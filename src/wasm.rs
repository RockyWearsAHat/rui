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
use crate::FrameDriver;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static COUNTER_DRIVER: RefCell<Option<FrameDriver<Counter>>> = const { RefCell::new(None) };
}

/// Initialize the counter app and store it for use by present/listen.
#[wasm_bindgen]
pub fn init_counter() -> i32 {
    COUNTER_DRIVER.with(|driver| {
        let app = demo::counter_app();
        let fonts = shell::load_system_fonts().expect("fonts should load in wasm");
        let frame_driver = FrameDriver::from_parts(
            app,
            fonts,
            demo::REFERENCE_WIDTH,
            demo::REFERENCE_HEIGHT,
            1.0,
        );
        *driver.borrow_mut() = Some(frame_driver);
    });
    0
}

/// Render the counter app to pixels and present them to the browser canvas.
///
/// Called by JavaScript after events have been collected via `listen_counter`.
#[wasm_bindgen]
pub fn present_counter() {
    COUNTER_DRIVER.with(|driver| {
        let mut driver_borrow = driver.borrow_mut();
        if let Some(driver) = driver_borrow.as_mut() {
            let appearance = shell::get_appearance();
            driver.set_appearance(appearance);
            let _ = driver.step();
            if driver.pixels_changed() {
                let _ = shell::present(driver.canvas());
            }
        }
    })
}

/// Collect events from the browser canvas and apply them.
///
/// Called by JavaScript before each `present_counter` to apply user input.
#[wasm_bindgen]
pub fn listen_counter() {
    COUNTER_DRIVER.with(|driver| {
        if let Some(d) = driver.borrow_mut().as_mut() {
            if let Ok(events) = d.collect_events() {
                d.apply_events(events);
            }
        }
    })
}

/// Get the current frame count of the counter app's memory.
///
/// Tests use this to verify that memory persists across frames.
/// The frame count increments each time the app draws, so if the same Memory
/// object is being reused, the counter will keep increasing. If Memory is
/// reallocated fresh each frame, the counter would reset to 1 each time.
#[wasm_bindgen]
pub fn counter_frame_count() -> u64 {
    COUNTER_DRIVER.with(|driver| {
        let driver_borrow = driver.borrow();
        if let Some(d) = driver_borrow.as_ref() {
            d.frame_count()
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
