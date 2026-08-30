//! Backend consistency test suite.
//!
//! Verifies that event handling is consistent across all backends (native, WASM).

use rui::testing::Harness;
use rui::{col, text, Point};

#[derive(Clone, Debug, Default)]
struct App;

fn view(_app: &App) -> rui::El<App> {
    col(text("Backend Consistency Test"))
}

/// Verify pointer coordinates are normalized by backend.
/// When clicking at (100, 100), Input reflects that position.
#[test]
fn pointer_coordinates_are_normalized() {
    let mut harness = Harness::new(App, view);

    harness.click(Point::new(100.0, 100.0));

    let input = harness.input();
    assert_eq!(
        input.pointer(),
        Point::new(100.0, 100.0),
        "pointer should reflect click position"
    );
}
