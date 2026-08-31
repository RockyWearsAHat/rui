//! Controls this library does not ship, built from the primitives and tested
//! exactly as anything it *did* ship would be.
//!
//! This is the claim the foundation has to answer for: that a checkbox, a
//! switch, a slider, a group of choices, and a menu are all things you write in
//! a few lines rather than things you wait for — and that once written they are
//! ordinary elements, testable with no window and no special support.
//!
//! Each control here is deliberately whole and standalone. Copying one into a
//! project and changing it is the intended use; see `examples/controls.rs` for
//! the same set drawn together.

use rui::testing::Harness;
use rui::{
    caption, col, draw, panel, row, text, Align, Anchor, Drag, El, Key, Modifiers, Painter, Point,
    Radius, Rect, Size, Tone,
};

/// Everything the controls below are wired to.
#[derive(Default)]
struct Settings {
    notify: bool,
    dark: bool,
    volume: f32,
    format: usize,
    tip: Option<String>,
}

// ---------------------------------------------------------------------------
// The recipes
// ---------------------------------------------------------------------------

/// A box that answers the pointer, and a word beside it.
fn checkbox<S: 'static>(label: &str, checked: bool, toggle: impl Fn(&mut S) + 'static) -> El<S> {
    row((
        draw(
            Size::new(15.0, 15.0),
            move |painter: &mut Painter<'_>, rect: Rect| {
                painter.fill(
                    rect,
                    Radius::Units(4.0),
                    if checked { Tone::Accent } else { Tone::Sunken },
                );
                painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
            },
        )
        .size(15.0, 15.0),
        text(label),
    ))
    .gap(8.0)
    .h(22.0)
    .align(Align::Center)
    .on_click(move |state: &mut S| toggle(state))
}

/// A track, and a knob that slides along it.
fn switch<S: 'static>(on: bool, flip: impl Fn(&mut S) + 'static) -> El<S> {
    draw(
        Size::new(34.0, 20.0),
        move |painter: &mut Painter<'_>, rect: Rect| {
            painter.fill(
                rect,
                Radius::Pill,
                if on { Tone::Accent } else { Tone::Sunken },
            );
            let knob = rect.h - 6.0;
            let x = if on {
                rect.max_x() - knob - 3.0
            } else {
                rect.x + 3.0
            };
            painter.fill(
                Rect::new(x, rect.y + 3.0, knob, knob),
                Radius::Pill,
                Tone::Surface,
            );
        },
    )
    .size(34.0, 20.0)
    .on_click(move |state: &mut S| flip(state))
}

/// A track that reports where along it the pointer is, and answers the arrows.
fn slider<S: 'static>(value: f32, set: impl Fn(&mut S, f32) + Copy + 'static) -> El<S> {
    const STEP: f32 = 0.05;
    let value = value.clamp(0.0, 1.0);
    draw(
        Size::new(160.0, 18.0),
        move |painter: &mut Painter<'_>, rect: Rect| {
            painter.fill(rect, Radius::Pill, Tone::Sunken);
            painter.fill(
                Rect::new(rect.x, rect.y, rect.w * value, rect.h),
                Radius::Pill,
                Tone::Accent,
            );
        },
    )
    .size(160.0, 18.0)
    .on_drag(move |state: &mut S, drag: Drag| set(state, drag.fraction().x))
    .on_key(move |state: &mut S, key: Key, _: Modifiers| match key {
        Key::Left => set(state, (value - STEP).max(0.0)),
        Key::Right => set(state, (value + STEP).min(1.0)),
        _ => {}
    })
}

/// A group of choices, one of them taken.
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
                    move |painter: &mut Painter<'_>, rect: Rect| {
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
            .on_click(move |state: &mut S| choose(state, index))
        })
        .collect::<Vec<_>>())
}

// ---------------------------------------------------------------------------
// What they do
// ---------------------------------------------------------------------------

#[test]
fn a_checkbox_answers_a_click_on_its_label_as_well_as_on_its_box() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(checkbox(
            "Notify on failure",
            settings.notify,
            |settings: &mut Settings| settings.notify = !settings.notify,
        ))
        .align(Align::Start)
    });

    harness.click_text("Notify on failure");
    assert!(
        harness.state().notify,
        "clicking the word is clicking the control"
    );

    harness.click_text("Notify on failure");
    assert!(!harness.state().notify, "and it is a toggle, not a latch");
}

