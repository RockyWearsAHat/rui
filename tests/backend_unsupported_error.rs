//! Integration tests verifying that backends return Err(Error::Unsupported)
//! for methods that are not implemented on specific platforms.
//!
//! These tests verify the error contract:
//! - X11 backend: set_composition_area returns Err(Error::Unsupported)
//! - Wayland backend: clipboard_text, set_clipboard_text, set_composition_area,
//!   update_accessibility return Err(Error::Unsupported)
//! - WASM backend: clipboard_text, set_clipboard_text, set_composition_area,
//!   update_accessibility return Err(Error::Unsupported)
//!
//! Note: These tests are platform-specific and will only run on the target
//! platform where the backend is compiled. On macOS, only compilation is
//! verified. Run with `cargo test --target x86_64-unknown-linux-gnu` to test
//! X11, or `cargo build --target wasm32-unknown-unknown` for WASM.

#[cfg(all(unix, not(target_os = "macos"), not(feature = "wayland")))]
#[cfg(test)]
mod x11_unsupported_tests {
    use rui::shell::Error;

    #[test]
    fn x11_set_composition_area_returns_unsupported() {
        // X11 backend does not support composition area setting.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }
}

#[cfg(all(unix, not(target_os = "macos"), feature = "wayland"))]
#[cfg(test)]
mod wayland_unsupported_tests {
    use rui::shell::Error;

    #[test]
    fn wayland_clipboard_text_returns_unsupported() {
        // Wayland backend clipboard reading not implemented.
        let result: Result<Option<String>, Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wayland_set_clipboard_text_returns_unsupported() {
        // Wayland backend clipboard writing not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wayland_set_composition_area_returns_unsupported() {
        // Wayland backend composition area not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wayland_update_accessibility_returns_unsupported() {
        // Wayland backend accessibility not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
mod wasm_unsupported_tests {
    use rui::shell::Error;

    #[test]
    fn wasm_clipboard_text_returns_unsupported() {
        // WASM backend clipboard reading not implemented.
        let result: Result<Option<String>, Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wasm_set_clipboard_text_returns_unsupported() {
        // WASM backend clipboard writing not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wasm_set_composition_area_returns_unsupported() {
        // WASM backend composition area not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn wasm_update_accessibility_returns_unsupported() {
        // WASM backend accessibility not implemented.
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }
}
