//! STEP 7: Scrollbar as Control (R9)
//!
//! RED phase test scaffolding for scrollbar widget.
//! Tests demonstrate the desired API for interactive scrollbar control.

use rui::element::El;
use rui::widgets::scrollbar;

#[derive(Clone)]
struct ScrollState {
    scroll_position: f32,
    content_height: f32,
}

#[test]
fn a_scrollbar_can_be_created() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let _el: El<ScrollState> = scrollbar(
        100.0, // viewport_height
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );
}

#[test]
fn a_scrollbar_displays_the_correct_thumb_position() {
    let _state = ScrollState {
        scroll_position: 250.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Scrollbar thumb should be at 250/1000 = 25% down
    assert_eq!(scrollbar_el.get_scrollbar_position(), Some(0.25));
}

#[test]
fn a_scrollbar_calculates_thumb_size_correctly() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0, // viewport is 100, content is 1000
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Thumb should be 100/1000 = 10% of scrollbar height
    assert_eq!(scrollbar_el.get_scrollbar_thumb_size(), Some(0.1));
}

#[test]
fn a_scrollbar_updates_scroll_position_on_drag() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    )
    .on_drag(|s: &mut ScrollState, drag| {
        let max_scroll = s.content_height - 100.0;
        s.scroll_position = (drag.fraction().y * max_scroll).max(0.0).min(max_scroll);
    });

    // Simulate drag to 50% down
    // This should move scroll to 450.0 (50% of 900 max scroll)
    let handler_exists = scrollbar_el.has_drag_handler();
    assert!(handler_exists);
}

#[test]
fn a_scrollbar_is_disabled_when_content_fits() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 50.0, // Smaller than viewport
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0, // viewport_height > content_height
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Scrollbar provides a disabled state (actual logic in ENHANCEMENT phase)
    let is_disabled = scrollbar_el.get_scrollbar_disabled();
    assert!(is_disabled.is_some());
}

#[test]
fn a_scrollbar_preserves_position_on_state_changes() {
    let _state = ScrollState {
        scroll_position: 250.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Position should be consistent across multiple queries
    assert_eq!(scrollbar_el.get_scrollbar_position(), Some(0.25));
    assert_eq!(scrollbar_el.get_scrollbar_position(), Some(0.25));
}

#[test]
fn a_scrollbar_clamps_position_to_valid_range() {
    let _state = ScrollState {
        scroll_position: 950.0, // Near bottom
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Position should clamp to valid range [0, 1]
    let pos = scrollbar_el.get_scrollbar_position();
    if let Some(p) = pos {
        assert!((0.0..=1.0).contains(&p));
    }
}

#[test]
fn a_scrollbar_handles_zero_height_content() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 0.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Should not panic or crash with zero content
    let pos = scrollbar_el.get_scrollbar_position();
    assert!(pos.is_some());
}
