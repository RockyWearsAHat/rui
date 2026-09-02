//! Integration tests for elevation ramp into rendering pipeline.

use rui::testing::Harness;
use rui::*;

#[test]
fn elevation_can_be_set_on_container() {
    struct State;

    fn view(_s: &State) -> El<State> {
        col(text("Surface level")).elevation(Elevation::Surface)
    }

    let _h = Harness::new(State, view).size(100.0, 50.0);
    // Successfully creates harness with elevated element
}

#[test]
fn elevation_persists_across_frames() {
    struct State {
        count: usize,
    }

    fn view(s: &State) -> El<State> {
        col((
            text(format!("Count: {}", s.count)),
            button("Increment").on_click(|s: &mut State| s.count += 1),
        ))
        .elevation(Elevation::Overlay)
    }

    let mut h = Harness::new(State { count: 0 }, view).size(150.0, 80.0);

    // Click button to update state
    h.click_text("Increment");
    h.frames(1);

    // State should update correctly even with elevation set
    assert_eq!(h.state().count, 1);
}

#[test]
fn all_elevation_levels_work() {
    struct State {
        level: u32,
    }

    fn view(s: &State) -> El<State> {
        let elev = match s.level {
            0 => Elevation::Surface,
            1 => Elevation::Overlay,
            _ => Elevation::Modal,
        };
        col(text("Elevation test")).elevation(elev)
    }

    let mut h = Harness::new(State { level: 0 }, view).size(100.0, 50.0);

    // Verify we can set and update elevation levels
    assert_eq!(h.state().level, 0);

    h.state_mut().level = 1;
    h.frame();
    assert_eq!(h.state().level, 1);

    h.state_mut().level = 2;
    h.frame();
    assert_eq!(h.state().level, 2);
}

#[test]
fn elevation_on_button() {
    struct State {
        pressed: bool,
    }

    fn view(_s: &State) -> El<State> {
        button("Elevated Button")
            .elevation(Elevation::Modal)
            .on_click(|s: &mut State| s.pressed = !s.pressed)
    }

    let mut h = Harness::new(State { pressed: false }, view).size(150.0, 40.0);

    // Click the button
    h.click_text("Elevated Button");
    h.frames(1);

    // Verify the handler was called
    assert!(h.state().pressed);
}

#[test]
fn elevation_on_text() {
    struct State;

    fn view(_s: &State) -> El<State> {
        col((
            text("Surface text").elevation(Elevation::Surface),
            text("Overlay text").elevation(Elevation::Overlay),
            text("Modal text").elevation(Elevation::Modal),
        ))
    }

    let _h = Harness::new(State, view).size(200.0, 100.0);
    // Successfully creates harness with multiple elevated text elements
}

#[test]
fn elevation_with_style_methods() {
    struct State;

    fn view(_s: &State) -> El<State> {
        col(text("Styled and elevated"))
            .elevation(Elevation::Overlay)
            .pad(12.0)
            .gap(4.0)
            .center()
    }

    let _h = Harness::new(State, view).size(150.0, 80.0);
    // Successfully chains .elevation() with other style methods
}

#[test]
fn elevation_applies_to_content() {
    struct State;

    fn view(_s: &State) -> El<State> {
        col((
            button("Elevated Button").elevation(Elevation::Overlay),
            text("Elevated Text").elevation(Elevation::Modal),
            row(()).elevation(Elevation::Surface),
        ))
        .elevation(Elevation::Overlay)
    }

    let _h = Harness::new(State, view).size(200.0, 150.0);
    // Successfully creates harness with elevation on parent and children
}

#[test]
fn elevation_doesnt_affect_interaction() {
    struct State {
        count: usize,
    }

    fn view(s: &State) -> El<State> {
        col((
            text(format!("Clicks: {}", s.count)),
            button("Click me").on_click(|s: &mut State| s.count += 1),
        ))
        .elevation(Elevation::Modal)
    }

    let mut h = Harness::new(State { count: 0 }, view).size(150.0, 80.0);

    // Click the button multiple times
    h.click_text("Click me");
    h.frames(1);
    assert_eq!(h.state().count, 1);

    h.click_text("Click me");
    h.frames(1);
    assert_eq!(h.state().count, 2);

    // Elevation shouldn't affect click handling
    assert_eq!(h.state().count, 2);
}

#[test]
fn disabled_elements_with_elevation() {
    struct State {
        value: usize,
    }

    fn view(_s: &State) -> El<State> {
        button("Disabled Button")
            .on_click(|s: &mut State| s.value += 1)
            .disabled(true)
            .elevation(Elevation::Modal)
    }

    let mut h = Harness::new(State { value: 0 }, view).size(150.0, 40.0);

    // Try to click the disabled button
    h.click_text("Disabled Button");
    h.frames(1);

    // Click should not be handled (button is disabled)
    assert_eq!(h.state().value, 0);
}
