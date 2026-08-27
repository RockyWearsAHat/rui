//! Test that embedded fonts work in a rendering scenario using Harness.
//!
//! This verifies that fonts loaded via `load_system_fonts()` (which uses embedded
//! fonts on wasm targets) can successfully render text that is discoverable by
//! Harness::shows().

use rui::shell::load_system_fonts;
use rui::testing::Harness;
use rui::{col, text, El};

#[derive(Default)]
struct State;

fn view(_state: &State) -> El<State> {
    col((
        text("Embedded Font Test"),
        text("Hello World"),
        text("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    ))
}

#[test]
fn embedded_fonts_load_and_render_text() {
    // Load the embedded fonts (or system fonts on native)
    let loaded_fonts = load_system_fonts().expect("embedded fonts should load");

    // Create a Harness with the loaded fonts
    let mut harness = Harness::with_fonts(State, view, loaded_fonts);

    // Verify that text rendered with the embedded font is discoverable
    assert!(
        harness.shows("Embedded Font Test"),
        "UI font text should render"
    );
    assert!(harness.shows("Hello World"), "Basic text should render");
    assert!(
        harness.shows("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        "Alphabet should render"
    );
}

#[test]
fn embedded_fonts_work_with_control_patterns() {
    // Load fonts
    let loaded_fonts = load_system_fonts().expect("embedded fonts should load");

    // Use the recipes pattern: a simple checkbox-like control
    #[derive(Default)]
    struct Settings {
        checked: bool,
    }

    fn control_view(s: &Settings) -> El<Settings> {
        col((
            text(if s.checked { "Checked" } else { "Unchecked" }),
            text("Settings").size(18.0, 18.0),
        ))
    }

    let mut harness = Harness::with_fonts(Settings::default(), control_view, loaded_fonts);

    // Verify the initial state renders
    assert!(
        harness.shows("Unchecked"),
        "initial state text should display"
    );

    // Verify we can identify dynamic content
    assert!(harness.shows("Settings"));
}
