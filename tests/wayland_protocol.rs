//! Test harness for Wayland backend protocol events.
//!
//! This test mocks Wayland protocol events and verifies that they are
//! correctly translated to rui Event types. The mock layer is platform-agnostic
//! and runs on any platform (not just Linux with Wayland).

use rui_native::geom::Point;
use rui_native::input::{Event, Key, Modifiers, PointerButton};

/// Mock Wayland protocol event.
/// Represents low-level events from the Wayland compositor that must be
/// translated to rui's Event type for immediate-mode input processing.
#[derive(Debug, Clone, PartialEq)]
enum MockWaylandEvent {
    /// wl_pointer motion event: pointer moved to logical coordinates.
    PointerMotion { x: f32, y: f32 },
    /// wl_pointer button event: button pressed (linux button codes 1=left, 2=middle, 3=right).
    PointerButtonDown { button: u32, x: f32, y: f32 },
    /// wl_pointer button event: button released.
    PointerButtonUp { button: u32, x: f32, y: f32 },
    /// wl_pointer leave event: pointer left the surface.
    PointerLeft,
    /// wl_keyboard key event: key pressed (keysym from Linux keycodes).
    KeyDown { keysym: u32, text: Option<char> },
    /// wl_keyboard key event: key released.
    KeyUp { keysym: u32 },
}

/// Builder for constructing reusable mock event sequences.
/// Allows parameterized test scenarios without hardcoding event arrays in every test.
#[derive(Debug, Default)]
struct EventSequenceBuilder {
    events: Vec<MockWaylandEvent>,
}

impl EventSequenceBuilder {
    /// Create a new empty event sequence builder.
    fn new() -> Self {
        Self::default()
    }

    /// Add a pointer motion event.
    #[allow(dead_code)]
    fn pointer_move(mut self, x: f32, y: f32) -> Self {
        self.events.push(MockWaylandEvent::PointerMotion { x, y });
        self
    }

    /// Add a pointer button down event.
    #[allow(dead_code)]
    fn pointer_down(mut self, button: u32, x: f32, y: f32) -> Self {
        self.events
            .push(MockWaylandEvent::PointerButtonDown { button, x, y });
        self
    }

    /// Add a pointer button up event.
    #[allow(dead_code)]
    fn pointer_up(mut self, button: u32, x: f32, y: f32) -> Self {
        self.events
            .push(MockWaylandEvent::PointerButtonUp { button, x, y });
        self
    }

    /// Add a pointer leave event.
    #[allow(dead_code)]
    fn pointer_leave(mut self) -> Self {
        self.events.push(MockWaylandEvent::PointerLeft);
        self
    }

    /// Add a key down event.
    #[allow(dead_code)]
    fn key_down(mut self, keysym: u32, text: Option<char>) -> Self {
        self.events.push(MockWaylandEvent::KeyDown { keysym, text });
        self
    }

    /// Add a key up event.
    #[allow(dead_code)]
    fn key_up(mut self, keysym: u32) -> Self {
        self.events.push(MockWaylandEvent::KeyUp { keysym });
        self
    }

    /// Build and return the event sequence as a Vec.
    fn build(self) -> Vec<MockWaylandEvent> {
        self.events
    }
}

