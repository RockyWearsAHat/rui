//! WASM entry point for the counter app.
//!
//! Exports functions that JavaScript can call to initialize and render the
//! counter interface. The rendering pipeline is identical to the native backend:
//! the view function is called, the result is laid out and drawn to a canvas.

use crate::input::Event;
use crate::shell;
use crate::theme::Appearance;
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
}

impl CounterApp {
    /// Create a new counter app with default window size.
    pub fn new() -> Self {
        let app = App::new("Counter", Counter { count: 0 }, counter_view);
        let fonts = shell::load_system_fonts().expect("fonts should load in wasm");

        // Default wasm canvas size (960x640, same as default WindowOptions)
        let canvas = Canvas::new(960, 640, 1.0);

        Self { app, canvas, fonts }
    }
}

impl Default for CounterApp {
    fn default() -> Self {
        Self::new()
    }
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
/// Returns the canvas dimensions on success for testing.
pub fn present_counter() -> Result<(u32, u32), crate::Error> {
    COUNTER_APP.with(|app| {
        let mut app_borrow = app.borrow_mut();
        if let Some(app) = app_borrow.as_mut() {
            let appearance = Appearance::Light; // TODO: read from browser prefers-color-scheme

            // Draw the app into the canvas
            app.app.draw_into(
                &mut app.canvas,
                &mut app.fonts,
                appearance,
                &mut crate::Memory::new(),
            );

            // Present the canvas to the browser
            shell::present(&app.canvas)?;

            Ok((app.canvas.width(), app.canvas.height()))
        } else {
            Err(crate::Error::Platform("app not initialized".to_string()))
        }
    })
}

/// Collect events from the browser canvas and return them.
///
/// Called by JavaScript before each `present_counter` to apply user input.
pub fn listen_counter() -> Result<Vec<Event>, crate::Error> {
    shell::listen()
}
