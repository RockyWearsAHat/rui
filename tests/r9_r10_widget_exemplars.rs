//! Widget exemplars: slider and radio button controls.
//!
//! These tests verify that slider and radio button can be built from primitives
//! with full keyboard and pointer support, state persistence, and accessibility
//! compliance. They serve as exemplars for building custom interactive controls.
//!
//! Proof pattern: Every exemplar is whole (copy into a project and use directly),
//! built from primitives with no framework magic, testable with Harness headless,
//! and fully keyboard-navigable and accessible.

use rui::testing::Harness;
use rui::{Align, Drag, El, Key, Modifiers, Painter, Radius, Size, Tone, col, draw, row, text};

#[derive(Clone, Copy, Default)]
struct ControlsState {
    volume: f32,
    format: usize,
}

/// A slider control: single-dimension numeric selector with drag and arrow key support.
///
/// Exemplar pattern: State (volume: f32) → view (slider) → handlers (on_drag + on_key).
/// Built from primitives: draw for bar, on_drag for pointer, on_key for arrows.
fn slider<S: 'static>(value: f32, set: impl Fn(&mut S, f32) + Copy + 'static) -> El<S> {
    const STEP: f32 = 0.05;
    let value = value.clamp(0.0, 1.0);
    draw(
        Size::new(160.0, 18.0),
        move |painter: &mut Painter<'_>, rect: rui::Rect| {
            painter.fill(rect, Radius::Pill, Tone::Sunken);
            painter.fill(
                rui::Rect::new(rect.x, rect.y, rect.w * value, rect.h),
                Radius::Pill,
                Tone::Accent,
            );
        },
    )
    .size(160.0, 18.0)
    .role(rui::Role::Slider)
    .label("Volume")
    .value(format!("{:.0}%", value * 100.0))
    .on_drag(move |state: &mut S, drag: Drag| set(state, drag.fraction().x))
    .on_key(move |state: &mut S, key: Key, _: Modifiers| match key {
        Key::Left => set(state, (value - STEP).max(0.0)),
        Key::Right => set(state, (value + STEP).min(1.0)),
        _ => {}
    })
}

/// A radio button group: mutually-exclusive choice selector.
///
/// Exemplar pattern: State (chosen: usize) → view (radio_group) → handler (choose).
/// Built from primitives: draw for button, text for label, on_click for selection.
fn radio_group<S: 'static>(
    labels: &[&str],
    chosen: usize,
    choose: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    col(labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let taken = index == chosen;
            row((
                draw(
                    Size::new(15.0, 15.0),
                    move |painter: &mut Painter<'_>, rect: rui::Rect| {
                        painter.fill(
                            rect,
                            Radius::Pill,
                            if taken { Tone::Accent } else { Tone::Sunken },
                        );
                    },
                )
                .size(15.0, 15.0),
                text(*label),
            ))
            .key(*label)
            .h(22.0)
            .gap(8.0)
            .align(Align::Center)
            .role(rui::Role::Radio)
            .selected(taken)
            .on_click(move |state: &mut S| choose(state, index))
        })
        .collect::<Vec<_>>())
}

#[test]
fn a_slider_moves_with_pointer_drag() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        col(slider(state.volume, |state: &mut ControlsState, v| {
            state.volume = v
        })
        .key("volume"))
        .align(Align::Start)
    });

    let rect = harness.find_key("volume").expect("slider exists").rect;
    harness.drag(
        rui::Point::new(rect.x + 40.0, rect.center().y),
        rui::Point::new(rect.x + 120.0, rect.center().y),
    );
    assert!(
        (harness.state().volume - 0.75).abs() < 0.01,
        "slider moves proportionally with pointer"
    );
}

#[test]
fn a_slider_can_be_keyboard_controlled_with_arrows() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        col(slider(state.volume, |state: &mut ControlsState, v| {
            state.volume = v
        }))
    });

    // Tab to focus the slider
    harness.tab();

    harness.key(Key::Right);
    assert!(harness.state().volume > 0.0, "right arrow increases volume");

    let mid = harness.state().volume;
    assert!(mid >= 0.05);

    harness.key(Key::Left);
    assert!(harness.state().volume < mid, "left arrow decreases volume");
}

#[test]
fn a_slider_clamps_values_to_0_1_range() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        col(slider(state.volume, |state: &mut ControlsState, v| {
            state.volume = v
        }))
    });

    harness.tab();

    // Drag all the way right (should not exceed 1.0)
    for _ in 0..20 {
        harness.key(Key::Right);
    }
    assert!(
        harness.state().volume <= 1.0,
        "slider value clamped to max 1.0"
    );

    // Drag all the way left (should not go below 0.0)
    for _ in 0..50 {
        harness.key(Key::Left);
    }
    assert!(
        harness.state().volume >= 0.0,
        "slider value clamped to min 0.0"
    );
}

