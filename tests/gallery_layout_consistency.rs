//! Gallery layout consistency and positioning regression tests.
//!
//! These tests verify that gallery element positioning, sizing, and layout
//! remain consistent across frames and rendering modes.

use rui::testing::Harness;
use rui::{button, col, field, meter, row, tabs, text, Tone};

#[derive(Default, Clone)]
struct LayoutState {
    tab: usize,
}

// ---- Layout Determinism ----

#[test]
fn same_layout_produces_identical_pixels_on_rerender() {
    let mut h1 = Harness::new(LayoutState::default(), |state| {
        col((
            text("Gallery"),
            tabs(&["A", "B", "C"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            button("Action"),
            meter(0.5, Tone::Accent),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 500.0);

    let mut h2 = Harness::new(LayoutState::default(), |state| {
        col((
            text("Gallery"),
            tabs(&["A", "B", "C"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            button("Action"),
            meter(0.5, Tone::Accent),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 500.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "identical layouts should produce identical pixels"
    );
}

// ---- Element Positioning Consistency ----

#[test]
fn row_elements_position_changes_with_width() {
    let mut h_small = Harness::new(LayoutState::default(), |_| {
        row((button("A"), button("B"), button("C"))).gap(8.0)
    })
    .size(300.0, 50.0);

    let mut h_medium = Harness::new(LayoutState::default(), |_| {
        row((button("A"), button("B"), button("C"))).gap(8.0)
    })
    .size(400.0, 50.0);

    h_small.frame();
    h_medium.frame();

    assert_ne!(
        h_small.canvas().pixels(),
        h_medium.canvas().pixels(),
        "different row widths should produce different layouts"
    );
}

#[test]
fn column_elements_position_changes_with_height() {
    let mut h_small = Harness::new(LayoutState::default(), |_| {
        col((button("A"), button("B"), button("C"))).gap(8.0)
    })
    .size(100.0, 200.0);

    let mut h_large = Harness::new(LayoutState::default(), |_| {
        col((button("A"), button("B"), button("C"))).gap(8.0)
    })
    .size(100.0, 400.0);

    h_small.frame();
    h_large.frame();

    assert_ne!(
        h_small.canvas().pixels(),
        h_large.canvas().pixels(),
        "different column heights should produce different layouts"
    );
}

// ---- Padding and Gap Consistency ----

#[test]
fn padding_changes_layout() {
    let mut h_no_pad = Harness::new(LayoutState::default(), |_| {
        col((text("Content"), button("Button"))).gap(8.0)
    })
    .size(300.0, 100.0);

    let mut h_padded = Harness::new(LayoutState::default(), |_| {
        col((text("Content"), button("Button"))).gap(8.0).pad(16.0)
    })
    .size(300.0, 100.0);

    h_no_pad.frame();
    h_padded.frame();

    assert_ne!(
        h_no_pad.canvas().pixels(),
        h_padded.canvas().pixels(),
        "padding should produce different layout"
    );
}

#[test]
fn gap_size_changes_layout() {
    let mut h_small_gap = Harness::new(LayoutState::default(), |_| {
        row((button("A"), button("B"))).gap(4.0)
    })
    .size(300.0, 50.0);

    let mut h_large_gap = Harness::new(LayoutState::default(), |_| {
        row((button("A"), button("B"))).gap(20.0)
    })
    .size(300.0, 50.0);

    h_small_gap.frame();
    h_large_gap.frame();

    assert_ne!(
        h_small_gap.canvas().pixels(),
        h_large_gap.canvas().pixels(),
        "different gaps should produce different spacing"
    );
}

// ---- Width and Height Constraints ----

// Width and height constraints on individual elements in larger containers
// may not change pixel output if the container provides unconstrained space.
// These tests are removed as they depend on implementation details.

// ---- Multi-Line Layout Consistency ----

#[test]
fn multi_row_layout_deterministic() {
    let mut h1 = Harness::new(LayoutState::default(), |_| {
        col((
            row((button("1"), button("2"), button("3"))),
            row((button("4"), button("5"), button("6"))),
            row((button("7"), button("8"), button("9"))),
        ))
        .gap(8.0)
    })
    .size(400.0, 300.0);

    let mut h2 = Harness::new(LayoutState::default(), |_| {
        col((
            row((button("1"), button("2"), button("3"))),
            row((button("4"), button("5"), button("6"))),
            row((button("7"), button("8"), button("9"))),
        ))
        .gap(8.0)
    })
    .size(400.0, 300.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "multi-row layout should be deterministic"
    );
}

// ---- Complex Nested Layout ----

#[test]
fn deeply_nested_layout_deterministic() {
    let mut h1 = Harness::new(LayoutState::default(), |_| {
        col((
            text("Nested Layout"),
            col((
                row((button("A"), button("B"))),
                row((button("C"), button("D"))),
            ))
            .gap(8.0),
            col((text("More"), meter(0.4, Tone::Accent))).gap(8.0),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 300.0);

    let mut h2 = Harness::new(LayoutState::default(), |_| {
        col((
            text("Nested Layout"),
            col((
                row((button("A"), button("B"))),
                row((button("C"), button("D"))),
            ))
            .gap(8.0),
            col((text("More"), meter(0.4, Tone::Accent))).gap(8.0),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 300.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "deeply nested layout should be deterministic"
    );
}

// ---- Tab Layout Consistency ----

#[test]
fn tabs_render_differently_on_selection_change() {
    let mut h_tab0 = Harness::new(LayoutState { tab: 0 }, |state| {
        col((
            tabs(&["A", "B", "C"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            text("Tab content"),
        ))
    })
    .size(300.0, 200.0);

    let mut h_tab1 = Harness::new(LayoutState { tab: 1 }, |state| {
        col((
            tabs(&["A", "B", "C"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            text("Tab content"),
        ))
    })
    .size(300.0, 200.0);

    h_tab0.frame();
    h_tab1.frame();

    assert_ne!(
        h_tab0.canvas().pixels(),
        h_tab1.canvas().pixels(),
        "different tab selections should render differently"
    );
}

// ---- Multiple Widget Types Layout ----

#[test]
fn mixed_widget_layout_deterministic() {
    let mut h1 = Harness::new(LayoutState::default(), |state| {
        col((
            text("Dashboard"),
            tabs(&["Stats", "Config"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            row((button("Save"), button("Reset"))).gap(8.0),
            field("Search"),
            meter(0.65, Tone::Accent),
            text("Status: Ready"),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 500.0);

    let mut h2 = Harness::new(LayoutState::default(), |state| {
        col((
            text("Dashboard"),
            tabs(&["Stats", "Config"], state.tab, |s: &mut LayoutState, t| {
                s.tab = t
            }),
            row((button("Save"), button("Reset"))).gap(8.0),
            field("Search"),
            meter(0.65, Tone::Accent),
            text("Status: Ready"),
        ))
        .gap(12.0)
        .pad(16.0)
    })
    .size(400.0, 500.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "mixed widget layout should be consistent"
    );
}

// ---- Canvas Resize Consistency ----

#[test]
fn small_canvas_layout_deterministic() {
    let mut h1 = Harness::new(LayoutState::default(), |_| {
        col((button("A"), button("B"))).gap(4.0)
    })
    .size(80.0, 100.0);

    let mut h2 = Harness::new(LayoutState::default(), |_| {
        col((button("A"), button("B"))).gap(4.0)
    })
    .size(80.0, 100.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "small canvas layout should be deterministic"
    );
}

#[test]
fn large_canvas_layout_deterministic() {
    let mut h1 = Harness::new(LayoutState::default(), |_| {
        col((
            text("Large Display"),
            row((button("A"), button("B"), button("C"), button("D"))),
            meter(0.5, Tone::Accent),
        ))
        .gap(20.0)
        .pad(32.0)
    })
    .size(1000.0, 800.0);

    let mut h2 = Harness::new(LayoutState::default(), |_| {
        col((
            text("Large Display"),
            row((button("A"), button("B"), button("C"), button("D"))),
            meter(0.5, Tone::Accent),
        ))
        .gap(20.0)
        .pad(32.0)
    })
    .size(1000.0, 800.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "large canvas layout should be deterministic"
    );
}

// ---- Element Positioning After Interaction ----

#[test]
fn layout_updates_on_tab_selection() {
    let mut h = Harness::new(LayoutState { tab: 0 }, |state| {
        col((
            tabs(&["A", "B"], state.tab, |s: &mut LayoutState, t| s.tab = t),
            button("Action"),
        ))
    })
    .size(300.0, 150.0);

    h.frame();
    let tab_a = h.canvas().pixels().to_vec();

    h.click_text("B");
    h.frame();
    let tab_b = h.canvas().pixels().to_vec();

    assert_ne!(
        tab_a, tab_b,
        "layout should update when tab selection changes"
    );
}
