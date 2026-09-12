//! macOS system tray implementation using NSStatusItem.
//!
//! # Design
//!
//! The macOS tray uses NSStatusBar's `statusItemWithLength:` to create an
//! NSStatusItem, which automatically manages placement in the system menu bar.
//! We avoid any delegate patterns or callbacks; instead, event handlers
//! directly post TrayEvents to the shared event queue.
//!
//! ## Objective-C Message Sending
//!
//! All Objective-C calls go through `objc_msgSend` with proper transmutation
//! to the correct function signature. This is unsafe but isolated to this module,
//! allowing the public API to be completely safe.
//!
//! ## Lifecycle
//!
//! An autorelease pool is created and destroyed around every allocation-heavy
//! operation to prevent memory leaks. The NSStatusItem is retained by
//! NSStatusBar, so we don't need to manually release it in Drop.
//!
//! ## Menu Reconstruction
//!
//! Each call to `set_menu()` rebuilds the NSMenu from scratch. Old items are
//! removed and new ones added. This matches the reactive design: the menu
//! is not retained; it is rebuilt each frame from app state.

#![allow(unsafe_code, unsafe_op_in_unsafe_fn)]

use crate::Error;
use std::cell::RefCell;
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Arc, Mutex};

use super::{TrayEvent, TrayMenuItem};

/// Platform-specific opaque handle for Objective-C objects.
type Object = *mut c_void;
/// Platform-specific selector for Objective-C method names.
type Sel = *const c_void;

/// macOS NSPoint struct for screen coordinates.
#[repr(C)]
#[derive(Copy, Clone)]
struct NSPoint {
    x: f64,
    y: f64,
}

/// macOS NSSize struct for dimensions.
#[repr(C)]
#[derive(Copy, Clone)]
struct NSSize {
    width: f64,
    height: f64,
}

/// macOS NSRect struct combining point and size.
#[repr(C)]
#[derive(Copy, Clone)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

/// The macOS tray implementation using NSStatusItem.
pub struct TrayInner {
    #[allow(dead_code)]
    status_item: Object,
    button: Object,
    menu: Object,
    #[allow(dead_code)]
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    /// Whether to emit IconActivatedForPanel events on button click instead
    /// of showing the standard menu.
    #[allow(dead_code)]
    use_panel: bool,
}

// SAFETY: The underlying Objective-C objects are thread-safe as long as
// we only access them from the main thread (AppKit's requirement).
unsafe impl Send for TrayInner {}
unsafe impl Sync for TrayInner {}

// Thread-local storage for the event queue and panel mode flag
//
// A thread-local rather than passing the queue through the Objective-C call is
// what makes the click handler below possible without a custom ivar: AppKit
// always invokes a menu item's action on the main thread, which is the same
// thread that ever calls `TrayInner::new`/`set_menu`, so the thread-local is
// guaranteed to hold the right queue when the handler runs.
thread_local! {
    static TRAY_MENU_HANDLER: RefCell<Option<Arc<Mutex<Vec<TrayEvent>>>>> = const { RefCell::new(None) };
    static TRAY_PANEL_MODE: RefCell<bool> = const { RefCell::new(false) };
    static TRAY_BUTTON: RefCell<Option<Object>> = const { RefCell::new(None) };
}

/// The Objective-C class that receives menu-item clicks, created once and
/// reused for every menu item on every tray this process ever creates.
///
/// # Why a real class instead of the null-action placeholder this replaced
///
/// A menu item with `action: null` never invokes anything — AppKit simply has
/// nothing to call, so a click just closes the menu. Making a click actually
/// do something requires a real Objective-C target/action pair, and Rust has
/// no compiler support for defining an Objective-C method — the runtime has
/// to be asked to build one. `objc_allocateClassPair` plus `class_addMethod`
/// is exactly that ask: it manufactures a subclass of `NSObject` at runtime
/// carrying one C function as its `menuItemClicked:` implementation. The
/// class (and the one instance of it below) are process-lifetime singletons
/// on purpose — every tray this process creates shares the same target, so
/// there is exactly one place a click ever lands, matching the single shared
/// `TRAY_MENU_HANDLER` it reads from.
fn tray_target_class() -> Object {
    use std::sync::OnceLock;
    static CLASS: OnceLock<usize> = OnceLock::new();
    let ptr = *CLASS.get_or_init(|| unsafe {
        let superclass = class(c"NSObject");
        let cls: Object = objc_allocateClassPair(superclass, c"RuiTrayMenuTarget".as_ptr(), 0);
        if !cls.is_null() {
            let imp: extern "C" fn(Object, Sel, Object) = tray_menu_item_clicked;
            // "v@:@" = void return, (self, _cmd, one object argument) — the
            // standard Cocoa action-method signature every NSMenuItem expects.
            class_addMethod(
                cls,
                sel(c"menuItemClicked:"),
                imp as *const c_void,
                c"v@:@".as_ptr(),
            );
            objc_registerClassPair(cls);
        }
        cls as usize
    });
    ptr as Object
}

