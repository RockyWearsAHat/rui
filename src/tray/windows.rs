//! Windows system tray implementation using Shell_NotifyIcon and TrackPopupMenu.
//!
//! # Design
//!
//! The Windows tray uses `Shell_NotifyIconW()` to register an icon in the system tray.
//! The icon is associated with a message-only window (HWND_MESSAGE) that receives
//! WM_APP notifications when the tray icon is clicked.
//!
//! ## Message-Only Window
//!
//! A message-only window (created with parent `HWND_MESSAGE = -3`) is invisible and
//! only exists to receive messages. It cannot be seen or interacted with directly;
//! it is purely a target for tray notifications. This avoids the overhead of a
//! visible window.
//!
//! ## Event Flow
//!
//! 1. Right-click on tray icon → WM_APP + WM_RBUTTONUP
//! 2. Window proc calls TrackPopupMenu(), which shows the context menu
//! 3. User clicks menu item → menu command is posted to event queue
//! 4. Left-click on tray icon → WM_APP + other button code → IconActivated posted
//!
//! ## Menu Item IDs
//!
//! Menu items are created with their `id` as the menu command ID. When
//! TrackPopupMenu returns, the returned command ID is posted as
//! MenuItemClicked. This avoids any separate mapping.
//!
//! ## Icon Data
//!
//! TODO: Currently uses the default application icon. Full PNG→HICON conversion
//! would require external image processing; this is left for a future enhancement.
//! For now, set_icon() uses the default icon to avoid complexity.

#![allow(unsafe_code, dead_code, unused_variables)]

use crate::Error;
use std::cell::RefCell;
use std::ffi::{c_void, OsStr};
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::{Arc, Mutex};

use super::{TrayEvent, TrayMenuItem};

/// Windows tray implementation using Shell_NotifyIcon.
pub struct TrayInner {
    hwnd: *mut c_void,
    icon_id: u32,
    icon_handle: RefCell<*mut c_void>, // HICON, can be updated
    menu_items: RefCell<Vec<TrayMenuItem>>,
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
}

// SAFETY: The window handle and icon handle are managed safely in Drop.
unsafe impl Send for TrayInner {}
unsafe impl Sync for TrayInner {}

type Handle = *mut c_void;
type WordParameter = usize;
type LongParameter = isize;

// ============================================================================
// Win32 FFI Declarations
// ============================================================================

#[allow(clashing_extern_declarations)]
#[link(name = "user32")]
unsafe extern "system" {
    fn CreateWindowExW(
        extended_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        parameter: *mut c_void,
    ) -> Handle;

    fn DefWindowProcW(
        window: Handle,
        message: u32,
        word: WordParameter,
        long: LongParameter,
    ) -> LongParameter;

    fn DestroyWindow(window: Handle) -> i32;
    fn RegisterClassExW(class: *const WindowClass) -> u16;
    fn PeekMessageW(
        message: *mut Message,
        window: Handle,
        first: u32,
        last: u32,
        remove: u32,
    ) -> i32;
    fn DispatchMessageW(message: *const Message) -> LongParameter;
    fn LoadIconW(instance: Handle, name: *const u16) -> Handle;
    fn CreatePopupMenu() -> Handle;
    fn AppendMenuW(menu: Handle, flags: u32, id: usize, text: *const u16) -> i32;
    fn DestroyMenu(menu: Handle) -> i32;
    fn TrackPopupMenu(
        menu: Handle,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        window: Handle,
        rect: *const WindowRect,
    ) -> i32;
    fn PostMessageW(window: Handle, message: u32, word: WordParameter, long: LongParameter) -> i32;
    fn GetCursorPos(point: *mut Point) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(action: u32, data: *const NotifyIconData) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
}

// ============================================================================
// Win32 Structures
// ============================================================================

#[repr(C)]
struct WindowClass {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc:
        unsafe extern "system" fn(Handle, u32, WordParameter, LongParameter) -> LongParameter,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: Handle,
    h_icon: Handle,
    h_cursor: Handle,
    hbr_background: Handle,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: Handle,
}

