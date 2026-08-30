//! Integration test for WASM parity verification.
//!
//! Generates light and dark reference frames using embedded font pair for comparison with WASM-rendered output.

use rui::demo::{self, Counter, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::testing::Harness;
use rui::{image, Appearance};

/// Reference frame data: pixel buffer and metadata about dimensions.
struct ReferenceFrame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

/// Generate a reference frame for the given appearance mode.
fn reference_frame(appearance: Appearance) -> ReferenceFrame {
    let harness = Harness::new(Counter { count: 0 }, |counter: &Counter| {
        demo::counter_view(counter)
    })
    .size(REFERENCE_WIDTH as f32, REFERENCE_HEIGHT as f32)
    .appearance(appearance);

    let canvas = harness.canvas();
    let width = canvas.width();
    let height = canvas.height();
    let pixels = image::rgba(canvas);

    ReferenceFrame {
        pixels,
        width,
        height,
    }
}

fn parity_frames() -> [(Appearance, Vec<u8>); 2] {
    let mut frames = [
        (Appearance::Light, Vec::new()),
        (Appearance::Dark, Vec::new()),
    ];

    for (i, &appearance) in [Appearance::Light, Appearance::Dark].iter().enumerate() {
        let frame = reference_frame(appearance);
        frames[i] = (appearance, frame.pixels);
    }

    frames
}

#[test]
fn reference_frames_generate_successfully() {
    let light_frame = reference_frame(Appearance::Light);
    let dark_frame = reference_frame(Appearance::Dark);

    println!("Light frame: {}x{}", light_frame.width, light_frame.height);
    println!("Dark frame: {}x{}", dark_frame.width, dark_frame.height);

    assert_eq!(light_frame.width, REFERENCE_WIDTH);
    assert_eq!(light_frame.height, REFERENCE_HEIGHT);
    assert_eq!(dark_frame.width, REFERENCE_WIDTH);
    assert_eq!(dark_frame.height, REFERENCE_HEIGHT);
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
fn wasm_parity_generates_reference() {
    let temp_dir = std::path::PathBuf::from("/tmp/rui-wasm-parity");
    std::fs::create_dir_all(&temp_dir).expect("failed to create temp directory");

    for (name, appearance) in [("light", Appearance::Light), ("dark", Appearance::Dark)] {
        let frame = reference_frame(appearance);

        let rgba_path = temp_dir.join(format!("parity-{name}.rgba"));
        std::fs::write(&rgba_path, &frame.pixels).expect("failed to write RGBA file");

        let png = image::png(frame.width, frame.height, &frame.pixels)
            .expect("PNG encoding should succeed");
        let png_path = temp_dir.join(format!("parity-{name}.png"));
        std::fs::write(&png_path, png).expect("failed to write PNG file");
    }

    assert!(temp_dir.join("parity-light.rgba").exists());
    assert!(temp_dir.join("parity-light.png").exists());
    assert!(temp_dir.join("parity-dark.rgba").exists());
    assert!(temp_dir.join("parity-dark.png").exists());
}

#[test]
fn all_pixels_are_opaque() {
    for appearance in [Appearance::Light, Appearance::Dark] {
        let frame = reference_frame(appearance);

        for (i, chunk) in frame.pixels.chunks_exact(4).enumerate() {
            let alpha = chunk[3];
            assert_eq!(
                alpha, 0xFF,
                "Pixel {} has alpha={}, expected 0xFF (fully opaque)",
                i, alpha
            );
        }
    }
}

#[test]
fn light_and_dark_differ() {
    let light_frame = reference_frame(Appearance::Light);
    let dark_frame = reference_frame(Appearance::Dark);

    assert_ne!(
        light_frame.pixels, dark_frame.pixels,
        "light and dark mode frames should have different pixel data"
    );
}
