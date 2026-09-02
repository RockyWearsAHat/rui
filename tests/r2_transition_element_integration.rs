#![allow(missing_docs)]

use rui::*;
use std::time::Duration;

#[test]
fn element_accepts_transition_builder() {
    let el: El<()> = col(()).transition(Transition::fade_in(0.3));
    assert!(el.has_transition().is_some());
}

#[test]
fn element_transition_stored_correctly() {
    let transition = Transition::slide_in(SlideDirection::Left, 0.5);
    let el: El<()> = button("Test").transition(transition.clone());
    assert!(el.has_transition().is_some());
}

#[test]
fn element_can_chain_transition() {
    let el: El<()> = col(())
        .gap(8.0)
        .pad(12.0)
        .transition(Transition::fade_in(0.2));
    assert!(el.has_transition().is_some());
}

#[test]
fn memory_starts_transition() {
    let mut mem = Memory::new();
    mem.begin_frame(Duration::from_millis(100));
    let id = Id::new("test");
    mem.start_transition(id, 0.5);
    assert!(mem.is_animating());
}

#[test]
fn memory_tracks_transition_progress() {
    let mut mem = Memory::new();
    mem.begin_frame(Duration::from_secs(0));
    let id = Id::new("test");
    mem.start_transition(id, 1.0);

    // Progress starts at 0.0
    assert_eq!(mem.transition_progress(id), Some(0.0));

    // After ~0.0333 seconds (1/30th, clamped to max 1/15), progress should be ~0.0333
    mem.begin_frame(Duration::from_millis(33));
    let progress = mem.transition_progress(id).unwrap();
    assert!((progress - 0.033).abs() < 0.005);

    // Accumulate more time steps until we reach ~1.0 seconds
    for _ in 0..30 {
        mem.begin_frame(Duration::from_millis(33));
        if mem.transition_progress(id).unwrap() >= 1.0 {
            break;
        }
    }
    assert_eq!(mem.transition_progress(id), Some(1.0));
}

#[test]
fn memory_clears_transition() {
    let mut mem = Memory::new();
    mem.begin_frame(Duration::from_secs(0));
    let id = Id::new("test");
    mem.start_transition(id, 0.5);
    assert!(mem.transition_progress(id).is_some());

    mem.clear_transition(id);
    assert!(mem.transition_progress(id).is_none());
}

#[test]
fn multiple_transitions_tracked_independently() {
    let mut mem = Memory::new();
    mem.begin_frame(Duration::from_secs(0));

    let id1 = Id::new("test1");
    let id2 = Id::new("test2");

    mem.start_transition(id1, 1.0);
    mem.start_transition(id2, 0.5);

    mem.begin_frame(Duration::from_millis(250));
    let p1 = mem.transition_progress(id1).unwrap();
    let p2 = mem.transition_progress(id2).unwrap();

    // id2 should be further along (0.5s duration vs 1.0s)
    assert!(p2 > p1);
}

#[test]
fn transition_marks_animating() {
    let mut mem = Memory::new();
    mem.begin_frame(Duration::from_secs(0));
    assert!(!mem.is_animating());

    let id = Id::new("test");
    mem.start_transition(id, 1.0);
    assert!(mem.is_animating());

    mem.clear_transition(id);
    assert!(!mem.is_animating());
}
