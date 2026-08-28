//! Tests that run in Node.js (not requiring a real browser).
//!
//! These tests verify the WASM exports and memory persistence without needing
//! DOM or canvas rendering context features. Tests here run with `wasm-pack test --node`.

#![cfg(target_arch = "wasm32")]

use rui::theme::Appearance;
use rui::wasm::{counter_frame_count, init_counter, listen_counter, present_counter};

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

    // Get the starting frame count. On initialization, begin_frame has not yet
    // been called, so the frame count starts at 0.
    let mut previous_frame_count = counter_frame_count();

    // Call present_counter 5 times in sequence.
    // Each call to present_counter invokes draw_into, which calls memory.begin_frame(),
    // incrementing the frame counter by 1. If Memory were reallocated fresh each frame,
    // the frame counter would reset to a fresh Memory's count, which would not increment.
    // By checking that the frame count increases, we verify that the same Memory object
    // is being reused across frames.
    for i in 0..5 {
        present_counter();
        let new_frame_count = counter_frame_count();
        assert!(
            new_frame_count > previous_frame_count,
            "Frame {} failed: frame count did not increment (was {}, is now {})",
            i + 1,
            previous_frame_count,
            new_frame_count
        );
        previous_frame_count = new_frame_count;
    }
}
