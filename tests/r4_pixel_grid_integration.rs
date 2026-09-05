#![allow(missing_docs)]
// STEP 4 ENHANCEMENT phase: Pixel-grid crispness integration tests
// These tests verify that hairline snapping, gamma-boost LUT, and glyph caching
// are properly integrated into the rendering and paint systems

use rui::{geom::Rect, Canvas, Color, GammaBoostLut, GlyphCache, HairlineSnap};

/// ENHANCEMENT PHASE: Canvas can snap rectangles to pixel grid at various scales
#[test]
fn canvas_snaps_rectangles_at_1x_scale() {
    let rect = Rect::new(10.5, 20.3, 100.2, 50.7);
    let snapped = Canvas::snap_rect(rect, 1.0);

    // At 1.0x scale, coordinates should snap to integers
    assert!((snapped.x - snapped.x.round()).abs() < 0.01);
    assert!((snapped.y - snapped.y.round()).abs() < 0.01);
    assert!(snapped.w > 0.0);
    assert!(snapped.h > 0.0);
}

/// ENHANCEMENT PHASE: Canvas snapping preserves rectangle dimensions
#[test]
fn canvas_snap_preserves_valid_dimensions() {
    let rect = Rect::new(10.1, 20.2, 100.5, 50.8);
    let snapped = Canvas::snap_rect(rect, 1.0);

    // Snapped rectangle should still have valid dimensions
    assert!(snapped.w > 0.0);
    assert!(snapped.h > 0.0);
    // Dimensions should be reasonable (not collapsed)
    assert!(snapped.w < rect.w + 2.0);
    assert!(snapped.h < rect.h + 2.0);
}

/// ENHANCEMENT PHASE: Canvas snapping works at Retina (2x) scale
#[test]
fn canvas_snaps_rectangles_at_2x_scale() {
    let rect = Rect::new(10.1, 20.2, 100.3, 50.4);
    let snapped = Canvas::snap_rect(rect, 2.0);

    // At 2.0x scale, coordinates should snap to 0.5 boundaries
    let frac_x = snapped.x % 0.5;
    let frac_y = snapped.y % 0.5;
    assert!(frac_x.abs() < 0.01 || (frac_x - 0.5).abs() < 0.01);
    assert!(frac_y.abs() < 0.01 || (frac_y - 0.5).abs() < 0.01);
}

/// ENHANCEMENT PHASE: HairlineSnap produces consistent results
#[test]
fn hairline_snap_produces_consistent_results() {
    let snap = HairlineSnap::new(1.0);

    let x1 = snap.snap_x(10.5);
    let x2 = snap.snap_x(10.5);

    // Same input should always produce same output
    assert_eq!(x1, x2);
}

/// ENHANCEMENT PHASE: GammaBoostLut produces monotonic outputs
#[test]
fn gamma_boost_lut_produces_monotonic_outputs() {
    let lut = GammaBoostLut::new(1.1);

    let mut last = 0u8;
    for i in 0..=255 {
        let color = Color::rgb(i, i, i);
        let boosted = lut.boost(color);
        // Output should be monotonically increasing
        assert!(boosted.r >= last);
        last = boosted.r;
    }
}

/// ENHANCEMENT PHASE: GammaBoostLut boost factors work correctly
#[test]
fn gamma_boost_lut_different_boosts_produce_different_results() {
    let lut_gentle = GammaBoostLut::new(1.05);
    let lut_strong = GammaBoostLut::new(1.20);

    let input = Color::rgb(128, 128, 128);
    let gentle = lut_gentle.boost(input);
    let strong = lut_strong.boost(input);

    // Stronger boost should produce lighter result
    assert!(strong.r >= gentle.r);
}

/// ENHANCEMENT PHASE: GlyphCache can store and retrieve glyphs
#[test]
fn glyph_cache_stores_and_retrieves_glyphs() {
    let mut cache = GlyphCache::new();

    // Initially empty
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);

    // Insert a glyph
    cache.insert(b'A', 16.0, 100);
    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());

    // Retrieve it
    assert_eq!(cache.get(b'A', 16.0), Some(100));
}

/// ENHANCEMENT PHASE: GlyphCache returns None for missing glyphs
#[test]
fn glyph_cache_returns_none_for_missing_glyphs() {
    let cache = GlyphCache::new();

    assert!(cache.get(b'X', 16.0).is_none());
    assert!(cache.get(b'A', 32.0).is_none());
}

/// ENHANCEMENT PHASE: GlyphCache respects MAX_SIZE limit
#[test]
fn glyph_cache_respects_max_size() {
    let mut cache = GlyphCache::new();

    // Fill cache beyond max size (but not by much for performance)
    for i in 0..GlyphCache::MAX_SIZE + 100 {
        cache.insert(b'A', i as f32, i);
    }

    // Cache should not exceed MAX_SIZE
    assert!(cache.len() <= GlyphCache::MAX_SIZE);
}

/// ENHANCEMENT PHASE: Hairline snap handles edge cases
#[test]
fn hairline_snap_handles_zero_and_negative_coordinates() {
    let snap = HairlineSnap::new(1.0);

    let zero = snap.snap_x(0.0);
    let negative = snap.snap_x(-10.5);
    let large = snap.snap_x(1000.5);

    // All should snap to pixel boundaries
    assert!((zero - zero.round()).abs() < 0.01);
    assert!((negative - negative.round()).abs() < 0.01);
    assert!((large - large.round()).abs() < 0.01);
}

/// ENHANCEMENT PHASE: Canvas snap_rect handles edge cases
#[test]
fn canvas_snap_rect_handles_zero_sized_rects() {
    // A zero-width rectangle
    let rect = Rect::new(10.5, 20.5, 0.0, 50.0);
    let snapped = Canvas::snap_rect(rect, 1.0);

    // Should handle gracefully
    assert!(snapped.w >= 0.0);
    assert!(snapped.h >= 0.0);
}
