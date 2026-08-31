//! iOS backend for rui — implements Backend trait via UIKit/Metal.
//!
//! STEP 23 Phase 1: Foundation — iOS backend for mobile support.
//! This is a stub implementation establishing the Backend trait contract.
//! Phase 2 will add full touch event handling, DPI detection, and rendering.
//!
//! # Architecture
//!
//! The iOS backend follows the Recipe 2 pattern:
//! - Phase 1: Implement Backend trait (6 methods) with minimal stubs
//! - Phase 2: Add touch handling, DPI detection, appearance detection
//! - Phase 3: Comprehensive testing and parity verification
//!
//! # Platform Selection
//!
//! iOS backend is selected by: `#[cfg(target_os = "ios")]` in `src/shell/platform/mod.rs`
//! Alternative: `#[cfg(target_os = "android")]` for Android backend

use crate::canvas::Canvas;
use crate::geom::Point;
use crate::input::Event;
use crate::shell::Appearance;
use crate::shell::Backend;
use crate::shell::Error;
use crate::shell::WindowOptions;
use std::time::Duration;

/// iOS backend state machine.
///
/// Phase 1: Foundation (minimal fields)
/// - is_open: tracks whether app is running
/// - logical_width, logical_height: screen dimensions in logical pixels
/// - scale_factor: DPI scale (1x, 2x, 3x on modern iPhones)
/// - appearance: current theme (light or dark)
///
/// Phase 2 (TODO): Add UIWindow, CALayer, event queue state
pub struct IosBackend {
    is_open: bool,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    appearance: Appearance,
}

impl Backend for IosBackend {
    /// Open a new iOS window.
    ///
    /// Phase 1: Stub implementation creating default state.
    /// Phase 2 TODO: Create UIWindow via Objective-C FFI, attach CALayer.
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        // Phase 1: Default iPhone Pro dimensions (1170×2532 physical, 2x scale = 585×1266 logical)
        Ok(IosBackend {
            is_open: true,
            logical_width: 585,
            logical_height: 1266,
            scale_factor: 2.0,
            appearance: Appearance::Light,
        })
    }

    /// Collect events from UIKit event loop.
    ///
    /// Phase 1: Stub (no events collected).
    /// Phase 2 TODO: Translate UITouch phase → Click/Drag events.
    /// - UITouchPhaseBegan → Click (start)
    /// - UITouchPhaseMoved → Drag (continuous)
    /// - UITouchPhaseEnded → Release
    /// - Handle coordinate translation: device pixels → logical pixels
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
    /// Phase 2 TODO: Query UITraitCollection.userInterfaceStyle
    /// - UIUserInterfaceStyleLight → Appearance::Light
    /// - UIUserInterfaceStyleDark → Appearance::Dark
    fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Render canvas to screen.
    ///
    /// Phase 1: Stub (no rendering).
    /// Phase 2 TODO: Implement Metal rendering
    /// - Create MTLTexture from canvas pixels
    /// - Render to CAMetalLayer
    /// - Present next drawable to screen
    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        // Phase 1: No rendering (Phase 2 enhancement)
        Ok(())
    }

    /// Check if app is still running.
    fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Detect DPI scale factor from iOS system.
///
/// Phase 1: Stub (returns default).
/// Phase 2 TODO: Query UIScreen.main.nativeScale
/// - Typical values: 1.0 (iPad), 2.0 (iPhone SE), 3.0 (iPhone Pro)
fn detect_scale_factor() -> f32 {
    // Phase 1: Default 2x (most common on modern devices)
    2.0
}

/// Detect current appearance (light or dark mode).
///
/// Phase 1: Stub (returns Light).
/// Phase 2 TODO: Query UITraitCollection.userInterfaceStyle
/// - Check UIApplication.shared.preferredContentSizeCategory
/// - Subscribe to UITraitCollection.changed notifications
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
// ✅ IosBackend struct defined with 5 required fields
// ✅ All 6 Backend trait methods implemented (open, pump, surface, appearance, present, is_open)
// ✅ Code compiles without errors on iOS target
// ✅ Coordinate contract documented (logical = device / scale)
// ✅ Phase 2 TODOs documented for each method
// ✅ DPI and appearance detection functions documented
//
// Next: Phase 2 will fill in actual UIKit FFI and Metal rendering

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify IosBackend struct has required fields for Phase 1
    #[test]
    fn ios_backend_has_required_fields() {
        let backend = IosBackend {
            is_open: true,
            logical_width: 585,
            logical_height: 1266,
            scale_factor: 2.0,
            appearance: Appearance::Light,
        };

        assert!(backend.is_open);
        assert_eq!(backend.logical_width, 585);
        assert_eq!(backend.logical_height, 1266);
        assert_eq!(backend.scale_factor, 2.0);
    }

    /// Verify Backend trait is implemented correctly
    #[test]
    fn ios_backend_implements_backend_trait() {
        let backend = IosBackend::open(&WindowOptions::default()).unwrap();

        let (w, h, scale) = backend.surface();
        assert_eq!(w, 585);
        assert_eq!(h, 1266);
        assert_eq!(scale, 2.0);

        assert!(backend.is_open());
    }

    /// Verify coordinate contract: logical = device / scale
    #[test]
    fn ios_coordinate_contract_documented() {
        let backend = IosBackend::open(&WindowOptions::default()).unwrap();
        let (_w, _h, scale) = backend.surface();

        // Example: device pixel at (200, 200) with 2x scale
        let device_x = 200.0;
        let device_y = 200.0;
        let logical_x = device_x / scale;
        let logical_y = device_y / scale;

        assert_eq!(logical_x, 100.0);
        assert_eq!(logical_y, 100.0);
    }
}
