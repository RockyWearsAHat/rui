//! macOS NSPanel implementation for borderless floating windows.
//!
//! Creates and manages NSPanel windows with the correct configuration:
//! - No border or title bar (NSWindowStyleMaskBorderless)
//! - Non-activating (becomesKeyOnlyIfNeeded = true)
//! - Floating level (NSFloatingWindowLevel)
//! - Shown without stealing focus (orderFront: instead of makeKeyAndOrderFront:)
//! - Dismissible via Escape key and click-outside (wired to AppKit event monitors)
//! - Supports custom rendering via content view

#![allow(unsafe_code, unsafe_op_in_unsafe_fn)]

use crate::Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use super::PanelOptions;

/// A handle to one native control (a label or a button) placed on a panel
/// window's content view.
///
/// Opaque on purpose: what it wraps is a platform pointer, and the only
/// things a caller may do with it are the ones [`super::PanelWindow`] exposes
/// (`set_text`, `set_enabled`, `on_action`'s tag).
#[derive(Clone, Copy)]
pub struct Widget(pub(crate) Object);

unsafe impl Send for Widget {}
unsafe impl Sync for Widget {}

/// Name of the button-target class this backend builds once per process.
const ACTION_TARGET_CLASS: &CStr = c"RuiPanelActionTarget";

/// A panel's button-click callback, shared between its own struct and its
/// [`ACTION_CALLBACKS`] entry so updating one updates what `buttonClicked:`
/// invokes.
type ActionCallback = Arc<Mutex<Option<Box<dyn Fn(i64) + Send + Sync>>>>;

thread_local! {
    // Registry from a live action-target object (as a pointer) to the Rust
    // callback it should invoke. Keyed by pointer rather than carried in an
    // ivar: the callback is a fat `Box<dyn Fn>`, and an ivar only has room
    // for one pointer-sized slot, so the trait object lives here instead and
    // the ivar-sized identity (the object's own address) is the key.
    static ACTION_CALLBACKS: RefCell<HashMap<usize, ActionCallback>> =
        RefCell::new(HashMap::new());
}

/// Builds (or finds, on a later panel) the Objective-C class every panel
/// button's target is an instance of.
fn action_target_class() -> Result<Object, Error> {
    unsafe {
        let existing = class(ACTION_TARGET_CLASS);
        if !existing.is_null() {
            return Ok(existing);
        }
        let superclass = class(c"NSObject");
        if superclass.is_null() {
            return Err(Error::Platform(
                "AppKit is not loaded: there is no NSObject".into(),
            ));
        }
        let built = objc_allocateClassPair(superclass, ACTION_TARGET_CLASS.as_ptr(), 0);
        if built.is_null() {
            return Err(Error::Platform(
                "a panel action target class could not be created".into(),
            ));
        }
        // "v@:@" — void return, self, selector, one object argument (the sender).
        let added = class_addMethod(
            built,
            sel(c"buttonClicked:"),
            button_clicked as *const c_void,
            c"v@:@".as_ptr(),
        );
        if !added {
            return Err(Error::Platform(
                "the panel action target would not take buttonClicked:".into(),
            ));
        }
        objc_registerClassPair(built);
        Ok(built)
    }
}

/// The `buttonClicked:` implementation shared by every panel button.
///
/// Looks itself up in [`ACTION_CALLBACKS`] by its own address, reads the
/// sender's tag (how a widget names itself back to the caller), and calls the
/// registered closure with it.
extern "C" fn button_clicked(this: Object, _sel: Sel, sender: Object) {
    let key = this as usize;
    let callback = ACTION_CALLBACKS.with(|callbacks| callbacks.borrow().get(&key).cloned());
    let Some(callback) = callback else { return };
    let tag: i64 = unsafe { send(sender, sel(c"tag")) };
    if let Ok(guard) = callback.lock() {
        if let Some(f) = guard.as_ref() {
            f(tag);
        }
    };
}

