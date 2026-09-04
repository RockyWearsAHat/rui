//! Pixel format conversion utilities for rendering to different targets.
//!
//! The WASM backend renders to Canvas (u32 BGRA) and presents via putImageData (RGBA bytes).
//! This module provides the conversion logic that works on all platforms.

/// Converts pixels from Canvas format (u32 BGRA) to RGBA bytes for ImageData.
///
/// The Canvas stores pixels as packed u32 values in BGRA order (in little-endian memory).
/// ImageData expects RGBA bytes. This function unpacks and reorders the bytes.
///
/// # Example
/// ```ignore
/// let pixel = 0x220011FF; // B=0xFF, G=0x00, R=0x11, A=0x22
/// let rgba = convert_pixels_to_rgba(&[pixel]);
/// assert_eq!(&rgba[..], &[0x11, 0x00, 0xFF, 0x22]); // [R, G, B, A]
/// ```
pub fn convert_pixels_to_rgba(pixels: &[u32]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for &pixel in pixels {
        rgba.push((pixel >> 16) as u8); // R
        rgba.push((pixel >> 8) as u8); // G
        rgba.push(pixel as u8); // B
        rgba.push((pixel >> 24) as u8); // A
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_format_conversion_bgra_to_rgba() {
        // Test that u32 BGRA pixels convert correctly to RGBA byte sequence.
        // A pixel with B=0xFF, G=0x00, R=0x11, A=0x22 is stored as 0x221100FF
        // (A << 24 | R << 16 | G << 8 | B).
        let pixels = vec![0x221100FF];
        let rgba = convert_pixels_to_rgba(&pixels);

        assert_eq!(rgba.len(), 4);
        assert_eq!(rgba[0], 0x11); // R
        assert_eq!(rgba[1], 0x00); // G
        assert_eq!(rgba[2], 0xFF); // B
        assert_eq!(rgba[3], 0x22); // A
    }

    #[test]
    fn pixel_format_conversion_multiple_pixels() {
        // Test with multiple pixels to verify the conversion works for batches.
        let pixels = vec![
            0xFF000000, // Opaque black
            0x00FFFFFF, // Transparent white
        ];
        let rgba = convert_pixels_to_rgba(&pixels);

        assert_eq!(rgba.len(), 8);
        assert_eq!(&rgba[0..4], &[0x00, 0x00, 0x00, 0xFF]); // [R, G, B, A]
        assert_eq!(&rgba[4..8], &[0xFF, 0xFF, 0xFF, 0x00]); // [R, G, B, A]
    }

    #[test]
    fn pixel_format_conversion_empty() {
        // Test that empty pixel buffer converts to empty RGBA buffer.
        let pixels: Vec<u32> = vec![];
        let rgba = convert_pixels_to_rgba(&pixels);
        assert_eq!(rgba.len(), 0);
    }

    #[test]
    fn pixel_format_conversion_all_zero() {
        // Test conversion of a black, fully opaque pixel.
        let pixels = vec![0xFF000000];
        let rgba = convert_pixels_to_rgba(&pixels);
        assert_eq!(&rgba[..], &[0, 0, 0, 0xFF]);
    }

    #[test]
    fn pixel_format_conversion_all_white() {
        // Test conversion of a white, fully transparent pixel.
        let pixels = vec![0x00FFFFFF];
        let rgba = convert_pixels_to_rgba(&pixels);
        assert_eq!(&rgba[..], &[0xFF, 0xFF, 0xFF, 0]);
    }
}