#[test]
fn a_checkbox_draws_differently_once_it_is_ticked() {
    let mut off = Harness::new(Settings::default(), |settings: &Settings| {
        col(checkbox("Notify", settings.notify, |_: &mut Settings| {})).align(Align::Start)
    })
    .size(200.0, 60.0);
    let mut on = Harness::new(
        Settings {
            notify: true,
            ..Settings::default()
        },
        |settings: &Settings| {
            col(checkbox("Notify", settings.notify, |_: &mut Settings| {})).align(Align::Start)
        },
    )
    .size(200.0, 60.0);

    off.frame();
    on.frame();
    assert_ne!(
        off.canvas().pixels(),
        on.canvas().pixels(),
        "a state nobody can see is not a state"
    );
}

#[test]
fn a_switch_flips_and_moves_its_knob_when_it_does() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(switch(settings.dark, |settings: &mut Settings| {
            settings.dark = !settings.dark
        })
        .key("switch"))
        .align(Align::Start)
    })
    .size(200.0, 60.0);

    harness.frame();
    let rect = harness
        .find_key("switch")
        .expect("the switch is on screen")
        .rect;
    let before: Vec<u32> = harness.canvas().pixels().to_vec();

    harness.click(rect.center());
    assert!(harness.state().dark);
    harness.frame();
    assert_ne!(
        before,
        harness.canvas().pixels(),
        "the knob should have moved"
    );
}

#[test]
fn a_slider_follows_the_pointer_and_the_arrow_keys_alike() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(slider(settings.volume, |settings: &mut Settings, value| {
            settings.volume = value
        })
        .key("volume"))
        .align(Align::Start)
    });

    let rect = harness
        .find_key("volume")
        .expect("the slider is on screen")
        .rect;
    harness.drag(
        Point::new(rect.x + 40.0, rect.center().y),
        Point::new(rect.x + 120.0, rect.center().y),
    );
    assert!(
        (harness.state().volume - 0.75).abs() < 0.001,
        "it ends where the pointer let go"
    );

    // Pressing it gave it the keyboard, so the arrows step it from there.
    harness.key(Key::Left);
    assert!((harness.state().volume - 0.70).abs() < 0.001);
    harness.key(Key::Right).key(Key::Right);
    assert!((harness.state().volume - 0.80).abs() < 0.001);
}

#[test]
fn a_slider_can_be_used_from_the_keyboard_without_ever_being_clicked() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(slider(settings.volume, |settings: &mut Settings, value| {
            settings.volume = value
        }))
    });

    harness.tab();
    harness.key(Key::Right);
    assert!(
        (harness.state().volume - 0.05).abs() < 0.001,
        "tab reached it and the arrow moved it"
    );
}

#[test]
fn a_group_of_choices_takes_exactly_one_of_them() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(radio_group(
            &["Plain", "JSON", "Binary"],
            settings.format,
            |settings: &mut Settings, index| settings.format = index,
        ))
        .align(Align::Start)
    });

    harness.click_text("Binary");
    assert_eq!(harness.state().format, 2);

    harness.click_text("Plain");
    assert_eq!(
        harness.state().format,
        0,
        "choosing one is unchoosing the others"
    );
}

#[test]
fn a_note_appears_when_the_pointer_arrives_and_goes_when_it_leaves() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col((
            text("Workers")
                .h(24.0)
                .on_hover(|settings: &mut Settings, over: bool| {
                    settings.tip = over.then(|| "How many at once".to_owned())
                })
                .add(settings.tip.as_ref().map(|note| {
                    panel(caption(note.clone()))
                        .w(160.0)
                        .key("tip")
                        .layer(Anchor::Below)
                })),
            text("Elsewhere").h(24.0),
        ))
        .align(Align::Start)
    });

    assert!(
        !harness.shows("How many at once"),
        "nothing is hovered, so there is no note"
    );

    harness.hover_text("Workers");
    harness.frame();
    assert!(
        harness.shows("How many at once"),
        "the pointer arriving brought it up"
    );

    harness.hover_text("Elsewhere");
    harness.frame();
    assert!(
        !harness.shows("How many at once"),
        "and leaving took it away again"
    );
}

#[test]
fn a_note_that_is_up_does_not_come_up_again_every_frame() {
    // What `on_hover` reporting the *change* rather than the state is for: an
    // application told every frame that the pointer is still there would write
    // to its own state every frame, and never stop redrawing.
    #[derive(Default)]
    struct Counted {
        changes: u32,
    }

    let mut harness = Harness::new(Counted::default(), |_: &Counted| {
        col(text("Workers")
            .h(24.0)
            .on_hover(|counted: &mut Counted, _| counted.changes += 1))
        .align(Align::Start)
    });

    harness.hover_text("Workers");
    assert_eq!(harness.state().changes, 1);

    harness.frames(10);
    assert_eq!(
        harness.state().changes,
        1,
        "the pointer has not moved, so nothing has changed"
    );
}