/// Platform-specific opaque handle for Objective-C objects.
type Object = *mut c_void;
/// Platform-specific selector for Objective-C method names.
type Sel = *const c_void;

/// macOS key code for Escape key
const KEY_CODE_ESCAPE: i64 = 53;

/// macOS CGRect struct combining point and size.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

/// macOS CGPoint struct for screen coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

/// macOS CGSize struct for dimensions.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

// Thread-local storage for the panel window object, allowing safe cleanup.
thread_local! {
    static PANEL_WINDOWS: RefCell<Vec<Object>> = const { RefCell::new(Vec::new()) };
}

/// Type-erased dismissal callback (must be Send + Sync).
type DismissCallback = Box<dyn Fn() + Send + Sync>;

/// Wrapper for the callback that explicitly implements Send + Sync.
struct SendableCallback(Option<DismissCallback>);
unsafe impl Send for SendableCallback {}
unsafe impl Sync for SendableCallback {}

/// macOS-specific panel window using NSPanel.
pub struct PanelWindowInner {
    /// The NSPanel object handle, retained by this struct.
    panel: Object,
    /// Optional dismissal callback, invoked when the panel is dismissed.
    dismiss_callback: Arc<Mutex<SendableCallback>>,
    /// Shared flag to signal the event monitor thread to stop.
    shutdown_signal: Arc<AtomicBool>,
    /// Receiver to acknowledge shutdown from the monitor thread.
    #[allow(dead_code)]
    shutdown_rx: Arc<Mutex<Option<mpsc::Receiver<()>>>>,
    /// This panel's own button target, an instance of [`action_target_class`].
    /// Retained for the panel's lifetime; every button's `setTarget:` points
    /// here, and [`ACTION_CALLBACKS`] is keyed by its address.
    action_target: Object,
    /// The one callback a caller has registered via `on_action`, shared with
    /// the [`ACTION_CALLBACKS`] entry so updating it here updates what
    /// `buttonClicked:` invokes.
    action_callback: ActionCallback,
}