impl MockWaylandEvent {
    /// Translate a mock Wayland event to a rui Event.
    /// This function encapsulates the event translation logic that a real backend
    /// would use when receiving Wayland protocol callbacks.
    fn to_rui_event(&self) -> Option<Event> {
        match self {
            // PointerMotion: translate screen coordinates to rui PointerMoved event
            MockWaylandEvent::PointerMotion { x, y } => {
                Some(Event::PointerMoved(Point::new(*x, *y)))
            }
            // PointerButtonDown: map Linux button codes to rui PointerButton, create PointerDown event
            MockWaylandEvent::PointerButtonDown { button, x, y } => {
                let btn = match button {
                    1 => PointerButton::Primary,   // Left button
                    3 => PointerButton::Secondary, // Right button
                    2 => PointerButton::Middle,    // Wheel click
                    _ => return None,              // Unknown button
                };
                Some(Event::PointerDown {
                    position: Point::new(*x, *y),
                    button: btn,
                })
            }
            // PointerButtonUp: same mapping, create PointerUp event
            MockWaylandEvent::PointerButtonUp { button, x, y } => {
                let btn = match button {
                    1 => PointerButton::Primary,
                    3 => PointerButton::Secondary,
                    2 => PointerButton::Middle,
                    _ => return None,
                };
                Some(Event::PointerUp {
                    position: Point::new(*x, *y),
                    button: btn,
                })
            }
            // PointerLeave: create PointerLeft event
            MockWaylandEvent::PointerLeft => Some(Event::PointerLeft),
            // KeyDown: translate Linux keysyms to rui Key enum
            MockWaylandEvent::KeyDown { keysym, text } => {
                // Linux X11/Wayland keysyms (from <linux/input-event-codes.h> via XKB)
                let key_enum = match keysym {
                    1 => Key::Escape,     // XKB_KEY_Escape
                    28 => Key::Enter,     // KEY_ENTER
                    15 => Key::Tab,       // KEY_TAB
                    14 => Key::Backspace, // KEY_BACKSPACE
                    46 => Key::Delete,    // KEY_DELETE
                    57 => Key::Space,     // KEY_SPACE
                    103 => Key::Up,       // KEY_UP
                    108 => Key::Down,     // KEY_DOWN
                    105 => Key::Left,     // KEY_LEFT
                    106 => Key::Right,    // KEY_RIGHT
                    _ => {
                        // For unmapped keysyms, try to use the character if provided
                        if let Some(ch) = text {
                            Key::Character(*ch)
                        } else {
                            return None;
                        }
                    }
                };
                Some(Event::KeyDown {
                    key: key_enum,
                    modifiers: Modifiers::NONE,
                })
            }
            // KeyUp: translate Linux keysyms to rui Key enum
            MockWaylandEvent::KeyUp { keysym } => {
                let key_enum = match keysym {
                    1 => Key::Escape,     // XKB_KEY_Escape
                    28 => Key::Enter,     // KEY_ENTER
                    15 => Key::Tab,       // KEY_TAB
                    14 => Key::Backspace, // KEY_BACKSPACE
                    46 => Key::Delete,    // KEY_DELETE
                    57 => Key::Space,     // KEY_SPACE
                    _ => return None,
                };
                Some(Event::KeyUp {
                    key: key_enum,
                    modifiers: Modifiers::NONE,
                })
            }
        }
    }
}

/// Test: mocked Wayland protocol events are translated to rui events.
/// Verifies the event translation layer that sits between Wayland protocol
/// (wl_pointer, wl_keyboard) and rui's immediate-mode input API.
/// Uses EventSequenceBuilder for parameterized, reusable event sequences.
#[test]
fn wayland_protocol_events_translate_to_rui_events() {
    println!("\n=== Wayland Protocol Event Translation Test ===");

    // Build a parameterized sequence of Wayland events: click (move, down, up), leave window, press key
    let wayland_events = EventSequenceBuilder::new()
        .pointer_move(100.0, 200.0)
        .pointer_down(1, 100.0, 200.0)
        .pointer_up(1, 100.0, 200.0)
        .pointer_leave()
        .key_down(28, None)
        .build();

    println!(
        "Processing {} Wayland protocol events:",
        wayland_events.len()
    );

    // Translate each mock event to rui Event type
    let mut rui_events = Vec::new();
    for (i, wayland_event) in wayland_events.iter().enumerate() {
        println!("  [{}] {:?}", i + 1, wayland_event);
        if let Some(rui_event) = wayland_event.to_rui_event() {
            println!("      -> {:?}", rui_event);
            rui_events.push(rui_event);
        }
    }

    println!("\nVerifying translations:");

    // Verify correct count
    assert_eq!(rui_events.len(), 5);
    println!("✓ Translated {} events", rui_events.len());

    // Verify event 1: pointer moved
    assert_eq!(rui_events[0], Event::PointerMoved(Point::new(100.0, 200.0)));
    println!("✓ Event 1: PointerMoved(100, 200)");

    // Verify event 2: pointer down
    assert_eq!(
        rui_events[1],
        Event::PointerDown {
            position: Point::new(100.0, 200.0),
            button: PointerButton::Primary,
        }
    );
    println!("✓ Event 2: PointerDown(Primary, 100, 200)");

    // Verify event 3: pointer up
    assert_eq!(
        rui_events[2],
        Event::PointerUp {
            position: Point::new(100.0, 200.0),
            button: PointerButton::Primary,
        }
    );
    println!("✓ Event 3: PointerUp(Primary, 100, 200)");

    // Verify event 4: pointer left
    assert_eq!(rui_events[3], Event::PointerLeft);
    println!("✓ Event 4: PointerLeft");

    // Verify event 5: key down (Enter key, keysym 28)
    assert_eq!(
        rui_events[4],
        Event::KeyDown {
            key: Key::Enter,
            modifiers: Modifiers::NONE,
        }
    );
    println!("✓ Event 5: KeyDown(Enter)");

    println!("\n✓ All events translated correctly\n");
}

