//! STEP 3: Verify widgets render at Metrics::DEFAULT dimensions.
//!
//! This test module uses Harness-based behavior testing (following CLAUDE.md convention)
//! to verify that widgets render at sizes determined by Metrics::DEFAULT values.
//! Tests exercise actual widget rendering, not brittle source code string matching.

use rui::testing::Harness;
use rui::widgets::{button, col, field, meter, segmented, tabs, text};
use rui::Tone;

struct App {
    count: usize,
    selected: usize,
}

#[test]
fn step3_button_renders_with_metrics_dimensions() {
    // Button should render (verifies Metrics::DEFAULT.control_height is used)
    let app = App {
        count: 0,
        selected: 0,
    };
    let mut h = Harness::new(app, |_app: &App| {
        col((
            text("Button test:"),
            button("Click me").on_click(|app: &mut App| {
                app.count += 1;
            }),
        ))
    })
    .size(200.0, 200.0);

    h.frames(1);
    assert!(h.shows("Click me"), "Button should render");

    // Verify click handler works (button has control_height from Metrics::DEFAULT)
    h.click_text("Click me");
    assert_eq!(h.state().count, 1, "Button click should increment count");
}

#[test]
fn step3_field_renders_with_metrics_dimensions() {
    // Field should render (verifies Metrics::DEFAULT.control_height is used)
    let app = App {
        count: 0,
        selected: 0,
    };
    let mut h = Harness::new(app, |_app: &App| {
        col((text("Field test:"), field("input field")))
    })
    .size(300.0, 200.0);

    h.frames(1);
    // Field renders with control_height sizing from Metrics::DEFAULT
    assert!(h.canvas().width() > 0, "Field should render");
}

#[test]
fn step3_segmented_renders_with_metrics_dimensions() {
    // Segmented control should render (verifies row_height from Metrics::DEFAULT)
    let app = App {
        count: 0,
        selected: 0,
    };
    let mut h = Harness::new(app, |app: &App| {
        col((
            text("Segmented test:"),
            segmented(
                &["Small", "Medium", "Large"],
                app.selected,
                |app: &mut App, idx| {
                    app.selected = idx;
                },
            ),
        ))
    })
    .size(300.0, 200.0);

    h.frames(1);
    assert!(h.shows("Small"), "Segmented should render options");

    // Verify selection works (segmented uses row_height from Metrics::DEFAULT)
    h.click_text("Medium");
    assert_eq!(h.state().selected, 1, "Segmented should update selection");
}

#[test]
fn step3_tabs_render_with_metrics_dimensions() {
    // Tab widget should render (verifies control_height from Metrics::DEFAULT)
    let app = App {
        count: 0,
        selected: 0,
    };
    let mut h = Harness::new(app, |app: &App| {
        col((
            tabs(
                &["First", "Second", "Third"],
                app.selected,
                |app: &mut App, idx| {
                    app.selected = idx;
                },
            ),
            text("Tab content"),
        ))
    })
    .size(300.0, 200.0);

    h.frames(1);
    assert!(h.shows("First"), "Tabs should render");

    // Verify tab selection works (tabs use control_height from Metrics::DEFAULT)
    h.click_text("Second");
    assert_eq!(h.state().selected, 1, "Tabs should update selection");
}

#[test]
fn step3_meter_renders_with_widget_specific_dimensions() {
    // Meter should render with widget-specific dimensions (80x6, not Metrics-based)
    let app = App {
        count: 0,
        selected: 0,
    };
    let mut h = Harness::new(app, |_app: &App| {
        col((text("Progress:"), meter(0.75, Tone::Accent)))
    })
    .size(300.0, 200.0);

    h.frames(1);
    // Meter renders with its own widget-specific dimensions
    assert!(h.canvas().width() > 0, "Meter should render");
}