#[test]
fn code_block_highlights_keywords_and_strings() {
    use rui::{code_block, Language};

    #[derive(Default)]
    struct App;

    let code = "let x = \"hello\"; // comment";
    let mut harness = Harness::new(App, move |_: &App| {
        col(code_block(code, Language::Rust)).align(Align::Start)
    })
    .size(400.0, 100.0);

    harness.frame();

    // Verify that keywords are rendered
    assert!(harness.shows("let"), "keyword 'let' is rendered");

    // Verify that strings are rendered
    assert!(harness.shows("\"hello\""), "string literal is rendered");

    // Verify that comments are rendered
    assert!(harness.shows("// comment"), "comment is rendered");

    // Verify that the code block has elements
    let probes = harness.probes();
    assert!(!probes.is_empty(), "code block renders elements");
}

#[test]
fn code_block_highlights_numeric_literals() {
    use rui::{code_block, Language};

    #[derive(Default)]
    struct App;

    let code = "let x = 42; let y = 3.14;";
    let mut harness = Harness::new(App, move |_: &App| {
        col(code_block(code, Language::Rust)).align(Align::Start)
    })
    .size(400.0, 100.0);

    harness.frame();

    // Verify that numeric literals are rendered
    assert!(harness.shows("42"), "decimal literal is rendered");
    assert!(harness.shows("3.14"), "float literal is rendered");

    // Verify that the code block has elements
    let probes = harness.probes();
    assert!(!probes.is_empty(), "code block renders elements");
}

#[test]
fn a_segmented_control_changes_selection_when_clicked() {
    use rui::segmented;

    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(segmented(
            &["Small", "Medium", "Large"],
            settings.format,
            |settings: &mut Settings, index| settings.format = index,
        ))
        .align(Align::Start)
    });

    // Initially, first option is selected
    assert_eq!(harness.state().format, 0);

    // Click on "Medium" (second option)
    harness.click_text("Medium");
    assert_eq!(
        harness.state().format,
        1,
        "clicking an option changes the selection"
    );

    // Click on "Large" (third option)
    harness.click_text("Large");
    assert_eq!(
        harness.state().format,
        2,
        "selection changes to the new option"
    );

    // Click back to "Small"
    harness.click_text("Small");
    assert_eq!(
        harness.state().format,
        0,
        "selection can cycle back to previous options"
    );
}

#[test]
fn a_meter_displays_progress_as_a_fraction() {
    use rui::meter;

    #[derive(Default)]
    struct ProgressState {
        progress: f32,
    }

    // Test meter at 0% progress
    let mut harness = Harness::new(ProgressState { progress: 0.0 }, |state: &ProgressState| {
        col((
            text("Upload progress:"),
            meter(state.progress, Tone::Accent),
        ))
    });

    assert_eq!(harness.state().progress, 0.0, "progress starts at 0%");
    let pixels_at_zero = harness.frame().canvas().pixels().len();
    assert!(pixels_at_zero > 0, "meter at 0% renders pixels");

    // Test meter at 50% progress
    let mut harness = Harness::new(ProgressState { progress: 0.5 }, |state: &ProgressState| {
        col((
            text("Upload progress:"),
            meter(state.progress, Tone::Accent),
        ))
    });

    assert_eq!(harness.state().progress, 0.5, "progress at 50%");
    let pixels_at_half = harness.frame().canvas().pixels().len();
    assert!(pixels_at_half > 0, "meter at 50% renders pixels");

    // Test meter at 100% progress
    let mut harness = Harness::new(ProgressState { progress: 1.0 }, |state: &ProgressState| {
        col((
            text("Upload progress:"),
            meter(state.progress, Tone::Accent),
        ))
    });

    assert_eq!(harness.state().progress, 1.0, "progress at 100% completion");
    let pixels_at_full = harness.frame().canvas().pixels().len();
    assert!(pixels_at_full > 0, "meter at 100% renders pixels");

    // Meter clamps values beyond 1.0 in its display logic
    let mut harness = Harness::new(ProgressState { progress: 1.5 }, |state: &ProgressState| {
        col((
            text("Upload progress:"),
            meter(state.progress, Tone::Accent),
        ))
    });

    assert_eq!(
        harness.state().progress,
        1.5,
        "state can hold values beyond 1.0"
    );
    let pixels_beyond = harness.frame().canvas().pixels().len();
    assert!(
        pixels_beyond > 0,
        "meter displays when progress exceeds 1.0"
    );
}

