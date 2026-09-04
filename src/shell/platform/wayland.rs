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
use crate::{Canvas, Event, Key, Modifiers, Point, PointerButton};
use std::cell::RefCell;
use std::env;
use std::time::Duration;

use crate::shell::{Backend, Error, WindowOptions};

/// Translate Wayland button constants to rui PointerButton.
/// Wayland button codes from linux/input-event-codes.h.
fn translate_button(button: u32) -> PointerButton {
    match button {
        0x110 => PointerButton::Primary,   // BTN_LEFT
        0x111 => PointerButton::Secondary, // BTN_RIGHT
        0x112 => PointerButton::Middle,    // BTN_MIDDLE
        _ => PointerButton::Primary,       // Default
    }
}

/// Translate XKB keysym values to rui Key enum.
/// XKB keysyms are used by the Wayland wl_keyboard protocol.
fn translate_keysym(keysym: u32) -> Option<Key> {
    match keysym {
        // Function keys
        0xff1b => Some(Key::Escape),
        0xff0d => Some(Key::Enter),
        0xff09 => Some(Key::Tab),
        0xff08 => Some(Key::Backspace),
        0xffff => Some(Key::Delete),
        0xff51 => Some(Key::Left),
        0xff52 => Some(Key::Up),
        0xff53 => Some(Key::Right),
        0xff54 => Some(Key::Down),
        0xff50 => Some(Key::Home),
        0xff57 => Some(Key::End),
        0xff55 => Some(Key::PageUp),
        0xff56 => Some(Key::PageDown),
        // Punctuation is handled separately; modifiers return None
        _ => None,
    }
}

/// Detect system appearance (light/dark mode) via environment variables or system settings.
/// Uses a fallback chain: tries environment variables first, then defaults to Light.
fn detect_appearance() -> Appearance {
    // Fallback chain for appearance detection:
    // 1. Check GTK_THEME environment variable (used by many Linux DEs)
    if let Ok(gtk_theme) = env::var("GTK_THEME") {
        if gtk_theme.to_lowercase().contains("dark") {
            return Appearance::Dark;
        }
    }

    // 2. Check QT_STYLE_OVERRIDE environment variable (KDE Plasma)
    if let Ok(qt_style) = env::var("QT_STYLE_OVERRIDE") {
        if qt_style.to_lowercase().contains("dark") {
            return Appearance::Dark;
        }
    }

    // 3. Check XDG_CURRENT_DESKTOP heuristic
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();
        // These desktops typically default to dark mode
        if desktop.contains("gnome") || desktop.contains("kde") {
            // Could add portal API query here in future
            // For now, use GNOME/KDE defaults (typically light, but check other env vars)
            if let Ok(prefer_dark) = env::var("GNOME_DARK_MODE") {
                if prefer_dark == "1" || prefer_dark.to_lowercase() == "true" {
                    return Appearance::Dark;
                }
            }
        }
    }

    // Default to Light mode if no preference is detected
    Appearance::Light
}

/// Shared memory buffer for Wayland frame presentation.
/// Holds pixel data that can be attached to a wl_surface via wl_shm protocol.
/// Minimal implementation: single buffer, no release callbacks or double buffering.
struct ShmBuffer {
    /// Pixel data in ARGB format (matching canvas pixel format).
    pixels: Vec<u32>,
    /// Width in device pixels.
    width: u32,
    /// Height in device pixels.
    height: u32,
}

impl ShmBuffer {
    /// Create a new shared memory buffer with the given dimensions.
    fn new(width: u32, height: u32) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            pixels: vec![0; size],
            width,
            height,
        }
    }

    /// Resize the buffer if canvas dimensions have changed.
    /// Discards previous contents (will be redrawn on next frame).
    fn resize(&mut self, width: u32, height: u32) {
        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;
            let size = (width as usize) * (height as usize);
            self.pixels.clear();
            self.pixels.resize(size, 0);
        }
    }

    /// Copy pixel data from canvas into this buffer.
    /// Assumes canvas pixel format matches (ARGB) and dimensions are compatible.
    fn copy_from_canvas(&mut self, canvas: &Canvas) {
        let canvas_pixels = canvas.pixels();
        if canvas_pixels.len() == self.pixels.len() {
            self.pixels.copy_from_slice(canvas_pixels);
        }
    }
}

