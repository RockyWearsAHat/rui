//! Visual depth through elevation levels.
//!
//! Elevation creates visual hierarchy by boosting lightness in dark mode,
//! or through shadow/border in light mode. Three standard levels are provided:
//! Surface (baseline), Overlay (raised), and Modal (highest).

/// Elevation level for visual depth.
///
/// Elevation creates visual hierarchy through lightness changes (dark elevation).
/// Each level boosts lightness to appear higher in the z-order.
/// In light mode, elevation may be expressed through shadow or border.
/// In dark mode, elevation is expressed through lightness boost (WCAG-accessible).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    /// Surface level (baseline, no boost)
    Surface,
    /// Overlay level (moderate boost)
    Overlay,
    /// Modal level (maximum boost)
    Modal,
}

impl Elevation {
    /// Lightness boost factor (0.0-0.15 range) for this elevation level.
    /// Used in dark mode to brighten colors and create depth perception.
    pub fn lightness_boost(self) -> f32 {
        match self {
            Elevation::Surface => 0.00,
            Elevation::Overlay => 0.07,
            Elevation::Modal => 0.14,
        }
    }

    /// Apply this elevation's lightness boost to a color.
    /// In dark mode: increases lightness by boost factor.
    /// In light mode: can be ignored or expressed through shadow/border.
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
