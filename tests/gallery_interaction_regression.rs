//! Interaction and state visual regression tests for gallery widgets.
//!
//! These tests verify that widgets respond correctly to user interactions
//! and state changes are reflected in the rendered output or handler execution.

use rui::testing::Harness;
use rui::{button, col, field, row, tabs};

#[derive(Default, Clone)]
struct InteractionState {
    tab: usize,
    clicks: usize,
}

// ---- Button Interaction States ----

#[test]
fn button_click_runs_handler() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        col(button("Click").on_click(|s: &mut InteractionState| s.clicks += 1))
    })
    .size(200.0, 100.0);

    h.frame();
    assert_eq!(h.state().clicks, 0, "no clicks initially");

    h.click_text("Click");
    assert_eq!(h.state().clicks, 1, "click handler should have run");
}

// ---- Tabs Interaction ----

#[test]
fn tab_selection_change_renders_differently() {
    let mut h = Harness::new(
        InteractionState {
            tab: 0,
            ..Default::default()
        },
        |state| {
            tabs(
                &["A", "B", "C"],
                state.tab,
                |s: &mut InteractionState, t| {
                    s.tab = t;
                },
            )
        },
    )
    .size(400.0, 50.0);
    h.frame();
    let tab_0 = h.canvas().pixels().to_vec();

    h.click_text("B");
    h.frame();
    let tab_1 = h.canvas().pixels().to_vec();

    assert_ne!(tab_0, tab_1, "tab selection should render differently");
    assert_eq!(h.state().tab, 1, "tab selection should update state");
}

// ---- Field (Input) Interaction ----

#[test]
fn field_focus_renders_focus_ring() {
    let mut h =
        Harness::new(InteractionState::default(), |_| col(field("Value"))).size(300.0, 50.0);
    h.frame();
    let unfocused = h.canvas().pixels().to_vec();

    // Tab to focus the field
    h.key(rui::Key::Tab);
    h.frame();
    let focused = h.canvas().pixels().to_vec();

    assert_ne!(
        unfocused, focused,
        "focused field should render with focus ring"
    );
}

#[test]
fn field_contains_text_rendered_visibly() {
    let mut h_empty =
        Harness::new(InteractionState::default(), |_| col(field(""))).size(300.0, 50.0);

    let mut h_filled =
        Harness::new(InteractionState::default(), |_| col(field("Some text"))).size(300.0, 50.0);

    h_empty.frame();
    h_filled.frame();

    assert_ne!(
        h_empty.canvas().pixels(),
        h_filled.canvas().pixels(),
        "field with text should render differently"
    );
}

// ---- Disabled State Interaction ----

#[test]
fn disabled_button_does_not_respond_to_clicks() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        col(button("Click")
            .on_click(|s: &mut InteractionState| s.clicks += 1)
            .disabled(true))
    })
    .size(200.0, 100.0);

    h.frame();
    h.click_text("Click");

    // Handler should not have run because button is disabled
    assert_eq!(
        h.state().clicks,
        0,
        "disabled button should not respond to clicks"
    );
}

#[test]
fn disabled_button_hover_does_not_animate() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        col(button("Click")
            .on_click(|_: &mut InteractionState| {})
            .disabled(true))
    })
    .size(200.0, 100.0);

    h.frame();

    // Try to hover disabled button
    h.move_pointer(rui::Point::new(100.0, 50.0));
    h.frame();

    // Disabled button should not animate on hover
    assert!(
        !h.is_animating(),
        "disabled button should not start animation"
    );
}

// ---- Tab Selection Persistence ----

#[test]
fn tab_selection_persists_across_frames() {
    let mut h = Harness::new(
        InteractionState {
            tab: 0,
            ..Default::default()
        },
        |state| {
            tabs(
                &["A", "B", "C"],
                state.tab,
                |s: &mut InteractionState, t| {
                    s.tab = t;
                },
            )
        },
    )
    .size(400.0, 50.0);

    h.frame();
    h.click_text("C");
    h.frame();
    let after_click = h.canvas().pixels().to_vec();

    // Several frames later, same tab should still be selected
    h.frames(10);
    let after_delay = h.canvas().pixels().to_vec();

    assert_eq!(h.state().tab, 2, "tab should remain selected");
    assert_eq!(
        after_click, after_delay,
        "tab selection should persist visually"
    );
}

// ---- Row/Col Layout Interaction ----

#[test]
fn row_with_buttons_all_interactive() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        row((
            button("Left").on_click(|s: &mut InteractionState| s.clicks += 1),
            button("Right").on_click(|s: &mut InteractionState| s.clicks += 1),
        ))
        .gap(8.0)
    })
    .size(400.0, 50.0);

    h.frame();
    h.click_text("Left");
    assert_eq!(h.state().clicks, 1);

    h.click_text("Right");
    assert_eq!(h.state().clicks, 2, "both buttons should be clickable");
}

// ---- Keyboard Navigation ----

#[test]
fn tab_key_navigates_between_focusable_elements() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        col((
            button("B1").on_click(|s: &mut InteractionState| s.clicks += 1),
            button("B2").on_click(|s: &mut InteractionState| s.clicks += 1),
        ))
    })
    .size(200.0, 100.0);

    h.frame();
    // Tab should navigate to first button
    h.key(rui::Key::Tab);
    h.frame();

    // Enter should click focused button
    h.key(rui::Key::Enter);
    assert_eq!(h.state().clicks, 1, "enter should activate focused button");
}

// ---- Multiple Clicks ----

#[test]
fn multiple_clicks_increment_state() {
    let mut h = Harness::new(InteractionState::default(), |_| {
        col(button("Inc").on_click(|s: &mut InteractionState| s.clicks += 1))
    })
    .size(200.0, 100.0);

    h.frame();
    for i in 1..=5 {
        h.click_text("Inc");
        assert_eq!(h.state().clicks, i, "click {} should increment", i);
    }
}

// ---- Tab Navigation with Arrow Key ----

#[test]
fn tab_click_updates_state() {
    let mut h = Harness::new(
        InteractionState {
            tab: 0,
            ..Default::default()
        },
        |state| {
            tabs(&["A", "B"], state.tab, |s: &mut InteractionState, t| {
                s.tab = t
            })
        },
    )
    .size(400.0, 50.0);

    h.frame();
    assert_eq!(h.state().tab, 0, "starts at first tab");

    // Click second tab
    h.click_text("B");
    h.frame();

    // Verify state changed
    assert_eq!(h.state().tab, 1, "click should navigate tabs");
}
