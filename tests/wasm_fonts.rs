//! Test that embedded fonts work in a rendering scenario using Harness.
//!
//! This verifies that fonts loaded via `load_system_fonts()` (which uses embedded
//! fonts on wasm targets) can successfully render text that is discoverable by
//! Harness::shows().

use rui_native::shell::load_system_fonts;
use rui_native::testing::Harness;
use rui_native::theme::{Appearance, Theme};
use rui_native::{col, text, El};

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

#[test]
fn embedded_fonts_render_glyphs_with_pixel_data() {
    // Load the embedded fonts (or system fonts on native)
    let loaded_fonts = load_system_fonts().expect("embedded fonts should load");

    // Create a simple text view and render it
    #[derive(Default)]
    struct TextApp;

    fn text_view(_state: &TextApp) -> El<TextApp> {
        col((text("A"), text("B")))
    }

    let mut harness = Harness::with_fonts(TextApp, text_view, loaded_fonts);

    // Verify that glyphs are actually rendered by checking pixel data exists
    let frame = harness.frame();
    let canvas = frame.canvas();

    // Text should occupy some area; verify the canvas has the expected dimensions
    assert!(canvas.width() > 0, "canvas should have width");
    assert!(canvas.height() > 0, "canvas should have height");

    // Verify pixels exist (non-transparent pixels where text was rendered)
    // Canvas stores pixels as u32 RGBA values; check for any with alpha > 0
    let pixels = canvas.pixels();
    let mut found_text_pixels = false;

    for &pixel in pixels {
        // Extract alpha channel (top byte in RGBA)
        let alpha = (pixel >> 24) & 0xFF;
        if alpha > 0 {
            found_text_pixels = true;
            break;
        }
    }

    assert!(
        found_text_pixels,
        "embedded fonts should render glyphs as visible pixels"
    );
}
