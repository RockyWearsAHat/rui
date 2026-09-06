//! Tests for the avatar component.

use rui::testing::Harness;
use rui::{avatar, avatar_color, col, Color, Rect};

#[test]
fn avatar_is_deterministic_for_a_name() {
    // Test that avatar_color returns the same value for the same name
    let color1 = avatar_color("Alice");
    let color2 = avatar_color("Alice");
    assert_eq!(
        color1, color2,
        "avatar_color should return the same color for the same name"
    );

    // Render twice and verify both render successfully
    let mut harness1 = Harness::new((), |_| col((avatar("Alice", 32.0),))).size(100.0, 100.0);
    harness1.frame();

    let mut harness2 = Harness::new((), |_| col((avatar("Alice", 32.0),))).size(100.0, 100.0);
    harness2.frame();

    // Both should render without error
}

#[test]
fn avatar_differs_between_names() {
    let color_alice = avatar_color("Alice");
    let color_bob = avatar_color("Bob");
    assert_ne!(
        color_alice, color_bob,
        "Different names should produce different colors"
    );
}

#[test]
fn avatar_is_case_and_whitespace_insensitive() {
    let color1 = avatar_color("Test User");
    let color2 = avatar_color("test user");
    let color3 = avatar_color(" test user ");

    assert_eq!(color1, color2, "Case should not matter");
    assert_eq!(color1, color3, "Whitespace should not matter");
}

#[test]
fn avatar_fits_its_box() {
    let mut harness = Harness::new((), |_| col((avatar("Test", 32.0),))).size(100.0, 100.0);
    harness.frame();

    // Avatar should be contained within the 32x32 box
    // Check that pixels far outside the avatar box are not marked
    let far_rect = Rect {
        x: 80.0,
        y: 80.0,
        w: 20.0,
        h: 20.0,
    };
    assert!(
        !harness.marked(far_rect),
        "Avatar should not overflow far outside its box"
    );
}

#[test]
fn avatar_names_itself_for_accessibility() {
    let mut harness = Harness::new((), |_| col((avatar("Alice", 32.0),))).size(100.0, 100.0);
    harness.frame();

    // Check accessibility
    harness.assert_accessible();
}

#[test]
fn avatar_color_is_legible_on_the_dark_ground() {
    // Test that avatar colors have sufficient contrast against the dark background #0f1115
    let dark_bg = Color::rgb(0x0f, 0x11, 0x15);

    for name in &["Alice", "Bob", "Charlie", "Test", "User"] {
        let color = avatar_color(name);
        let contrast_ratio = contrast(color, dark_bg);
        assert!(
            contrast_ratio >= 3.0,
            "Color for '{}' has contrast {} against dark background",
            name,
            contrast_ratio
        );
    }
}

/// Calculate WCAG 2.0 contrast ratio between two colors.
fn contrast(color1: Color, color2: Color) -> f32 {
    let lum1 = relative_luminance(color1);
    let lum2 = relative_luminance(color2);
    let (lighter, darker) = if lum1 > lum2 {
        (lum1, lum2)
    } else {
        (lum2, lum1)
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// Calculate relative luminance using WCAG 2.0 formula.
fn relative_luminance(color: Color) -> f32 {
    let r = normalize_channel(color.r);
    let g = normalize_channel(color.g);
    let b = normalize_channel(color.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Convert 8-bit channel to linear RGB.
fn normalize_channel(channel: u8) -> f32 {
    let c = channel as f32 / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
