//! The per-platform backends, and the choice between them.
//!
//! Exactly one is compiled, and each implements the same four-method
//! [`Backend`](crate::Backend). `unsafe` is confined to these files: the run
//! loop above them and the toolkit beneath them contain none.

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod backend;

#[cfg(target_os = "ios")]
#[path = "ios.rs"]
#[allow(unsafe_code, reason = "UIKit and Metal are Objective-C and C")]
mod backend;

#[cfg(target_os = "android")]
#[path = "android.rs"]
#[allow(unsafe_code, reason = "Android NDK is C and Java")]
mod backend;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
#[allow(unsafe_code, reason = "AppKit and Core Graphics are C and Objective-C")]
mod backend;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
#[allow(unsafe_code, reason = "the Win32 window and bitmap calls are C")]
mod backend;

#[cfg(all(target_os = "linux", not(target_arch = "wasm32"), feature = "wayland"))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol is C")]
mod backend;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    not(feature = "wayland")
))]
#[path = "x11.rs"]
#[allow(unsafe_code, reason = "Xlib is C")]
mod backend;

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "ios",
    target_os = "android",
    unix,
    target_arch = "wasm32"
)))]
#[path = "unsupported.rs"]
mod backend;

pub(crate) use backend::Window;
