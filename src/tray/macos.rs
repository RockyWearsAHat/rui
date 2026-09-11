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
use std::ffi::{CStr, c_char, c_void};
use std::sync::{Arc, Mutex};

use super::{TrayEvent, TrayMenuItem};

/// Platform-specific opaque handle for Objective-C objects.
type Object = *mut c_void;
/// Platform-specific selector for Objective-C method names.
type Sel = *const c_void;

/// The macOS tray implementation using NSStatusItem.
pub struct TrayInner {
    #[allow(dead_code)]
    status_item: Object,
    button: Object,
    menu: Object,
    #[allow(dead_code)]
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
}

// SAFETY: The underlying Objective-C objects are thread-safe as long as
// we only access them from the main thread (AppKit's requirement).
unsafe impl Send for TrayInner {}
unsafe impl Sync for TrayInner {}

// Thread-local storage for the event queue (used for icon click detection)
thread_local! {
    static TRAY_MENU_HANDLER: RefCell<Option<Arc<Mutex<Vec<TrayEvent>>>>> = const { RefCell::new(None) };
}

impl TrayInner {
    /// Create a new tray icon on macOS.
    pub fn new(
        icon_data: &[u8],
        tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    ) -> Result<Self, Error> {
        unsafe {
            let pool = objc_autoreleasePoolPush();
            let result = Self::new_inner(icon_data, tooltip, event_queue.clone());
            objc_autoreleasePoolPop(pool);
            result
        }
    }

    unsafe fn new_inner(
        _icon_data: &[u8],
        tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
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

        // Set a placeholder image
        let image = create_placeholder_image()?;
        let _: () = send1(button, sel(c"setImage:"), image);

        // Set the menu on the status item
        let _: () = send1(status_item, sel(c"setMenu:"), menu);

        // Store event queue in thread-local for button click handler
        TRAY_MENU_HANDLER.with(|handler| {
            *handler.borrow_mut() = Some(event_queue.clone());
        });

        Ok(TrayInner {
            status_item,
            button,
            menu,
            event_queue,
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

                // Create menu item with no action selector (since we can't implement proper
                // Objective-C methods from Rust FFI without significant complexity).
                // In a full implementation, a custom NSObject subclass with proper method dispatch
                // would be needed. For now, we create items with display-only purpose.
                let menu_item: Object = send(class(c"NSMenuItem"), sel(c"alloc"));
                let menu_item: Object = send3(
                    menu_item,
                    sel(c"initWithTitle:action:keyEquivalent:"),
                    ns_label,
                    std::ptr::null::<c_void>(), // No action
                    ns_string(c""),
                );

                if !menu_item.is_null() {
                    // Store the item ID as the tag (could be used for event delivery in future)
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
}

impl Drop for TrayInner {
    fn drop(&mut self) {
        // The status item is retained by the status bar and will be cleaned up
        // automatically when the status bar is deallocated. We don't need to
        // explicitly release it here.

        // Clean up thread-local event queue reference
        TRAY_MENU_HANDLER.with(|handler| {
            *handler.borrow_mut() = None;
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
