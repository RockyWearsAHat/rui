//! Generate and verify parity reference frames for WASM backend testing.
//!
//! This test generates light and dark reference frames in-process using the
//! rendering pipeline, storing them as temporary files that can be used by
//! WASM parity tests to verify pixel-perfect rendering.

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::{image, Appearance};
use std::fs;

const SCALE: f32 = 1.0;

#[test]
fn wasm_parity_generates_reference() {
    let temp_dir = std::env::temp_dir().join("rui-wasm-parity");
    fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

    for (name, appearance) in [("light", Appearance::Light), ("dark", Appearance::Dark)] {
        let canvas = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, SCALE, appearance)
            .expect("reference frame generation failed");
        let pixels = image::rgba(&canvas);

        let raw_path = temp_dir.join(format!("parity-{}.rgba", name));
        fs::write(&raw_path, &pixels).expect("failed to write .rgba file");
        assert!(raw_path.exists(), "parity-{}.rgba was not created", name);

        let png_bytes =
            image::png(canvas.width(), canvas.height(), &pixels).expect("PNG encoding failed");
        let png_path = temp_dir.join(format!("parity-{}.png", name));
        fs::write(&png_path, png_bytes).expect("failed to write .png file");
        assert!(png_path.exists(), "parity-{}.png was not created", name);
    }
}
