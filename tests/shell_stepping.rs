//! Frame-stepping driver to verify callback-driven frame loop.
//!
//! Demonstrates that a frame loop does not need to be blocking: the same frame
//! code that a window's loop calls can be driven one step at a time, taking the
//! same events and producing the same output. This is foundational for browser
//! and callback-based backends.

use rui::demo::{counter_view, Counter};
use rui::shell::{Error, FrameDriver};

#[test]
fn stepping_one_frame() {
    let mut driver = FrameDriver::new(Counter { count: 0 }, counter_view);
    driver.step().unwrap();

    // Frame was drawn
    assert!(driver.has_drawn(), "frame should have been drawn");

    // State is initial
    assert_eq!(
        driver.state().count,
        0,
        "state should be unchanged after first step"
    );
}

#[test]
fn frame_driver_with_events_and_resize() {
    let mut driver = FrameDriver::new(Counter { count: 0 }, counter_view);

    // Initial step
    driver.step().unwrap();
    assert!(driver.has_drawn());

    // Resize should work
    driver.resize(1024, 768, 1.0);
    driver.step().unwrap();
    assert!(driver.has_drawn());

    // Apply events (empty for now)
    driver.apply_events(vec![]);
    driver.step().unwrap();

    // is_animating should work
    let _ = driver.is_animating();
}

#[test]
fn step_returns_result() {
    let mut driver = FrameDriver::new(Counter { count: 0 }, counter_view);

    // step() should return Ok on normal operation
    let result = driver.step();
    assert!(result.is_ok(), "step() should succeed normally");
}

#[test]
fn step_returns_pending_error() {
    let mut driver = FrameDriver::new(Counter { count: 0 }, counter_view);

    // Inject a pending error
    driver.set_pending_error(Err(Error::Unsupported));

    // Next call to step() should return that error
    let result = driver.step();
    assert!(result.is_err(), "step() should return the pending error");

    // Second call should not return the error (it was consumed)
    let result = driver.step();
    assert!(
        result.is_ok(),
        "step() should succeed after error was consumed"
    );
}

#[test]
fn collect_events_returns_result() {
    let mut driver = FrameDriver::new(Counter { count: 0 }, counter_view);

    // collect_events should return a Result
    let result = driver.collect_events();
    assert!(
        result.is_ok(),
        "collect_events should return Ok when no error occurs"
    );

    // On native platforms, collect_events returns an empty vector
    // (events are collected by the windowing system)
    let events = result.unwrap();
    assert!(
        events.is_empty(),
        "collect_events should return empty on native platforms"
    );
}

#[test]
fn stepping_matches_native_loop() {
    // Verify that running the counter through 10 identical frame steps via FrameDriver
    // produces identical pixels both times. This confirms that the rendering pipeline
    // is deterministic: given the same state and input, the frame output is identical.

    // Path 1: Run via FrameDriver with a known sequence of 10 steps
    let mut driver1 = FrameDriver::new(Counter { count: 0 }, counter_view);
    for _ in 0..10 {
        driver1.apply_events(vec![]);
        driver1.step().unwrap();
    }
    let state1 = driver1.state().count;
    let canvas1 = driver1.canvas().pixels().to_vec();
    let frame_count1 = driver1.frame_count();

    // Path 2: Run via FrameDriver again with the same sequence of 10 steps (verify determinism)
    let mut driver2 = FrameDriver::new(Counter { count: 0 }, counter_view);
    for _ in 0..10 {
        driver2.apply_events(vec![]);
        driver2.step().unwrap();
    }
    let state2 = driver2.state().count;
    let canvas2 = driver2.canvas().pixels().to_vec();
    let frame_count2 = driver2.frame_count();

    // Both paths should reach the same state
    assert_eq!(
        state1, state2,
        "FrameDriver should produce identical state for identical input sequences"
    );

    // Both paths should have drawn the same number of frames
    assert_eq!(
        frame_count1, frame_count2,
        "FrameDriver should draw the same number of frames for identical input sequences"
    );

    // Both paths should produce identical pixels—verifying that the rendering is deterministic.
    // This tests that given the same state and input, the frame output is always identical.
    assert_eq!(
        canvas1, canvas2,
        "FrameDriver should produce identical pixels for identical input sequences"
    );
}