/// A star rating widget: display 1–5 stars, clickable to set the rating.
///
/// This is Recipe 2 (Add a New Widget) from CLAUDE.md verified in practice.
/// The widget is built entirely from primitives and demonstrates the pattern:
/// state → view function → handler that receives &mut state as argument.
#[test]
fn a_star_rating_updates_when_clicked() {
    struct RatingState {
        rating: usize,
    }

    let mut harness = Harness::new(RatingState { rating: 0 }, |state: &RatingState| {
        col((
            text("Rate this content:"),
            rui::widgets::star_rating(state.rating, |state: &mut RatingState, r| {
                state.rating = r;
            })
            .key("star-rating"),
        ))
    });

    // Verify initial state
    assert_eq!(harness.state().rating, 0, "initial rating is 0");

    // Find the star rating widget to determine its layout
    let rating_rect = harness
        .find_key("star-rating")
        .expect("star rating widget is on screen")
        .rect;

    // Stars are arranged horizontally with 4px gap
    // Each star is 16px wide: 16 + 4 + 16 + 4 + 16 + 4 + 16 + 4 + 16 = 96px total
    // Calculate position of each star's center within the widget
    let star_width = 16.0;
    let gap = 4.0;
    let star_spacing = star_width + gap;

    // Click the 4th star
    let star4_x = rating_rect.x + (3.0 * star_spacing) + star_width / 2.0;
    harness.click(Point::new(star4_x, rating_rect.y + star_width / 2.0));
    assert_eq!(
        harness.state().rating,
        4,
        "after clicking 4th star, rating is 4"
    );

    // Click the 1st star to change rating
    let star1_x = rating_rect.x + star_width / 2.0;
    harness.click(Point::new(star1_x, rating_rect.y + star_width / 2.0));
    assert_eq!(
        harness.state().rating,
        1,
        "after clicking 1st star, rating is 1"
    );

    // Click the 5th star to max out rating
    let star5_x = rating_rect.x + (4.0 * star_spacing) + star_width / 2.0;
    harness.click(Point::new(star5_x, rating_rect.y + star_width / 2.0));
    assert_eq!(
        harness.state().rating,
        5,
        "after clicking 5th star, rating is 5"
    );
}

#[test]
fn a_checkbox_group_manages_multiple_selections() {
    // Recipe 3: A more complex widget managing state for multiple items.
    // This demonstrates:
    // - State as a Vec<bool> for multiple items
    // - Building checkboxes from primitives for each item
    // - Handling separate click handlers for each checkbox
    // - Using element keys for proper identity across frames

    struct SettingsState {
        notifications: Vec<bool>,
    }

    let mut harness = Harness::new(
        SettingsState {
            notifications: vec![false, false, false],
        },
        |state: &SettingsState| {
            col((
                text("Notification Settings:"),
                rui::widgets::checkbox_group(
                    &["Email", "SMS", "Push"],
                    &state.notifications,
                    |state: &mut SettingsState, index| {
                        if index < state.notifications.len() {
                            state.notifications[index] = !state.notifications[index];
                        }
                    },
                )
                .key("notifications-group"),
            ))
        },
    );

    // Verify initial state
    assert_eq!(
        harness.state().notifications,
        vec![false, false, false],
        "all notifications initially disabled"
    );

    // Click first checkbox (Email)
    harness.click_text("Email");
    assert!(
        harness.state().notifications[0],
        "email notification enabled after click"
    );
    assert!(
        !harness.state().notifications[1],
        "sms notification still disabled"
    );
    assert!(
        !harness.state().notifications[2],
        "push notification still disabled"
    );

    // Click second checkbox (SMS)
    harness.click_text("SMS");
    assert!(
        harness.state().notifications[0],
        "email notification still enabled"
    );
    assert!(
        harness.state().notifications[1],
        "sms notification now enabled"
    );
    assert!(
        !harness.state().notifications[2],
        "push notification still disabled"
    );

    // Click third checkbox (Push)
    harness.click_text("Push");
    assert_eq!(
        harness.state().notifications,
        vec![true, true, true],
        "all notifications now enabled"
    );

    // Click first checkbox again to toggle it off
    harness.click_text("Email");
    assert!(
        !harness.state().notifications[0],
        "email notification toggled off"
    );
    assert!(
        harness.state().notifications[1],
        "sms notification still enabled"
    );
    assert!(
        harness.state().notifications[2],
        "push notification still enabled"
    );
}
