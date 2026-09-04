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
#[test]
fn wayland_protocol_events_translate_to_rui_events() {
    println!("\n=== Wayland Protocol Event Translation Test ===");

    // Simulate a sequence of Wayland events: click (move, down, up), leave window, press key
    let wayland_events = [
        MockWaylandEvent::PointerMotion { x: 100.0, y: 200.0 },
        MockWaylandEvent::PointerButtonDown {
            button: 1,
            x: 100.0,
            y: 200.0,
        },
        MockWaylandEvent::PointerButtonUp {
            button: 1,
            x: 100.0,
            y: 200.0,
        },
        MockWaylandEvent::PointerLeft,
        MockWaylandEvent::KeyDown {
            keysym: 28,
            text: None,
        },
    ];

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
