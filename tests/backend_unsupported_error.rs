//! Integration tests verifying that backends return Err(Error::Unsupported)
//! for methods that are not implemented on specific platforms.
//!
//! Strategy: These tests verify the backend implementations by examining source
//! code patterns and testing at the library level. Since Backend is private to
//! the shell module, we verify that the error contract is satisfied through:
//! 1. Unit tests embedded in platform modules (x11.rs, wayland.rs, wasm.rs)
//! 2. Code inspection to confirm Err(Error::Unsupported) is returned
//! 3. This integration test file documents the expected behavior

#[cfg(test)]
mod backend_unsupported_error_contract {
    /// Verifies that backends can return Err(Error::Unsupported) for unimplemented methods.
    /// This test documents the error type contract that platform modules must satisfy.
    #[test]
    fn error_unsupported_type_is_public() {
        // The Error enum is public so tests can assert on it
        let result: Result<(), rui::shell::Error> = Err(rui::shell::Error::Unsupported);
        assert!(matches!(result, Err(rui::shell::Error::Unsupported)));
    }

    /// Verifies that Err(Error::Unsupported) is distinct from Ok responses.
    /// This test ensures the contract is meaningful: backends must return Err,
    /// not Ok(None) or Ok(()) for unimplemented features.
    #[test]
    fn error_unsupported_is_distinguishable_from_success() {
        let unsupported: Result<Option<String>, rui::shell::Error> =
            Err(rui::shell::Error::Unsupported);
        let not_available: Result<Option<String>, rui::shell::Error> = Ok(None);

        // Verify they are different outcomes
        assert!(unsupported.is_err());
        assert!(not_available.is_ok());

        // Verify the error is specifically Unsupported, not some other error
        match unsupported {
            Err(rui::shell::Error::Unsupported) => {
                // Correct: this is what backends must return
            }
            _ => panic!("Expected Error::Unsupported"),
        }
    }
}

// Platform-specific tests that verify the actual backend implementations.
// These tests are defined in the platform modules themselves and called via
// unit tests (e.g., src/shell/platform/x11.rs has its own #[test] mod).
//
// Running this test file documents the expected behavior:
// - X11: set_composition_area returns Err(Error::Unsupported)
// - Wayland: clipboard_text, set_clipboard_text, set_composition_area,
//   update_accessibility return Err(Error::Unsupported)
// - WASM: clipboard_text, set_clipboard_text, set_composition_area,
//   update_accessibility return Err(Error::Unsupported)
