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

// Platform-specific backend implementations (Linux platforms)
#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol and system calls are C")]
mod wayland_backend;

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
#[path = "x11.rs"]
#[allow(unsafe_code, reason = "Xlib is C")]
mod x11_backend;

// On Linux, use an enum for runtime auto-detection between Wayland and X11
#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
pub enum Window {
    Wayland(wayland_backend::Window),
    X11(x11_backend::Window),
}

#[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
impl Backend for Window {
    fn open(options: &WindowOptions) -> Result<Self, Error> {
        // Try Wayland first
        match wayland_backend::Window::open(options) {
            Ok(wayland_window) => return Ok(Window::Wayland(wayland_window)),
            Err(_wayland_err) => {
                // Wayland failed, try X11
                match x11_backend::Window::open(options) {
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
        match self {
            Window::Wayland(w) => w.pump(timeout, events, &mut |_| redraw(self)),
            Window::X11(w) => w.pump(timeout, events, &mut |_| redraw(self)),
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

// On other platforms, re-export from the platform-specific backend module
#[cfg(not(all(unix, not(target_os = "macos"), not(target_arch = "wasm32"))))]
pub(crate) use backend::Window;

// Public re-exports for testing platform-specific backends
pub mod wayland {
    //! Wayland backend for testing.
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    pub use super::wayland_backend::Window;

    #[cfg(not(all(unix, not(target_os = "macos"), not(target_arch = "wasm32"))))]
    /// Stub window type for non-Linux platforms.
    pub struct Window;
}

pub mod x11 {
    //! X11 backend for testing.
    #[cfg(all(unix, not(target_os = "macos"), not(target_arch = "wasm32")))]
    pub use super::x11_backend::Window;

    #[cfg(not(all(unix, not(target_os = "macos"), not(target_arch = "wasm32"))))]
    /// Stub window type for non-Linux platforms.
    pub struct Window;
}
