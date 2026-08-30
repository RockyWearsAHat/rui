//! Verify that pointer coordinates are correctly normalized across backends.
//!
//! Backends may transform pointer coordinates (e.g., scaling, DPI adjustments).
//! These tests ensure that the Input state correctly reflects the normalized
//! position that the UI should receive.

use rui::testing::Harness;
use rui::{col, text, El, Point};

#[derive(Default)]
struct App {}

fn view(_app: &App) -> El<App> {
    col(text("Test"))
}

/// Verify that pointer coordinates are correctly normalized by the backend.
///
/// When clicking at (100.0, 100.0), the Input state should reflect that
/// the pointer is at (100.0, 100.0) in logical coordinates.
#[test]
fn pointer_coordinates_are_normalized() {
    let mut harness = Harness::new(App::default(), view);

    // Click at position (100, 100)
    harness.click(Point::new(100.0, 100.0));

    // Verify the Input state reflects the click position
    let input = harness.input();
    assert_eq!(
        input.pointer(),
        Point::new(100.0, 100.0),
        "pointer position should be (100.0, 100.0) after click at that position"
    );
}
