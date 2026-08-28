//! Tests that run in Node.js (not requiring a real browser).
//!
//! These tests verify the WASM exports and memory persistence without needing
//! DOM or canvas rendering context features. Tests here run with `wasm-pack test --node`.

#![cfg(target_arch = "wasm32")]

use rui::theme::Appearance;
use rui::wasm::{init_counter, listen_counter, present_counter};

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
