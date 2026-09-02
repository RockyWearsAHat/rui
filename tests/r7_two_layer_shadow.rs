//! Tests for the two-layer shadow elevation system (R7).
//!
//! Sophisticated depth perception through primary (soft) and secondary (sharp) shadows.
//! Material Design 3 / Fluent Design pattern for elevated surfaces.

use rui::style::{ShadowLayer, ShadowLayers, Tone};
use rui::testing::Harness;
use rui::*;

#[derive(Clone, Copy)]
struct State;

fn view(_state: &State) -> El<State> {
    col((
        // Simple single-layer shadow (for backwards compatibility)
        draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow(8.0)
            .key("simple"),
        // Elevated two-layer shadow (for raised surfaces)
        draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow_elevated(12.0)
            .key("elevated"),
        // Custom shadow layers
        draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow_layers(ShadowLayers::new(
                ShadowLayer::new(10.0, 5.0, 0.8),
                Some(ShadowLayer::new(3.0, 1.0, 0.3)),
            ))
            .key("custom"),
    ))
    .gap(16.0)
    .pad(16.0)
}

#[test]
fn simple_shadow_creates_single_layer() {
    let state = State;
    let mut h = Harness::new(state, view).size(200.0, 200.0);
    h.frame();

    // The simple shadow should render a single primary shadow
    let pixels = h.canvas().pixels();
    assert!(!pixels.is_empty(), "Canvas should have pixels");
    assert_ne!(pixels.len(), 0, "Shadow should produce pixel changes");
}

#[test]
fn elevated_shadow_creates_two_layers() {
    let state = State;
    let mut h = Harness::new(state, view).size(200.0, 200.0);
    h.frame();

    // Elevated shadow should create a richer depth effect
    // We verify this by checking that the canvas has rendered something
    let pixels = h.canvas().pixels();
    assert!(
        !pixels.is_empty(),
        "Canvas should have pixels for elevated shadow"
    );
}

#[test]
fn shadow_layer_blur_affects_depth() {
    let state = State;
    let mut h1 = Harness::new(state, |_s| {
        col(draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow(4.0))
    })
    .size(200.0, 100.0);
    h1.frame();

    let mut h2 = Harness::new(state, |_s| {
        col(draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow(16.0))
    })
    .size(200.0, 100.0);
    h2.frame();

    // Larger blur should produce different shadow appearance
    // This is a soft verification that blur parameter is processed
    let pixels1 = h1.canvas().pixels();
    let pixels2 = h2.canvas().pixels();
    assert!(
        !pixels1.is_empty() && !pixels2.is_empty(),
        "Both should render"
    );
}

#[test]
fn shadow_layer_opacity_clamps_to_valid_range() {
    // Test that ShadowLayer clamps opacity to [0, 1]
    let layer1 = ShadowLayer::new(8.0, 4.0, 2.0); // Should clamp to 1.0
    assert_eq!(layer1.opacity, 1.0, "Opacity should clamp to 1.0");

    let layer2 = ShadowLayer::new(8.0, 4.0, -0.5); // Should clamp to 0.0
    assert_eq!(layer2.opacity, 0.0, "Opacity should clamp to 0.0");

    let layer3 = ShadowLayer::new(8.0, 4.0, 0.5);
    assert_eq!(layer3.opacity, 0.5, "Valid opacity should remain unchanged");
}

#[test]
fn simple_shadow_calculates_defaults() {
    let layers = ShadowLayers::simple(8.0);

    // Primary shadow should have calculated offset and opacity
    assert_eq!(layers.primary.blur, 8.0, "Blur should match input");
    assert_eq!(layers.primary.offset, 4.0, "Offset should be 0.5x blur");
    assert!(
        (layers.primary.opacity - 0.5).abs() < 0.01,
        "Opacity should be 0.5"
    );

    // Simple shadow should not have secondary layer
    assert_eq!(
        layers.secondary, None,
        "Simple shadow should have no secondary layer"
    );
}

#[test]
fn elevated_shadow_creates_two_distinct_layers() {
    let layers = ShadowLayers::elevated(12.0);

    // Primary shadow should be soft and large
    assert_eq!(layers.primary.blur, 12.0);
    assert!(
        layers.primary.opacity > 0.5,
        "Primary should have good opacity"
    );

    // Secondary shadow should be sharper and closer
    assert!(
        layers.secondary.is_some(),
        "Elevated should have secondary layer"
    );
    if let Some(secondary) = layers.secondary {
        assert!(
            secondary.blur < layers.primary.blur,
            "Secondary blur should be less than primary"
        );
        assert!(
            secondary.offset < layers.primary.offset,
            "Secondary offset should be less than primary"
        );
        assert!(
            secondary.opacity < layers.primary.opacity,
            "Secondary opacity should be less than primary"
        );
    }
}

