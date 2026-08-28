//! WASM entry point for the counter app.
//!
//! Exports functions that JavaScript can call to initialize and render the
//! counter interface. The rendering pipeline is identical to the native backend:
//! the view function is called, the result is laid out and drawn to a canvas.

use crate::shell;
use crate::{App, Canvas, El};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static COUNTER_APP: RefCell<Option<CounterApp>> = RefCell::new(None);
}

/// The Counter application state.
#[derive(Clone)]
struct Counter {
    count: i32,
}

/// Render the counter interface.
fn counter_view(counter: &Counter) -> El<Counter> {
    use crate::{button, col, row, title};
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
        let app = App::new("Counter", Counter { count: 0 }, counter_view);
        let fonts = shell::load_system_fonts().expect("fonts should load in wasm");

        // Default wasm canvas size (960x640, same as default WindowOptions)
        let canvas = Canvas::new(960, 640, 1.0);
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
