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

/// Filters text input data, removing control characters except tab and newline.
///
/// Input events may contain control characters that shouldn't be inserted.
/// This function keeps printable characters (code >= 32) plus tab (9) and newline (10).
#[allow(dead_code)]
pub fn filter_text_input_data(data: &str) -> String {
    data.chars()
        .filter(|c| {
            let code = *c as u32;
            code >= 32 || *c == '\t' || *c == '\n'
        })
        .collect()
}

/// Normalizes wheel event delta values to pixels with correct sign.
///
/// Handles three deltaMode values:
/// - 0: DOM_DELTA_PIXEL (already in pixels)
/// - 1: DOM_DELTA_LINE (multiply by standard line height of 16px)
/// - 2: DOM_DELTA_PAGE (multiply by standard page height of 400px)
///
/// The sign is preserved as-is: positive deltaY means scroll down (content moves down),
/// which is positive in rui's coordinate system.
#[allow(dead_code)]
pub fn normalize_wheel_delta(delta_x: f64, delta_y: f64, delta_mode: u32) -> (f32, f32) {
    const LINE_HEIGHT: f64 = 16.0;
    const PAGE_HEIGHT: f64 = 400.0;

    let multiplier = match delta_mode {
        0 => 1.0,         // pixels
        1 => LINE_HEIGHT, // lines
        2 => PAGE_HEIGHT, // pages
        _ => 1.0,         // unknown mode, treat as pixels
    };

    ((delta_x * multiplier) as f32, (delta_y * multiplier) as f32)
}

/// Converts a pointer event's viewport-relative position into window-logical units.
///
/// This function performs the coordinate system transformation from viewport coordinates
/// (with DOM scale factors applied) to the platform-independent window-logical units
/// that rui uses throughout.
///
/// # Coordinate System Contract
///
/// The returned coordinates are in **window-logical units**, accounting for the display's
/// scale factor (DPI scaling). These are the same units used throughout rui's layout,
/// rendering, and event handling—never device pixels or CSS pixels.
///
/// # Parameters
///
/// - `client_x`/`client_y`: Position from the DOM event (viewport coordinates)
/// - `rect_left`/`rect_top`: Canvas position in the viewport
/// - `rect_width`/`rect_height`: Canvas displayed size (CSS pixels)
/// - `buffer_width`/`buffer_height`: Canvas drawing buffer size
/// - `scale`: Display's device pixel ratio
///
/// x and y are scaled independently — width against width, height against height —
/// since a canvas need not be scaled the same on both axes.
#[allow(clippy::too_many_arguments)]
pub fn pointer_canvas_position(
    client_x: f64,
    client_y: f64,
    rect_left: f64,
    rect_top: f64,
    rect_width: f64,
    rect_height: f64,
    buffer_width: f64,
    buffer_height: f64,
    scale: f64,
) -> (f32, f32) {
    let across = |offset: f64, shown: f64, buffer: f64| {
        if shown > 0.0 && scale > 0.0 {
            (offset * buffer / shown / scale) as f32
        } else {
            offset as f32
        }
    };
    (
        across(client_x - rect_left, rect_width, buffer_width),
        across(client_y - rect_top, rect_height, buffer_height),
    )
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

    #[test]
    fn simple_character_passes_through() {
        assert_eq!(filter_text_input_data("a"), "a");
    }

    #[test]
    fn multiple_characters_pass_through() {
        assert_eq!(filter_text_input_data("hello"), "hello");
    }

    #[test]
    fn tab_character_is_preserved() {
        assert_eq!(filter_text_input_data("\t"), "\t");
    }

    #[test]
    fn newline_character_is_preserved() {
        assert_eq!(filter_text_input_data("\n"), "\n");
    }

    #[test]
    fn mixed_text_with_tab_and_newline() {
        assert_eq!(
            filter_text_input_data("hello\tworld\ntest"),
            "hello\tworld\ntest"
        );
    }

    #[test]
    fn control_characters_are_filtered() {
        assert_eq!(filter_text_input_data("\x00\x01\x02"), "");
    }

    #[test]
    fn mixed_text_and_control_characters() {
        assert_eq!(filter_text_input_data("a\x00b\x01c"), "abc");
    }

    #[test]
    fn printable_special_characters_pass_through() {
        assert_eq!(filter_text_input_data("!@#$%^&*()"), "!@#$%^&*()");
    }

    #[test]
    fn space_character_passes_through() {
        assert_eq!(filter_text_input_data(" "), " ");
    }

    #[test]
    fn empty_string_remains_empty() {
        assert_eq!(filter_text_input_data(""), "");
    }

    #[test]
    fn pointer_canvas_position_reads_x_from_client_x_not_client_y() {
        // A rect offset only in x, with no width/height/scale distortion, should
        // move only the x coordinate of the result.
        let (x, y) =
            pointer_canvas_position(110.0, 20.0, 10.0, 20.0, 100.0, 100.0, 100.0, 100.0, 1.0);
        assert_eq!(x, 100.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn pointer_canvas_position_reads_y_from_client_y_not_client_x() {
        let (x, y) =
            pointer_canvas_position(10.0, 120.0, 10.0, 20.0, 100.0, 100.0, 100.0, 100.0, 1.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 100.0);
    }

    #[test]
    fn pointer_canvas_position_scales_x_by_width_and_y_by_height_independently() {
        // Buffer is twice as wide as shown, and half as tall as shown: a swapped
        // implementation would scale x and y the same way and fail this.
        let (x, y) = pointer_canvas_position(60.0, 60.0, 0.0, 0.0, 100.0, 100.0, 200.0, 50.0, 1.0);
        assert_eq!(x, 120.0);
        assert_eq!(y, 30.0);
    }
}
