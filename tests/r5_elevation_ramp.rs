//! Tests for the elevation ramp (R7) feature.

use rui::*;

#[test]
fn elevation_levels_exist() {
    // RED phase: Test demonstrates desired API for elevation system
    // Elevation = visual depth through lightness boost in dark mode, not shadows
    let _level_0 = Elevation::Surface;
    let _level_1 = Elevation::Overlay;
    let _level_2 = Elevation::Modal;
}

#[test]
fn elevation_provides_lightness_boost() {
    // In dark mode: each elevation level boosts lightness to create depth
    // Surface (0) < Overlay (1) < Modal (2)
    let surface_boost = Elevation::Surface.lightness_boost();
    let overlay_boost = Elevation::Overlay.lightness_boost();
    let modal_boost = Elevation::Modal.lightness_boost();

    // Each level provides more lightness boost than the previous
    assert!(surface_boost < overlay_boost);
    assert!(overlay_boost < modal_boost);
}

#[test]
fn elevation_as_method_applies_to_element() {
    // Elements can declare elevation level via builder method
    struct State;
    let _el: El<State> = col((text("Modal content"),)).elevation(Elevation::Modal);
    // Should compile without errors
}

#[test]
fn elevation_getter_retrieves_applied_level() {
    // Can query applied elevation from element
    struct State;
    let el: El<State> = col((text("Overlay content"),)).elevation(Elevation::Overlay);

    // Should be able to retrieve the elevation that was set
    assert_eq!(el.get_elevation(), Some(Elevation::Overlay));
}

#[test]
fn surface_elevation_applies_no_boost() {
    // Surface is the baseline; no lightness boost
    let boost = Elevation::Surface.lightness_boost();
    assert_eq!(boost, 0.0);
}

#[test]
fn elevation_gradient_is_monotonic() {
    // Lightness boost increases smoothly across elevation levels
    let surface = Elevation::Surface.lightness_boost();
    let overlay = Elevation::Overlay.lightness_boost();
    let modal = Elevation::Modal.lightness_boost();

    assert!(surface >= 0.0);
    assert!(overlay > surface);
    assert!(modal > overlay);
    assert!(modal <= 0.15); // Reasonable cap on total boost
}

#[test]
fn elevation_none_is_surface_default() {
    // If no elevation set, element defaults to Surface (baseline)
    struct State;
    let el: El<State> = col(text("Default"));
    // No explicit elevation() call
    assert_eq!(el.get_elevation(), None); // Not set
    // But if queried in paint context, would resolve to Surface
}

#[test]
fn elevation_levels_apply_in_paint_context() {
    // In paint pipeline: element's color + elevation boost = final color
    // This test verifies the integration point exists
    let color = Color::rgb(128, 128, 128); // Medium gray

    // With Surface elevation: no boost
    let boosted_surface = Elevation::Surface.apply_to_color(color);
    assert_eq!(boosted_surface, color); // No change

    // With Overlay elevation: moderate boost (brightens)
    let boosted_overlay = Elevation::Overlay.apply_to_color(color);
    assert!(boosted_overlay.r >= color.r);
    assert!(boosted_overlay.g >= color.g);
    assert!(boosted_overlay.b >= color.b);

    // With Modal elevation: maximum boost (more brightening)
    let boosted_modal = Elevation::Modal.apply_to_color(color);
    assert!(boosted_modal.r >= boosted_overlay.r);
    assert!(boosted_modal.g >= boosted_overlay.g);
    assert!(boosted_modal.b >= boosted_overlay.b);
}