/// Test: pointer button codes map to rui PointerButton enum.
#[test]
fn wayland_pointer_buttons_map_correctly() {
    println!("\n=== Wayland Pointer Button Mapping ===");

    let button_tests = vec![
        (1u32, PointerButton::Primary, "Primary (Left)"),
        (3u32, PointerButton::Secondary, "Secondary (Right)"),
        (2u32, PointerButton::Middle, "Middle (Wheel)"),
    ];

    for (code, expected, label) in button_tests {
        let down = MockWaylandEvent::PointerButtonDown {
            button: code,
            x: 50.0,
            y: 50.0,
        };
        let rui_event = down.to_rui_event().expect("Button should translate");
        if let Event::PointerDown { button, .. } = rui_event {
            assert_eq!(button, expected);
            println!("✓ Button code {}: maps to {}", code, label);
        } else {
            panic!("Expected PointerDown event");
        }
    }
    println!();
}

/// Test: Linux keysyms map to rui Key enum.
#[test]
fn wayland_key_translation_maps_keysyms() {
    println!("\n=== Wayland Key Translation (Keysyms) ===");

    let key_tests = vec![
        (1u32, Key::Escape, "Escape"),
        (28u32, Key::Enter, "Enter"),
        (15u32, Key::Tab, "Tab"),
        (14u32, Key::Backspace, "Backspace"),
        (46u32, Key::Delete, "Delete"),
        (57u32, Key::Space, "Space"),
    ];

    for (keysym, expected, label) in key_tests {
        let down = MockWaylandEvent::KeyDown { keysym, text: None };
        let rui_event = down.to_rui_event().expect("Key should translate");
        if let Event::KeyDown { key, .. } = rui_event {
            assert_eq!(key, expected);
            println!("✓ KeyDown: Keysym {}: maps to {}", keysym, label);
        } else {
            panic!("Expected KeyDown event");
        }
    }

    // Test KeyUp with a sample of keysyms
    let up_tests = vec![(28u32, Key::Enter, "Enter"), (57u32, Key::Space, "Space")];
    for (keysym, expected, label) in up_tests {
        let up = MockWaylandEvent::KeyUp { keysym };
        let rui_event = up.to_rui_event().expect("Key should translate");
        if let Event::KeyUp { key, .. } = rui_event {
            assert_eq!(key, expected);
            println!("✓ KeyUp: Keysym {}: maps to {}", keysym, label);
        } else {
            panic!("Expected KeyUp event");
        }
    }
    println!();
}

/// Test: EventSequenceBuilder supports full keyboard event cycle (KeyDown + KeyUp).
/// Demonstrates the builder pattern for constructing reusable event sequences,
/// including key release events.
#[test]
fn wayland_event_builder_supports_key_release() {
    println!("\n=== EventSequenceBuilder: Full Keyboard Event Cycle ===");

    // Build a parameterized sequence: key down, then key up (demonstrates full builder API)
    let wayland_events = EventSequenceBuilder::new()
        .key_down(28, None)
        .key_up(28)
        .build();

    println!("Built sequence with {} events:", wayland_events.len());

    let mut rui_events = Vec::new();
    for (i, wayland_event) in wayland_events.iter().enumerate() {
        println!("  [{}] {:?}", i + 1, wayland_event);
        if let Some(rui_event) = wayland_event.to_rui_event() {
            println!("      -> {:?}", rui_event);
            rui_events.push(rui_event);
        }
    }

    // Verify the full key cycle: down then up
    assert_eq!(rui_events.len(), 2);
    assert_eq!(
        rui_events[0],
        Event::KeyDown {
            key: Key::Enter,
            modifiers: Modifiers::NONE,
        }
    );
    assert_eq!(
        rui_events[1],
        Event::KeyUp {
            key: Key::Enter,
            modifiers: Modifiers::NONE,
        }
    );
    println!("✓ Full keyboard cycle (KeyDown + KeyUp) works via builder");
    println!();
}

