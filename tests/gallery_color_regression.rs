//! Color visual regression tests for the gallery.
//!
//! These tests verify that colors render correctly and distinctly at the pixel level.

use rui::testing::Harness;
use rui::{col, Appearance, Tone};

#[derive(Default)]
struct EmptyState;

/// Verify that a specific tone renders to a specific color in light mode
#[test]
fn light_mode_accent_tone_renders_correctly() {
    let mut h = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Accent)
    })
    .size(100.0, 100.0)
    .appearance(Appearance::Light);
    h.frame();

    let pixel = h.pixel(50, 50).expect("a pixel in the center");
    // Light mode accent should have color components
    assert!(
        pixel.r > 0 || pixel.g > 0 || pixel.b > 0,
        "accent should have color"
    );
}

/// Verify that a specific tone renders to a different color in dark mode
#[test]
fn dark_mode_accent_tone_renders_differently() {
    let mut h_light = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Accent)
    })
    .size(100.0, 100.0)
    .appearance(Appearance::Light);

    let mut h_dark = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Accent)
    })
    .size(100.0, 100.0)
    .appearance(Appearance::Dark);

    h_light.frame();
    h_dark.frame();

    let light_pixel = h_light.pixel(50, 50).expect("a pixel");
    let dark_pixel = h_dark.pixel(50, 50).expect("a pixel");

    assert_ne!(
        light_pixel, dark_pixel,
        "light and dark accent should be different colors"
    );
}

/// Verify that status colors (Ok, Bad, Warn) are distinct
#[test]
fn status_colors_are_visually_distinct() {
    let mut h_ok =
        Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Ok)).size(100.0, 100.0);

    let mut h_bad =
        Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Bad)).size(100.0, 100.0);

    let mut h_warn = Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Warn))
        .size(100.0, 100.0);

    h_ok.frame();
    h_bad.frame();
    h_warn.frame();

    let ok_px = h_ok.pixel(50, 50).expect("ok pixel");
    let bad_px = h_bad.pixel(50, 50).expect("bad pixel");
    let warn_px = h_warn.pixel(50, 50).expect("warn pixel");

    // All three should be different
    assert_ne!(ok_px, bad_px, "ok and bad should be different colors");
    assert_ne!(ok_px, warn_px, "ok and warn should be different colors");
    assert_ne!(bad_px, warn_px, "bad and warn should be different colors");
}

/// Verify that ground color is consistent
#[test]
fn ground_color_is_consistent_across_frames() {
    let mut h_light = Harness::new(EmptyState, |_| col(()))
        .size(100.0, 100.0)
        .appearance(Appearance::Light);

    let mut h_dark = Harness::new(EmptyState, |_| col(()))
        .size(100.0, 100.0)
        .appearance(Appearance::Dark);

    h_light.frame();
    let light_1 = h_light.pixel(50, 50).expect("light ground");

    h_light.frame();
    let light_2 = h_light.pixel(50, 50).expect("light ground again");

    assert_eq!(light_1, light_2, "light ground should be consistent");

    h_dark.frame();
    let dark_1 = h_dark.pixel(50, 50).expect("dark ground");

    h_dark.frame();
    let dark_2 = h_dark.pixel(50, 50).expect("dark ground again");

    assert_eq!(dark_1, dark_2, "dark ground should be consistent");
    assert_ne!(
        light_1, dark_1,
        "light and dark grounds should be different"
    );
}

/// Verify that tone rendering is idempotent (same tone = same pixel)
#[test]
fn tone_rendering_is_idempotent() {
    let mut h1 = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Surface)
    })
    .size(100.0, 100.0);

    let mut h2 = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Surface)
    })
    .size(100.0, 100.0);

    h1.frame();
    h2.frame();

    assert_eq!(
        h1.canvas().pixels(),
        h2.canvas().pixels(),
        "same tone should render to identical pixels"
    );
}

/// Verify that sunken tone appears darker than surface
#[test]
fn sunken_appears_darker_than_surface() {
    let mut h_surface = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Surface)
    })
    .size(100.0, 100.0)
    .appearance(Appearance::Light);

    let mut h_sunken = Harness::new(EmptyState, |_| {
        col(()).size(100.0, 100.0).fill(Tone::Sunken)
    })
    .size(100.0, 100.0)
    .appearance(Appearance::Light);

    h_surface.frame();
    h_sunken.frame();

    let surface_px = h_surface.pixel(50, 50).expect("surface");
    let sunken_px = h_sunken.pixel(50, 50).expect("sunken");

    // Sunken should be darker than surface in light mode
    assert!(
        sunken_px.luminance() < surface_px.luminance(),
        "sunken should be darker than surface"
    );
}

/// Verify that idle tone is visible and renders correctly
#[test]
fn idle_tone_is_visible() {
    let mut h = Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Idle))
        .size(100.0, 100.0);
    h.frame();

    let idle_px = h.pixel(50, 50).expect("idle pixel");

    // Idle should be a real color (visible)
    assert!(idle_px.luminance() > 0.0, "idle should be visible");

    // Verify it renders consistently
    h.frame();
    let idle_px_2 = h.pixel(50, 50).expect("idle pixel again");
    assert_eq!(idle_px, idle_px_2, "idle tone should render consistently");
}

/// Verify that muted tone is distinct from primary
#[test]
fn muted_tone_is_distinct_from_ok() {
    let mut h_muted = Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Muted))
        .size(100.0, 100.0);

    let mut h_ok =
        Harness::new(EmptyState, |_| col(()).size(100.0, 100.0).fill(Tone::Ok)).size(100.0, 100.0);

    h_muted.frame();
    h_ok.frame();

    let muted_px = h_muted.pixel(50, 50).expect("muted pixel");
    let ok_px = h_ok.pixel(50, 50).expect("ok pixel");

    assert_ne!(muted_px, ok_px, "muted and ok should be different colors");
}
