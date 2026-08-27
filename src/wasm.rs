//! WASM entry point for the counter app.
//!
//! Exports functions that JavaScript can call to initialize and render the
//! counter interface. The rendering pipeline is identical to the native backend:
//! the view function is called, the result is laid out and drawn to a canvas.

use crate::{App, El};
use wasm_bindgen::prelude::*;

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

/// Initialize and set up the counter app for the wasm backend.
///
/// This is called from JavaScript to start the counter application in the
/// browser. Returns 0 on success.
#[wasm_bindgen]
pub fn init_counter() -> i32 {
    // Create the initial state
    let _app = App::new("Counter", Counter { count: 0 }, counter_view);

    // The wasm backend will take over from here, setting up requestAnimationFrame
    // and DOM event listeners. For now, we just return success.
    // The full wasm backend integration happens in step 2.
    0
}
