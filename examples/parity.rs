//! The native half of the backend parity check: one frame of the counter,
//! drawn with no window, written out for a browser to be compared against.
//!
//! Run with `cargo run -p rui --example parity -- <directory>`. It writes four
//! files per appearance:
//!
//! * `parity-<appearance>.png` — for a person to look at.
//! * `parity-<appearance>.rgba` — the same pixels, raw, for the page to diff.
//!
//! Two files rather than one because they answer different questions. A PNG is
//! what a reviewer wants and the worst possible thing to compare against: a
//! browser decodes it through its own colour management, so a diff against a
//! decoded PNG measures the image pipeline as much as the renderer. The `.rgba`
//! file is `width * height * 4` bytes in the order `putImageData` wants them,
//! which the page can `fetch()` and compare with nothing in between.
//!
//! `examples/parity.html` loads the wasm build, has it present the same frame
//! through the real browser backend, reads the `<canvas>` back with
//! `getImageData`, and reports the first byte that differs — or that none does.

use rui_native::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui_native::{image, Appearance};

/// Every pixel is drawn at one device pixel per logical unit.
///
/// Not because the renderer cannot do better — the gallery draws at 2.0 — but
/// because a `<canvas>` element's backing store is its `width`/`height`
/// attributes, and pinning both sides to 1.0 keeps the browser's own device
/// pixel ratio out of a comparison that is not about it.
const SCALE: f32 = 1.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args().nth(1).unwrap_or_else(|| ".".into());

    for (name, appearance) in [("light", Appearance::Light), ("dark", Appearance::Dark)] {
        let canvas = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, SCALE, appearance)?;
        let pixels = image::rgba(&canvas);

        let raw = format!("{directory}/parity-{name}.rgba");
        std::fs::write(&raw, &pixels)?;
        println!(
            "wrote {raw} ({}x{}, {} bytes)",
            canvas.width(),
            canvas.height(),
            pixels.len()
        );

        let png = image::png(canvas.width(), canvas.height(), &pixels)
            .ok_or("the frame could not be encoded")?;
        let path = format!("{directory}/parity-{name}.png");
        std::fs::write(&path, png)?;
        println!("wrote {path}");
    }
    Ok(())
}
