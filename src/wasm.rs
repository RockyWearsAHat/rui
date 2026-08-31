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

use crate::app::App;
use crate::canvas::Canvas;
use crate::demo::{self, Counter};
use crate::input::{Event, Input};
use crate::memory::Memory;
use crate::shell::{self, LoadedFonts};
use crate::text::Fonts;
use crate::theme::{Appearance, Theme};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static COUNTER_APP: RefCell<Option<CounterState>> = const { RefCell::new(None) };
}

struct CounterState {
    app: App<Counter>,
    fonts: Fonts,
    canvas: Canvas,
    memory: Memory,
    input: Input,
    events: Vec<Event>,
    ui_font: crate::text::FontId,
    mono_font: crate::text::FontId,
    last_frame_time: crate::shell::clock::Moment,
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
    demo::counter_app()
        .run_with_fonts(fonts)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

/// Initialize the counter app and store it for use by present/listen.
#[wasm_bindgen]
pub fn init_counter() -> i32 {
    COUNTER_APP.with(|state| {
        let app = demo::counter_app();
        let LoadedFonts {
            fonts,
            ui_font,
            mono_font,
        } = shell::load_system_fonts().expect("fonts should load in wasm");
        let canvas = Canvas::new(demo::REFERENCE_WIDTH, demo::REFERENCE_HEIGHT, 1.0);
        *state.borrow_mut() = Some(CounterState {
            app,
            fonts,
            canvas,
            memory: Memory::new(),
            input: Input::new(),
            events: Vec::new(),
            ui_font,
            mono_font,
            last_frame_time: crate::shell::clock::Moment::now(),
        });
    });
    0
}

/// Render the counter app to pixels and present them to the browser canvas.
///
/// Called by JavaScript after events have been collected via `listen_counter`.
#[wasm_bindgen]
pub fn present_counter() {
    COUNTER_APP.with(|state| {
        if let Some(counter) = state.borrow_mut().as_mut() {
            let appearance = shell::get_appearance();
            let theme = Theme::new(appearance, counter.ui_font, counter.mono_font);

            counter
                .canvas
                .clear_vertical(theme.palette.background, theme.palette.background_deep);

            let now = crate::shell::clock::Moment::now();
            let elapsed = now.since(counter.last_frame_time);
            counter.last_frame_time = now;

            counter.memory.begin_frame(elapsed);
            counter.input.begin_frame();
            for event in counter.events.drain(..) {
                counter.input.apply(event);
            }

            counter.app.frame(
                &mut counter.canvas,
                &mut counter.fonts,
                &counter.input,
                &mut counter.memory,
                &theme,
            );

            counter.memory.end_frame(&counter.input);

            let _ = shell::present(&counter.canvas);
        }
    })
}

/// Collect events from the browser canvas and apply them.
///
/// Called by JavaScript before each `present_counter` to apply user input.
#[wasm_bindgen]
pub fn listen_counter() {
    if let Ok(events) = shell::listen() {
        COUNTER_APP.with(|state| {
            if let Some(counter) = state.borrow_mut().as_mut() {
                counter.events.extend(events);
            }
        })
    }
}

/// Get the current frame count of the counter app's memory.
///
/// Tests use this to verify that memory persists across frames.
/// The frame count increments each time the app draws, so if the same Memory
/// object is being reused, the counter will keep increasing. If Memory is
/// reallocated fresh each frame, the counter would reset to 1 each time.
#[wasm_bindgen]
pub fn counter_frame_count() -> u64 {
    COUNTER_APP.with(|state| {
        if let Some(counter) = state.borrow().as_ref() {
            counter.memory.frame_count()
        } else {
            0
        }
    })
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

/// Render a parity frame for WASM-based headless testing.
#[wasm_bindgen]
pub fn render_wasm_parity_frame(dark: bool) -> Result<Vec<u8>, JsValue> {
    demo::render_parity_frame_rgba(dark).map_err(|error| JsValue::from_str(&error))
}
