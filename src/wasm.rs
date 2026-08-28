//! WASM entry point for the counter app.
//!
//! Exports functions that JavaScript can call to initialize and render the
//! counter interface. The rendering pipeline is identical to the native backend:
//! the view function is called, the result is laid out and drawn to a canvas.
//!
//! The description itself is not written here. It lives in [`crate::demo`],
//! which `examples/counter.rs` opens in a native window and `examples/parity.rs`
//! draws to a PNG — one function, three drivers, so "the same frame everywhere"
//! is something that can be checked rather than only asserted.

use crate::demo::{self, Counter};
use crate::shell;
use crate::theme::Appearance;
use crate::{App, Canvas};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static COUNTER_APP: RefCell<Option<CounterApp>> = const { RefCell::new(None) };
}

/// The counter app wrapper for wasm, holding app state and rendering surfaces.
pub struct CounterApp {
    app: App<Counter>,
    canvas: Canvas,
    fonts: crate::LoadedFonts,
    memory: crate::Memory,
}

impl CounterApp {
    /// Create a new counter app with default window size.
    pub fn new() -> Self {
        let app = demo::counter_app();
        let fonts = shell::load_system_fonts().expect("fonts should load in wasm");

        // Default wasm canvas size (960x640, same as default WindowOptions)
        let canvas = Canvas::new(demo::REFERENCE_WIDTH, demo::REFERENCE_HEIGHT, 1.0);
        let memory = crate::Memory::new();

        Self {
            app,
            canvas,
            fonts,
            memory,
        }
    }
}

impl Default for CounterApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts the counter and lets the library drive its own frames.
///
/// The whole of what a page has to do. Everything below this is the same as it
/// is on a desktop — the same `App`, the same view, the same loop body — and
/// the call returns at once rather than when the interface is finished, because
/// what it starts is a `requestAnimationFrame` loop and not a `while`.
///
/// The three functions below are the other way of doing this, for a page that
/// already has a loop of its own and wants to draw one frame inside it.
#[wasm_bindgen]
pub fn start_counter() -> Result<(), JsValue> {
    let fonts =
        shell::load_system_fonts().map_err(|error| JsValue::from_str(&error.to_string()))?;
    App::new("Counter", Counter { count: 0 }, counter_view)
        .run_with_fonts(fonts)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Initialize the counter app and store it for use by present/listen.
#[wasm_bindgen]
pub fn init_counter() -> i32 {
    COUNTER_APP.with(|app| {
        *app.borrow_mut() = Some(CounterApp::new());
    });
    0
}

/// Render the counter app to pixels and present them to the browser canvas.
///
/// Called by JavaScript after events have been collected via `listen_counter`.
#[wasm_bindgen]
pub fn present_counter() {
    COUNTER_APP.with(|app| {
        let mut app_borrow = app.borrow_mut();
        if let Some(app) = app_borrow.as_mut() {
            let appearance = shell::get_appearance();

            // Draw the app into the canvas
            app.app
                .draw_into(&mut app.canvas, &mut app.fonts, appearance, &mut app.memory);

            // Present the canvas to the browser
            let _ = shell::present(&app.canvas);
        }
    })
}

/// Collect events from the browser canvas and apply them.
///
/// Called by JavaScript before each `present_counter` to apply user input.
#[wasm_bindgen]
pub fn listen_counter() {
    let _ = shell::listen();
}

/// Draw the parity frame and present it through the browser backend.
///
/// The frame is [`crate::demo::reference_frame`] — the same call
/// `examples/parity.rs` makes on a desktop, with the same size, the same scale,
/// the same embedded faces and a freshly zeroed [`crate::Memory`] — and it
/// reaches the page through the ordinary [`shell::present`], so what ends up on
/// the `<canvas>` has been through the real backend rather than a shortcut cut
/// for the test. Reading those pixels back and diffing them against the desktop
/// file is then a measurement of the backend and nothing else.
///
/// Deliberately not driven from the animation loop: it takes its appearance as
/// an argument instead of asking the page, so the same call can be made for
/// light and for dark whatever the browser is set to.
#[wasm_bindgen]
pub fn present_parity_frame(dark: bool) -> Result<(), JsValue> {
    let appearance = if dark {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    let canvas = demo::reference_frame(
        demo::REFERENCE_WIDTH,
        demo::REFERENCE_HEIGHT,
        1.0,
        appearance,
    )
    .map_err(|error| JsValue::from_str(&format!("the parity frame could not be drawn: {error}")))?;

    shell::present(&canvas).map_err(|error| {
        JsValue::from_str(&format!("the parity frame could not be shown: {error}"))
    })
}

/// The size the parity frame is drawn at, as `[width, height]`.
///
/// The page sizes its `<canvas>` from this rather than repeating the numbers,
/// because a `<canvas>` of the wrong size would make every pixel differ for a
/// reason that has nothing to do with the backend.
#[wasm_bindgen]
pub fn parity_frame_size() -> Vec<u32> {
    vec![demo::REFERENCE_WIDTH, demo::REFERENCE_HEIGHT]
}
