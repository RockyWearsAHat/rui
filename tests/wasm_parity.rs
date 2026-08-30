//! Integration test for WASM parity verification.
//!
//! Verifies that reference frames can be generated deterministically using embedded fonts
//! for later comparison with WASM-rendered output.

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::Appearance;

/// Compare two RGBA byte buffers pixel-by-pixel.
/// Returns (differing_pixel_count, total_pixels).
pub fn compare_frames(expected: &[u8], actual: &[u8]) -> (usize, usize) {
    if expected.len() != actual.len() {
        return (usize::MAX, expected.len() / 4); // Signal size mismatch
    }
    let total_pixels = expected.len() / 4;
    let diff_count = expected
        .chunks(4)
        .zip(actual.chunks(4))
        .filter(|(exp, act)| exp != act)
        .count();
    (diff_count, total_pixels)
}

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
fn reference_frames_generate_successfully() {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    println!("Light frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);
    println!("Dark frame: {}x{}", REFERENCE_WIDTH, REFERENCE_HEIGHT);

    assert!(
        !light.pixels().is_empty(),
        "light frame should contain pixels"
    );
    assert!(
        !dark.pixels().is_empty(),
        "dark frame should contain pixels"
    );
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

#[test]
fn frames_are_deterministic() {
    let frames1 = parity_frames();
    let frames2 = parity_frames();

    for ((_, bytes1), (_, bytes2)) in frames1.iter().zip(frames2.iter()) {
        assert_eq!(
            bytes1, bytes2,
            "parity frames should be identical across multiple generations (deterministic rendering)"
        );
    }
}

#[test]
fn frame_comparison_detects_identity() {
    let frames = parity_frames();
    let (_, light_bytes) = &frames[0];
    let (diff_count, total_pixels) = compare_frames(light_bytes, light_bytes);

    assert_eq!(
        diff_count, 0,
        "comparing a frame to itself should show 0 differing pixels"
    );
    assert_eq!(
        total_pixels,
        (REFERENCE_WIDTH * REFERENCE_HEIGHT) as usize,
        "total pixels should match frame dimensions"
    );
}

#[test]
fn parity_frames_roundtrip_to_rgba() {
    let frames = parity_frames();
    let (_, original_bytes) = &frames[0];

    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let test_path = temp_dir.join("parity_roundtrip_test.rgba");

    std::fs::write(&test_path, original_bytes).expect("should write RGBA bytes to temporary file");

    // Read back from file
    let loaded_bytes =
        std::fs::read(&test_path).expect("should read RGBA bytes from temporary file");

    // Verify content matches
    assert_eq!(
        &loaded_bytes, original_bytes,
        "RGBA bytes should round-trip through file I/O correctly"
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_path);
}

#[test]
fn parity_frame_byte_count_correct() {
    let frames = parity_frames();

    for (appearance, bytes) in frames {
        let expected_byte_count = (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as usize;
        assert_eq!(
            bytes.len(),
            expected_byte_count,
            "{:?} frame should have {} bytes (width × height × 4), got {}",
            appearance,
            expected_byte_count,
            bytes.len()
        );
    }
}
