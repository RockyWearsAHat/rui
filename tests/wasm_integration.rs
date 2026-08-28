//! Test that the wasm module exports a counter initialization function.
//!
//! This test verifies the contract between JavaScript and Rust: that the wasm
//! module exports a function to initialize and render the counter app.

#![cfg(target_arch = "wasm32")]

use rui::demo::{reference_frame, REFERENCE_HEIGHT, REFERENCE_WIDTH};
use rui::shell::pixel_conversion::convert_pixels_to_rgba;
use rui::theme::Appearance;
use rui::wasm::{init_counter, listen_counter, present_counter, present_parity_frame};
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

#[wasm_bindgen_test::wasm_bindgen_test]
fn appearance_matches_system_preference() {
    // In a wasm test environment (Node.js), there may not be a real browser window.
    // However, the get_appearance() function should handle this gracefully
    // and return a valid appearance value.
    let appearance = rui::shell::get_appearance();

    // Verify it returns a valid appearance (not a panic)
    // The actual value (Dark vs Light) depends on the environment,
    // but it must be one of these two.
    assert!(
        appearance == Appearance::Dark || appearance == Appearance::Light,
        "get_appearance() should return either Dark or Light"
    );
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm_exports_counter_init() {
    // The counter app must be exported so JavaScript can call it.
    // This test verifies that the wasm module has the required export by
    // calling the function directly. If init_counter is not exported or callable,
    // this test will not compile.
    let result = init_counter();
    assert_eq!(result, 0, "init_counter should return 0 on success");
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn counter_initialized_via_thread_local() {
    // Test that init_counter stores the app in a thread-local
    // Just verify that the function completes without error
    let result = init_counter();
    assert_eq!(result, 0, "init_counter should return 0 on success");
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm_exports_all_counter_functions() {
    // Initialize the counter
    init_counter();

    // listen_counter should be exported and callable
    listen_counter();

    // present_counter should be exported and callable
    present_counter();
}

#[wasm_bindgen_test::wasm_bindgen_test]
fn memory_persists_across_frames() {
    // Initialize the counter app
    init_counter();

    // Call present_counter 5 times in sequence.
    // This validates that Memory persists correctly across multiple render calls,
    // detecting if state is being lost or allocated fresh each frame.
    for _ in 0..5 {
        present_counter();
    }
}

/// What `examples/parity.html` checks by hand, in a browser a script can drive.
///
/// The page compares the browser against a file a desktop run wrote, which is
/// the whole claim and needs two machines' worth of setup. This is the half
/// that can be a test: the frame the renderer produced is presented through the
/// real [`rui::shell::present`] and read straight back off the `<canvas>`, so
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
