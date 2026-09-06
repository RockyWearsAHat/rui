//! Tooltip component tests.

use rui::testing::Harness;
use rui::{button, tooltip, El};

/// Simple state for tooltip tests.
#[derive(Default)]
struct State {
    open: bool,
    hover: bool,
}

/// A button with a tooltip.
fn view(state: &State) -> El<State> {
    tooltip(
        button("Hover me"),
        "This is a tooltip",
        state.open,
        |state: &mut State, hover| {
            state.hover = hover;
            state.open = hover;
        },
    )
}

#[test]
fn tooltip_is_absent_until_open() {
    let mut harness = Harness::new(State::default(), view);
    harness.frame();

    // When open is false, the tooltip bubble should not exist
    assert!(
        harness.find_key("tooltip").is_none(),
        "tooltip bubble should not exist when open=false"
    );
}

#[test]
fn tooltip_bubble_is_layered_when_open() {
    let mut harness = Harness::new(
        State {
            open: true,
            ..Default::default()
        },
        view,
    );
    harness.frame();

    // When open is true, the tooltip should be layered
    let probe = harness
        .find_key("tooltip")
        .expect("tooltip bubble should exist when open=true");
    assert!(
        probe.layered,
        "tooltip bubble should have layered=true when open=true"
    );
}

#[test]
fn tooltip_names_its_child_even_when_closed() {
    let mut harness = Harness::new(State::default(), view);
    harness.frame();

    // The accessible names should include the tooltip label even when closed
    let names = harness.accessible_names();
    assert!(
        names.iter().any(|n| n.contains("This is a tooltip")),
        "tooltip label should be in accessible names even when closed: {:?}",
        names
    );
}

#[test]
fn tooltip_hover_reports_true_then_false() {
    let mut harness = Harness::new(State::default(), view);
    harness.frame();

    // Find the button
    let button_probe = harness
        .find_key("Hover me")
        .or_else(|| harness.find_key("Hover me"))
        .or_else(|| {
            // Try to click the button to find it
            for probe in harness.probes() {
                if let Some(text) = &probe.text {
                    if text.contains("Hover me") {
                        return Some(probe.clone());
                    }
                }
            }
            None
        })
        .expect("should find the button");

    // Move pointer over the button
    harness.move_pointer(button_probe.rect.center());
    harness.frame();

    // Check that state.open is now true
    assert!(harness.state().open, "tooltip should be open after hover");

    // Move pointer away
    harness.move_pointer(rui::Point::new(-100.0, -100.0));
    harness.frame();

    // Check that state.open is now false
    assert!(
        !harness.state().open,
        "tooltip should be closed after hover leave"
    );
}

#[test]
fn tooltip_is_never_focusable() {
    let mut harness = Harness::new(
        State {
            open: true,
            ..Default::default()
        },
        view,
    );
    harness.frame();

    // The tooltip bubble should never be focusable
    let probe = harness
        .find_key("tooltip")
        .expect("tooltip bubble should exist when open=true");
    assert!(!probe.focusable, "tooltip bubble should never be focusable");
}