impl PanelWindowInner {
    /// Create a new NSPanel with the given options.
    pub fn new(options: PanelOptions) -> Result<Self, Error> {
        unsafe {
            // Create an autorelease pool for this operation.
            let pool = objc_autoreleasePoolPush();

            // Build the content rect for the window.
            let content_rect = CGRect {
                origin: CGPoint {
                    x: options.x,
                    y: options.y,
                },
                size: CGSize {
                    width: options.width,
                    height: options.height,
                },
            };

            // Allocate NSPanel.
            let panel_class = class(c"NSPanel");
            if panel_class.is_null() {
                objc_autoreleasePoolPop(pool);
                return Err(Error::Platform("NSPanel class not found".into()));
            }

            let panel: Object = send(panel_class, sel(c"alloc"));
            if panel.is_null() {
                objc_autoreleasePoolPop(pool);
                return Err(Error::Platform("Failed to allocate NSPanel".into()));
            }

            // Initialize with borderless style mask (0), buffered backing, and non-deferred.
            // NSWindowStyleMaskBorderless = 0
            // NSBackingStoreBuffered = 2
            let panel: Object = send4(
                panel,
                sel(c"initWithContentRect:styleMask:backing:defer:"),
                content_rect,
                0i64,  // NSWindowStyleMaskBorderless
                2i64,  // NSBackingStoreBuffered
                false, // defer
            );

            if panel.is_null() {
                objc_autoreleasePoolPop(pool);
                return Err(Error::Platform("Failed to initialize NSPanel".into()));
            }

            // Set becomesKeyOnlyIfNeeded to true so it doesn't become key unless clicked.
            let _: () = send1(panel, sel(c"setBecomesKeyOnlyIfNeeded:"), true);

            // Set the window level to floating (NSFloatingWindowLevel = 2).
            let _: () = send1(panel, sel(c"setLevel:"), 2i64);

            // A borderless NSPanel is otherwise opaque white behind whatever
            // the content view draws, which is exactly what makes a rounded
            // content-view corner show square white corners poking out from
            // behind it. Both of these together are what let the content
            // view's own background and corner radius (set by the caller via
            // `style`) actually show, edge to edge.
            let _: () = send1(panel, sel(c"setOpaque:"), false);
            let clear: Object = send(class(c"NSColor"), sel(c"clearColor"));
            let _: () = send1(panel, sel(c"setBackgroundColor:"), clear);
            // Dark, regardless of the system's own appearance setting: this
            // panel's palette is fixed (it matches the app's own dark theme),
            // and without this a light-system-appearance user would get
            // light-mode default control colors — light NSButton bezels,
            // light-on-light where text does not override its own color —
            // fighting the dark background it sits on.
            let dark_name = ns_string("NSAppearanceNameDarkAqua");
            let dark: Object = send1(class(c"NSAppearance"), sel(c"appearanceNamed:"), dark_name);
            let _: () = send1(panel, sel(c"setAppearance:"), dark);

            // Show the window without making it key or activating the app.
            // orderFront: brings the window to the front without stealing focus.
            let _: () = send1(panel, sel(c"orderFront:"), std::ptr::null_mut::<c_void>());

            // Retain the panel so we own it.
            let _: Object = send(panel, sel(c"retain"));

            objc_autoreleasePoolPop(pool);

            // Store a reference so we can release it later.
            PANEL_WINDOWS.with(|pw| {
                pw.borrow_mut().push(panel);
            });

            let shutdown_signal = Arc::new(AtomicBool::new(false));
            let (shutdown_tx, shutdown_rx) = mpsc::channel();

            let target_class = action_target_class()?;
            let action_target: Object = send(target_class, sel(c"alloc"));
            let action_target: Object = send(action_target, sel(c"init"));
            let _: Object = send(action_target, sel(c"retain"));
            let action_callback: ActionCallback = Arc::new(Mutex::new(None));
            ACTION_CALLBACKS.with(|callbacks| {
                callbacks
                    .borrow_mut()
                    .insert(action_target as usize, Arc::clone(&action_callback));
            });

            let inner = PanelWindowInner {
                panel,
                dismiss_callback: Arc::new(Mutex::new(SendableCallback(None))),
                shutdown_signal: shutdown_signal.clone(),
                shutdown_rx: Arc::new(Mutex::new(Some(shutdown_rx))),
                action_target,
                action_callback,
            };

            // Set up event monitors after creation.
            inner.setup_event_monitors(shutdown_signal, shutdown_tx)?;

            Ok(inner)
        }
    }

