//! WebAssembly bindings for rui components.
//!
//! This module exposes rui's component API and examples via `wasm-bindgen`,
//! allowing JavaScript to instantiate and interact with rui components in a browser.

#![cfg(target_arch = "wasm32")]

use crate::El;
use wasm_bindgen::prelude::*;

/// A counter application state.
#[wasm_bindgen]
pub struct Counter {
    count: i32,
}

#[wasm_bindgen]
impl Counter {
    /// Create a new counter starting at 0.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { count: 0 }
    }

    /// Increment the counter by 1.
    pub fn increment(&mut self) {
        self.count += 1;
    }

    /// Decrement the counter by 1.
    pub fn decrement(&mut self) {
        self.count -= 1;
    }

    /// Reset the counter to 0.
    pub fn reset(&mut self) {
        self.count = 0;
    }

    /// Get the current count value.
    pub fn get_count(&self) -> i32 {
        self.count
    }

    /// Set the count to a specific value.
    pub fn set_count(&mut self, value: i32) {
        self.count = value;
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the counter view to a canvas element by ID.
///
/// # Arguments
/// * `canvas_id` - The HTML id of the canvas element to render to
/// * `count` - The current counter value
#[wasm_bindgen]
pub fn render_counter(canvas_id: &str, count: i32) {
    // Stub implementation: rendering to canvas requires a full wasm backend
    // (see src/shell/platform/wasm.rs worklist item).
    // This function can be called from JavaScript but currently is a no-op.
    let _ = (canvas_id, count);
}

/// Create a counter view element for use in rui applications.
///
/// This mirrors the counter example from `examples/counter.rs`.
pub fn counter_view(counter: &Counter) -> El<Counter> {
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
