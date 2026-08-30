//! Integration test for WASM parity verification.
//!
//! Verifies that reference frames can be generated deterministically using embedded fonts
//! for later comparison with WASM-rendered output.

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::Appearance;

#[test]
fn wasm_parity_reference_frames() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    let light_pixels = light.pixels();
    let dark_pixels = dark.pixels();

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