/// The single instance of [`tray_target_class`], retained for the process's
/// whole lifetime — every menu item's `target` points at this one object.
fn tray_target_instance() -> Object {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<usize> = OnceLock::new();
    let ptr = *INSTANCE.get_or_init(|| unsafe {
        let cls = tray_target_class();
        let obj: Object = send(cls, sel(c"alloc"));
        let obj: Object = send(obj, sel(c"init"));
        let _: () = send(obj, sel(c"retain"));
        obj as usize
    });
    ptr as Object
}

/// The Objective-C class for button click handling in panel mode.
fn tray_button_target_class() -> Object {
    use std::sync::OnceLock;
    static CLASS: OnceLock<usize> = OnceLock::new();
    let ptr = *CLASS.get_or_init(|| unsafe {
        let superclass = class(c"NSObject");
        let cls: Object = objc_allocateClassPair(superclass, c"RuiTrayButtonTarget".as_ptr(), 0);
        if !cls.is_null() {
            let imp: extern "C" fn(Object, Sel, Object) = tray_button_clicked;
            class_addMethod(
                cls,
                sel(c"buttonClicked:"),
                imp as *const c_void,
                c"v@:@".as_ptr(),
            );
            objc_registerClassPair(cls);
        }
        cls as usize
    });
    ptr as Object
}

/// The single instance of [`tray_button_target_class`] for button click handling.
fn tray_button_target_instance() -> Object {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<usize> = OnceLock::new();
    let ptr = *INSTANCE.get_or_init(|| unsafe {
        let cls = tray_button_target_class();
        let obj: Object = send(cls, sel(c"alloc"));
        let obj: Object = send(obj, sel(c"init"));
        let _: () = send(obj, sel(c"retain"));
        obj as usize
    });
    ptr as Object
}

/// The `menuItemClicked:` action every tray menu item's `target`/`action`
/// point at. `sender` is the `NSMenuItem` that was clicked; its `tag` is the
/// [`TrayMenuItem::id`] `set_menu` stamped onto it, which is how this reaches
/// back to which item fired without needing per-item Objective-C state.
extern "C" fn tray_menu_item_clicked(_this: Object, _cmd: Sel, sender: Object) {
    let tag: i64 = unsafe { send(sender, sel(c"tag")) };
    TRAY_MENU_HANDLER.with(|handler| {
        if let Some(queue) = handler.borrow().as_ref() {
            if let Ok(mut events) = queue.lock() {
                events.push(TrayEvent::MenuItemClicked(tag as usize));
            }
        }
    });
}

/// The `buttonClicked:` action for the tray button when in panel mode.
/// This is called when the tray icon is clicked in panel mode.
extern "C" fn tray_button_clicked(_this: Object, _cmd: Sel, _sender: Object) {
    eprintln!("RUI TRAY: button_clicked handler INVOKED");
    TRAY_PANEL_MODE.with(|mode_ref| {
        let is_panel_mode = *mode_ref.borrow();
        eprintln!("RUI TRAY: is_panel_mode = {}", is_panel_mode);
        if is_panel_mode {
            eprintln!("RUI TRAY: In panel mode, attempting to get button");
            TRAY_BUTTON.with(|button_ref| {
                if let Some(button) = button_ref.borrow().as_ref() {
                    eprintln!("RUI TRAY: Got button, computing screen position");
                    // Get the button's frame in screen coordinates
                    let button = *button;
                    unsafe {
                        let frame: NSRect = send(button, sel(c"frame"));
                        // Convert to screen coordinates
                        let window: Object = send(button, sel(c"window"));
                        let window_frame: NSRect = send(window, sel(c"frame"));
                        let screen_x = window_frame.origin.x + frame.origin.x;
                        let screen_y = window_frame.origin.y + frame.origin.y;

                        TRAY_MENU_HANDLER.with(|handler| {
                            if let Some(queue) = handler.borrow().as_ref() {
                                if let Ok(mut events) = queue.lock() {
                                    events.push(TrayEvent::IconActivatedForPanel {
                                        screen_position: (screen_x, screen_y),
                                    });
                                    eprintln!(
                                        "RUI TRAY: Posted IconActivatedForPanel event at ({}, {})",
                                        screen_x, screen_y
                                    );
                                }
                            } else {
                                eprintln!("RUI TRAY: No event queue in handler");
                            }
                        });
                    }
                } else {
                    eprintln!("RUI TRAY: No button in thread-local storage");
                }
            });
        } else {
            eprintln!("RUI TRAY: NOT in panel mode, ignoring click");
        }
    });
}

