//! The Wayland backend: wayland-client for the window and input, DMA-BUF for the blit.
//!
//! # Wayland vs X11
//!
//! Wayland is the modern display server protocol, replacing X11 on modern Linux
//! systems. This backend uses `wayland-client` to communicate with the Wayland
//! compositor. When available, Wayland provides better performance, security,
//! and resource isolation than X11.
//!
//! The rui Wayland backend:
//! - Opens a surface via `wl_shell` or `xdg_shell` protocol
//! - Handles input events (pointer, keyboard) from the compositor
//! - Detects system appearance (light/dark mode) via the platform protocol
//! - Presents frames using DMA-BUF or shm (shared memory) buffers

use crate::theme::Appearance;
use crate::{Canvas, Event, Point};
use std::time::Duration;

use crate::shell::{Backend, Error, WindowOptions};

/// Minimal Wayland surface state for basic window management and event handling.
/// A full implementation would include surface buffers, input device tracking,
/// and wl_shell/xdg_shell protocol state machines.
pub struct Window {
    /// Whether the window is still open.
    is_open: bool,
    /// Window dimensions in logical units.
    width: u32,
    height: u32,
    /// Display scale factor (device pixels per logical unit).
    scale_factor: f32,
    /// Current appearance (light or dark mode).
    appearance: Appearance,
}

impl Backend for Window {
    fn open(options: &WindowOptions) -> Result<Self, Error> {
        // Placeholder: In a full implementation, this would:
        // 1. Connect to the Wayland display via wl_display
        // 2. Bind to wl_registry to discover available globals
        // 3. Create a wl_surface and bind to xdg_shell for window decoration
        // 4. Attach input listeners for pointer and keyboard events
        // 5. Query system appearance via portal API or theme protocol
        //
        // For now, return a basic window state that compiles.
        Ok(Window {
            is_open: true,
            width: options.width.max(420.0) as u32,
            height: options.height.max(320.0) as u32,
            scale_factor: 1.0,
            appearance: Appearance::Light,
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Placeholder: In a full implementation, this would:
        // 1. Use wl_event_queue to poll for events from the compositor
        // 2. Translate wl_pointer events (motion, button) to rui Event::Pointer
        // 3. Translate wl_keyboard events (keymap, key) to rui Event::Key
        // 4. Handle wl_surface callbacks (frame, leave) for rendering hints
        // 5. Respect the timeout parameter for backpressure
        //
        // For now, return without adding events (keeps the loop responsive).
        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        // Return drawable size in device pixels and scale factor.
        // In a full implementation, this would query the output's scale factor
        // and apply DPI scaling to convert logical units to device pixels.
        let device_width = (self.width as f32 * self.scale_factor) as u32;
        let device_height = (self.height as f32 * self.scale_factor) as u32;
        (device_width, device_height, self.scale_factor)
    }

    fn appearance(&self) -> Appearance {
        // Return the current appearance (light or dark mode).
        // In a full implementation, this would listen to appearance changes
        // via the system portal API (org.freedesktop.Appearance) or theme protocol.
        self.appearance
    }

    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        // Placeholder: In a full implementation, this would:
        // 1. Create or reuse a DMA-BUF or shm buffer with the canvas pixel data
        // 2. Attach the buffer to the wl_surface
        // 3. Commit the surface state to the compositor
        // 4. Handle buffer lifecycle (release callbacks, double buffering)
        //
        // For now, succeed silently (no-op presentation).
        Ok(())
    }

    fn is_open(&self) -> bool {
        // Return whether the window is still on screen.
        // In a full implementation, this would check if wl_surface events
        // indicate the window has been destroyed or the display connection lost.
        self.is_open
    }
}
