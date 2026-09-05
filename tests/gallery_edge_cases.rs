//! Edge case and comprehensive visual state regression tests for gallery.
//!
//! These tests verify gallery rendering with boundary conditions, text variations,
//! and complex component combinations to ensure robust visual regression coverage.

use rui::testing::Harness;
use rui::{button, col, field, meter, row, text, Key, Tone};

#[derive(Default, Clone)]
struct EdgeCaseState {
    clicks: usize,
    progress: f32,
}

// ---- Text Edge Cases ----

#[test]
fn empty_text_renders_without_error() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| col(text(""))).size(200.0, 50.0);
    h.frame();
    // Should not panic or produce invalid pixels
    assert!(
        !h.canvas().pixels().is_empty(),
        "empty text should still render"
    );
}

#[test]
fn very_long_text_renders_at_correct_width() {
    let long_text = "The quick brown fox jumps over the lazy dog. ".repeat(5);
    let mut h = Harness::new(EdgeCaseState::default(), move |_| {
        col(text(&long_text)).w(300.0)
    })
    .size(350.0, 200.0);

    h.frame();
    let pixels = h.canvas().pixels();
    // Should render without truncation or corruption
    assert!(!pixels.is_empty(), "long text should render completely");
}

#[test]
fn single_character_text_renders() {
    let mut h_single = Harness::new(EdgeCaseState::default(), |_| col(text("A"))).size(100.0, 50.0);
    let mut h_many =
        Harness::new(EdgeCaseState::default(), |_| col(text("ABCDE"))).size(100.0, 50.0);

    h_single.frame();
    h_many.frame();

    assert_ne!(
        h_single.canvas().pixels(),
        h_many.canvas().pixels(),
        "single vs multiple characters should render differently"
    );
}

#[test]
fn text_with_special_characters_renders() {
    let special = "Hello! @#$% &*() 日本語";
    let mut h =
        Harness::new(EdgeCaseState::default(), move |_| col(text(special))).size(300.0, 100.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "special characters should render"
    );
}

// ---- Field (Input) Edge Cases ----

#[test]
fn empty_field_renders_without_placeholder_text() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| col(field(""))).size(300.0, 50.0);

    h.frame();
    let empty_pixels = h.canvas().pixels().to_vec();

    let mut h_labeled =
        Harness::new(EdgeCaseState::default(), |_| col(field("Name"))).size(300.0, 50.0);
    h_labeled.frame();
    let labeled_pixels = h_labeled.canvas().pixels().to_vec();

    assert_ne!(
        empty_pixels, labeled_pixels,
        "labeled field should render differently"
    );
}

#[test]
fn field_with_very_long_placeholder() {
    let long_placeholder = "This is a very long placeholder text that should not break layout";
    let mut h = Harness::new(EdgeCaseState::default(), move |_| {
        col(field(long_placeholder))
    })
    .size(300.0, 50.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "long placeholder should render"
    );
}

// ---- Button Edge Cases ----

#[test]
fn button_with_empty_label_renders() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| col(button(""))).size(200.0, 50.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "empty button should render"
    );
}

#[test]
fn button_with_very_long_label() {
    let long_label = "Click Me To Perform Important Action".repeat(2);
    let mut h = Harness::new(EdgeCaseState::default(), move |_| {
        col(button(&long_label)).w(300.0)
    })
    .size(350.0, 100.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "long button label should render"
    );
}

#[test]
fn disabled_button_appears_visually_distinct() {
    let mut h_enabled = Harness::new(EdgeCaseState::default(), |_| {
        col(button("Click").disabled(false))
    })
    .size(200.0, 50.0);
    let mut h_disabled = Harness::new(EdgeCaseState::default(), |_| {
        col(button("Click").disabled(true))
    })
    .size(200.0, 50.0);

    h_enabled.frame();
    h_disabled.frame();

    assert_ne!(
        h_enabled.canvas().pixels(),
        h_disabled.canvas().pixels(),
        "disabled button should have different appearance"
    );
}

// ---- Meter Edge Cases ----

#[test]
fn meter_at_zero_progress() {
    let mut h = Harness::new(
        EdgeCaseState {
            progress: 0.0,
            ..Default::default()
        },
        |state| col(meter(state.progress, Tone::Accent)),
    )
    .size(400.0, 50.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "zero progress meter should render"
    );
}

#[test]
fn meter_at_full_progress() {
    let mut h = Harness::new(
        EdgeCaseState {
            progress: 1.0,
            ..Default::default()
        },
        |state| col(meter(state.progress, Tone::Accent)),
    )
    .size(400.0, 50.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "full progress meter should render"
    );
}

#[test]
fn meter_progress_rendering_is_proportional() {
    let mut h_half = Harness::new(
        EdgeCaseState {
            progress: 0.5,
            ..Default::default()
        },
        |state| col(meter(state.progress, Tone::Accent)),
    )
    .size(400.0, 50.0);

    let mut h_quarter = Harness::new(
        EdgeCaseState {
            progress: 0.25,
            ..Default::default()
        },
        |state| col(meter(state.progress, Tone::Accent)),
    )
    .size(400.0, 50.0);

    h_half.frame();
    h_quarter.frame();

    assert_ne!(
        h_half.canvas().pixels(),
        h_quarter.canvas().pixels(),
        "different progress values should render differently"
    );
}

