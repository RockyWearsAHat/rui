//! Integration test for WASM parity verification.
//!
//! Verifies that reference frames can be generated deterministically using embedded fonts
//! for later comparison with WASM-rendered output.

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::Appearance;

/// Generate both light and dark reference frames as RGBA byte buffers.
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
fn wasm_parity_reference_frames_generate_successfully() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should generate");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should generate");

    assert!(!light.pixels().is_empty(), "light frame pixels generated");
    assert!(!dark.pixels().is_empty(), "dark frame pixels generated");

    println!("Light frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);
    println!("Dark frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);
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
                "{:?} frame pixel {} has alpha={:02x}, expected 0xff",
                appearance, i, alpha
            );
        }
    }
}

#[test]
fn wasm_parity_light_and_dark_differ() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    assert_ne!(
        light.pixels(),
        dark.pixels(),
        "light and dark mode frames should differ"
    );
}

#[test]
fn wasm_parity_frame_dimensions_are_correct() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("reference frame should render successfully");

    let pixel_count = light.pixels().len();
    let expected_count = (REFERENCE_WIDTH * REFERENCE_HEIGHT) as usize;

    assert_eq!(
        pixel_count, expected_count,
        "frame should have {}x{} pixels, got {}",
        REFERENCE_WIDTH, REFERENCE_HEIGHT, pixel_count
    );
}

#[test]
fn wasm_parity_rendering_is_deterministic() {
    let frame1 = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("first render");
    let frame2 = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("second render");

    assert_eq!(
        frame1.pixels(),
        frame2.pixels(),
        "identical parameters should produce identical pixels"
    );
}

#[test]
fn wasm_parity_scale_factor_affects_rendering() {
    let scale_1x = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("1x scale render");
    let scale_2x = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 2.0, Appearance::Light)
        .expect("2x scale render");

    assert_ne!(
        scale_1x.pixels(),
        scale_2x.pixels(),
        "different scales should produce different pixels"
    );
}

#[test]
fn wasm_parity_dark_mode_is_darker() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light render");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark render");

    let light_brightness: u64 = light
        .pixels()
        .iter()
        .map(|&pixel| {
            let r = (pixel & 0xFF) as u64;
            let g = ((pixel >> 8) & 0xFF) as u64;
            let b = ((pixel >> 16) & 0xFF) as u64;
            r + g + b
        })
        .sum();

    let dark_brightness: u64 = dark
        .pixels()
        .iter()
        .map(|&pixel| {
            let r = (pixel & 0xFF) as u64;
            let g = ((pixel >> 8) & 0xFF) as u64;
            let b = ((pixel >> 16) & 0xFF) as u64;
            r + g + b
        })
        .sum();

    assert!(
        dark_brightness < light_brightness,
        "dark mode should be darker than light mode"
    );
}

#[test]
fn wasm_parity_light_frame_reproducible() {
    let frame1 =
        reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light).expect("first");
    let frame2 =
        reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light).expect("second");

    assert_eq!(frame1.pixels(), frame2.pixels(), "light frames match");
}

#[test]
fn wasm_parity_dark_frame_reproducible() {
    let frame1 =
        reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark).expect("first");
    let frame2 =
        reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark).expect("second");

    assert_eq!(frame1.pixels(), frame2.pixels(), "dark frames match");
}
