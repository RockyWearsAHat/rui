//! Tests for WASM event handling, particularly pointer events with button mapping.
#![cfg(target_arch = "wasm32")]

use rui::input::{Key, PointerButton};
use rui::shell::event_mapping::{map_keyboard_code_to_key, map_pointer_button};

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

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
    use rui::shell::event_mapping::filter_text_input_data;

    // Verify that a simple character produces Event::Text
    assert_eq!(
        filter_text_input_data("a"),
        "a",
        "Simple character should pass through"
    );

    // Verify that control characters are filtered
    assert_eq!(
        filter_text_input_data("a\x00b"),
        "ab",
        "Control characters should be filtered"
    );

    // Verify tab is preserved
    assert_eq!(
        filter_text_input_data("a\tb"),
        "a\tb",
        "Tab should be preserved"
    );

    // Verify newline is preserved
    assert_eq!(
        filter_text_input_data("a\nb"),
        "a\nb",
        "Newline should be preserved"
    );
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn test_wasm_scroll_events() {
    use rui::shell::event_mapping::normalize_wheel_delta;

    // Test that positive wheel deltaY (scroll down) produces positive y scroll
    // W3C spec: positive deltaY = wheel moved away from user = content scrolls down
    let (_x, y) = normalize_wheel_delta(0.0, 100.0, 0);
    assert!(y > 0.0, "positive deltaY should produce positive scroll");

    // Test that negative wheel deltaY (scroll up) produces negative y scroll
    let (_x, y) = normalize_wheel_delta(0.0, -100.0, 0);
    assert!(y < 0.0, "negative deltaY should produce negative scroll");

    // Test that deltaMode 1 (lines) is normalized to pixels
    let (_x, y) = normalize_wheel_delta(0.0, 1.0, 1);
    assert!(
        y > 0.0,
        "line mode should produce positive scroll for positive delta"
    );

    // Test that deltaMode 2 (pages) is normalized to pixels
    let (_x, y) = normalize_wheel_delta(0.0, 1.0, 2);
    assert!(
        y > 0.0,
        "page mode should produce positive scroll for positive delta"
    );

    // Test horizontal scroll
    let (x, y) = normalize_wheel_delta(50.0, 0.0, 0);
    assert!(x > 0.0, "positive deltaX should produce positive x scroll");
    assert_eq!(y, 0.0, "deltaY should be zero");
}
