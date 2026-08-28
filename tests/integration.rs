//! Integration tests for widget combinations.

use rui::testing::Harness;
use rui::{col, text, Align};

/// State for split pane test.
#[derive(Default, Clone)]
struct SplitState {
    split_ratio: f32,
}

#[test]
fn split_pane_divider_appears_and_is_draggable() {
    let mut harness = Harness::new(SplitState { split_ratio: 0.5 }, |state: &SplitState| {
        rui::split(
            col(text("Left pane")).align(Align::Start),
            col(text("Right pane")).align(Align::Start),
            state.split_ratio,
            |state: &mut SplitState, ratio: f32| {
                state.split_ratio = ratio;
            },
        )
    })
    .size(400.0, 200.0);

    // Draw initial frame
    harness.frame();

    // Find the split divider by key
    let divider = harness
        .find_key("split_divider")
        .expect("divider should be on screen");

    // Verify divider is at 50% initially
    // Container 400px, divider 4px, so left gets (400-4)*0.5 = 198px
    let expected_x = 198.0;
    assert!(
        (divider.rect.x - expected_x).abs() < 1.0,
        "divider should be at 50% position, got x={}",
        divider.rect.x
    );

    // Drag the divider to 70% (280px)
    harness.drag(divider.rect.center(), rui::Point::new(280.0, 100.0));

    // Verify the state was updated
    assert!(
        (harness.state().split_ratio - 0.7).abs() < 0.01,
        "split ratio should be ~0.7 after dragging to 280px, got {}",
        harness.state().split_ratio
    );

    // Redraw and verify divider is at new position
    harness.frame();
    let divider = harness
        .find_key("split_divider")
        .expect("divider should still be on screen");
    assert!(
        (divider.rect.x - 280.0).abs() < 1.0,
        "divider should be at 70% position after drag"
    );
}
