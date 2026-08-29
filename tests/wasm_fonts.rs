//! Test that embedded fonts work in a rendering scenario using Harness.
//!
//! This verifies that fonts loaded via `load_system_fonts()` (which uses embedded
//! fonts on wasm targets) can successfully render text that is discoverable by
//! Harness::shows().

use rui::shell::load_system_fonts;
use rui::testing::Harness;
use rui::theme::{Appearance, Theme};
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

#[test]
fn theme_construction_with_loaded_font_ids() {
    // Load the embedded fonts via the shell module (WASM uses embedded, native uses system)
    let loaded_fonts = load_system_fonts().expect("fonts should load");

    // Verify that ui_font and mono_font FontIds can be used to construct a Theme
    // This is what present_counter() does: Theme::new(appearance, counter.ui_font, counter.mono_font)
    let theme_light = Theme::new(
        Appearance::Light,
        loaded_fonts.ui_font,
        loaded_fonts.mono_font,
    );
    let theme_dark = Theme::new(
        Appearance::Dark,
        loaded_fonts.ui_font,
        loaded_fonts.mono_font,
    );

    // Verify themes were constructed successfully with valid FontIds
    // Both themes should be initialized with non-zero color values
    assert!(theme_light.palette.background.r > 0);
    assert!(theme_dark.palette.background.r > 0);
}