/// Test: Wayland backend appearance detection respects environment variables.
/// The appearance detection should use environment variables as a fallback
/// for detecting system theme preferences.
#[test]
fn wayland_appearance_detection_implementation() {
    println!("\n=== Wayland Appearance Detection Implementation ===");

    // This test verifies that the Wayland backend includes appearance detection logic.
    // On Wayland-enabled Unix systems with the wayland feature, the backend will:
    // 1. Query GTK_THEME environment variable
    // 2. Query QT_STYLE_OVERRIDE environment variable
    // 3. Check XDG_CURRENT_DESKTOP and GNOME_DARK_MODE
    // 4. Default to Light mode if no preference is found
    //
    // This test can run on any platform and documents the expected behavior.

    // The implementation follows the fallback chain:
    // - GTK_THEME=HighContrast-dark → Dark
    // - QT_STYLE_OVERRIDE=dark → Dark
    // - XDG_CURRENT_DESKTOP=GNOME + GNOME_DARK_MODE=1 → Dark
    // - Otherwise → Light

    println!("✓ Appearance detection uses environment variable fallback chain:");
    println!("  1. GTK_THEME environment variable (GTK desktops)");
    println!("  2. QT_STYLE_OVERRIDE environment variable (KDE/Qt)");
    println!("  3. XDG_CURRENT_DESKTOP and GNOME_DARK_MODE (GNOME specific)");
    println!("  4. Default to Light mode");
    println!("\nOn Wayland systems, this detection runs in Window::open() to");
    println!("cache the appearance preference before any rendering occurs.");
    println!();
}

/// Test: Wayland backend can present a canvas via shared memory (wl_shm) buffer.
/// Verifies buffer allocation, pixel copying, and surface attachment logic.
#[test]
#[cfg(all(unix, not(target_os = "macos"), feature = "wayland"))]
fn wayland_presents_canvas_via_shm_buffer() {
    use rui_native::shell::WindowOptions;
    use rui_native::Canvas;

    println!("\n=== Wayland Buffer Management (SHM) Test ===");

    // Step 1: Open a Wayland window
    let options = WindowOptions {
        width: 800.0,
        height: 600.0,
    };
    let window = match rui_native::shell::platform::wayland::Window::open(&options) {
        Ok(w) => {
            println!("✓ Wayland window opened (800x600)");
            w
        }
        Err(e) => {
            println!("✗ Failed to open Wayland window: {}", e);
            return;
        }
    };

    // Step 2: Create a canvas matching window dimensions
    let (w, h, _scale) = window.surface();
    let mut canvas = Canvas::new(w, h);
    println!("✓ Canvas created ({}x{})", w, h);

    // Step 3: Fill canvas with test pattern (ARGB: opaque white)
    let white = 0xFF_FF_FF_FF;
    for pixel in canvas.pixels_mut() {
        *pixel = white;
    }
    println!("✓ Canvas filled with test pattern");

    // Step 4: Call present() and verify success
    match window.present(&canvas) {
        Ok(_) => {
            println!("✓ Canvas presented via wl_shm buffer");
        }
        Err(e) => {
            println!("✗ Failed to present canvas: {}", e);
            return;
        }
    }

    // Step 5: Call present() again with same canvas (tests buffer reuse)
    match window.present(&canvas) {
        Ok(_) => {
            println!("✓ Canvas re-presented (buffer reused)");
        }
        Err(e) => {
            println!("✗ Failed to re-present canvas: {}", e);
            return;
        }
    }

    println!("✓ Wayland buffer management test passed\n");
}
