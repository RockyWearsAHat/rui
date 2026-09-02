//! Tests for Palette::derive dynamic theme generation.

use rui::{Appearance, Color, Palette};

#[test]
fn derive_creates_a_light_palette_from_a_base_accent() {
    let accent = Color::rgb(0x25, 0x63, 0xd4); // Blue
    let palette = Palette::derive(accent, Appearance::Light);

    // Palette should be legible
    palette.assert_legible("derived_light");

    // Accent should match the requested color
    assert_eq!(palette.accent, accent);

    // Surfaces should be neutral greys with proper hierarchy
    assert!(palette.surface.luminance() > palette.background.luminance());
    assert!(palette.raised.luminance() < palette.surface.luminance()); // raised is darker in light theme
}

#[test]
fn derive_creates_a_dark_palette_from_a_base_accent() {
    let accent = Color::rgb(0x4f, 0x8f, 0xf7); // Lighter blue for dark theme
    let palette = Palette::derive(accent, Appearance::Dark);

    // Palette should be legible
    palette.assert_legible("derived_dark");

    // Accent should match the requested color
    assert_eq!(palette.accent, accent);

    // Surface hierarchy should be correct (value ascending in dark theme)
    assert!(palette.background_deep.luminance() < palette.background.luminance());
    assert!(palette.background.luminance() < palette.surface.luminance());
    assert!(palette.surface.luminance() < palette.raised.luminance());
}

#[test]
fn derive_generates_all_status_colors() {
    let accent = Color::rgb(0x25, 0x63, 0xd4);
    let palette = Palette::derive(accent, Appearance::Light);

    // Status colors should be generated and distinct from accent
    assert_ne!(palette.ok, palette.accent);
    assert_ne!(palette.warn, palette.accent);
    assert_ne!(palette.bad, palette.accent);
    assert_ne!(palette.idle, palette.accent);

    // Status tints should pair with their status hues
    palette.assert_legible("derived_light");
}

#[test]
fn derive_accent_variants_are_distinct() {
    let accent = Color::rgb(0x25, 0x63, 0xd4);
    let palette = Palette::derive(accent, Appearance::Light);

    // accent_deep should be darker than accent
    assert!(palette.accent_deep.luminance() < palette.accent.luminance());

    // accent_light should be lighter than accent
    assert!(palette.accent_light.luminance() > palette.accent.luminance());
}

#[test]
fn derive_respects_appearance_in_text_color() {
    let accent = Color::rgb(0x25, 0x63, 0xd4);
    let light_palette = Palette::derive(accent, Appearance::Light);
    let dark_palette = Palette::derive(accent, Appearance::Dark);

    // Light theme should have dark text
    assert!(light_palette.text.luminance() < 0.5);

    // Dark theme should have light text
    assert!(dark_palette.text.luminance() > 0.5);
}

#[test]
fn derive_red_accent_creates_distinct_status_colors() {
    let red = Color::rgb(0xd9, 0x30, 0x33);
    let palette = Palette::derive(red, Appearance::Light);

    // All status colors should be present and different from each other
    palette.assert_legible("red_accent_light");
    assert_ne!(palette.ok, palette.bad);
    assert_ne!(palette.warn, palette.bad);
}

#[test]
fn derive_green_accent_maintains_contrast() {
    let green = Color::rgb(0x12, 0x7a, 0x45);
    let palette = Palette::derive(green, Appearance::Dark);

    palette.assert_legible("green_accent_dark");
}

#[test]
fn multiple_derives_are_independent() {
    let blue = Color::rgb(0x25, 0x63, 0xd4);
    let red = Color::rgb(0xd9, 0x30, 0x33);

    let blue_palette = Palette::derive(blue, Appearance::Light);
    let red_palette = Palette::derive(red, Appearance::Light);

    // Accents should differ
    assert_eq!(blue_palette.accent, blue);
    assert_eq!(red_palette.accent, red);

    // Both should be legible
    blue_palette.assert_legible("blue_light");
    red_palette.assert_legible("red_light");
}