#[repr(C)]
struct Message {
    hwnd: Handle,
    message: u32,
    word: WordParameter,
    long: LongParameter,
    time: u32,
    pt: Point,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct WindowRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct NotifyIconData {
    cb_size: u32,
    hwnd: Handle,
    uid: u32,
    u_flags: u32,
    u_callback_message: u32,
    h_icon: Handle,
    sz_tip: [u16; 128],
    dw_state: u32,
    dw_state_mask: u32,
    sz_info: [u16; 256],
    u_timeout: u32,
    sz_info_title: [u16; 64],
    dw_info_flags: u32,
    // Extended fields (Windows Vista+) - version 6
    guid: [u8; 16],
    h_balloon_icon: Handle,
}

// ============================================================================
// Win32 Constants
// ============================================================================

const WM_APP: u32 = 0x8000;
const WM_TRAY_ICON: u32 = WM_APP + 1;
const WM_TRAY_MENU: u32 = WM_APP + 2;

const NIM_ADD: u32 = 0;
const NIM_MODIFY: u32 = 1;
const NIM_DELETE: u32 = 2;

const NIF_MESSAGE: u32 = 1;
const NIF_ICON: u32 = 2;
const NIF_TIP: u32 = 4;
const NIF_STATE: u32 = 8;

const NIS_HIDDEN: u32 = 1;
const NIS_SHAREDICON: u32 = 2;

const WM_RBUTTONUP: u32 = 0x0205;
const WM_CONTEXTMENU: u32 = 0x007B;

const MFT_STRING: u32 = 0;
const MFT_SEPARATOR: u32 = 0x800;
const MFS_ENABLED: u32 = 0;
const MFS_DISABLED: u32 = 1;
const MFS_CHECKED: u32 = 8;

const TPM_LEFTALIGN: u32 = 0;
const TPM_RIGHTBUTTON: u32 = 2;
const TPM_RETURNCMD: u32 = 256;

// ============================================================================
// Global State for Message Window
// ============================================================================

thread_local! {
    static TRAY_INNER_PTR: RefCell<Option<*mut TrayInner>> = RefCell::new(None);
}

// ============================================================================
// Window Procedure
// ============================================================================

unsafe extern "system" fn tray_window_proc(
    hwnd: Handle,
    message: u32,
    word: WordParameter,
    long: LongParameter,
) -> LongParameter {
    if message == WM_TRAY_ICON {
        TRAY_INNER_PTR.with(|ptr| {
            if let Some(inner_ptr) = *ptr.borrow() {
                let inner = &*inner_ptr;
                if word == WM_RBUTTONUP as usize {
                    // Right-click: show context menu
                    let mut cursor_pos = Point { x: 0, y: 0 };
                    if GetCursorPos(&mut cursor_pos) == 0 {
                        return;
                    }

                    let items = inner.menu_items.borrow();
                    let menu = CreatePopupMenu();
                    if menu.is_null() {
                        return;
                    }

                    for item in items.iter() {
                        let mut label_wide: Vec<u16> = OsStr::new(&item.label)
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();

                        let mut flags = MFT_STRING;
                        if !item.enabled {
                            flags |= MFS_DISABLED;
                        }
                        if item.selected {
                            flags |= MFS_CHECKED;
                        }

                        AppendMenuW(menu, flags, item.id, label_wide.as_mut_ptr());
                    }

                    // Track and execute popup menu
                    let cmd = TrackPopupMenu(
                        menu,
                        TPM_LEFTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
                        cursor_pos.x,
                        cursor_pos.y,
                        0,
                        hwnd,
                        ptr::null(),
                    );

                    if cmd > 0 {
                        if let Ok(mut queue) = inner.event_queue.lock() {
                            queue.push(TrayEvent::MenuItemClicked(cmd as usize));
                        }
                    }

                    DestroyMenu(menu);
                } else {
                    // Left-click or other activation
                    if let Ok(mut queue) = inner.event_queue.lock() {
                        queue.push(TrayEvent::IconActivated);
                    }
                }
            }
        });
        return 0;
    }

    DefWindowProcW(hwnd, message, word, long)
}

// ============================================================================
// Implementation
// ============================================================================

impl TrayInner {
    /// Create a new tray icon on Windows.
    pub fn new(
        icon_data: &[u8],
        tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    ) -> Result<Self, Error> {
        unsafe {
            // Register the window class
            let class_name = wide_string("RuiTrayWindow");
            let wnd_class = WindowClass {
                cb_size: mem::size_of::<WindowClass>() as u32,
                style: 0,
                lpfn_wnd_proc: tray_window_proc,
                cb_cls_extra: 0,
                cb_wnd_extra: 0,
                h_instance: GetModuleHandleW(ptr::null()),
                h_icon: ptr::null_mut(),
                h_cursor: ptr::null_mut(),
                hbr_background: ptr::null_mut(),
                lpsz_menu_name: ptr::null(),
                lpsz_class_name: class_name.as_ptr(),
                h_icon_sm: ptr::null_mut(),
            };

            let class_atom = RegisterClassExW(&wnd_class);
            if class_atom == 0 {
                return Err(Error::Platform(
                    "Failed to register tray window class".into(),
                ));
            }

            // Create message-only window
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                (-3i32) as *mut c_void, // HWND_MESSAGE = -3
                ptr::null_mut(),
                GetModuleHandleW(ptr::null()),
                ptr::null_mut(),
            );

            if hwnd.is_null() {
                return Err(Error::Platform("Failed to create tray window".into()));
            }

            // Load and convert icon
            let icon_handle = if icon_data.is_empty() {
                // Use default application icon
                LoadIconW(ptr::null_mut(), (-1i32) as *const u16) // IDI_APPLICATION
            } else {
                // TODO: For now, use default. Full implementation would use CreateIconFromResourceEx
                // or convert PNG to BMP and use CreateDIBIcon, but that is complex.
                LoadIconW(ptr::null_mut(), (-1i32) as *const u16)
            };

            if icon_handle.is_null() {
                DestroyWindow(hwnd);
                return Err(Error::Platform("Failed to load icon".into()));
            }

            let tray = TrayInner {
                hwnd,
                icon_id: 1,
                icon_handle: RefCell::new(icon_handle),
                menu_items: RefCell::new(Vec::new()),
                event_queue,
            };

            // Register the tray icon
            let mut tooltip_wide = wide_string(tooltip);
            tooltip_wide.truncate(127); // Limit to 127 chars (leave room for null terminator)
            let mut sz_tip = [0u16; 128];
            for (i, &ch) in tooltip_wide.iter().enumerate() {
                if i >= 128 {
                    break;
                }
                sz_tip[i] = ch;
            }

            let notify_data = NotifyIconData {
                cb_size: mem::size_of::<NotifyIconData>() as u32,
                hwnd,
                uid: tray.icon_id,
                u_flags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                u_callback_message: WM_TRAY_ICON,
                h_icon: icon_handle,
                sz_tip,
                dw_state: 0,
                dw_state_mask: 0,
                sz_info: [0; 256],
                u_timeout: 0,
                sz_info_title: [0; 64],
                dw_info_flags: 0,
                guid: [0; 16],
                h_balloon_icon: ptr::null_mut(),
            };

            if Shell_NotifyIconW(NIM_ADD, &notify_data) == 0 {
                DestroyWindow(hwnd);
                return Err(Error::Platform("Failed to add tray icon".into()));
            }

            // Store pointer for message window access
            TRAY_INNER_PTR.with(|ptr| {
                *ptr.borrow_mut() = Some(&tray as *const _ as *mut _);
            });

            Ok(tray)
        }
    }

