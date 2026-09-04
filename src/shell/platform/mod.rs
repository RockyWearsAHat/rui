//! The per-platform backends, and the choice between them.
//!
//! Each platform backend implements the [`Backend`](crate::Backend) trait.
//! `unsafe` is confined to these files; the run loop and toolkit contain none.
//!
//! # Platform Selection
//!
//! On Linux, Wayland and X11 backends are both compiled, and [`Window::open`]
//! implements runtime auto-detection:
//! 1. Attempts to open a Wayland connection first (modern display server)
//! 2. Falls back to X11 if Wayland is unavailable (XWayland compatibility)
//!
//! This gives the best of both: native Wayland when available, X11 fallback
//! for compatibility with systems lacking Wayland support.

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
use crate::shell::{Backend, Error, WindowOptions};
#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
use std::time::Duration;

// Platform-specific backend implementations (non-Linux platforms)
#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
#[allow(unsafe_code, reason = "FFI and DOM interop are inherently unsafe")]
mod backend;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
#[allow(unsafe_code, reason = "AppKit and Core Graphics are C and Objective-C")]
mod backend;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
#[allow(unsafe_code, reason = "the Win32 window and bitmap calls are C")]
mod backend;

// Wayland backend (with feature gate for Linux)
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol and system calls are C")]
mod backend;

// X11 backend (Linux without Wayland feature, or as fallback when Wayland feature is enabled)
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    not(feature = "wayland")
))]
#[path = "x11.rs"]
#[allow(unsafe_code, reason = "Xlib is C")]
mod backend;

// Unsupported platform fallback
#[cfg(not(any(
    target_arch = "wasm32",
    target_os = "macos",
    target_os = "windows",
    all(unix, not(target_os = "macos"))
)))]
#[path = "unsupported.rs"]
mod backend;

// Internal modules for Wayland/X11 when both are compiled (for runtime auto-detection)
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol and system calls are C")]
mod wayland_impl;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
#[path = "x11.rs"]
#[allow(unsafe_code, reason = "Xlib is C")]
mod x11_impl;

// On Linux with Wayland feature enabled, use an enum for runtime auto-detection
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
pub enum Window {
    Wayland(wayland_impl::Window),
    X11(x11_impl::Window),
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
))]
#[allow(
    unsafe_code,
    reason = "Adapter callback uses pointer to work around borrow checker; safe because pointer derived from mutable reference"
)]
impl Backend for Window {
    fn open(options: &WindowOptions) -> Result<Self, Error> {
        // Try Wayland first
        match wayland_impl::Window::open(options) {
            Ok(wayland_window) => return Ok(Window::Wayland(wayland_window)),
            Err(_wayland_err) => {
                // Wayland failed, try X11
                match x11_impl::Window::open(options) {
                    Ok(x11_window) => Ok(Window::X11(x11_window)),
                    Err(x11_err) => Err(x11_err),
                }
            }
        }
    }

    fn pump(
        &mut self,
        timeout: Duration,
        events: &mut Vec<crate::shell::Event>,
        redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Create a wrapper that can reconstruct the enum reference from backend calls
        struct RedrawWrapper<'a> {
            window: *mut Window,
            inner_redraw: &'a mut dyn FnMut(&Window),
        }

        impl<'a> RedrawWrapper<'a> {
            fn call(&mut self) {
                // SAFETY: The pointer is valid for the lifetime of this wrapper because we hold
                // a mutable reference to the Window from which it was derived.
                (self.inner_redraw)(unsafe { &*self.window });
            }
        }

        let mut wrapper = RedrawWrapper {
            window: self as *mut _,
            inner_redraw: redraw,
        };

        match self {
            Window::Wayland(w) => w.pump(timeout, events, &mut |_| wrapper.call()),
            Window::X11(w) => w.pump(timeout, events, &mut |_| wrapper.call()),
        }
    }

    fn surface(&self) -> (u32, u32, f32) {
        match self {
            Window::Wayland(w) => w.surface(),
            Window::X11(w) => w.surface(),
        }
    }

    fn appearance(&self) -> crate::theme::Appearance {
        match self {
            Window::Wayland(w) => w.appearance(),
            Window::X11(w) => w.appearance(),
        }
    }

    fn present(&self, canvas: &crate::canvas::Canvas) -> Result<(), Error> {
        match self {
            Window::Wayland(w) => w.present(canvas),
            Window::X11(w) => w.present(canvas),
        }
    }

    fn is_open(&self) -> bool {
        match self {
            Window::Wayland(w) => w.is_open(),
            Window::X11(w) => w.is_open(),
        }
    }
}

// On non-Linux platforms and Linux without Wayland feature, use the backend module directly
#[cfg(not(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    feature = "wayland"
)))]
pub(crate) use backend::Window;

// Public re-exports for testing platform-specific backends
pub mod wayland {
    //! Wayland backend for testing.
    #[cfg(all(
        unix,
        not(target_os = "macos"),
        not(target_arch = "wasm32"),
        feature = "wayland"
    ))]
    pub use super::wayland_impl::Window;

    #[cfg(not(all(
        unix,
        not(target_os = "macos"),
        not(target_arch = "wasm32"),
        feature = "wayland"
    )))]
    /// Stub window type (Wayland feature not enabled or non-Linux platform).
    pub struct Window;
}

pub mod x11 {
    //! X11 backend for testing.
    #[cfg(all(
        unix,
        not(target_os = "macos"),
        not(target_arch = "wasm32"),
        not(feature = "wayland")
    ))]
    pub use super::backend::Window;

    #[cfg(all(
        unix,
        not(target_os = "macos"),
        not(target_arch = "wasm32"),
        feature = "wayland"
    ))]
    pub use super::x11_impl::Window;

    #[cfg(not(any(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")),)))]
    /// Stub window type (non-Linux platform).
    pub struct Window;
}
