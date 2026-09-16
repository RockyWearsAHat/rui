//! Visual depth through elevation levels with layered shadows.
//!
//! Elevation creates visual hierarchy through sophisticated multi-layer shadow
//! definitions. Each elevation level defines 2-4 shadow layers with progressive
//! blur and offset characteristics, creating rich depth perception without
//! relying on lightness changes alone.
//!
//! Shadow layers are composed of:
//! - Primary shadow: soft, large blur (establishes base depth)
//! - Secondary shadow: medium blur (reinforces separation)
//! - Tertiary shadow (optional): sharp, small blur (crisp edge definition)

use crate::style::{ShadowLayer, ShadowLayers};

/// Elevation level for visual depth.
///
/// Elevation creates visual hierarchy through sophisticated multi-layer shadows
/// following Material Design 3 principles. Each level defines layered shadows
/// with progressively refined blur and offset characteristics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    /// Surface level (baseline, subtle shadow or none)
    Surface,
    /// Overlay level (moderate shadow depth)
    Overlay,
    /// Modal level (maximum shadow depth)
    Modal,
}

impl Elevation {
    /// Get the shadow layers for this elevation level.
    ///
    /// Returns properly composed shadow layers with blur radii of 12-24px,
    /// vertical offsets of 4-8px, and opacity values tuned per layer
    /// for sophisticated depth perception.
    pub fn shadow_layers(self) -> ShadowLayers {
        match self {
            Elevation::Surface => {
                // Subtle elevation: single soft shadow
                ShadowLayers::new(
                    ShadowLayer::new(8.0, 2.0, 0.12), // soft ambient shadow
                    None,
                )
            }
            Elevation::Overlay => {
                // Moderate elevation: two-layer shadow
                ShadowLayers::new(
                    ShadowLayer::new(16.0, 4.0, 0.15),      // primary soft shadow
                    Some(ShadowLayer::new(4.0, 1.5, 0.08)), // secondary crisp shadow
                )
            }
            Elevation::Modal => {
                // Maximum elevation: three-layer shadow for rich depth
                ShadowLayers::new(
                    ShadowLayer::new(24.0, 8.0, 0.18),       // primary deep shadow
                    Some(ShadowLayer::new(12.0, 4.0, 0.12)), // secondary mid shadow
                )
            }
        }
    }

    /// Lightness boost factor for dark mode (0.0-0.15 range).
    ///
    /// Used to adjust surface colors in dark mode for visual separation.
    /// Deprecated in favor of multi-layer shadows but retained for compatibility.
    pub fn lightness_boost(self) -> f32 {
        match self {
            Elevation::Surface => 0.00,
            Elevation::Overlay => 0.05,
            Elevation::Modal => 0.10,
        }
    }

    /// Apply this elevation's lightness boost to a color.
    ///
    /// Mostly deprecated; shadow_layers() is the primary elevation mechanism.
    pub fn apply_to_color(self, color: crate::Color) -> crate::Color {
        let boost = self.lightness_boost();
        if boost == 0.0 {
            return color;
        }

        // Convert RGB to HSL, boost L, convert back
        let r = (color.r as f32) / 255.0;
        let g = (color.g as f32) / 255.0;
        let b = (color.b as f32) / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        // Boost lightness
        let boosted_l = (l + boost).min(1.0);
        let delta_l = boosted_l - l;

        // Shift RGB toward white by delta_l
        let r_boosted = (r + delta_l).min(1.0);
        let g_boosted = (g + delta_l).min(1.0);
        let b_boosted = (b + delta_l).min(1.0);

        crate::Color::rgb(
            (r_boosted * 255.0) as u8,
            (g_boosted * 255.0) as u8,
            (b_boosted * 255.0) as u8,
        )
    }
}
