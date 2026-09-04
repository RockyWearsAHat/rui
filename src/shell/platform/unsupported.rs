//! Stub for unsupported platforms.
//!
//! rui supports:
//! - **macOS** (Cocoa backend)
//! - **Windows** (WinAPI backend)
//! - **Linux** (X11 and optionally Wayland via --features wayland)
//! - **WebAssembly** (Canvas backend for browsers)
//!
//! This stub is selected when compiling for an unsupported platform,
//! causing a clear compile error rather than a linker error later.

#[allow(non_camel_case_types)]
pub struct Window;

impl Window {
    #[allow(unconditional_panic)]
    pub fn open(_: &crate::shell::WindowOptions) -> Result<Self, crate::shell::Error> {
        panic!("rui: unsupported platform.\n\
                Supported platforms: macOS, Windows, Linux (X11/Wayland), and WebAssembly.\n\
                If targeting Linux, ensure you are cross-compiling with the correct target triple.\n\
                For Wayland support on Linux, build with: cargo build --features wayland");
    }
}
