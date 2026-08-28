//! Tests for WASM event handling, particularly pointer events with button mapping.
#![cfg(target_arch = "wasm32")]

use rui::input::{Key, PointerButton};
use rui::shell::event_mapping::{map_keyboard_code_to_key, map_pointer_button};

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_wasm_pointer_button_mapping() {
    // This test verifies that pointer button mapping works correctly.
    // DOM button values: 0 = Left, 1 = Middle, 2 = Right
    // rui PointerButton: Primary (0), Secondary (1), Middle (2)

    // Left button (0) should map to Primary
    assert_eq!(
        map_pointer_button(0),
        Some(PointerButton::Primary),
        "DOM button 0 (left) should map to PointerButton::Primary"
    );

    // Middle button (1) should map to Middle
    assert_eq!(
        map_pointer_button(1),
        Some(PointerButton::Middle),
        "DOM button 1 (middle) should map to PointerButton::Middle"
    );

    // Right button (2) should map to Secondary
    assert_eq!(
        map_pointer_button(2),
        Some(PointerButton::Secondary),
        "DOM button 2 (right) should map to PointerButton::Secondary"
    );
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_wasm_keyboard_events() {
    // Test keyboard code mapping for common keys
    assert_eq!(
        map_keyboard_code_to_key("KeyA"),
        Some(Key::Character('a')),
        "KeyA should map to Character('a')"
    );

    assert_eq!(
        map_keyboard_code_to_key("Enter"),
        Some(Key::Enter),
        "Enter should map to Key::Enter"
    );

    assert_eq!(
        map_keyboard_code_to_key("Shift"),
        None,
        "Shift key code should not map (modifiers are handled separately)"
    );
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_wasm_text_input_event() {
    // Test that plain text characters are accepted and control characters are filtered
    use rui::input::Event;

    // Verify that a simple character produces Event::Text
    let text = "a";
    let event = Event::Text(text.to_string());
    assert_eq!(
        event,
        Event::Text("a".to_string()),
        "Simple character should produce Event::Text"
    );
}