    /// Set up background event monitoring for keyboard (Escape) and mouse (click-outside).
    fn setup_event_monitors(
        &self,
        shutdown_signal: Arc<AtomicBool>,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Result<(), Error> {
        // Store panel pointer as a usize to avoid Send issues with raw pointers.
        let panel_ptr = self.panel as usize;
        let dismiss_callback = Arc::clone(&self.dismiss_callback);
        let shutdown = shutdown_signal.clone();

        // Spawn a background thread to monitor events.
        let monitor_thread = thread::spawn(move || {
            let panel = panel_ptr as Object; // Convert back to Object pointer
            let mut prev_buttons = 0u64;
            let mut dismissed = false;

            while !shutdown.load(Ordering::Relaxed) && !dismissed {
                // Small sleep to avoid busy-waiting.
                thread::sleep(Duration::from_millis(50));

                unsafe {
                    let pool = objc_autoreleasePoolPush();

                    // Check mouse position for click-outside detection.
                    let mouse_location: CGPoint = send(class(c"NSEvent"), sel(c"mouseLocation"));

                    // Get the panel's frame to check bounds.
                    let frame: CGRect = send(panel, sel(c"frame"));

                    // Check if mouse is outside the panel bounds.
                    let is_outside_panel = mouse_location.x < frame.origin.x
                        || mouse_location.x >= (frame.origin.x + frame.size.width)
                        || mouse_location.y < frame.origin.y
                        || mouse_location.y >= (frame.origin.y + frame.size.height);

                    // Get the current pressed mouse buttons using NSEvent's pressedMouseButtons.
                    // This gives us a bitmask of which buttons are currently pressed.
                    let buttons: u64 = send(class(c"NSEvent"), sel(c"pressedMouseButtons"));

                    // Detect if a mouse button was just pressed (changed from not pressed to pressed).
                    let new_button_press = buttons != 0 && prev_buttons == 0;

                    // Get the current event to check for Escape key.
                    let app_class = class(c"NSApplication");
                    let app: Object = send(app_class, sel(c"sharedApplication"));
                    let current_event: Object = send(app, sel(c"currentEvent"));

                    // Check for Escape key (key code 53).
                    let mut should_dismiss = false;

                    if !current_event.is_null() {
                        let event_type: i64 = send(current_event, sel(c"type"));

                        // NSEventTypeKeyDown = 10
                        if event_type == 10 {
                            let key_code: i64 = send(current_event, sel(c"keyCode"));
                            if key_code == KEY_CODE_ESCAPE {
                                should_dismiss = true;
                            }
                        }
                    }

                    // Also dismiss on mouse click outside the panel.
                    if new_button_press && is_outside_panel {
                        should_dismiss = true;
                    }

                    if should_dismiss {
                        // Invoke the dismissal callback if set.
                        if let Ok(cb) = dismiss_callback.lock() {
                            if let Some(callback) = cb.0.as_ref() {
                                callback();
                            }
                        }
                        // Mark as dismissed and exit the monitoring loop.
                        dismissed = true;
                    }

                    prev_buttons = buttons;
                    objc_autoreleasePoolPop(pool);
                }
            }

            // Signal that the thread is shutting down.
            let _ = shutdown_tx.send(());
        });

        // Don't detach; let the thread run in the background until shutdown.
        std::mem::forget(monitor_thread);

        Ok(())
    }

    /// Show the panel window.
    pub fn show(&self) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let _: () = send1(
                self.panel,
                sel(c"orderFront:"),
                std::ptr::null_mut::<c_void>(),
            );
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Hide the panel window.
    pub fn hide(&self) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let _: () = send(self.panel, sel(c"orderOut:"));
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Move the panel to a new position.
    pub fn set_position(&self, x: f64, y: f64) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();

            // Get the current frame.
            let frame: CGRect = send(self.panel, sel(c"frame"));

            // Create a new frame with the updated origin.
            let new_frame = CGRect {
                origin: CGPoint { x, y },
                size: frame.size,
            };

            // Set the new frame.
            let _: () = send2(self.panel, sel(c"setFrame:display:"), new_frame, true);

            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Resize the panel window.
    pub fn set_size(&self, width: f64, height: f64) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();

            // Get the current frame.
            let frame: CGRect = send(self.panel, sel(c"frame"));

            // Create a new frame with the updated size.
            let new_frame = CGRect {
                origin: frame.origin,
                size: CGSize { width, height },
            };

            // Set the new frame.
            let _: () = send2(self.panel, sel(c"setFrame:display:"), new_frame, true);

            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> Result<bool, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let visible: bool = send(self.panel, sel(c"isVisible"));
            objc_autoreleasePoolPop(pool);
            Ok(visible)
        }
    }

    /// Register a dismissal callback.
    ///
    /// The callback is stored and can be invoked manually by the application
    /// or integrated with the app's event handling loop.
    pub fn on_dismiss(&self, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), Error> {
        // Store the callback for later invocation
        if let Ok(mut cb) = self.dismiss_callback.lock() {
            cb.0 = Some(callback);
        }
        Ok(())
    }

    /// Invoke the dismissal callback if one is registered.
    ///
    /// This is called when the panel should be dismissed (e.g., on Escape key
    /// or click outside). Applications can call this method directly or wire it
    /// into their event handling.
    pub fn invoke_dismiss(&self) {
        if let Ok(cb) = self.dismiss_callback.lock() {
            if let Some(callback) = cb.0.as_ref() {
                callback();
            }
        }
    }

    /// Check if this panel point is inside the panel bounds.
    ///
    /// Useful for implementing click-away dismissal: if a click point is outside
    /// the panel frame, the application can call `invoke_dismiss()`.
    pub fn contains_point(&self, x: f64, y: f64) -> Result<bool, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let frame: CGRect = send(self.panel, sel(c"frame"));
            let contains = x >= frame.origin.x
                && x < (frame.origin.x + frame.size.width)
                && y >= frame.origin.y
                && y < (frame.origin.y + frame.size.height);
            objc_autoreleasePoolPop(pool);
            Ok(contains)
        }
    }

    /// Sets the content view's background color, corner radius, and a
    /// hairline border — the look of the panel as a whole, as opposed to any
    /// one control on it. `background` and `border` are sRGB `(r, g, b)`,
    /// each `0.0..=1.0`; `border_width` in points (`0.0` omits the border).
    pub fn style(
        &self,
        background: (f32, f32, f32),
        corner_radius: f64,
        border: (f32, f32, f32),
        border_width: f64,
    ) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let content_view: Object = send(self.panel, sel(c"contentView"));
            let _: () = send1(content_view, sel(c"setWantsLayer:"), true);
            let layer: Object = send(content_view, sel(c"layer"));
            let bg = cg_color(background, 1.0);
            let _: () = send1(layer, sel(c"setBackgroundColor:"), bg);
            let _: () = send1(layer, sel(c"setCornerRadius:"), corner_radius);
            let _: () = send1(layer, sel(c"setMasksToBounds:"), true);
            if border_width > 0.0 {
                let border_color = cg_color(border, 1.0);
                let _: () = send1(layer, sel(c"setBorderColor:"), border_color);
                let _: () = send1(layer, sel(c"setBorderWidth:"), border_width);
            }
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Adds a non-interactive line of text to the panel's content view.
    pub fn add_label(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text: &str,
    ) -> Result<Widget, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let field_class = class(c"NSTextField");
            if field_class.is_null() {
                objc_autoreleasePoolPop(pool);
                return Err(Error::Platform("NSTextField class not found".into()));
            }
            let field: Object = send(field_class, sel(c"alloc"));
            let frame = CGRect {
                origin: CGPoint { x, y },
                size: CGSize { width, height },
            };
            let field: Object = send1(field, sel(c"initWithFrame:"), frame);
            let value = ns_string(text);
            let _: () = send1(field, sel(c"setStringValue:"), value);
            let _: () = send1(field, sel(c"setEditable:"), false);
            let _: () = send1(field, sel(c"setSelectable:"), false);
            let _: () = send1(field, sel(c"setBezeled:"), false);
            let _: () = send1(field, sel(c"setDrawsBackground:"), false);
            let content_view: Object = send(self.panel, sel(c"contentView"));
            let _: () = send1(content_view, sel(c"addSubview:"), field);
            objc_autoreleasePoolPop(pool);
            Ok(Widget(field))
        }
    }

    /// Adds a clickable button to the panel's content view. `tag` is the
    /// value handed to the [`on_action`](Self::on_action) callback when this
    /// button is pressed — the caller's own way of telling buttons apart.
    pub fn add_button(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        title: &str,
        tag: i64,
    ) -> Result<Widget, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let button_class = class(c"NSButton");
            if button_class.is_null() {
                objc_autoreleasePoolPop(pool);
                return Err(Error::Platform("NSButton class not found".into()));
            }
            let button: Object = send(button_class, sel(c"alloc"));
            let frame = CGRect {
                origin: CGPoint { x, y },
                size: CGSize { width, height },
            };
            let button: Object = send1(button, sel(c"initWithFrame:"), frame);
            let value = ns_string(title);
            let _: () = send1(button, sel(c"setTitle:"), value);
            // NSBezelStyleRounded = 1
            let _: () = send1(button, sel(c"setBezelStyle:"), 1i64);
            let _: () = send1(button, sel(c"setTag:"), tag);
            let _: () = send1(button, sel(c"setTarget:"), self.action_target);
            let _: () = send1(button, sel(c"setAction:"), sel(c"buttonClicked:"));
            let content_view: Object = send(self.panel, sel(c"contentView"));
            let _: () = send1(content_view, sel(c"addSubview:"), button);
            objc_autoreleasePoolPop(pool);
            Ok(Widget(button))
        }
    }

    /// Changes a label's or button's text.
    pub fn set_text(&self, widget: &Widget, text: &str) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            // NSTextField reads back through `stringValue`/`setStringValue:`;
            // NSButton's `setTitle:` covers a button the same call is used on.
            let class_name = send::<Object>(widget.0, sel(c"className"));
            let name = from_ns_string(class_name);
            let value = ns_string(text);
            if name == "NSButton" {
                let _: () = send1(widget.0, sel(c"setTitle:"), value);
            } else {
                let _: () = send1(widget.0, sel(c"setStringValue:"), value);
            }
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Enables or disables a button (a no-op, harmlessly, on a label).
    pub fn set_enabled(&self, widget: &Widget, enabled: bool) -> Result<(), Error> {
        unsafe {
            let _: () = send1(widget.0, sel(c"setEnabled:"), enabled);
            Ok(())
        }
    }

    /// Tints a label's text (a no-op, harmlessly, on a button — a button's
    /// color follows the platform's control style, not arbitrary tinting).
    pub fn set_text_color(&self, widget: &Widget, rgb: (f32, f32, f32)) -> Result<(), Error> {
        unsafe {
            let class_name = send::<Object>(widget.0, sel(c"className"));
            if from_ns_string(class_name) == "NSButton" {
                return Ok(());
            }
            let pool = objc_autoreleasePoolPush();
            let color: Object = send4(
                class(c"NSColor"),
                sel(c"colorWithSRGBRed:green:blue:alpha:"),
                rgb.0 as f64,
                rgb.1 as f64,
                rgb.2 as f64,
                1.0f64,
            );
            let _: () = send1(widget.0, sel(c"setTextColor:"), color);
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Tints a button's bezel (a no-op, harmlessly, on a label). `emphasis`
    /// draws it filled in that color with a light title, matching a primary
    /// action; otherwise it is a quieter outline in that color, for a
    /// secondary one.
    pub fn set_button_tint(
        &self,
        widget: &Widget,
        rgb: (f32, f32, f32),
        emphasis: bool,
    ) -> Result<(), Error> {
        unsafe {
            let class_name = send::<Object>(widget.0, sel(c"className"));
            if from_ns_string(class_name) != "NSButton" {
                return Ok(());
            }
            let pool = objc_autoreleasePoolPush();
            // The stock bezel chrome (rounded-rect bevel/highlight) draws over
            // whatever the layer behind it shows, so a tint is only visible at
            // all once the button stops drawing that chrome itself.
            let _: () = send1(widget.0, sel(c"setBordered:"), false);
            let _: () = send1(widget.0, sel(c"setWantsLayer:"), true);
            let layer: Object = send(widget.0, sel(c"layer"));
            let _: () = send1(layer, sel(c"setCornerRadius:"), 6.0f64);
            let title_color = if emphasis {
                let fill = cg_color(rgb, 1.0);
                let _: () = send1(layer, sel(c"setBackgroundColor:"), fill);
                // A light title on a filled, saturated bezel; not the tint
                // color itself, which would vanish against its own fill.
                (0.04, 0.05, 0.07)
            } else {
                let clear = cg_color((0.0, 0.0, 0.0), 0.0);
                let _: () = send1(layer, sel(c"setBackgroundColor:"), clear);
                let border = cg_color(rgb, 0.55);
                let _: () = send1(layer, sel(c"setBorderColor:"), border);
                let _: () = send1(layer, sel(c"setBorderWidth:"), 1.0f64);
                rgb
            };
            set_button_title_color(widget.0, title_color);
            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Registers the one callback every button on this panel reports its
    /// press to, carrying the tag it was created with. Replaces any callback
    /// registered earlier.
    pub fn on_action<F>(&self, callback: F) -> Result<(), Error>
    where
        F: Fn(i64) + Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.action_callback.lock() {
            *guard = Some(Box::new(callback));
        }
        Ok(())
    }

    /// Close and release the panel.
    pub fn close(&mut self) -> Result<(), Error> {
        // Signal the event monitor thread to shut down.
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Wait for the shutdown confirmation with a timeout.
        if let Ok(mut rx_guard) = self.shutdown_rx.lock() {
            if let Some(rx) = rx_guard.take() {
                let _ = rx.recv_timeout(Duration::from_millis(500));
            }
        }

        unsafe {
            let pool = objc_autoreleasePoolPush();

            // Close the window.
            let _: () = send1(self.panel, sel(c"close"), true);

            // Release our retained reference.
            let _: Object = send(self.panel, sel(c"release"));

            objc_autoreleasePoolPop(pool);

            // Remove from the thread-local registry.
            PANEL_WINDOWS.with(|pw| {
                let mut windows = pw.borrow_mut();
                windows.retain(|&w| w != self.panel);
            });
            ACTION_CALLBACKS.with(|callbacks| {
                callbacks
                    .borrow_mut()
                    .remove(&(self.action_target as usize));
            });
            let _: Object = send(self.action_target, sel(c"release"));

            Ok(())
        }
    }
}

impl Drop for PanelWindowInner {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

// SAFETY: The underlying Objective-C objects are thread-safe as long as
// we only access them from the main thread (AppKit's requirement).
unsafe impl Send for PanelWindowInner {}
unsafe impl Sync for PanelWindowInner {}

// Objective-C FFI declarations.
unsafe extern "C" {
    fn objc_msgSend();
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
    fn objc_getClass(name: *const c_char) -> Object;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_allocateClassPair(
        superclass: Object,
        name: *const c_char,
        extra_bytes: usize,
    ) -> Object;
    fn objc_registerClassPair(class: Object);
    fn class_addMethod(class: Object, name: Sel, imp: *const c_void, types: *const c_char) -> bool;
}

/// A `CGColorRef` for `rgb` (sRGB, each channel `0.0..=1.0`) at `alpha`, valid
/// until the pool is popped — layer properties (`backgroundColor`,
/// `borderColor`) want a `CGColorRef` specifically, unlike a control's
/// `NSColor`-typed properties (`setTextColor:` and the like).
fn cg_color(rgb: (f32, f32, f32), alpha: f32) -> Object {
    unsafe {
        let ns_color: Object = send4(
            class(c"NSColor"),
            sel(c"colorWithSRGBRed:green:blue:alpha:"),
            rgb.0 as f64,
            rgb.1 as f64,
            rgb.2 as f64,
            alpha as f64,
        );
        send(ns_color, sel(c"CGColor"))
    }
}

/// Recolors a button's current title in place, via an attributed string —
/// `-setTitleColor:` does not exist on `NSButton`; this is the real way.
///
/// Only holds until the next plain `-setTitle:` (what [`PanelWindowInner::set_text`]
/// uses), which resets the title to unattributed text and so back to the
/// default (readable, under this panel's forced dark appearance) title
/// color — acceptable for a button whose *label* changes state
/// (Connect/Disconnect/Cancel) more often than its tint does.
unsafe fn set_button_title_color(button: Object, rgb: (f32, f32, f32)) {
    unsafe {
        let title: Object = send(button, sel(c"title"));
        let attributed: Object = send(class(c"NSMutableAttributedString"), sel(c"alloc"));
        let attributed: Object = send1(attributed, sel(c"initWithString:"), title);
        let color = {
            let ns_color: Object = send4(
                class(c"NSColor"),
                sel(c"colorWithSRGBRed:green:blue:alpha:"),
                rgb.0 as f64,
                rgb.1 as f64,
                rgb.2 as f64,
                1.0f64,
            );
            ns_color
        };
        let key = ns_string("NSColor");
        let length: i64 = send(title, sel(c"length"));
        let range = NsRangeLocal {
            location: 0,
            length,
        };
        let _: () = send3(
            attributed,
            sel(c"addAttribute:value:range:"),
            key,
            color,
            range,
        );
        let _: () = send1(button, sel(c"setAttributedTitle:"), attributed);
    }
}

/// An `NSRange`, passed by value the same way the rest of this file's structs
/// are — a local definition rather than importing the shell backend's, since
/// this file has none of its own C-struct plumbing otherwise.
#[repr(C)]
struct NsRangeLocal {
    location: i64,
    length: i64,
}

/// An `NSString` holding `text`, valid until the pool is popped.
fn ns_string(text: &str) -> Object {
    let text = std::ffi::CString::new(text).unwrap_or_default();
    unsafe {
        send1(
            class(c"NSString"),
            sel(c"stringWithUTF8String:"),
            text.as_ptr(),
        )
    }
}

/// The Rust string behind an `NSString`, or empty when there is none.
fn from_ns_string(string: Object) -> String {
    if string.is_null() {
        return String::new();
    }
    let utf8: *const c_char = unsafe { send(string, sel(c"UTF8String")) };
    if utf8.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(utf8) }
        .to_string_lossy()
        .into_owned()
}

/// Get an Objective-C class by name.
#[inline]
unsafe fn class(name: &CStr) -> Object {
    objc_getClass(name.as_ptr())
}

/// Get or register an Objective-C selector.
#[inline]
unsafe fn sel(name: &CStr) -> Sel {
    sel_registerName(name.as_ptr())
}

/// Send a message with no arguments.
#[inline]
unsafe fn send<R>(obj: Object, selector: Sel) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(obj, selector)
}

/// Send a message with one argument.
#[inline]
unsafe fn send1<R, A>(obj: Object, selector: Sel, arg: A) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(obj, selector, arg)
}