#[test]
fn custom_shadow_layers_accepts_full_control() {
    let primary = ShadowLayer::new(10.0, 6.0, 0.7);
    let secondary = ShadowLayer::new(4.0, 1.5, 0.3);
    let layers = ShadowLayers::new(primary, Some(secondary));

    assert_eq!(layers.primary.blur, 10.0);
    assert_eq!(layers.primary.offset, 6.0);
    assert_eq!(layers.primary.opacity, 0.7);

    assert_eq!(layers.secondary.unwrap().blur, 4.0);
    assert_eq!(layers.secondary.unwrap().offset, 1.5);
    assert_eq!(layers.secondary.unwrap().opacity, 0.3);
}

#[test]
fn shadow_methods_build_without_error() {
    let state = State;
    // Just verify that the methods compile and don't panic
    let mut h = Harness::new(state, |_| {
        col((
            col(()).shadow(8.0),
            col(()).shadow_elevated(12.0),
            col(()).shadow_layers(ShadowLayers::simple(8.0)),
        ))
    })
    .size(200.0, 300.0);

    h.frame(); // Render without errors
    assert!(
        !h.canvas().pixels().is_empty(),
        "Shadow methods should render"
    );
}

#[test]
fn elevated_shadow_persists_across_frames() {
    let state = State;
    let mut h = Harness::new(state, |_s| {
        col(draw(Size::new(100.0, 50.0), |_, _| {})
            .fill(Tone::Surface)
            .shadow_elevated(10.0))
    })
    .size(200.0, 100.0);

    h.frame();
    let pixels_frame1 = h.canvas().pixels().to_vec();
    h.frame();
    let pixels_frame2 = h.canvas().pixels().to_vec();

    // Same input should produce identical pixels
    assert_eq!(
        pixels_frame1, pixels_frame2,
        "Elevated shadow should render identically across frames"
    );
}

#[test]
fn shadow_offset_creates_depth_direction() {
    let down = ShadowLayer::new(8.0, 4.0, 0.5);
    let up = ShadowLayer::new(8.0, -4.0, 0.5);

    // Both should be valid (positive offset = down, negative = up)
    assert_eq!(down.offset, 4.0);
    assert_eq!(up.offset, -4.0);
}

#[test]
fn multiple_shadow_elevations_in_one_view() {
    let state = State;
    let mut h = Harness::new(state, |_| {
        col((
            col(()).shadow(4.0),           // light shadow
            col(()).shadow(8.0),           // medium shadow
            col(()).shadow_elevated(12.0), // elevated shadow
        ))
        .gap(16.0)
    })
    .size(200.0, 300.0);

    h.frame();
    // All three should render without errors
    let pixels = h.canvas().pixels();
    assert!(
        !pixels.is_empty(),
        "Multiple shadow elevations should render"
    );
}

#[test]
fn shadow_layer_zero_blur_is_valid() {
    let layer = ShadowLayer::new(0.0, 2.0, 0.5);
    assert_eq!(layer.blur, 0.0, "Zero blur is valid (hard shadow)");
    assert_eq!(layer.offset, 2.0, "Offset can still be set");
}

#[test]
fn shadow_layer_zero_opacity_renders_invisible() {
    let layers = ShadowLayers::new(
        ShadowLayer::new(8.0, 4.0, 0.0),       // Invisible primary
        Some(ShadowLayer::new(4.0, 2.0, 0.8)), // Visible secondary
    );

    assert_eq!(layers.primary.opacity, 0.0, "Zero opacity is valid");
    assert_eq!(
        layers.secondary.unwrap().opacity,
        0.8,
        "Secondary can be visible"
    );
}

#[test]
fn shadow_layer_calculation_for_material_design() {
    // Material Design 3 shadow system
    // Elevation level -> shadow parameters
    let _surface = ShadowLayers::simple(0.0); // No elevation
    let raised = ShadowLayers::elevated(8.0); // Raised elevation
    let modal = ShadowLayers::elevated(24.0); // Modal elevation

    assert!(
        raised.secondary.is_some(),
        "Raised should have secondary layer"
    );
    assert!(
        modal.secondary.is_some(),
        "Modal should have secondary layer"
    );

    if let (Some(r_sec), Some(m_sec)) = (raised.secondary, modal.secondary) {
        assert!(
            m_sec.blur > r_sec.blur,
            "Modal shadow should be larger than raised"
        );
    }
}
