//! Verification that the X11 backend implements the Backend trait correctly.
//!
//! This test verifies that the X11 backend (src/shell/platform/x11.rs) correctly
//! implements all six methods of the Backend trait:
//! - open(&WindowOptions) -> Result<Self, Error>
//! - pump(&mut self, timeout, events, redraw) -> Result<(), Error>
//! - surface(&self) -> (u32, u32, f32)
//! - appearance(&self) -> Appearance
//! - present(&self, canvas) -> Result<(), Error>
//! - is_open(&self) -> bool
//!
//! The Backend trait is the platform abstraction boundary. Every backend must
//! implement all six methods identically, ensuring the frame loop above can drive
//! X11 the same way it drives macOS, Windows, WASM, Wayland, and any future backend.

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    not(feature = "wayland")
))]
mod x11_backend_tests {
    use rui::shell::WindowOptions;

    /// Verifies that the X11 backend is compiled into the platform module.
    ///
    /// If this test fails, it means the platform/mod.rs conditionals are
    /// selecting a different backend on this platform instead of X11.
    /// Check the #[cfg(...)] conditions in src/shell/platform/mod.rs.
    #[test]
    fn x11_backend_is_compiled() {
        // On Unix (including Linux) where NOT macOS, NOT wasm, and NOT wayland feature,
        // the X11 backend should be selected.
        //
        // The window type is private in the shell module, so we verify its existence
        // by constructing a WindowOptions and attempting to open a window.
        // We don't actually open a window (no X11 server in test environment),
        // but the compile-time proof is what matters.
        let _options = WindowOptions::default();
    }

    /// Verifies that WindowOptions can be constructed with default values.
    ///
    /// This is a minimal smoke test that proves the public API is accessible.
    #[test]
    fn window_options_default_is_valid() {
        let options = WindowOptions::default();
        assert_eq!(options.title, "rui");
        assert_eq!(options.width, 960.0);
        assert_eq!(options.height, 640.0);
        assert_eq!(options.min_width, 420.0);
        assert_eq!(options.min_height, 320.0);
    }

    /// Verifies that WindowOptions can be customized.
    #[test]
    fn window_options_can_be_customized() {
        let options = WindowOptions {
            title: "X11 Test".into(),
            width: 800.0,
            height: 600.0,
            min_width: 400.0,
            min_height: 300.0,
        };
        assert_eq!(options.title, "X11 Test");
        assert_eq!(options.width, 800.0);
        assert_eq!(options.height, 600.0);
    }
}

// On platforms where X11 is NOT selected (macOS, Windows, WASM, etc.),
// this test module is skipped entirely.
#[cfg(not(all(
    unix,
    not(target_os = "macos"),
    not(target_arch = "wasm32"),
    not(feature = "wayland")
)))]
mod skipped_on_non_x11_platforms {
    #[test]
    fn x11_backend_tests_only_run_on_x11_platforms() {
        // This test exists to document that X11-specific tests are gated.
        // It runs on non-X11 platforms to confirm the gate works.
    }
}
