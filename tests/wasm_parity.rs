//! Generate native parity reference frames in-process for WASM backend comparison.
//!
//! This test produces the same reference frames as `examples/parity.rs` does,
//! but callable from `cargo test` instead of as a standalone binary.
//! The frames are stored as both `.rgba` (raw pixels for byte-for-byte comparison)
//! and `.png` (for human inspection).

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::{image, Appearance};

/// Every pixel is drawn at one device pixel per logical unit.
/// This matches examples/parity.rs and pins both backend comparisons to the same scale.
const SCALE: f32 = 1.0;

#[test]
fn wasm_parity_generates_reference() {
    let temp_dir = std::path::PathBuf::from("/tmp/rui-wasm-parity");
    std::fs::create_dir_all(&temp_dir).expect("failed to create temp directory");

    for (name, appearance) in [("light", Appearance::Light), ("dark", Appearance::Dark)] {
        let canvas = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, SCALE, appearance)
            .expect("reference_frame should succeed");

        // Write raw RGBA pixels for byte-for-byte comparison
        let pixels = image::rgba(&canvas);
        let rgba_path = temp_dir.join(format!("parity-{name}.rgba"));
        std::fs::write(&rgba_path, &pixels).expect("failed to write RGBA file");

        // Write PNG for human inspection
        let png = image::png(canvas.width(), canvas.height(), &pixels)
            .expect("PNG encoding should succeed");
        let png_path = temp_dir.join(format!("parity-{name}.png"));
        std::fs::write(&png_path, png).expect("failed to write PNG file");
    }

    // Verify both appearances were written
    assert!(temp_dir.join("parity-light.rgba").exists());
    assert!(temp_dir.join("parity-light.png").exists());
    assert!(temp_dir.join("parity-dark.rgba").exists());
    assert!(temp_dir.join("parity-dark.png").exists());
}
