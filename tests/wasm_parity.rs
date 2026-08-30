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
fn wasm_parity_frames_available() {
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
fn reference_frames_generate_successfully() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    let light_pixels = light.pixels();
    let dark_pixels = dark.pixels();

    println!("Light frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);
    println!("Dark frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);

    assert!(
        !light_pixels.is_empty(),
        "light frame should contain pixels"
    );
    assert!(!dark_pixels.is_empty(), "dark frame should contain pixels");
    assert!(
        light_pixels.iter().all(|pixel| pixel >> 24 == 0xff),
        "all light frame pixels should be opaque"
    );
    assert!(
        dark_pixels.iter().all(|pixel| pixel >> 24 == 0xff),
        "all dark frame pixels should be opaque"
    );
    assert_ne!(
        light_pixels, dark_pixels,
        "light and dark frames should differ"
    );
}

#[test]
fn wasm_parity_all_pixels_are_opaque() {
    for &appearance in &[Appearance::Light, Appearance::Dark] {
        let frame = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, appearance)
            .expect("reference frame should render successfully");

        for (i, &pixel) in frame.pixels().iter().enumerate() {
            let alpha = pixel >> 24;
            assert_eq!(
                alpha, 0xff,
                "{:?} frame pixel {} has alpha={:02x}, expected 0xff (fully opaque)",
                appearance, i, alpha
            );
        }
    }
}

#[test]
fn light_and_dark_differ() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    assert_ne!(
        light.pixels(),
        dark.pixels(),
        "light and dark mode frames should have different pixel data"
    );
}
