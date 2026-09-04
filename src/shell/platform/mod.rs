//! The per-platform backends, and the choice between them.
//!
//! Exactly one is compiled, and each implements the same four-method
//! [`Backend`](crate::Backend). `unsafe` is confined to these files: the run
//! loop above them and the toolkit beneath them contain none.
//!
//! # Platform Selection
//!
//! The `wayland` feature flag selects between Wayland and X11 on Linux:
//! - `cargo build --features wayland`: Uses Wayland backend (modern display server)
//! - `cargo build`: Uses X11 backend (default; compatible with both X11 and XWayland)
//!
//! Auto-detection: X11 is the default for Linux because it runs on both X11 and
//! Wayland systems (via XWayland), providing broader compatibility. Enable the
//! `wayland` feature to use the native Wayland protocol on systems that support it.

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod backend;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
#[allow(unsafe_code, reason = "AppKit and Core Graphics are C and Objective-C")]
mod backend;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
#[allow(unsafe_code, reason = "the Win32 window and bitmap calls are C")]
mod backend;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol and system calls are C")]
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
    unix,
    target_arch = "wasm32"
)))]
#[path = "unsupported.rs"]
mod backend;

pub(crate) use backend::Window;