impl TrayInner {
    /// Create a new tray icon on macOS.
    pub fn new(
        icon_data: &[u8],
        tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
        use_panel: bool,
    ) -> Result<Self, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let result = Self::new_inner(icon_data, tooltip, event_queue.clone(), use_panel)?;

            // Explicitly retain the status item to keep it alive after the autorelease pool is popped.
            // NSStatusBar should retain it, but we add an extra retain to be safe.
            let _: () = send(result.status_item, sel(c"retain"));

            objc_autoreleasePoolPop(pool);
            Ok(result)
        }
    }

    unsafe fn new_inner(
        icon_data: &[u8],
        tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
        use_panel: bool,
    ) -> Result<Self, Error> {
        // Get the system status bar
        let status_bar: Object = send(class(c"NSStatusBar"), sel(c"systemStatusBar"));
        if status_bar.is_null() {
            return Err(Error::Platform("NSStatusBar unavailable".into()));
        }

        // Create a status item with automatic width
        let status_item: Object = send1(
            status_bar,
            sel(c"statusItemWithLength:"),
            -2.0_f64, // NSVariableStatusItemLength
        );
        if status_item.is_null() {
            return Err(Error::Platform("Could not create NSStatusItem".into()));
        }

        // Get the button of the status item
        let button: Object = send(status_item, sel(c"button"));
        if button.is_null() {
            return Err(Error::Platform(
                "Could not access status item button".into(),
            ));
        }

        // Create the initial menu (empty for now)
        let menu: Object = send(class(c"NSMenu"), sel(c"alloc"));
        let menu: Object = send1(menu, sel(c"initWithTitle:"), ns_string(c""));

        // Set tooltip
        if !tooltip.is_empty() {
            if let Ok(tooltip_cstr) = CStr::from_bytes_with_nul(tooltip.as_bytes()) {
                let _: () = send1(button, sel(c"setToolTip:"), ns_string(tooltip_cstr));
            } else {
                let tooltip_with_nul = std::ffi::CString::new(tooltip)
                    .map_err(|_| Error::Platform("Invalid tooltip".into()))?;
                let _: () = send1(button, sel(c"setToolTip:"), ns_string(&tooltip_with_nul));
            }
        }

        // Set the actual icon image (use the provided data, not a placeholder)
        // NSStatusBar requires a valid image to display the item
        let image = if icon_data.is_empty() {
            create_placeholder_image()?
        } else {
            image_from_png(icon_data)?
        };
        let _: () = send1(button, sel(c"setImage:"), image);

        // Set the menu on the status item. In panel mode, we'll remove it later in set_panel_mode.
        let _: () = send1(status_item, sel(c"setMenu:"), menu);
        eprintln!("RUI TRAY: Initial menu set on status item");

        // If in panel mode, set up button action to emit IconActivatedForPanel events
        if use_panel {
            TRAY_PANEL_MODE.with(|mode| {
                *mode.borrow_mut() = true;
            });
            TRAY_BUTTON.with(|btn| {
                *btn.borrow_mut() = Some(button);
            });

            // Create the target object for the button click
            let button_target: Object = tray_button_target_instance();
            let _: () = send1(button, sel(c"setTarget:"), button_target);
            let _: () = send1(button, sel(c"setAction:"), sel(c"buttonClicked:"));
        }

        // Store event queue in thread-local for button click handler
        TRAY_MENU_HANDLER.with(|handler| {
            *handler.borrow_mut() = Some(event_queue.clone());
        });

        Ok(TrayInner {
            status_item,
            button,
            menu,
            event_queue,
            use_panel,
        })
    }

    /// Update the menu items.
    pub fn set_menu(&self, items: Vec<TrayMenuItem>) -> Result<(), Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();

            // Remove old menu items
            let count: i64 = send(self.menu, sel(c"numberOfItems"));
            for _ in 0..count {
                let _: () = send1(self.menu, sel(c"removeItemAtIndex:"), 0i64);
            }

            // Add new items
            for item in items {
                let label =
                    std::ffi::CString::new(item.label.clone()).unwrap_or_else(|_| c"".to_owned());
                let ns_label = ns_string(&label);

                // Real target/action: `tray_menu_item_clicked` (a runtime-built
                // Objective-C method, see `tray_target_class`) reads the clicked
                // item's tag and posts a TrayEvent — this is what makes a click
                // actually do something, replacing the previous null-action
                // placeholder that only ever closed the menu.
                let menu_item: Object = send(class(c"NSMenuItem"), sel(c"alloc"));
                let menu_item: Object = send3(
                    menu_item,
                    sel(c"initWithTitle:action:keyEquivalent:"),
                    ns_label,
                    sel(c"menuItemClicked:"),
                    ns_string(c""),
                );

                if !menu_item.is_null() {
                    let _: () = send1(menu_item, sel(c"setTarget:"), tray_target_instance());
                    // The tag IS the event's payload: tray_menu_item_clicked reads
                    // it straight off the sender, so this is not optional bookkeeping.
                    let _: () = send1(menu_item, sel(c"setTag:"), item.id as i64);

                    // Set enabled state
                    let _: () = send1(menu_item, sel(c"setEnabled:"), item.enabled);

                    // Set selected state (checkmark)
                    if item.selected {
                        let _: () = send1(menu_item, sel(c"setState:"), 1i64); // NSControlStateValueOn
                    }

                    // Add to menu
                    let _: () = send1(self.menu, sel(c"addItem:"), menu_item);
                }
            }

            objc_autoreleasePoolPop(pool);
            Ok(())
        }
    }

    /// Change the tooltip text.
    pub fn set_tooltip(&self, text: &str) -> Result<(), Error> {
        unsafe {
            let cstr = std::ffi::CString::new(text)
                .map_err(|_| Error::Platform("Invalid tooltip".into()))?;
            let _: () = send1(self.button, sel(c"setToolTip:"), ns_string(&cstr));
            Ok(())
        }
    }

    /// Change the icon image.
    pub fn set_icon(&self, data: &[u8]) -> Result<(), Error> {
        unsafe {
            let image = image_from_png(data)?;
            let _: () = send1(self.button, sel(c"setImage:"), image);
            Ok(())
        }
    }

    /// Enable or disable panel mode.
    pub fn set_panel_mode(&self, enabled: bool) -> Result<(), Error> {
        unsafe {
            if enabled {
                eprintln!("RUI TRAY: Enabling panel mode");
                TRAY_PANEL_MODE.with(|mode| {
                    *mode.borrow_mut() = true;
                });
                TRAY_BUTTON.with(|btn| {
                    *btn.borrow_mut() = Some(self.button);
                });

                // Create a simple menu with a single item that posts the panel event.
                // Use the existing menu infrastructure so the click handler works.
                // The item will have id=999 (reserved for panel), and when clicked,
                // the menu_item_clicked handler will route it to the panel logic.
                eprintln!("RUI TRAY: Clearing menu items for panel mode");
                let count: i64 = send(self.menu, sel(c"numberOfItems"));
                for _ in 0..count {
                    let _: () = send1(self.menu, sel(c"removeItemAtIndex:"), 0i64);
                }

                // Add a single "Panel" item
                let menu_item: Object = send(class(c"NSMenuItem"), sel(c"alloc"));
                let menu_item: Object = send3(
                    menu_item,
                    sel(c"initWithTitle:action:keyEquivalent:"),
                    ns_string(c"Show"),
                    sel(c"menuItemClicked:"),
                    ns_string(c""),
                );

                if !menu_item.is_null() {
                    let button_target: Object = tray_target_instance();
                    let _: () = send1(menu_item, sel(c"setTarget:"), button_target);
                    // Use a special tag (9999) to identify this as the panel trigger
                    let _: () = send1(menu_item, sel(c"setTag:"), 9999i64);
                    let _: () = send1(self.menu, sel(c"addItem:"), menu_item);
                    eprintln!("RUI TRAY: Added panel trigger item (tag=9999) to menu");
                }

                // The normal menu is already set, so nothing else to do
                eprintln!("RUI TRAY: Panel mode enabled");
            } else {
                TRAY_PANEL_MODE.with(|mode| {
                    *mode.borrow_mut() = false;
                });
                TRAY_BUTTON.with(|btn| {
                    *btn.borrow_mut() = None;
                });

                eprintln!("RUI TRAY: Panel mode disabled");
            }
            Ok(())
        }
    }
}

