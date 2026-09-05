//! The counter program, kept in the library so every backend draws the *same* one.
//!
//! [`crate::shell`]'s own doc comment claims that a backend does five small
//! things and that everything above it is decided identically everywhere — so a
//! native window and a browser canvas showing the same application must show
//! the same pixels. Checking that claim needs the two sides to be running one
//! description, not two copies of one that have not drifted apart yet, which is
//! why the counter lives here rather than in `examples/counter.rs` alone.
//!
//! Three drivers, one description: `examples/counter.rs` opens it in a native
//! window, [`crate::wasm`] drives it from `requestAnimationFrame` in a browser,
//! and [`reference_frame`] draws one frame of it with no window at all.
//! `examples/parity.rs` writes that frame out, and the page in
//! `examples/parity.html` compares it — byte for byte — with what the browser
//! backend actually put on its canvas.

use crate::shell::embedded_fonts::{embedded_mono_font, embedded_ui_font};
use crate::shell::Error;
use crate::text::Fonts;
use crate::{button, col, row, title, App, Appearance, Canvas, El, LoadedFonts};

/// The width the parity frame is drawn at, matching [`crate::WindowOptions`].
pub const REFERENCE_WIDTH: u32 = 960;

/// The height the parity frame is drawn at, matching [`crate::WindowOptions`].
pub const REFERENCE_HEIGHT: u32 = 640;

/// Everything this program knows.
pub struct Counter {
    /// How far the counter has been counted.
    pub count: i32,
}

/// What should be on screen, given that.
pub fn counter_view(counter: &Counter) -> El<Counter> {
    col((
        title(format!("{}", counter.count))
            .text_size(56.0)
            .bold()
            .center_text(),
        row((
            button("−")
                .w(56.0)
                .on_click(|counter: &mut Counter| counter.count -= 1),
            button("Reset")
                .w(80.0)
                .on_click(|counter: &mut Counter| counter.count = 0),
            button("+")
                .primary()
                .w(56.0)
                .on_click(|counter: &mut Counter| counter.count += 1),
        ))
        .gap(8.0),
    ))
    .gap(20.0)
    .pad(32.0)
    .center()
}

/// A counter application at zero, ready for a window or a canvas.
pub fn counter_app() -> App<Counter> {
    App::new("Counter", Counter { count: 0 }, counter_view)
}

/// The two faces the parity check pins both sides to.
///
/// [`crate::shell::load_system_fonts`] answers the desktop's own faces on a
/// desktop and the embedded ones in a browser. That is the right answer for
/// either on its own and the wrong one for comparing them: two different
/// typefaces cannot rasterise to the same pixels, so a diff between them would
/// only ever be measuring the font search. This answers the embedded pair
/// everywhere, which is what makes the comparison about the backend.
pub fn embedded_pair() -> Result<LoadedFonts, Error> {
    let mut fonts = Fonts::new();
    let ui_font = fonts.add(embedded_ui_font()?);
    let mono_font = fonts.add(embedded_mono_font()?);
    Ok(LoadedFonts {
        fonts,
        ui_font,
        mono_font,
    })
}

/// One frame of the counter, drawn with no window and no history.
///
/// Deterministic on purpose: a fresh [`crate::Memory`] every time, so nothing
/// mid-animation and nothing hovered can make the same call answer two
/// different pictures. That is what lets a native run and a browser run be
/// compared byte for byte rather than approximately.
pub fn reference_frame(
    width: u32,
    height: u32,
    scale: f32,
    appearance: Appearance,
) -> Result<Canvas, Error> {
    let mut fonts = embedded_pair()?;
    Ok(counter_app().render(width, height, scale, appearance, &mut fonts))
}

/// Generate both light and dark reference frames as RGBA byte buffers.
///
/// Returns a 2-element array: `[(Appearance::Light, pixels), (Appearance::Dark, pixels)]`.
/// Uses the same encoding as [`crate::image::rgba`]: R, G, B channels extracted from
/// pixel u32, alpha always 0xFF.
///
/// This helper is used by parity verification tests to compare native and WASM
/// rendering pixel-for-pixel.
pub fn parity_frames() -> [(Appearance, Vec<u8>); 2] {
    let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
        .expect("light reference frame should render successfully");
    let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
        .expect("dark reference frame should render successfully");

    let light_bytes = crate::image::rgba(&light);
    let dark_bytes = crate::image::rgba(&dark);

    [
        (Appearance::Light, light_bytes),
        (Appearance::Dark, dark_bytes),
    ]
}

/// Render a parity frame and return its pixels as RGBA bytes.
///
/// Core logic for rendering a parity frame to RGBA bytes. Used by the WASM
/// parity frame rendering function and testable from native code.
///
/// Returns the RGBA bytes (4 bytes per pixel, 960x640 = 2,457,600 bytes)
/// encoded in the same format as [`crate::image::rgba`].
pub fn render_parity_frame_rgba(dark: bool) -> Result<Vec<u8>, String> {
    let appearance = if dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    let canvas = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, appearance)
        .map_err(|error| format!("the parity frame could not be drawn: {error}"))?;

    Ok(crate::image::rgba(&canvas))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reference_frame_is_the_same_picture_every_time() {
        let first = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
            .expect("the embedded faces should parse");
        let again = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
            .expect("the embedded faces should parse");
        assert_eq!(
            first.pixels(),
            again.pixels(),
            "a reference frame that is not reproducible cannot be compared against"
        );
    }

    #[test]
    fn every_reference_pixel_is_opaque() {
        // The browser un-premultiplies whatever `putImageData` was handed, so a
        // frame carrying anything but a full alpha byte would read back a
        // different colour than it went in with, and the comparison would be
        // measuring the canvas rather than the renderer.
        let frame = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
            .expect("the embedded faces should parse");
        assert!(
            frame.pixels().iter().all(|pixel| pixel >> 24 == 0xff),
            "the canvas handed to a backend must be opaque"
        );
    }

    #[test]
    fn light_and_dark_are_different_pictures() {
        let light = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Light)
            .expect("the embedded faces should parse");
        let dark = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, Appearance::Dark)
            .expect("the embedded faces should parse");
        assert_ne!(light.pixels(), dark.pixels());
    }

    /// Wrapper function for tests that need a synchronous Vec<u8> result.
    fn render_wasm_parity_frame(dark: bool) -> Vec<u8> {
        render_parity_frame_rgba(dark).expect("parity frame should render successfully")
    }

    #[test]
    fn render_wasm_parity_frame_light_produces_correct_buffer() {
        let pixels = render_wasm_parity_frame(false);
        let expected_bytes = (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as usize;
        assert_eq!(
            pixels.len(),
            expected_bytes,
            "light frame should return {}x{}x4 = {} bytes",
            REFERENCE_WIDTH,
            REFERENCE_HEIGHT,
            expected_bytes
        );
        assert!(!pixels.is_empty(), "light frame buffer should not be empty");
    }

    #[test]
    fn render_wasm_parity_frame_dark_produces_correct_buffer() {
        let pixels = render_wasm_parity_frame(true);
        let expected_bytes = (REFERENCE_WIDTH * REFERENCE_HEIGHT * 4) as usize;
        assert_eq!(
            pixels.len(),
            expected_bytes,
            "dark frame should return {}x{}x4 = {} bytes",
            REFERENCE_WIDTH,
            REFERENCE_HEIGHT,
            expected_bytes
        );
        assert!(!pixels.is_empty(), "dark frame buffer should not be empty");
    }
}