    /// Update the menu items.
    pub fn set_menu(&self, items: Vec<TrayMenuItem>) -> Result<(), Error> {
        *self.menu_items.borrow_mut() = items;
        Ok(())
    }

    /// Change the tooltip text.
    pub fn set_tooltip(&self, text: &str) -> Result<(), Error> {
        unsafe {
            let mut tooltip_wide = wide_string(text);
            tooltip_wide.truncate(127);
            let mut sz_tip = [0u16; 128];
            for (i, &ch) in tooltip_wide.iter().enumerate() {
                if i >= 128 {
                    break;
                }
                sz_tip[i] = ch;
            }

            let notify_data = NotifyIconData {
                cb_size: mem::size_of::<NotifyIconData>() as u32,
                hwnd: self.hwnd,
                uid: self.icon_id,
                u_flags: NIF_TIP,
                u_callback_message: WM_TRAY_ICON,
                h_icon: ptr::null_mut(),
                sz_tip,
                dw_state: 0,
                dw_state_mask: 0,
                sz_info: [0; 256],
                u_timeout: 0,
                sz_info_title: [0; 64],
                dw_info_flags: 0,
                guid: [0; 16],
                h_balloon_icon: ptr::null_mut(),
            };

            if Shell_NotifyIconW(NIM_MODIFY, &notify_data) == 0 {
                return Err(Error::Platform("Failed to update tooltip".into()));
            }

            Ok(())
        }
    }