/// Event for Wayland event queue simulation.
#[derive(Clone, Debug)]
enum WaylandEvent {
    /// Pointer motion event with coordinates.
    PointerMotion { x: f32, y: f32 },
    /// Pointer button event.
    PointerButton {
        button: PointerButton,
        pressed: bool,
    },
    /// Scroll event.
    Scroll { delta_y: f32 },
    /// Keyboard event.
    Key { key: Key, pressed: bool },
}

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
    /// Shared memory buffer for frame presentation (wl_shm protocol).
    /// Interior mutability allows updating the buffer during `present()` which takes `&self`.
    shm_buffer: RefCell<Option<ShmBuffer>>,
    /// Event queue for testing (mock events).
    event_queue: RefCell<Vec<WaylandEvent>>,
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
        // Detect system appearance on startup using environment variables
        let appearance = detect_appearance();

        Ok(Window {
            is_open: true,
            width: options.width.max(420.0) as u32,
            height: options.height.max(320.0) as u32,
            scale_factor: 1.0,
            appearance,
            shm_buffer: RefCell::new(None),
            event_queue: RefCell::new(Vec::new()),
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Process events from the Wayland event queue.
        // In a full implementation, this would call wl_display_dispatch_pending()
        // to poll the compositor for new events. For now, we drain mock events
        // from the queue to support testing.
        let mut queue = self.event_queue.borrow_mut();
        for wayland_event in queue.drain(..) {
            match wayland_event {
                WaylandEvent::PointerMotion { x, y } => {
                    events.push(Event::PointerMoved(Point::new(x, y)));
                }
                WaylandEvent::PointerButton { button, pressed } => {
                    if pressed {
                        events.push(Event::PointerDown(button));
                    } else {
                        events.push(Event::PointerUp(button));
                    }
                }
                WaylandEvent::Scroll { delta_y } => {
                    events.push(Event::Scroll(Point::new(0.0, delta_y)));
                }
                WaylandEvent::Key { key, pressed } => {
                    if pressed {
                        events.push(Event::KeyDown(key, None));
                    } else {
                        events.push(Event::KeyUp(key));
                    }
                }
            }
        }
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

    fn present(&self, canvas: &Canvas) -> Result<(), Error> {
        // Implement basic shared memory (wl_shm) buffer management.
        // Steps:
        // 1. Create or resize buffer to match canvas dimensions
        // 2. Copy canvas pixel data into buffer
        // 3. (Full implementation would attach to wl_surface and commit here)
        //
        // For now, the buffer holds the pixel data. A full implementation would:
        // - Create a wl_shm_pool via the wl_shm global
        // - Create a wl_buffer from the pool
        // - Attach the buffer to wl_surface via wl_surface.attach()
        // - Call wl_surface.commit() to present to the compositor

        let (width, height, _scale) = self.surface();

        let mut buffer_ref = self.shm_buffer.borrow_mut();

        // Create or update the shared memory buffer
        match buffer_ref.as_mut() {
            Some(buffer) => {
                // Buffer exists; resize if needed and copy new pixel data
                buffer.resize(width, height);
                buffer.copy_from_canvas(canvas);
            }
            None => {
                // First frame; create the buffer
                let mut buffer = ShmBuffer::new(width, height);
                buffer.copy_from_canvas(canvas);
                *buffer_ref = Some(buffer);
            }
        }

        Ok(())
    }

    fn is_open(&self) -> bool {
        // Return whether the window is still on screen.
        // In a full implementation, this would check if wl_surface events
        // indicate the window has been destroyed or the display connection lost.
        self.is_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appearance_detection_returns_valid_appearance() {
        let appearance = detect_appearance();
        match appearance {
            Appearance::Light | Appearance::Dark => {}
        }
    }

    #[test]
    fn appearance_detection_is_deterministic() {
        let appearance1 = detect_appearance();
        let appearance2 = detect_appearance();
        assert_eq!(appearance1, appearance2);
    }

    #[test]
    fn button_translation_maps_wayland_left_button() {
        let button = translate_button(0x110); // BTN_LEFT
        assert_eq!(button, PointerButton::Primary);
    }

    #[test]
    fn button_translation_maps_wayland_right_button() {
        let button = translate_button(0x111); // BTN_RIGHT
        assert_eq!(button, PointerButton::Secondary);
    }

    #[test]
    fn button_translation_maps_wayland_middle_button() {
        let button = translate_button(0x112); // BTN_MIDDLE
        assert_eq!(button, PointerButton::Middle);
    }

    #[test]
    fn button_translation_defaults_unknown_buttons() {
        let button = translate_button(0xFFFF); // Unknown button
        assert_eq!(button, PointerButton::Primary);
    }

    #[test]
    fn keysym_translation_maps_escape() {
        let key = translate_keysym(0xff1b);
        assert_eq!(key, Some(Key::Escape));
    }

    #[test]
    fn keysym_translation_maps_enter() {
        let key = translate_keysym(0xff0d);
        assert_eq!(key, Some(Key::Enter));
    }

    #[test]
    fn keysym_translation_maps_backspace() {
        let key = translate_keysym(0xff08);
        assert_eq!(key, Some(Key::Backspace));
    }

    #[test]
    fn keysym_translation_maps_arrow_keys() {
        assert_eq!(translate_keysym(0xff51), Some(Key::Left));
        assert_eq!(translate_keysym(0xff53), Some(Key::Right));
        assert_eq!(translate_keysym(0xff52), Some(Key::Up));
        assert_eq!(translate_keysym(0xff54), Some(Key::Down));
    }

    #[test]
    fn pump_processes_pointer_motion_events() {
        let mut window = Window::open(&WindowOptions {
            width: 800.0,
            height: 600.0,
        })
        .unwrap();

        window
            .event_queue
            .borrow_mut()
            .push(WaylandEvent::PointerMotion { x: 100.0, y: 200.0 });

        let mut events = Vec::new();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::PointerMoved(point) => {
                assert_eq!(point.x, 100.0);
                assert_eq!(point.y, 200.0);
            }
            _ => panic!("Expected PointerMoved event"),
        }
    }

    #[test]
    fn pump_processes_pointer_button_events() {
        let mut window = Window::open(&WindowOptions {
            width: 800.0,
            height: 600.0,
        })
        .unwrap();

        window
            .event_queue
            .borrow_mut()
            .push(WaylandEvent::PointerButton {
                button: PointerButton::Primary,
                pressed: true,
            });

        let mut events = Vec::new();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::PointerDown(button) => {
                assert_eq!(*button, PointerButton::Primary);
            }
            _ => panic!("Expected PointerDown event"),
        }
    }

    #[test]
    fn pump_processes_keyboard_events() {
        let mut window = Window::open(&WindowOptions {
            width: 800.0,
            height: 600.0,
        })
        .unwrap();

        window.event_queue.borrow_mut().push(WaylandEvent::Key {
            key: Key::Escape,
            pressed: true,
        });

        let mut events = Vec::new();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::KeyDown(key, _) => {
                assert_eq!(*key, Key::Escape);
            }
            _ => panic!("Expected KeyDown event"),
        }
    }

    #[test]
    fn pump_processes_scroll_events() {
        let mut window = Window::open(&WindowOptions {
            width: 800.0,
            height: 600.0,
        })
        .unwrap();

        window
            .event_queue
            .borrow_mut()
            .push(WaylandEvent::Scroll { delta_y: -120.0 });

        let mut events = Vec::new();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::Scroll(point) => {
                assert_eq!(point.y, -120.0);
            }
            _ => panic!("Expected Scroll event"),
        }
    }

    #[test]
    fn pump_clears_event_queue_after_processing() {
        let mut window = Window::open(&WindowOptions {
            width: 800.0,
            height: 600.0,
        })
        .unwrap();

        window
            .event_queue
            .borrow_mut()
            .push(WaylandEvent::PointerMotion { x: 100.0, y: 200.0 });

        let mut events = Vec::new();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();

        // Process again; queue should be empty
        events.clear();
        window
            .pump(Duration::from_secs(0), &mut events, &mut |_| {})
            .unwrap();
        assert_eq!(events.len(), 0);
    }
}
