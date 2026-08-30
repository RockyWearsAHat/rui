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
/// Uses the same encoding as `image::rgba()`: R, G, B channels extracted from pixel u32,
/// alpha always 0xFF.
fn parity_frames() -> [(Appearance, Vec<u8>); 2] {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    let light_bytes: Vec<u8> = light
        .pixels()
        .iter()
        .flat_map(|&pixel| vec![(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8, 0xff])
        .collect();
    let dark_bytes: Vec<u8> = dark
        .pixels()
        .iter()
        .flat_map(|&pixel| vec![(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8, 0xff])
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

/// Write parity reference frames to disk in the format `examples/parity.html` expects.
///
/// Writes two files per appearance:
/// - `parity-<appearance>.rgba` — raw RGBA bytes for byte-accurate comparison
///
/// This function enables programmatic frame generation for the browser parity workflow.
pub fn write_parity_frames_to_directory(directory: &str) -> std::io::Result<()> {
    use std::path::Path;

    let dir = Path::new(directory);
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    let frames = parity_frames();
    for (appearance, bytes) in frames {
        let name = match appearance {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        };

        let rgba_path = dir.join(format!("parity-{}.rgba", name));
        std::fs::write(&rgba_path, &bytes)?;
    }

    Ok(())
}

#[test]
fn parity_frames_can_be_serialized_for_browser() {
    let frames = parity_frames();

    // Simulate what parity.html expects: raw RGBA byte buffers for light and dark
    for (appearance, bytes) in frames {
        let appearance_name = match appearance {
            Appearance::Light => "light",
            Appearance::Dark => "dark",
        };

        // Verify the bytes are in the format getImageData would return
        // (width * height * 4 bytes in RGBA order)
        assert_eq!(
            bytes.len(),
            (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as usize,
            "frame should be serializable as raw RGBA for browser comparison"
        );

        // Every 4 bytes should represent one pixel (RGBA)
        for chunk in bytes.chunks(4) {
            assert_eq!(
                chunk.len(),
                4,
                "each pixel in {} frame should be exactly 4 bytes (RGBA)",
                appearance_name
            );
        }
    }
}

#[test]
fn parity_frames_can_write_to_browser_directory() {
    let temp_dir = std::env::temp_dir().join("rui_parity_test");
    let dir_str = temp_dir.to_string_lossy().to_string();

    // Write frames to temporary directory
    write_parity_frames_to_directory(&dir_str).expect("should write parity frames to directory");

    // Verify both .rgba files exist and have correct byte counts
    for appearance_name in &["light", "dark"] {
        let rgba_path = temp_dir.join(format!("parity-{}.rgba", appearance_name));
        assert!(
            rgba_path.exists(),
            "parity-{}.rgba should exist after write",
            appearance_name
        );

        let metadata = std::fs::metadata(&rgba_path).expect("should read metadata for .rgba file");
        let expected_size = (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as u64;
        assert_eq!(
            metadata.len(),
            expected_size,
            "parity-{}.rgba should be {} bytes, got {}",
            appearance_name,
            expected_size,
            metadata.len()
        );
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn programmatic_frames_match_example_output() {
    // Generate frames using the parity example (reference implementation)
    let example_dir = std::env::temp_dir().join("rui_parity_example_verify");
    let example_dir_str = example_dir.to_string_lossy().to_string();

    // Clean up any prior run
    let _ = std::fs::remove_dir_all(&example_dir);

    // Create directory for example output
    std::fs::create_dir_all(&example_dir)
        .expect("should create temporary directory for example output");

    // Run the parity example to generate reference frames
    let output = std::process::Command::new("cargo")
        .args(["run", "-p", "rui", "--example", "parity", "--"])
        .arg(&example_dir_str)
        .output()
        .expect("should run parity example");

    if !output.status.success() {
        panic!(
            "parity example failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Load the example-generated RGBA files
    let example_light_path = example_dir.join("parity-light.rgba");
    let example_dark_path = example_dir.join("parity-dark.rgba");

    let example_light_bytes = std::fs::read(&example_light_path)
        .expect("should read parity-light.rgba from example output");
    let example_dark_bytes = std::fs::read(&example_dark_path)
        .expect("should read parity-dark.rgba from example output");

    // Generate frames programmatically using the test helper
    let frames = parity_frames();
    let (_, programmatic_light_bytes) = &frames[0];
    let (_, programmatic_dark_bytes) = &frames[1];

    // Compare light frames
    let (diff_light, total_pixels) = compare_frames(&example_light_bytes, programmatic_light_bytes);
    assert_eq!(
        diff_light, 0,
        "programmatic light frame should match example output exactly ({} pixels)",
        total_pixels
    );

    // Compare dark frames
    let (diff_dark, _) = compare_frames(&example_dark_bytes, programmatic_dark_bytes);
    assert_eq!(
        diff_dark, 0,
        "programmatic dark frame should match example output exactly"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&example_dir);
}

#[test]
fn programmatic_frames_ready_for_browser_parity() {
    // This test verifies the complete workflow: programmatic frames can be
    // written to the location `examples/parity.html` expects for browser comparison.
    // The parity.html script loads frames from `/target/parity/parity-{light,dark}.rgba`
    // and compares them byte-for-byte against what the WASM backend draws.

    let target_parity_dir = "target/parity";

    // Write programmatic frames to the exact location parity.html will fetch from
    write_parity_frames_to_directory(target_parity_dir)
        .expect("should write frames to target/parity for browser parity verification");

    // Verify both files exist and are the correct size
    for appearance_name in &["light", "dark"] {
        let rgba_path = std::path::PathBuf::from(target_parity_dir)
            .join(format!("parity-{}.rgba", appearance_name));

        assert!(
            rgba_path.exists(),
            "parity-{}.rgba should exist at target/parity for browser to load",
            appearance_name
        );

        let metadata =
            std::fs::metadata(&rgba_path).expect("should read metadata for reference frame");
        let expected_size = (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as u64;

        assert_eq!(
            metadata.len(),
            expected_size,
            "parity-{}.rgba should be {} bytes for {}x{} frame",
            appearance_name,
            expected_size,
            REFERENCE_WIDTH,
            REFERENCE_HEIGHT
        );
    }

    // Load and verify the content is correct
    let light_bytes =
        std::fs::read(std::path::PathBuf::from(target_parity_dir).join("parity-light.rgba"))
            .expect("should read parity-light.rgba");

    let dark_bytes =
        std::fs::read(std::path::PathBuf::from(target_parity_dir).join("parity-dark.rgba"))
            .expect("should read parity-dark.rgba");

    // Frames should not be empty
    assert!(!light_bytes.is_empty(), "light frame should contain pixels");
    assert!(!dark_bytes.is_empty(), "dark frame should contain pixels");

    // Frames should be different (light vs dark modes render differently)
    assert_ne!(
        light_bytes, dark_bytes,
        "light and dark frames should differ (different appearance = different rendering)"
    );
}
