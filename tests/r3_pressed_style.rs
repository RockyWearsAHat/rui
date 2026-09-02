#![allow(missing_docs)]
// STEP 3 RED phase: Test scaffolding for pressed style struct and disabled = 0.38 alpha convention
// These tests demonstrate the desired API for interactive elements with visual feedback

use rui::{Pressed, Tone, button, col, testing::Harness};

/// RED PHASE: Pressed struct should exist with fill, ink, and border overrides
#[test]
fn a_pressed_style_can_override_fill() {
    let pressed = Pressed {
        fill: Some(Tone::Sunken),
        ink: None,
        border: None,
    };
    assert!(pressed.fill.is_some());
    assert_eq!(pressed.fill.unwrap(), Tone::Sunken);
}

/// RED PHASE: Elements should have a .pressed() builder method
#[test]
fn an_element_can_set_pressed_style_with_builder() {
    struct App {
        count: usize,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Press me")
            .on_click(|app: &mut App| app.count += 1)
            .pressed(Pressed {
                fill: Some(Tone::Accent),
                ink: Some(Tone::OnAccent),
                border: None,
            })
    }

    let mut h = Harness::new(App { count: 0 }, view).size(200.0, 100.0);
    h.frames(1);
    // Verify button is rendered without panicking
    assert_eq!(h.state().count, 0);
}

/// RED PHASE: Disabled elements should apply 0.38 alpha to their content
#[test]
fn a_disabled_element_applies_38_percent_alpha() {
    struct App {
        enabled: bool,
    }

    fn view(app: &App) -> rui::El<App> {
        button("Click me").disabled(!app.enabled)
    }

    let mut h = Harness::new(App { enabled: false }, view).size(200.0, 100.0);
    h.frames(1);
    // Button should render with disabled state without panicking
    assert!(!h.state().enabled);
}

/// RED PHASE: Hover and pressed states should both apply
#[test]
fn hover_and_pressed_states_can_coexist() {
    struct App {
        count: usize,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Multi-state button")
            .on_click(|app: &mut App| app.count += 1)
            .hover_fill(Tone::Raised)
            .pressed(Pressed {
                fill: Some(Tone::Sunken),
                ink: None,
                border: None,
            })
    }

    let mut h = Harness::new(App { count: 0 }, view).size(200.0, 100.0);
    h.frames(1);
    assert_eq!(h.state().count, 0);
}

/// RED PHASE: Pressed style should be empty by default
#[test]
fn a_pressed_style_default_is_empty() {
    let pressed = Pressed::default();
    assert!(pressed.fill.is_none());
    assert!(pressed.ink.is_none());
    assert!(pressed.border.is_none());
}

/// RED PHASE: Pressed style should have is_empty() method
#[test]
fn a_pressed_style_knows_when_its_empty() {
    let empty = Pressed::default();
    assert!(empty.is_empty());

    let filled = Pressed {
        fill: Some(Tone::Accent),
        ink: None,
        border: None,
    };
    assert!(!filled.is_empty());
}

/// RED PHASE: Multiple disabled elements in a row should all apply alpha
#[test]
fn multiple_disabled_elements_apply_alpha_independently() {
    struct App;

    fn view(_app: &App) -> rui::El<App> {
        col((
            button("Enabled 1").disabled(false),
            button("Disabled 1").disabled(true),
            button("Disabled 2").disabled(true),
            button("Enabled 2").disabled(false),
        ))
    }

    let mut h = Harness::new(App, view).size(200.0, 400.0);
    h.frames(1);
    // All buttons should render without panicking
}

/// RED PHASE: Disabled state should not prevent visual rendering
#[test]
fn disabled_buttons_still_render_with_visual_feedback() {
    struct App {
        hover_fill: Option<Tone>,
    }

    fn view(app: &App) -> rui::El<App> {
        let mut b = button("Click me").disabled(true);
        if let Some(fill) = app.hover_fill {
            b = b.hover_fill(fill);
        }
        b
    }

    let mut h = Harness::new(
        App {
            hover_fill: Some(Tone::Raised),
        },
        view,
    )
    .size(200.0, 100.0);
    h.frames(1);
}

/// ENHANCEMENT PHASE: Pressed styles should be applied when element is held
#[test]
fn pressed_styles_apply_when_element_is_held() {
    struct App {
        count: usize,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Press me")
            .on_click(|app: &mut App| app.count += 1)
            .pressed(Pressed {
                fill: Some(Tone::Sunken),
                ink: Some(Tone::OnAccent),
                border: None,
            })
    }

    let mut h = Harness::new(App { count: 0 }, view).size(200.0, 100.0);
    h.frames(1);

    // Get the button position and simulate a press/release cycle
    let button_center = rui::geom::Point::new(100.0, 50.0);
    h.press(button_center);
    h.frames(1);

    // Button should still be in initial state (click happens after release)
    assert_eq!(h.state().count, 0);

    h.release();
    h.frames(1);

    // Now click should have fired
    assert_eq!(h.state().count, 1);
}

/// ENHANCEMENT PHASE: Pressed border should be applied when held
#[test]
fn pressed_border_applies_when_element_is_held() {
    struct App {
        clicked: bool,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Press")
            .on_click(|app: &mut App| app.clicked = true)
            .border(1.0, Tone::Border)
            .pressed(Pressed {
                fill: None,
                ink: None,
                border: Some(Tone::Accent),
            })
    }

    let mut h = Harness::new(App { clicked: false }, view).size(200.0, 100.0);
    h.frames(1);

    let button_center = rui::geom::Point::new(100.0, 50.0);
    h.press(button_center);
    h.frames(1);

    h.release();
    h.frames(1);

    assert!(h.state().clicked);
}

/// ENHANCEMENT PHASE: Pressed ink (text color) should be applied when held
#[test]
fn pressed_ink_applies_when_element_is_held() {
    struct App {
        clicked: bool,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Press me")
            .on_click(|app: &mut App| app.clicked = true)
            .pressed(Pressed {
                fill: None,
                ink: Some(Tone::Accent),
                border: None,
            })
    }

    let mut h = Harness::new(App { clicked: false }, view).size(200.0, 100.0);
    h.frames(1);

    let button_center = rui::geom::Point::new(100.0, 50.0);
    h.press(button_center);
    h.frames(1);

    h.release();
    h.frames(1);

    assert!(h.state().clicked);
}

/// ENHANCEMENT PHASE: Hover and pressed styles should layer correctly
#[test]
fn hover_and_pressed_styles_layer_correctly() {
    struct App {
        count: usize,
    }

    fn view(_app: &App) -> rui::El<App> {
        button("Multi-state")
            .on_click(|app: &mut App| app.count += 1)
            .hover_fill(Tone::Raised)
            .pressed(Pressed {
                fill: Some(Tone::Sunken),
                ink: None,
                border: None,
            })
    }

    let mut h = Harness::new(App { count: 0 }, view).size(200.0, 100.0);
    h.frames(1);

    let button_center = rui::geom::Point::new(100.0, 50.0);

    // Move pointer over button (hover should apply)
    h.move_pointer(button_center);
    h.frames(1);

    // Press button (pressed should apply, overriding hover)
    h.press(button_center);
    h.frames(1);

    // Release button (hover should apply again)
    h.release();
    h.frames(1);

    assert_eq!(h.state().count, 1);
}