/// Send a message with two arguments.
#[inline]
unsafe fn send2<R, A, B>(obj: Object, selector: Sel, arg1: A, arg2: B) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A, B) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(obj, selector, arg1, arg2)
}

/// Send a message with three arguments.
#[inline]
#[allow(dead_code)]
unsafe fn send3<R, A, B, C>(obj: Object, selector: Sel, arg1: A, arg2: B, arg3: C) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A, B, C) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(obj, selector, arg1, arg2, arg3)
}

/// Send a message with four arguments.
#[inline]
unsafe fn send4<R, A, B, C, D>(
    obj: Object,
    selector: Sel,
    arg1: A,
    arg2: B,
    arg3: C,
    arg4: D,
) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A, B, C, D) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(obj, selector, arg1, arg2, arg3, arg4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cgpoint_creation() {
        let pt = CGPoint { x: 100.0, y: 200.0 };
        assert_eq!(pt.x, 100.0);
        assert_eq!(pt.y, 200.0);
    }

    #[test]
    fn cgsize_creation() {
        let size = CGSize {
            width: 300.0,
            height: 400.0,
        };
        assert_eq!(size.width, 300.0);
        assert_eq!(size.height, 400.0);
    }

    #[test]
    fn cgrect_creation() {
        let rect = CGRect {
            origin: CGPoint { x: 10.0, y: 20.0 },
            size: CGSize {
                width: 100.0,
                height: 200.0,
            },
        };
        assert_eq!(rect.origin.x, 10.0);
        assert_eq!(rect.origin.y, 20.0);
        assert_eq!(rect.size.width, 100.0);
        assert_eq!(rect.size.height, 200.0);
    }
}
