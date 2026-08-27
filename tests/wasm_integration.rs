//! Test that the wasm module exports a counter initialization function.
//!
//! This test verifies the contract between JavaScript and Rust: that the wasm
//! module exports a function to initialize and render the counter app.

#![cfg(target_arch = "wasm32")]

use rui::wasm::init_counter;

#[wasm_bindgen_test::wasm_bindgen_test]
fn wasm_exports_counter_init() {
    // The counter app must be exported so JavaScript can call it.
    // This test verifies that the wasm module has the required export by
    // calling the function directly. If init_counter is not exported or callable,
    // this test will not compile.
    let result = init_counter();
    assert_eq!(result, 0, "init_counter should return 0 on success");
}