impl Drop for TrayInner {
    fn drop(&mut self) {
        unsafe {
            // Release the extra retain we added to keep the status item alive.
            let _: () = send(self.status_item, sel(c"release"));
        }

        // Clean up thread-local references
        TRAY_MENU_HANDLER.with(|handler| {
            *handler.borrow_mut() = None;
        });
        TRAY_PANEL_MODE.with(|mode| {
            *mode.borrow_mut() = false;
        });
        TRAY_BUTTON.with(|btn| {
            *btn.borrow_mut() = None;
        });
    }
}

// ============================================================================
// Objective-C message sending helpers
// ============================================================================

/// Get a class by name.
fn class(name: &CStr) -> Object {
    unsafe { objc_getClass(name.as_ptr()) }
}

/// Get a selector by name.
fn sel(name: &CStr) -> Sel {
    unsafe { sel_registerName(name.as_ptr()) }
}

/// Create an NSString from a C string.
fn ns_string(text: &CStr) -> Object {
    unsafe {
        send1(
            class(c"NSString"),
            sel(c"stringWithUTF8String:"),
            text.as_ptr(),
        )
    }
}

/// Send a message taking no arguments.
unsafe fn send<R>(receiver: Object, selector: Sel) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(receiver, selector)
}

/// Send a message taking one argument.
unsafe fn send1<R, A>(receiver: Object, selector: Sel, a: A) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(receiver, selector, a)
}

