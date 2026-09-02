//! Pixel-grid crispness: hairline snapping, gamma-boost LUT, and glyph caching.
//!
//! Ensures rendering remains crisp at all scale factors by snapping hairlines to
//! device pixel boundaries and caching glyph rasterization.

use crate::Color;

/// Snaps coordinates to the pixel grid to keep hairlines crisp at any scale.
///
/// At a scale factor of 1.0, the pixel grid is every 1.0 logical unit.
/// At 2.0x, it is every 0.5 logical units (half-pixel grid for Retina).
#[derive(Debug, Clone, Copy)]
pub struct HairlineSnap {
    scale_factor: f32,
}

impl HairlineSnap {
    /// Create a new hairline snapper at a given scale factor.
    pub fn new(scale_factor: f32) -> Self {
        Self { scale_factor }
    }

    /// Snap an x-coordinate to the nearest pixel boundary.
    ///
    /// At 1.0x scale, snaps to integers. At 2.0x, snaps to 0.5 boundaries.
    pub fn snap_x(&self, x: f32) -> f32 {
        let pixel_size = 1.0 / self.scale_factor;
        (x / pixel_size).round() * pixel_size
    }

    /// Snap a y-coordinate to the nearest pixel boundary.
    pub fn snap_y(&self, y: f32) -> f32 {
        let pixel_size = 1.0 / self.scale_factor;
        (y / pixel_size).round() * pixel_size
    }

    /// Snap a coordinate pair.
    pub fn snap_point(&self, x: f32, y: f32) -> (f32, f32) {
        (self.snap_x(x), self.snap_y(y))
    }
}

/// Gamma-boost lookup table for improving contrast in dark mode.
///
/// Applies a power curve to RGB values to brighten mid-tones without affecting
/// blacks or whites, improving readability on displays with gamma characteristics.
#[derive(Debug, Clone)]
pub struct GammaBoostLut {
    /// Boost factor; 1.0 = no change, > 1.0 brightens mid-tones.
    #[allow(dead_code)]
    gamma: f32,
    /// Precomputed LUT for fast lookups (256 entries).
    lut: [u8; 256],
}

impl GammaBoostLut {
    /// Create a gamma-boost LUT with the given boost factor.
    pub fn new(gamma: f32) -> Self {
        let mut lut = [0u8; 256];
        for (i, entry) in lut.iter_mut().enumerate() {
            let normalized = (i as f32) / 255.0;
            // Apply power curve: output = input^(1/gamma)
            // Higher gamma = more boost toward white
            let boosted = normalized.powf(1.0 / gamma);
            *entry = (boosted * 255.0).round() as u8;
        }
        Self { gamma, lut }
    }

    /// Apply gamma boost to a color, preserving alpha.
    pub fn boost(&self, color: Color) -> Color {
        Color::rgba(
            self.lut[color.r as usize],
            self.lut[color.g as usize],
            self.lut[color.b as usize],
            color.a,
        )
    }
}

/// Cache for rendered glyphs to avoid re-rasterizing at the same size.
///
/// Stores pre-rendered glyph bitmaps keyed by (character, size) pair.
#[derive(Debug, Clone)]
pub struct GlyphCache {
    // Simplified cache: in a real implementation, this would store bitmaps
    entries: std::collections::HashMap<(u8, u32), usize>,
}

impl GlyphCache {
    /// Maximum cache size (entries).
    pub const MAX_SIZE: usize = 512;

    /// Create a new glyph cache.
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// Get a cached glyph, if available.
    pub fn get(&self, ch: u8, size: f32) -> Option<usize> {
        self.entries.get(&(ch, size.round() as u32)).copied()
    }

    /// Store a glyph in the cache.
    pub fn insert(&mut self, ch: u8, size: f32, bitmap_id: usize) {
        if self.entries.len() >= Self::MAX_SIZE {
            // Simple eviction: remove oldest (in a real impl, use LRU)
            if let Some(first_key) = self.entries.keys().next().copied() {
                self.entries.remove(&first_key);
            }
        }
        self.entries.insert((ch, size.round() as u32), bitmap_id);
    }

    /// Number of cached glyphs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hairline_snap_1x_snaps_to_integers() {
        let snap = HairlineSnap::new(1.0);
        assert!((snap.snap_x(10.5) - 10.0).abs() < 0.01 || (snap.snap_x(10.5) - 11.0).abs() < 0.01);
    }

    #[test]
    fn hairline_snap_2x_snaps_to_half_pixels() {
        let snap = HairlineSnap::new(2.0);
        let snapped = snap.snap_x(10.25);
        // Should snap to 10.0, 10.5, or similar
        let frac = snapped % 0.5;
        assert!(frac.abs() < 0.01 || (frac - 0.5).abs() < 0.01);
    }

    #[test]
    fn gamma_boost_brightens_mid_tones() {
        let lut = GammaBoostLut::new(1.1);
        let gray = Color::rgba(128, 128, 128, 255);
        let boosted = lut.boost(gray);
        assert!(boosted.r >= gray.r);
    }

    #[test]
    fn gamma_boost_preserves_alpha() {
        let lut = GammaBoostLut::new(1.2);
        let color = Color::rgba(100, 150, 200, 128);
        let boosted = lut.boost(color);
        assert_eq!(boosted.a, color.a);
    }
}
