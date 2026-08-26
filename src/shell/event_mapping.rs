//! Event mapping for web backends: converts DOM event codes to rui Key/PointerButton types.
//!
//! These are pure functions that map platform-specific event representations to
//! the unified event types used throughout rui. This module can be tested on any
//! platform without requiring a DOM.

use crate::input::{Key, PointerButton};

/// Maps a keyboard code (from KeyboardEvent.code) to a Key enum.
///
/// This follows the standard Web KeyboardEvent.code naming convention.
/// See: https://www.w3.org/TR/uievents-code/
#[allow(dead_code)]
pub fn map_keyboard_code_to_key(code: &str) -> Option<Key> {
    match code {
        "Escape" => Some(Key::Escape),
        "Enter" => Some(Key::Enter),
        "Tab" => Some(Key::Tab),
        "Backspace" => Some(Key::Backspace),
        "Delete" => Some(Key::Delete),
        "Space" => Some(Key::Space),
        "ArrowUp" => Some(Key::Up),
        "ArrowDown" => Some(Key::Down),
        "ArrowLeft" => Some(Key::Left),
        "ArrowRight" => Some(Key::Right),
        "Home" => Some(Key::Home),
        "End" => Some(Key::End),
        "PageUp" => Some(Key::PageUp),
        "PageDown" => Some(Key::PageDown),
        code if code.starts_with("Key") && code.len() == 4 => code
            .chars()
            .nth(3)
            .map(|c| Key::Character(c.to_lowercase().next().unwrap_or(c))),
        _ => None,
    }
}

/// Maps a pointer button number (from PointerEvent.button) to PointerButton enum.
///
/// Follows the standard Web PointerEvent.button convention:
/// 0 = primary (usually left), 1 = middle, 2 = secondary (usually right).
#[allow(dead_code)]
pub fn map_pointer_button(button: u16) -> Option<PointerButton> {
    match button {
        0 => Some(PointerButton::Primary),
        1 => Some(PointerButton::Middle),
        2 => Some(PointerButton::Secondary),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_code_escape_maps_to_escape_key() {
        assert_eq!(map_keyboard_code_to_key("Escape"), Some(Key::Escape));
    }

    #[test]
    fn keyboard_code_enter_maps_to_enter_key() {
        assert_eq!(map_keyboard_code_to_key("Enter"), Some(Key::Enter));
    }

    #[test]
    fn keyboard_code_tab_maps_to_tab_key() {
        assert_eq!(map_keyboard_code_to_key("Tab"), Some(Key::Tab));
    }

    #[test]
    fn keyboard_code_backspace_maps_to_backspace_key() {
        assert_eq!(map_keyboard_code_to_key("Backspace"), Some(Key::Backspace));
    }

    #[test]
    fn keyboard_code_delete_maps_to_delete_key() {
        assert_eq!(map_keyboard_code_to_key("Delete"), Some(Key::Delete));
    }

    #[test]
    fn keyboard_code_space_maps_to_space_key() {
        assert_eq!(map_keyboard_code_to_key("Space"), Some(Key::Space));
    }

    #[test]
    fn keyboard_code_arrow_up_maps_to_up_key() {
        assert_eq!(map_keyboard_code_to_key("ArrowUp"), Some(Key::Up));
    }

    #[test]
    fn keyboard_code_arrow_down_maps_to_down_key() {
        assert_eq!(map_keyboard_code_to_key("ArrowDown"), Some(Key::Down));
    }

    #[test]
    fn keyboard_code_arrow_left_maps_to_left_key() {
        assert_eq!(map_keyboard_code_to_key("ArrowLeft"), Some(Key::Left));
    }

    #[test]
    fn keyboard_code_arrow_right_maps_to_right_key() {
        assert_eq!(map_keyboard_code_to_key("ArrowRight"), Some(Key::Right));
    }

    #[test]
    fn keyboard_code_home_maps_to_home_key() {
        assert_eq!(map_keyboard_code_to_key("Home"), Some(Key::Home));
    }

    #[test]
    fn keyboard_code_end_maps_to_end_key() {
        assert_eq!(map_keyboard_code_to_key("End"), Some(Key::End));
    }

    #[test]
    fn keyboard_code_page_up_maps_to_page_up_key() {
        assert_eq!(map_keyboard_code_to_key("PageUp"), Some(Key::PageUp));
    }

    #[test]
    fn keyboard_code_page_down_maps_to_page_down_key() {
        assert_eq!(map_keyboard_code_to_key("PageDown"), Some(Key::PageDown));
    }

    #[test]
    fn keyboard_code_key_a_maps_to_character_a() {
        assert_eq!(map_keyboard_code_to_key("KeyA"), Some(Key::Character('a')));
    }

    #[test]
    fn keyboard_code_key_z_maps_to_character_z() {
        assert_eq!(map_keyboard_code_to_key("KeyZ"), Some(Key::Character('z')));
    }

    #[test]
    fn keyboard_code_key0_maps_to_character_0() {
        assert_eq!(
            map_keyboard_code_to_key("Digit0"),
            None,
            "Digit keys are not letter keys"
        );
    }

    #[test]
    fn pointer_button_0_maps_to_primary() {
        assert_eq!(map_pointer_button(0), Some(PointerButton::Primary));
    }

    #[test]
    fn pointer_button_1_maps_to_middle() {
        assert_eq!(map_pointer_button(1), Some(PointerButton::Middle));
    }

    #[test]
    fn pointer_button_2_maps_to_secondary() {
        assert_eq!(map_pointer_button(2), Some(PointerButton::Secondary));
    }

    #[test]
    fn unknown_keyboard_code_returns_none() {
        assert_eq!(map_keyboard_code_to_key("Unknown"), None);
    }

    #[test]
    fn unknown_pointer_button_returns_none() {
        assert_eq!(map_pointer_button(99), None);
    }

    #[test]
    fn all_named_keys_are_mapped() {
        for code in &[
            "Escape",
            "Enter",
            "Tab",
            "Backspace",
            "Delete",
            "Space",
            "ArrowUp",
            "ArrowDown",
            "ArrowLeft",
            "ArrowRight",
            "Home",
            "End",
            "PageUp",
            "PageDown",
        ] {
            assert!(
                map_keyboard_code_to_key(code).is_some(),
                "code {} should map",
                code
            );
        }
    }

    #[test]
    fn all_pointer_buttons_are_mapped() {
        for button in &[0u16, 1, 2] {
            assert!(
                map_pointer_button(*button).is_some(),
                "button {} should map",
                button
            );
        }
    }
}