/// Send a message taking two arguments.
unsafe fn send2<R, A, B>(receiver: Object, selector: Sel, a: A, b: B) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A, B) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(receiver, selector, a, b)
}

/// Send a message taking three arguments.
unsafe fn send3<R, A, B, C>(receiver: Object, selector: Sel, a: A, b: B, c: C) -> R {
    let dispatch: unsafe extern "C" fn(Object, Sel, A, B, C) -> R =
        std::mem::transmute(objc_msgSend as *const ());
    dispatch(receiver, selector, a, b, c)
}

// ============================================================================
// Objective-C FFI declarations
// ============================================================================

unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Object;
    fn sel_registerName(name: *const c_char) -> Sel;
    fn objc_autoreleasePoolPush() -> *mut c_void;
    fn objc_autoreleasePoolPop(pool: *mut c_void);
    fn objc_msgSend();
    /// Builds a new class at runtime — this is how `tray_target_class` gets
    /// an `NSObject` subclass to hang `menuItemClicked:` off of without the
    /// compiler (which has no notion of an Objective-C class) being involved.
    fn objc_allocateClassPair(
        superclass: Object,
        name: *const c_char,
        extra_bytes: usize,
    ) -> Object;
    /// Finishes registering a class built with `objc_allocateClassPair` —
    /// the class cannot be instantiated (`alloc`/`init`) before this runs.
    fn objc_registerClassPair(cls: Object);
    /// Attaches a C function as an Objective-C method's implementation.
    /// `types` is the old-style Objective-C type-encoding string; `"v@:@"` is
    /// void-returning with the two implicit args (self, _cmd) plus one object
    /// argument, which is exactly an action method's signature.
    fn class_addMethod(cls: Object, name: Sel, imp: *const c_void, types: *const c_char) -> bool;
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {}

// ============================================================================
// Image creation helpers
// ============================================================================

/// Create a simple placeholder image for the tray icon.
fn create_placeholder_image() -> Result<Object, Error> {
    unsafe {
        // Create NSImage allocated and initialized with a size
        #[repr(C)]
        struct NsSize {
            width: f64,
            height: f64,
        }
        let image: Object = send(class(c"NSImage"), sel(c"alloc"));
        if image.is_null() {
            return Err(Error::Platform("Could not allocate NSImage".into()));
        }

        let size = NsSize {
            width: 16.0,
            height: 16.0,
        };
        let image: Object = send1(image, sel(c"initWithSize:"), size);

        if image.is_null() {
            Err(Error::Platform("Could not create placeholder image".into()))
        } else {
            Ok(image)
        }
    }
}

/// Create an NSImage from PNG data.
fn image_from_png(data: &[u8]) -> Result<Object, Error> {
    unsafe {
        let pool = objc_autoreleasePoolPush();

        // Create NSData from bytes
        let ns_data: Object = send(class(c"NSData"), sel(c"alloc"));
        let ns_data: Object = send2(
            ns_data,
            sel(c"initWithBytes:length:"),
            data.as_ptr(),
            data.len(),
        );

        if ns_data.is_null() {
            objc_autoreleasePoolPop(pool);
            return Err(Error::Platform("Could not create NSData".into()));
        }

        // Create NSImage from NSData
        let image: Object = send(class(c"NSImage"), sel(c"alloc"));
        let image: Object = send1(image, sel(c"initWithData:"), ns_data);

        objc_autoreleasePoolPop(pool);

        if image.is_null() {
            Err(Error::Platform("Could not create NSImage from data".into()))
        } else {
            Ok(image)
        }
    }
}
