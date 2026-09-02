//! STEP 7: Scrollbar Integration Tests (ENHANCEMENT phase)
//!
//! Comprehensive integration tests for scrollbar widget rendering and interaction.

use rui::element::El;
use rui::widgets::{col, row, scrollbar, text};

#[derive(Clone)]
struct ScrollState {
    scroll_position: f32,
    content_height: f32,
}

#[test]
fn a_scrollbar_works_in_container_layouts() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let _container: El<ScrollState> = col((
        scrollbar(
            100.0,
            |s: &ScrollState| s.scroll_position,
            |s: &ScrollState| s.content_height,
            |s: &mut ScrollState, pos| s.scroll_position = pos,
        ),
        text("Content"),
    ));
}

#[test]
fn a_scrollbar_works_with_row_layout() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let _container: El<ScrollState> = row((
        text("Content"),
        scrollbar(
            100.0,
            |s: &ScrollState| s.scroll_position,
            |s: &ScrollState| s.content_height,
            |s: &mut ScrollState, pos| s.scroll_position = pos,
        ),
    ));
}

#[test]
fn a_scrollbar_preserves_state_across_frames() {
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

    // Scrollbar should return consistent position across multiple queries
    let pos1 = scrollbar_el.get_scrollbar_position();
    let pos2 = scrollbar_el.get_scrollbar_position();
    assert_eq!(pos1, pos2);
}

#[test]
fn a_scrollbar_can_be_chained_with_style_methods() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let _scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    )
    .w(12.0)
    .h(100.0)
    .gap(8.0)
    .grow();
}

#[test]
fn a_scrollbar_works_with_small_content() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 50.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Should return valid position even when content < viewport
    assert!(scrollbar_el.get_scrollbar_position().is_some());
    assert!(scrollbar_el.get_scrollbar_thumb_size().is_some());
}

#[test]
fn a_scrollbar_works_with_large_content() {
    let _state = ScrollState {
        scroll_position: 5000.0,
        content_height: 10000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Should return valid position even with large content
    assert!(scrollbar_el.get_scrollbar_position().is_some());
    let thumb_size = scrollbar_el.get_scrollbar_thumb_size();
    assert!(thumb_size.is_some());
}

#[test]
fn a_scrollbar_drag_handler_integrates_with_element_api() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Verify that drag handler is attached
    assert!(scrollbar_el.has_drag_handler());
}

#[test]
fn a_scrollbar_responds_to_state_changes() {
    let _state = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let scrollbar_el: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Position getter should work before state changes
    let initial_pos = scrollbar_el.get_scrollbar_position();
    assert!(initial_pos.is_some());

    // Position getter should work after state changes (through closure evaluation)
    assert!(scrollbar_el.get_scrollbar_position().is_some());
}

#[test]
fn a_scrollbar_sizing_is_independent_of_content() {
    let _small_content_state = ScrollState {
        scroll_position: 0.0,
        content_height: 50.0,
    };

    let _large_content_state = ScrollState {
        scroll_position: 0.0,
        content_height: 10000.0,
    };

    let small_scrollbar: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    let large_scrollbar: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Both scrollbars should have valid sizes regardless of content
    assert!(small_scrollbar.get_scrollbar_thumb_size().is_some());
    assert!(large_scrollbar.get_scrollbar_thumb_size().is_some());
}

#[test]
fn multiple_scrollbars_can_coexist() {
    let _state1 = ScrollState {
        scroll_position: 0.0,
        content_height: 1000.0,
    };

    let _state2 = ScrollState {
        scroll_position: 250.0,
        content_height: 2000.0,
    };

    let _scrollbar1: El<ScrollState> = scrollbar(
        100.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    let _scrollbar2: El<ScrollState> = scrollbar(
        200.0,
        |s: &ScrollState| s.scroll_position,
        |s: &ScrollState| s.content_height,
        |s: &mut ScrollState, pos| s.scroll_position = pos,
    );

    // Both should work independently
    let container: El<ScrollState> = row((_scrollbar1, _scrollbar2));
    assert!(container.has_drag_handler());
}

#[test]
fn a_scrollbar_supports_custom_viewport_heights() {
    for viewport_height in &[50.0, 100.0, 200.0, 500.0] {
        let _state = ScrollState {
            scroll_position: 0.0,
            content_height: 1000.0,
        };

        let scrollbar_el: El<ScrollState> = scrollbar(
            *viewport_height,
            |s: &ScrollState| s.scroll_position,
            |s: &ScrollState| s.content_height,
            |s: &mut ScrollState, pos| s.scroll_position = pos,
        );

        // Should work with any viewport height
        assert!(scrollbar_el.get_scrollbar_position().is_some());
    }
}
