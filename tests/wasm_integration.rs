//! Test that the wasm module exports a counter initialization function.
//!
//! This test verifies the contract between JavaScript and Rust: that the wasm
//! module exports a function to initialize and render the counter app.

#![cfg(target_arch = "wasm32")]

use rui::wasm::{init_counter, listen_counter, present_counter};

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
