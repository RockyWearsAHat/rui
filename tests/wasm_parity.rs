//! Integration test for WASM parity verification.
//!
//! Verifies that reference frames can be generated deterministically using embedded fonts
//! for later comparison with WASM-rendered output.

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::Appearance;

/// Generate both light and dark reference frames as RGBA byte buffers.
/// Returns a 2-element array: [(Appearance::Light, pixels), (Appearance::Dark, pixels)].
fn parity_frames() -> [(Appearance, Vec<u8>); 2] {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    let light_bytes: Vec<u8> = light
        .pixels()
        .iter()
        .flat_map(|pixel| pixel.to_le_bytes())
        .collect();
    let dark_bytes: Vec<u8> = dark
        .pixels()
        .iter()
        .flat_map(|pixel| pixel.to_le_bytes())
        .collect();

    [
        (Appearance::Light, light_bytes),
        (Appearance::Dark, dark_bytes),
    ]
}

#[test]
fn parity_frames_available() {
    let frames = parity_frames();
    assert!(
        !frames[0].1.is_empty(),
        "light frame bytes should not be empty"
    );
    assert!(
        !frames[1].1.is_empty(),
        "dark frame bytes should not be empty"
    );
}

#[test]
fn all_pixels_are_opaque() {
    let frames = parity_frames();

    for (appearance, bytes) in frames {
        // Each pixel is 4 bytes: RGBA (little-endian u32)
        // Alpha is the 4th byte (index 3) of each pixel
        for (pixel_idx, chunk) in bytes.chunks(4).enumerate() {
            let alpha = chunk[3];
            assert_eq!(
                alpha, 0xFF,
                "pixel {} in {:?} frame should be opaque (alpha=0xFF), got alpha={:#x}",
                pixel_idx, appearance, alpha
            );
        }
    }
}
