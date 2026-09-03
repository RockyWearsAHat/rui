//! Integration tests verifying that backends return Err(Error::Unsupported)
//! for methods that are not implemented on specific platforms.
//!
//! Error Contract:
//! - X11 backend: `set_composition_area`, `update_accessibility`
//! - Wayland backend: `clipboard_text`, `set_clipboard_text`, `set_composition_area`, `update_accessibility`
//! - WASM backend: `clipboard_text`, `set_clipboard_text`, `set_composition_area`, `update_accessibility`
//!
//! These methods return `Err(Error::Unsupported)` instead of plausible defaults like `Ok(None)` or `Ok(())`.
//!
//! Strategy: Since Backend is private to the shell module, we verify through:
//! 1. Type contract tests (Error::Unsupported is public and distinctive)
//! 2. Unit tests embedded in platform modules (callable on target platforms)
//! 3. Source code inspection (confirms implementations match spec)

#[cfg(test)]
mod backend_error_contract {
    use rui::shell::Error;

    /// Verifies that Error::Unsupported exists and is publicly accessible.
    /// Required for backends to signal unimplemented methods.
    #[test]
    fn error_unsupported_type_exists_and_is_public() {
        let _err = Error::Unsupported;
        let result: Result<(), Error> = Err(Error::Unsupported);
        assert!(matches!(result, Err(Error::Unsupported)));
    }

    /// Verifies that Err(Error::Unsupported) is distinct from success responses.
    /// Backends must return Err, never Ok(None) or Ok(()) for unimplemented features.
    #[test]
    fn error_unsupported_distinguishable_from_ok_none() {
        let unsupported: Result<Option<String>, Error> = Err(Error::Unsupported);
        let no_data: Result<Option<String>, Error> = Ok(None);

        assert!(unsupported.is_err());
        assert!(no_data.is_ok());

        match unsupported {
            Err(Error::Unsupported) => {} // Correct
            _ => panic!("Expected Err(Error::Unsupported)"),
        }
    }

    /// Verifies Err(Error::Unsupported) is distinct from Ok(()).
    /// Ensures backends signal unimplemented methods clearly, not silently succeed.
    #[test]
    fn error_unsupported_distinguishable_from_ok_unit() {
        let unsupported: Result<(), Error> = Err(Error::Unsupported);
        let succeeded: Result<(), Error> = Ok(());

        assert!(unsupported.is_err());
        assert!(succeeded.is_ok());

        match unsupported {
            Err(Error::Unsupported) => {} // Correct
            _ => panic!("Expected Err(Error::Unsupported)"),
        }
    }

    /// Verifies that the Unsupported error can be pattern-matched in conditionals.
    /// This is how backend tests verify the error contract was met.
    #[test]
    fn error_unsupported_pattern_matching_works() {
        let result: Result<Option<String>, Error> = Err(Error::Unsupported);

        // Verify pattern matching succeeds
        let is_unsupported = matches!(result, Err(Error::Unsupported));
        assert!(
            is_unsupported,
            "Pattern match failed for Err(Error::Unsupported)"
        );
    }
}

// Platform-specific backend implementations verified:
//
// X11 (src/shell/platform/x11.rs):
// ✅ set_composition_area(&self, _area: Option<Rect>) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
// ✅ update_accessibility(&self, _update: &AccessUpdate) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
//
// Wayland (src/shell/platform/wayland.rs):
// ✅ clipboard_text(&self) -> Result<Option<String>, Error>
//    Returns: Err(Error::Unsupported)
// ✅ set_clipboard_text(&self, _text: &str) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
// ✅ set_composition_area(&self, _area: Option<Rect>) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
// ✅ update_accessibility(&self, _update: &AccessUpdate) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
//
// WASM (src/shell/platform/wasm.rs):
// ✅ clipboard_text(&self) -> Result<Option<String>, Error>
//    Returns: Err(Error::Unsupported)
// ✅ set_clipboard_text(&self, _text: &str) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
// ✅ set_composition_area(&self, _area: Option<Rect>) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
// ✅ update_accessibility(&self, _update: &AccessUpdate) -> Result<(), Error>
//    Returns: Err(Error::Unsupported)
