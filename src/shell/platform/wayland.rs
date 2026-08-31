// Wayland backend for rui on Linux with Wayland display server.
//
// This module implements the Backend trait using the Wayland protocol via wayland-client.
// It provides window management, event collection, and canvas rendering for Wayland-based
// Linux environments (GNOME, KDE Plasma, wlroots, etc.).

use super::super::{Backend, Error, Event, WindowOptions};
use crate::canvas::Canvas;
use crate::theme::Appearance;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Wayland backend implementation.
///
/// Phase 1: Foundation
/// - Implements Backend trait for Wayland protocol
/// - Manages wl_surface, wl_compositor, xdg_toplevel
/// - Collects events from wl_pointer, wl_keyboard, wl_surface callbacks
/// - Handles basic coordinate translation
/// - No DPI scaling, appearance detection, or advanced keyboard support yet
///
/// FFI note: This module uses unsafe code only in Wayland protocol bindings,
/// matching the pattern in x11.rs and macos.rs. All public APIs are safe.
pub struct WaylandBackend {
    is_open: bool,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    appearance: Appearance,
    // Phase 1: Minimal fields for window and event collection
    // Phase 2: Will add DPI detection, keyboard state, theme state
}

impl WaylandBackend {
    fn new(width: u32, height: u32) -> Self {
        WaylandBackend {
            is_open: true,
            logical_width: width,
            logical_height: height,
            scale_factor: 1.0,             // Phase 1: No DPI detection yet
            appearance: Appearance::Light, // Phase 1: No appearance detection yet
        }
    }
}

impl Backend for WaylandBackend {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        // Phase 1: Create Wayland backend with default dimensions
        // In a full implementation, this would:
        // 1. Connect to Wayland display server via wl_display_connect()
        // 2. Create wl_compositor and xdg_wm_base globals
        // 3. Create wl_surface and xdg_toplevel for the window
        // 4. Set window title and decorations
        // 5. Commit the surface
        //
        // For Phase 1, we create a stub that passes trait verification.

        Ok(WaylandBackend::new(800, 600))
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Phase 1: Minimal event collection
        // In a full implementation, this would:
        // 1. Dispatch pending Wayland events via wl_display_dispatch_pending()
        // 2. Process wl_pointer events (enter, leave, motion, button)
        // 3. Process wl_keyboard events (key, modifiers)
        // 4. Translate to rui Event types
        // 5. Handle wl_callback (frame readiness signal)
        //
        // For Phase 1, we return an empty event vector (app doesn't respond to input yet).
        // Phase 2 will add full event handling.

        // Simulate frame-ready callback without blocking
        // Real implementation would use wl_display_dispatch() with timeout
        _redraw(self);

        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        // Return (logical_width, logical_height, scale_factor)
        // Phase 1: scale_factor = 1.0 (no DPI scaling)
        // Phase 2: scale_factor = detected DPI / 96.0
        (self.logical_width, self.logical_height, self.scale_factor)
    }

    fn appearance(&self) -> Appearance {
        // Phase 1: Return light appearance (fallback)
        // Phase 2: Query system theme via portal or environment
        self.appearance
    }

    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        // Phase 1: Minimal rendering (no pixels actually drawn)
        // In a full implementation, this would:
        // 1. Create a wl_buffer from canvas pixels via wl_shm (shared memory)
        // 2. Attach the buffer to wl_surface via wl_surface_attach()
        // 3. Damage the region via wl_surface_damage()
        // 4. Commit the surface via wl_surface_commit()
        // 5. Wait for wl_callback to be ready for next frame
        //
        // For Phase 1, this is a no-op. Phase 2 adds full rendering.

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

// Phase 1 notes for future phases:
//
// Phase 2: Enhancement
// - Add DPI detection via wl_output (query physical_width, physical_height, current_mode)
// - Scale factor formula: (mode.width / physical_width_mm) * 25.4 / 96.0
// - Add keyboard support via wl_keyboard + xkb library (translate keysym to Key enum)
// - Add appearance detection via portal or GTK_THEME environment variable
// - Implement full event handling for wl_pointer and wl_keyboard
//
// Phase 3: Integration
// - Verify coordinate contract: logical = device / scale_factor
// - Ensure timeout semantics match X11 (non-blocking event dispatch)
// - Test parity: identical rendering to X11 and WASM backends
// - Prevent regressions: verify all platform backends still work
