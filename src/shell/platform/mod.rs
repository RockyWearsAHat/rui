//! The per-platform backends, and the choice between them.
//!
//! Exactly one is compiled, and each implements the same
//! [`Backend`](crate::shell::Backend) — whatever that trait currently asks for,
//! which is the point of naming it in one place. `unsafe` is confined to these
//! files: the run loop above them and the toolkit beneath them contain none.
//!
//! # Where a platform cannot do what the seam asks
//!
//! It says so, in its own module header, and returns the honest empty answer —
//! not a plausible-looking one. The X11 backend has no input method and says
//! that in place of pretending to compose; `unsupported.rs` does the same for
//! every method at once. A gap someone can read is a gap someone can close; a
//! silent one is a bug report from a person who thought their keyboard was
//! broken.

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
#[allow(unsafe_code, reason = "AppKit and Core Graphics are C and Objective-C")]
mod backend;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
#[allow(unsafe_code, reason = "the Win32 window and bitmap calls are C")]
mod backend;

#[cfg(all(unix, not(target_os = "macos"), not(feature = "wayland")))]
#[path = "x11.rs"]
#[allow(unsafe_code, reason = "Xlib is C")]
mod backend;

#[cfg(all(unix, not(target_os = "macos"), feature = "wayland"))]
#[path = "wayland.rs"]
#[allow(unsafe_code, reason = "Wayland protocol and platform calls")]
mod backend;

// Matched before the fallback arm below: wasm32 is `not(unix)` and would
// otherwise land in `unsupported.rs`, which is exactly the backend Forge's
// web UI was silently getting.
#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
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