    /// Change the icon image.
    ///
    /// # Note: Limitation on Windows
    /// This method currently ignores the provided PNG data and always uses the default
    /// application icon. Converting PNG data to Windows HICON format requires either
    /// external crates or complex Win32 API calls involving GDI+ or Direct2D. This is
    /// a known limitation; to work around it, use platform-specific APIs directly or
    /// consider using ICO/BMP formats instead.
    pub fn set_icon(&self, _icon_data: &[u8]) -> Result<(), Error> {
        unsafe {
            // NOTE: Full PNG->HICON conversion is non-trivial and would require either:
            // 1. External image processing crates (adds dependency)
            // 2. Complex GDI+ or Direct2D calls (large code, potential performance impact)
            // 3. Saving to temporary file and loading (I/O overhead)
            // For now, we use the default application icon as a fallback.
            // This is acceptable for system tray status indicators that use simple icons.
            let icon_handle = LoadIconW(ptr::null_mut(), (-1i32) as *const u16); // IDI_APPLICATION

            if icon_handle.is_null() {
                return Err(Error::Platform("Failed to load icon".into()));
            }

            let mut old_icon = self.icon_handle.borrow_mut();
            *old_icon = icon_handle;

            // Update the tray icon display
            let notify_data = NotifyIconData {
                cb_size: mem::size_of::<NotifyIconData>() as u32,
                hwnd: self.hwnd,
                uid: self.icon_id,
                u_flags: NIF_ICON,
                u_callback_message: WM_TRAY_ICON,
                h_icon: icon_handle,
                sz_tip: [0; 128],
                dw_state: 0,
                dw_state_mask: 0,
                sz_info: [0; 256],
                u_timeout: 0,
                sz_info_title: [0; 64],
                dw_info_flags: 0,
                guid: [0; 16],
                h_balloon_icon: ptr::null_mut(),
            };

            if Shell_NotifyIconW(NIM_MODIFY, &notify_data) == 0 {
                return Err(Error::Platform("Failed to update icon".into()));
            }

            Ok(())
        }
    }
}

impl Drop for TrayInner {
    fn drop(&mut self) {
        unsafe {
            // Remove the tray icon
            let notify_data = NotifyIconData {
                cb_size: mem::size_of::<NotifyIconData>() as u32,
                hwnd: self.hwnd,
                uid: self.icon_id,
                u_flags: 0,
                u_callback_message: 0,
                h_icon: ptr::null_mut(),
                sz_tip: [0; 128],
                dw_state: 0,
                dw_state_mask: 0,
                sz_info: [0; 256],
                u_timeout: 0,
                sz_info_title: [0; 64],
                dw_info_flags: 0,
                guid: [0; 16],
                h_balloon_icon: ptr::null_mut(),
            };

            let _ = Shell_NotifyIconW(NIM_DELETE, &notify_data);

            // Destroy the window
            if !self.hwnd.is_null() {
                let _ = DestroyWindow(self.hwnd);
            }

            // Clear the global pointer
            TRAY_INNER_PTR.with(|ptr| {
                *ptr.borrow_mut() = None;
            });
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
