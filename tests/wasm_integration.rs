//! Browser integration tests verifying pixel-perfect rendering.
//!
//! These tests require a real browser environment with DOM and canvas support.
//! Run with `wasm-pack test --headless --firefox` or similar.

#![cfg(target_arch = "wasm32")]

use rui_native::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui_native::shell::pixel_conversion::convert_pixels_to_rgba;
use rui_native::theme::Appearance;
use rui_native::wasm::{
    counter_frame_count, init_counter, listen_counter, present_counter, present_parity_frame,
};
use wasm_bindgen::JsCast;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

/// The `<canvas id="surface">` the backend looks for, made if the page has none.
///
/// A test page is not the harness page: nothing has put a surface on it, and a
/// backend that cannot find one reports so rather than drawing. Making it here
/// keeps that failure from reading as a pass.
fn surface() -> web_sys::HtmlCanvasElement {
    let document = web_sys::window()
        .expect("a browser test has a window")
        .document()
        .expect("a browser test has a document");

    let canvas = match document.get_element_by_id("surface") {
        Some(existing) => existing,
        None => {
            let made = document
                .create_element("canvas")
                .expect("a canvas element can be made");
            made.set_id("surface");
            document
                .body()
                .expect("a browser test has a body")
                .append_child(&made)
                .expect("the canvas can be added to the page");
            made
        }
    };

    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().expect("#surface is a canvas");
    canvas.set_width(REFERENCE_WIDTH);
    canvas.set_height(REFERENCE_HEIGHT);
    canvas
}

/// What `examples/parity.html` checks by hand, in a browser a script can drive.
///
/// The page compares the browser against a file a desktop run wrote, which is
/// the whole claim and needs two machines' worth of setup. This is the half
/// that can be a test: the frame the renderer produced is presented through the
/// real [`rui_native::shell::present`] and read straight back off the `<canvas>`, so
/// anything the backend loses on the way — a channel swapped, alpha
/// premultiplied, a row's stride misjudged — shows up as a differing byte.
///
/// Both appearances, because the two differ in almost every pixel and a
/// conversion bug that happens to be invisible on one rarely is on both.
#[wasm_bindgen_test::wasm_bindgen_test]
fn the_backend_presents_the_frame_it_was_given() {
    let canvas = surface();
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .expect("a 2d context can be asked for")
        .expect("a 2d context exists")
        .dyn_into()
        .expect("the context is a 2d one");

    for (name, dark, appearance) in [
        ("light", false, Appearance::Light),
        ("dark", true, Appearance::Dark),
    ] {
        present_parity_frame(dark).expect("the parity frame can be presented");

        let drawn = reference_frame(REFERENCE_WIDTH, REFERENCE_HEIGHT, 1.0, appearance)
            .expect("the embedded faces should parse");
        let expected = convert_pixels_to_rgba(drawn.pixels());

        let read_back = context
            .get_image_data(
                0.0,
                0.0,
                f64::from(REFERENCE_WIDTH),
                f64::from(REFERENCE_HEIGHT),
            )
            .expect("the surface can be read back")
            .data()
            .to_vec();

        assert_eq!(
            read_back.len(),
            expected.len(),
            "{name}: the surface is not the size the frame was drawn at"
        );

        let differing = read_back
            .iter()
            .zip(&expected)
            .filter(|(there, here)| there != here)
            .count();
        assert_eq!(
            differing,
            0,
            "{name}: {differing} of {} bytes changed between drawing the frame and reading it \
             back off the canvas",
            expected.len()
        );
    }
}

/// Verify that Memory state persists across multiple present_counter() calls.
///
/// The counter app owns its Memory instance across frames instead of allocating
/// fresh memory each frame. This test verifies that hover effects, focus states,
/// and animations survive the call to present_counter() by checking that the
/// frame count in Memory increases monotonically — if Memory were reallocated
/// fresh, the frame count would reset.
#[wasm_bindgen_test::wasm_bindgen_test]
fn memory_persists_across_present_counter_calls() {
    let _ = surface(); // Ensure the canvas exists for rendering

    // Initialize the counter app, which allocates Memory once.
    init_counter();

    // Present the first frame and record the frame count.
    present_counter();
    let frame_count_1 = counter_frame_count();
    assert!(
        frame_count_1 > 0,
        "frame count should be positive after first present_counter()"
    );

    // Collect any events (though we won't send any for this test).
    listen_counter();

    // Present the second frame and check that frame count increased.
    present_counter();
    let frame_count_2 = counter_frame_count();
    assert!(
        frame_count_2 > frame_count_1,
        "frame count should increase across present_counter() calls: {} -> {}",
        frame_count_1,
        frame_count_2
    );

    // Present a third frame to further verify persistence.
    listen_counter();
    present_counter();
    let frame_count_3 = counter_frame_count();
    assert!(
        frame_count_3 > frame_count_2,
        "frame count should continue increasing: {} -> {}",
        frame_count_2,
        frame_count_3
    );
}