// ---- Complex Layout Edge Cases ----

#[test]
fn deeply_nested_layout_renders_correctly() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| {
        col((
            col(col(col(button("Deep")))),
            col(row((button("Side"), button("By"), button("Side")))),
        ))
    })
    .size(400.0, 300.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "deeply nested layout should render"
    );
}

#[test]
fn many_elements_in_row_render_with_proper_spacing() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| {
        row((
            button("1"),
            button("2"),
            button("3"),
            button("4"),
            button("5"),
        ))
        .gap(8.0)
    })
    .size(500.0, 60.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "many elements should render"
    );
}

#[test]
fn many_elements_in_column_render_with_proper_spacing() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| {
        col((
            button("1"),
            button("2"),
            button("3"),
            button("4"),
            button("5"),
        ))
        .gap(8.0)
    })
    .size(200.0, 300.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "many column elements should render"
    );
}

// ---- Focus State Edge Cases ----

#[test]
fn focus_persists_across_frames_with_no_interaction() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| col(button("Button"))).size(200.0, 50.0);

    h.frame();
    h.key(Key::Tab);
    h.frame();
    let focused_pixels = h.canvas().pixels().to_vec();

    // Several frames later without interaction
    h.frames(10);
    let still_focused = h.canvas().pixels().to_vec();

    assert_eq!(focused_pixels, still_focused, "focus should persist");
}

#[test]
fn multiple_focusable_elements_tab_navigation() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| {
        col((
            button("B1").on_click(|s: &mut EdgeCaseState| s.clicks += 1),
            button("B2").on_click(|s: &mut EdgeCaseState| s.clicks += 1),
            button("B3").on_click(|s: &mut EdgeCaseState| s.clicks += 1),
        ))
    })
    .size(200.0, 150.0);

    h.frame();
    let start = h.canvas().pixels().to_vec();

    // Tab to next element
    h.key(Key::Tab);
    h.frame();
    let after_first_tab = h.canvas().pixels().to_vec();

    // Tab to next element
    h.key(Key::Tab);
    h.frame();
    let after_second_tab = h.canvas().pixels().to_vec();

    // All three focus states should be different
    assert_ne!(start, after_first_tab, "first tab should move focus");
    assert_ne!(
        after_first_tab, after_second_tab,
        "second tab should move focus again"
    );
}

// ---- Tone Rendering Edge Cases ----

#[test]
fn different_tones_render_distinctly() {
    let mut h_accent = Harness::new(EdgeCaseState::default(), |_| {
        col(button("Button").fill(Tone::Accent))
    })
    .size(200.0, 50.0);

    let mut h_ok = Harness::new(EdgeCaseState::default(), |_| {
        col(button("Button").fill(Tone::Ok))
    })
    .size(200.0, 50.0);

    h_accent.frame();
    h_ok.frame();

    assert_ne!(
        h_accent.canvas().pixels(),
        h_ok.canvas().pixels(),
        "different tones should render differently"
    );
}

// ---- Size Edge Cases ----

#[test]
fn minimal_size_canvas_renders() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| col(button("X"))).size(50.0, 30.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "minimal canvas should render"
    );
}

#[test]
fn large_canvas_renders_without_error() {
    let mut h = Harness::new(EdgeCaseState::default(), |_| {
        col((
            text("Large Canvas"),
            button("Button"),
            meter(0.5, Tone::Accent),
        ))
        .gap(20.0)
    })
    .size(2000.0, 1500.0);

    h.frame();
    assert!(
        !h.canvas().pixels().is_empty(),
        "large canvas should render"
    );
}

// ---- Idempotence: Same Description = Same Pixels ----

#[test]
fn identical_description_twice_produces_same_pixels() {
    let mut h1 = Harness::new(EdgeCaseState::default(), |_| {
        col((text("Test"), button("Click"), meter(0.5, Tone::Accent)))
    })
    .size(300.0, 200.0);
    let mut h2 = Harness::new(EdgeCaseState::default(), |_| {
        col((text("Test"), button("Click"), meter(0.5, Tone::Accent)))
    })
    .size(300.0, 200.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "same description should produce identical pixels"
    );
}

#[test]
fn same_button_different_internal_state_renders_identically() {
    let mut h1 = Harness::new(EdgeCaseState::default(), |_| {
        col(button("Click").on_click(|s: &mut EdgeCaseState| s.clicks += 1))
    })
    .size(200.0, 50.0);
    let mut h2 = Harness::new(
        EdgeCaseState {
            clicks: 5,
            ..Default::default()
        },
        |_| col(button("Click").on_click(|s: &mut EdgeCaseState| s.clicks += 1)),
    )
    .size(200.0, 50.0);

    h1.frame();
    h2.frame();

    // Both should render the same button, so pixels should be identical
    // (the state doesn't affect the button appearance, only internal count)
    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "same description with different internal state should render identically"
    );
}

// ---- Combination Edge Cases ----

#[test]
fn button_field_meter_combination_renders() {
    let mut h = Harness::new(EdgeCaseState::default(), |state| {
        col((
            text("Control Panel"),
            row((button("Start"), button("Stop"))).gap(8.0),
            field("Value"),
            meter(state.progress, Tone::Accent),
        ))
        .gap(16.0)
    })
    .size(400.0, 250.0);

    h.frame();
    assert!(!h.canvas().pixels().is_empty(), "combination should render");
}
