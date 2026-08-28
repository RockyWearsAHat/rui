//! Tests for WASM event handling, particularly pointer events with button mapping.
#![cfg(target_arch = "wasm32")]

use rui::input::PointerButton;
use rui::shell::event_mapping::map_pointer_button;

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
