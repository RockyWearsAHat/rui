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
