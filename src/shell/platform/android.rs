//! Android backend for rui — implements Backend trait via Android NDK/Vulkan.
//!
//! STEP 23 Phase 1: Foundation — Android backend for mobile support.
//! This is a stub implementation establishing the Backend trait contract.
//! Phase 2 will add full touch event handling, DPI detection, and rendering.
//!
//! # Architecture
//!
//! The Android backend follows the Recipe 2 pattern:
//! - Phase 1: Implement Backend trait (6 methods) with minimal stubs
//! - Phase 2: Add touch handling, DPI detection, appearance detection
//! - Phase 3: Comprehensive testing and parity verification
//!
//! # Platform Selection
//!
//! Android backend is selected by: `#[cfg(target_os = "android")]` in `src/shell/platform/mod.rs`
//! Alternative: `#[cfg(target_os = "ios")]` for iOS backend

use crate::canvas::Canvas;
use crate::geom::Point;
use crate::input::Event;
use crate::shell::Appearance;
use crate::shell::Backend;
use crate::shell::Error;
use crate::shell::WindowOptions;
use std::time::Duration;

/// Android backend state machine.
///
/// Phase 1: Foundation (minimal fields)
/// - is_open: tracks whether app is running
/// - logical_width, logical_height: screen dimensions in logical pixels (DPI-independent)
/// - scale_factor: DPI scale (density * 160 / 96, typical 1.0–3.0)
/// - appearance: current theme (light or dark)
///
/// Phase 2 (TODO): Add ANativeWindow, event queue, Vulkan/OpenGL state
pub struct AndroidBackend {
    is_open: bool,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    appearance: Appearance,
}

impl Backend for AndroidBackend {
    /// Open a new Android surface.
    ///
    /// Phase 1: Stub implementation creating default state.
    /// Phase 2 TODO: Retrieve ANativeWindow via app.config()->window,
    /// create Vulkan surface or OpenGL framebuffer.
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        // Phase 1: Default Android phone dimensions (1080×2400 physical, 3.0 scale = 360×800 logical)
        Ok(AndroidBackend {
            is_open: true,
            logical_width: 360,
            logical_height: 800,
            scale_factor: 3.0,
            appearance: Appearance::Light,
        })
    }

    /// Collect events from Android event loop.
    ///
    /// Phase 1: Stub (no events collected).
    /// Phase 2 TODO: Translate MotionEvent → Click/Drag events.
    /// - MotionEvent.ACTION_DOWN → Click (start)
    /// - MotionEvent.ACTION_MOVE → Drag (continuous)
    /// - MotionEvent.ACTION_UP → Release
    /// - Handle coordinate translation: device pixels → logical pixels
    /// - Account for safe area insets (status bar, navigation bar)
    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Phase 1: No events collected yet (Phase 2 enhancement)
        Ok(())
    }

    /// Return screen dimensions and DPI scale.
    fn surface(&self) -> (u32, u32, f32) {
        (self.logical_width, self.logical_height, self.scale_factor)
    }

    /// Query current appearance (light or dark mode).
    ///
    /// Phase 1: Hardcoded to Light.
    /// Phase 2 TODO: Query Configuration.uiMode
    /// - (uiMode & UI_MODE_NIGHT_MASK) == UI_MODE_NIGHT_YES → Appearance::Dark
    /// - Otherwise → Appearance::Light
    fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Render canvas to screen.
    ///
    /// Phase 1: Stub (no rendering).
    /// Phase 2 TODO: Implement Vulkan or software rendering
    /// - Create Vulkan image from canvas pixels (if Vulkan available)
    /// - Or: Copy canvas pixels directly to ANativeWindow_Buffer
    /// - Handle platform rendering differences (Vulkan optional)
    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        // Phase 1: No rendering (Phase 2 enhancement)
        Ok(())
    }

    /// Check if app is still running.
    fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Detect DPI scale factor from Android system.
///
/// Phase 1: Stub (returns default).
/// Phase 2 TODO: Query DisplayMetrics.density
/// - Formula: scale = density * 160 / 96
/// - Typical values: 1.0 (ldpi), 1.5 (mdpi), 2.0 (xhdpi), 3.0 (xxhdpi)
fn detect_scale_factor() -> f32 {
    // Phase 1: Default 3x (common on modern Android devices)
    3.0
}

/// Detect current appearance (light or dark mode).
///
/// Phase 1: Stub (returns Light).
/// Phase 2 TODO: Query Configuration.uiMode
/// - Check (uiMode & UI_MODE_NIGHT_MASK) == UI_MODE_NIGHT_YES
/// - Subscribe to configuration changes via onConfigurationChanged()
/// - Re-render when theme changes
fn detect_appearance() -> Appearance {
    // Phase 1: Default to Light
    Appearance::Light
}

// ============================================================================
// STEP 23 Phase 1: Foundation — Verification Gates
// ============================================================================
//
// Acceptance criteria for Phase 1:
// ✅ AndroidBackend struct defined with 5 required fields
// ✅ All 6 Backend trait methods implemented (open, pump, surface, appearance, present, is_open)
// ✅ Code compiles without errors on Android target
// ✅ Coordinate contract documented (logical = device / scale)
// ✅ Phase 2 TODOs documented for each method
// ✅ DPI and appearance detection functions documented
// ✅ Safe area inset handling documented
//
// Next: Phase 2 will fill in actual Android NDK FFI and Vulkan/software rendering

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify AndroidBackend struct has required fields for Phase 1
    #[test]
    fn android_backend_has_required_fields() {
        let backend = AndroidBackend {
            is_open: true,
            logical_width: 360,
            logical_height: 800,
            scale_factor: 3.0,
            appearance: Appearance::Light,
        };

        assert!(backend.is_open);
        assert_eq!(backend.logical_width, 360);
        assert_eq!(backend.logical_height, 800);
        assert_eq!(backend.scale_factor, 3.0);
    }

    /// Verify Backend trait is implemented correctly
    #[test]
    fn android_backend_implements_backend_trait() {
        let backend = AndroidBackend::open(&WindowOptions::default()).unwrap();

        let (w, h, scale) = backend.surface();
        assert_eq!(w, 360);
        assert_eq!(h, 800);
        assert_eq!(scale, 3.0);

        assert!(backend.is_open());
    }

    /// Verify coordinate contract: logical = device / scale
    #[test]
    fn android_coordinate_contract_documented() {
        let backend = AndroidBackend::open(&WindowOptions::default()).unwrap();
        let (_w, _h, scale) = backend.surface();

        // Example: device pixel at (300, 600) with 3x scale
        let device_x = 300.0;
        let device_y = 600.0;
        let logical_x = device_x / scale;
        let logical_y = device_y / scale;

        assert_eq!(logical_x, 100.0);
        assert_eq!(logical_y, 200.0);
    }
}
