//! Colours, and the one blend the whole toolkit is drawn with.
//!
//! Colours are 8-bit sRGB with a straight (not premultiplied) alpha. Straight
//! alpha is what a person writing a theme types, and premultiplying at the point
//! of use costs one multiply that the blend does anyway.
//!
//! # Why blending is not gamma-corrected
//!
//! Compositing sRGB values directly is, strictly, wrong: the values are not
//! linear light. Doing it properly means converting to linear, blending, and
//! converting back, on every pixel of every glyph. Every windowing system this
//! draws into composites the same wrong way, so matching them makes our text
//! sit correctly next to native chrome, and the alternative would make it look
//! subtly different from every other window on the screen. This is a deliberate
//! choice to be consistent rather than correct in isolation.

/// An 8-bit sRGB colour with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Opacity; 0 is invisible and 255 is opaque.
    pub a: u8,
}

impl Color {
    /// An opaque colour.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// A colour with an explicit alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Fully transparent; drawing with it is a no-op.
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Opaque black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);

    /// This colour at a different opacity.
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// This colour scaled to `factor` of its current opacity.
    ///
    /// Multiplying rather than replacing lets a caller fade something that is
    /// already translucent without having to know how translucent it was.
    pub fn fade(self, factor: f32) -> Self {
        let a = (self.a as f32 * factor.clamp(0.0, 1.0)).round() as u8;
        Self { a, ..self }
    }

    /// A colour `t` of the way from this one to `other`, channel by channel.
    ///
    /// `t` is clamped, so an over- or undershooting animation cannot produce a
    /// colour outside the pair it was told to move between.
    pub fn mix(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Self {
            r: lerp(self.r, other.r),
            g: lerp(self.g, other.g),
            b: lerp(self.b, other.b),
            a: lerp(self.a, other.a),
        }
    }

    /// Whether drawing with this colour could change any pixel.
    pub fn is_visible(self) -> bool {
        self.a > 0
    }

    /// This colour packed into the canvas's `0xAARRGGBB` word.
    pub const fn to_argb(self) -> u32 {
        (self.a as u32) << 24 | (self.r as u32) << 16 | (self.g as u32) << 8 | self.b as u32
    }

    /// The colour a packed `0xAARRGGBB` word represents.
    pub const fn from_argb(word: u32) -> Self {
        Self {
            a: (word >> 24) as u8,
            r: (word >> 16) as u8,
            g: (word >> 8) as u8,
            b: word as u8,
        }
    }

    /// The perceived lightness of the colour, from 0 (black) to 1 (white).
    ///
    /// Rec. 601 luma. Used to decide whether a theme is light or dark and which
    /// of a pair of foregrounds will be legible on it, which is a judgement
    /// about perception rather than about channel values.
    pub fn luminance(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }
}

/// Composites `source` over `destination` with `coverage` applied to its alpha.
///
/// `destination` is opaque — it is a pixel already in the buffer — so the result
/// is opaque too and the usual divide by the composed alpha falls away. Both
/// words are `0xAARRGGBB`.
///
/// `coverage` is the rasteriser's answer for how much of the pixel the shape
/// covers, which is why it multiplies the alpha rather than being a second
/// blend: a half-covered pixel of a half-transparent fill is a quarter of a
/// pixel of paint, and doing it in one step avoids rounding it twice.
pub fn blend_over(destination: u32, source: Color, coverage: u8) -> u32 {
    // `a * c / 255` with the divide replaced by the usual reciprocal trick, so
    // that 255 * 255 lands on exactly 255 rather than 254.
    let alpha = mul_255(source.a, coverage);
    if alpha == 0 {
        return destination;
    }
    if alpha == 255 {
        return source.to_argb() | 0xff00_0000;
    }

    let inverse = 255 - alpha;
    let channel = |src: u8, dst: u32| mul_255(src, alpha) as u32 + mul_255(dst as u8, inverse) as u32;

    let r = channel(source.r, (destination >> 16) & 0xff);
    let g = channel(source.g, (destination >> 8) & 0xff);
    let b = channel(source.b, destination & 0xff);
    0xff00_0000 | r << 16 | g << 8 | b
}

/// `a * b / 255`, rounded, without a divide.
fn mul_255(a: u8, b: u8) -> u8 {
    let product = a as u32 * b as u32 + 128;
    ((product + (product >> 8)) >> 8) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_coverage_of_an_opaque_colour_replaces_the_pixel() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!(blend_over(0xff00_00ff, red, 255), 0xffff_0000);
    }

    #[test]
    fn zero_coverage_leaves_the_pixel_alone() {
        let red = Color::rgb(255, 0, 0);
        assert_eq!(blend_over(0xff00_00ff, red, 0), 0xff00_00ff);
    }

    #[test]
    fn a_transparent_colour_never_draws_however_covered() {
        assert_eq!(blend_over(0xff12_3456, Color::TRANSPARENT, 255), 0xff12_3456);
    }

    #[test]
    fn half_coverage_lands_halfway_between_the_two_colours() {
        let white_over_black = blend_over(0xff00_0000, Color::WHITE, 128);
        let mid = white_over_black & 0xff;
        assert!((127..=129).contains(&mid), "expected about half, got {mid}");
    }

    #[test]
    fn blending_always_produces_an_opaque_pixel() {
        let faint = Color::rgba(10, 20, 30, 3);
        assert_eq!(blend_over(0xff00_0000, faint, 40) >> 24, 0xff);
    }

    #[test]
    fn coverage_and_alpha_multiply_rather_than_blending_twice() {
        // Half-transparent paint at half coverage is a quarter of a pixel.
        let half = Color::rgba(255, 255, 255, 128);
        let quarter = blend_over(0xff00_0000, half, 128) & 0xff;
        assert!((62..=66).contains(&quarter), "expected about a quarter, got {quarter}");
    }

    #[test]
    fn packing_round_trips() {
        let colour = Color::rgba(1, 2, 3, 4);
        assert_eq!(Color::from_argb(colour.to_argb()), colour);
    }

    #[test]
    fn mixing_is_clamped_at_both_ends() {
        let a = Color::rgb(0, 0, 0);
        let b = Color::rgb(255, 255, 255);
        assert_eq!(a.mix(b, -5.0), a);
        assert_eq!(a.mix(b, 5.0), b);
    }

    #[test]
    fn mul_255_reaches_both_extremes_exactly() {
        assert_eq!(mul_255(255, 255), 255);
        assert_eq!(mul_255(0, 255), 0);
        assert_eq!(mul_255(255, 0), 0);
    }

    #[test]
    fn luminance_orders_black_below_white() {
        assert!(Color::BLACK.luminance() < Color::WHITE.luminance());
        assert_eq!(Color::WHITE.luminance(), 1.0);
    }
}
