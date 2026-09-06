//! Visual parity test — renders the reference frame to PNG in light and dark modes.
//!
//! Usage: `cargo run -p rui --example parity`
//!
//! Generates parity-light.png and parity-dark.png showing the same view in both modes.
//! Used to verify pixel-for-pixel consistency between native and WebAssembly backends.

use rui::{demo, Appearance};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "examples";

    // Generate reference frames for light and dark modes
    let light_canvas = demo::reference_frame(
        demo::REFERENCE_WIDTH,
        demo::REFERENCE_HEIGHT,
        1.0,
        Appearance::Light,
    )?;

    let dark_canvas = demo::reference_frame(
        demo::REFERENCE_WIDTH,
        demo::REFERENCE_HEIGHT,
        1.0,
        Appearance::Dark,
    )?;

    // Convert canvases to RGBA
    let light_rgba = rui::image::rgba(&light_canvas);
    let dark_rgba = rui::image::rgba(&dark_canvas);

    // Encode as PNG
    let light_png = rui::image::png(demo::REFERENCE_WIDTH, demo::REFERENCE_HEIGHT, &light_rgba)
        .ok_or("Failed to encode light mode PNG")?;
    let dark_png = rui::image::png(demo::REFERENCE_WIDTH, demo::REFERENCE_HEIGHT, &dark_rgba)
        .ok_or("Failed to encode dark mode PNG")?;

    // Write to files
    let light_path = Path::new(output_dir).join("parity-light.png");
    let dark_path = Path::new(output_dir).join("parity-dark.png");

    fs::write(&light_path, light_png)?;
    fs::write(&dark_path, dark_png)?;

    println!(
        "✓ parity-light.png ({} bytes)",
        fs::metadata(&light_path)?.len()
    );
    println!(
        "✓ parity-dark.png ({} bytes)",
        fs::metadata(&dark_path)?.len()
    );
    println!();
    println!("Compare at: examples/parity.html");

    Ok(())
}
