#![allow(missing_docs)]

use rui::demo::{self, Counter, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::testing::Harness;
use rui::{image, Appearance};

#[test]
fn wasm_parity_generates_reference() {
    let temp_dir = std::path::PathBuf::from("/tmp/rui-wasm-parity");
    std::fs::create_dir_all(&temp_dir).expect("failed to create temp directory");

    for (name, appearance) in [("light", Appearance::Light), ("dark", Appearance::Dark)] {
        let harness = Harness::new(Counter { count: 0 }, |counter: &Counter| {
            demo::counter_view(counter)
        })
        .size(REFERENCE_WIDTH as f32, REFERENCE_HEIGHT as f32)
        .appearance(appearance);

        let canvas = harness.canvas();
        let pixels = image::rgba(canvas);
        let rgba_path = temp_dir.join(format!("parity-{name}.rgba"));
        std::fs::write(&rgba_path, &pixels).expect("failed to write RGBA file");

        let png = image::png(canvas.width(), canvas.height(), &pixels)
            .expect("PNG encoding should succeed");
        let png_path = temp_dir.join(format!("parity-{name}.png"));
        std::fs::write(&png_path, png).expect("failed to write PNG file");
    }

    assert!(temp_dir.join("parity-light.rgba").exists());
    assert!(temp_dir.join("parity-light.png").exists());
    assert!(temp_dir.join("parity-dark.rgba").exists());
    assert!(temp_dir.join("parity-dark.png").exists());
}
