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
use std::ffi::{c_char, c_void, CStr};
use std::sync::mpsc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use super::PanelOptions;

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

            let inner = PanelWindowInner {
                panel,
                dismiss_callback: Arc::new(Mutex::new(SendableCallback(None))),
                shutdown_signal: shutdown_signal.clone(),
                shutdown_rx: Arc::new(Mutex::new(Some(shutdown_rx))),
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