#[test]
fn a_slider_is_keyboard_accessible() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        slider(state.volume, |state: &mut ControlsState, v| {
            state.volume = v
        })
    });

    harness.assert_accessible();
}

#[test]
fn a_radio_group_selects_one_choice_at_a_time() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        col((
            text("Choose format:"),
            radio_group(
                &["Plain", "JSON", "Binary"],
                state.format,
                |state: &mut ControlsState, index| state.format = index,
            ),
        ))
        .gap(8.0)
    });

    assert_eq!(harness.state().format, 0, "defaults to first choice");

    harness.click_text("JSON");
    assert_eq!(harness.state().format, 1);

    harness.click_text("Binary");
    assert_eq!(harness.state().format, 2);

    // Clicking the same choice again doesn't change it
    harness.click_text("Binary");
    assert_eq!(harness.state().format, 2);
}

#[test]
fn a_radio_group_unchooses_previous_selection() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        radio_group(
            &["A", "B", "C"],
            state.format,
            |state: &mut ControlsState, index| state.format = index,
        )
    });

    harness.click_text("B");
    assert_eq!(harness.state().format, 1);

    harness.click_text("C");
    assert_eq!(harness.state().format, 2, "selecting C unchooses B");
}

#[test]
fn a_radio_group_persists_selection_across_frames() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        radio_group(
            &["X", "Y", "Z"],
            state.format,
            |state: &mut ControlsState, index| state.format = index,
        )
    });

    harness.click_text("Y");
    assert_eq!(harness.state().format, 1);

    // Step many frames
    for _ in 0..100 {
        harness.frame();
    }

    assert_eq!(
        harness.state().format,
        1,
        "selection persists across frames"
    );
}

#[test]
fn a_radio_group_renders_filled_circle_for_selection() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        radio_group(
            &["Red", "Green"],
            state.format,
            |state: &mut ControlsState, index| state.format = index,
        )
    });

    let first_frame = harness.canvas().pixels().to_vec();

    harness.click_text("Green");
    let second_frame = harness.canvas().pixels().to_vec();

    assert_ne!(
        first_frame, second_frame,
        "radio group visual changes when selection changes"
    );
}

#[test]
fn a_radio_group_is_keyboard_accessible() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        radio_group(
            &["First", "Second", "Third"],
            state.format,
            |state: &mut ControlsState, index| state.format = index,
        )
    });

    harness.assert_accessible();
    harness.assert_tab_order();
}

#[test]
fn a_slider_step_size_is_consistent() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        col(slider(state.volume, |state: &mut ControlsState, v| {
            state.volume = v
        }))
    });

    harness.tab();

    harness.key(Key::Right);
    let step1 = harness.state().volume;

    harness.key(Key::Right);
    let step2 = harness.state().volume;

    // Steps should be equal (0.05 each)
    assert!(
        (step2 - step1 - (step1 - 0.0)).abs() < 0.001,
        "slider steps are consistent at 0.05 per arrow key"
    );
}

#[test]
fn a_radio_group_with_single_choice_renders() {
    let mut harness = Harness::new(ControlsState::default(), |state: &ControlsState| {
        radio_group(&["Only"], state.format, |state: &mut ControlsState, _| {
            state.format = 0;
        })
    });

    assert_eq!(harness.state().format, 0);
    harness.assert_accessible();
}

#[test]
fn a_slider_and_radio_group_work_together_in_one_view() {
    #[derive(Clone, Copy, Default)]
    struct CombinedState {
        volume: f32,
        format: usize,
    }

    let mut harness = Harness::new(CombinedState::default(), |state: &CombinedState| {
        col((
            row((
                text("Volume:"),
                slider(state.volume, |s: &mut CombinedState, v| s.volume = v).key("slider"),
            )),
            row((
                text("Format:"),
                radio_group(
                    &["Low", "High"],
                    state.format,
                    |s: &mut CombinedState, i| s.format = i,
                ),
            )),
        ))
        .gap(16.0)
    });

    // Adjust slider via drag
    let slider_rect = harness.find_key("slider").expect("slider exists").rect;
    harness.drag(
        rui::Point::new(slider_rect.x + 40.0, slider_rect.center().y),
        rui::Point::new(slider_rect.x + 80.0, slider_rect.center().y),
    );
    let after_slider = harness.state().volume;
    assert!(after_slider > 0.0, "slider responds to pointer drag");

    // Change radio selection (shouldn't affect slider)
    harness.click_text("High");
    assert_eq!(harness.state().format, 1);
    assert_eq!(
        harness.state().volume,
        after_slider,
        "radio selection doesn't affect slider value"
    );

    harness.assert_accessible();
}
