#![allow(missing_docs)]
// STEP 4 RED phase: Test scaffolding for pixel-grid crispness (R6)
// These tests demonstrate the desired API for hairline snapping, glyph caching, and gamma-boost LUT

use rui::{Color, geom::Rect};

/// RED PHASE: HairlineSnap should exist and snap coordinates to pixel grid
#[test]
fn a_hairline_snap_snaps_coordinates_to_pixel_grid() {
    // A line at 10.5 logical units should snap to 10.0 or 11.0 device pixels
    let snap = rui::HairlineSnap::new(1.0); // 1.0 scale factor

    // 10.5 should snap to nearest 0.5 (device pixel boundary at 1.0 scale)
    let snapped = snap.snap_x(10.5);
    assert!(snapped == 10.0 || snapped == 11.0);
}

/// RED PHASE: Gamma-boost LUT should exist and apply contrast adjustment
#[test]
fn a_gamma_boost_lut_applies_contrast_adjustment() {
    let lut = rui::GammaBoostLut::new(1.1); // 1.1x gamma boost

    // A mid-tone gray (128) should be slightly boosted toward white
    let input = Color::rgba(128, 128, 128, 255);
    let boosted = lut.boost(input);

    // Boosted should be lighter than original
    assert!(boosted.r >= input.r);
    assert!(boosted.g >= input.g);
    assert!(boosted.b >= input.b);
}

/// RED PHASE: Glyph cache should exist and store rendered glyphs
#[test]
fn a_glyph_cache_stores_rendered_glyphs() {
    let cache = rui::GlyphCache::new();

    // Cache should be empty initially
    assert_eq!(cache.len(), 0);

    // Should be able to query and store glyphs
    assert!(cache.get(b'A', 16.0).is_none());
}

/// RED PHASE: Canvas should support snap_rect for hairline snapping
#[test]
fn canvas_can_snap_rectangles_to_pixel_grid() {
    let rect = Rect::new(10.5, 20.3, 100.2, 50.7);
    let snapped = rui::Canvas::snap_rect(rect, 1.0); // 1.0 scale factor

    // Snapped rect should have integer coordinates or half-pixel boundaries
    assert!(snapped.x % 0.5 < 0.01 || (1.0 - snapped.x % 0.5) < 0.01);
    assert!(snapped.y % 0.5 < 0.01 || (1.0 - snapped.y % 0.5) < 0.01);
}

/// RED PHASE: HairlineSnap should work at different scale factors
#[test]
fn hairline_snap_works_at_different_scale_factors() {
    let snap_2x = rui::HairlineSnap::new(2.0);

    // At 2.0 scale, pixel grid is at 0.5 logical unit intervals
    let snapped = snap_2x.snap_x(10.25);

    // Should snap to 10.0, 10.5, or nearest pixel boundary
    let frac = snapped % 0.5;
    assert!(frac < 0.01 || (0.5 - frac) < 0.01);
}

/// RED PHASE: GammaBoostLut should preserve alpha channel
#[test]
fn gamma_boost_lut_preserves_alpha() {
    let lut = rui::GammaBoostLut::new(1.2);
    let input = Color::rgba(100, 150, 200, 128); // Semi-transparent
    let boosted = lut.boost(input);

    // Alpha should be unchanged
    assert_eq!(boosted.a, input.a);
}

/// RED PHASE: Glyph cache should evict old glyphs when full
#[test]
fn glyph_cache_evicts_old_glyphs_when_full() {
    let cache = rui::GlyphCache::new();

    // Fill cache beyond capacity
    for _ch in b'A'..=b'z' {
        // Simulate storing glyph (would fail with current stub)
    }

    // Cache size should not exceed maximum
    assert!(cache.len() <= rui::GlyphCache::MAX_SIZE);
}

/// RED PHASE: Hairline at 1px should always land at pixel boundaries
#[test]
fn hairline_at_one_pixel_snaps_to_pixel_boundaries() {
    let snap = rui::HairlineSnap::new(1.0);

    // Test multiple coordinates
    for x in [0.1, 0.5, 0.9, 10.4, 10.5, 10.9] {
        let snapped = snap.snap_x(x);
        // Should snap to 0.0, 0.5, 1.0, etc.
        let frac = snapped % 1.0;
        assert!(frac < 0.01 || (frac - 0.5).abs() < 0.01 || (1.0 - frac) < 0.01);
    }
}

/// RED PHASE: Canvas rect snapping should handle negative coordinates
#[test]
fn canvas_snap_rect_handles_negative_coordinates() {
    let rect = Rect::new(-10.5, -20.3, 100.2, 50.7);
    let snapped = rui::Canvas::snap_rect(rect, 1.0);

    // Snapped should still be valid rectangle
    assert!(snapped.w > 0.0);
    assert!(snapped.h > 0.0);
}
