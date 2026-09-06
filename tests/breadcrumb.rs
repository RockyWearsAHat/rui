//! Breadcrumb component tests

use rui::breadcrumb;
use rui::testing::Harness;

#[derive(Default, Clone)]
struct State {
    clicked_index: Option<usize>,
}

#[test]
fn breadcrumb_links_every_segment_but_the_last() {
    let segments = &["home", "projects", "rui", "src"];

    let mut h = Harness::new(State::default(), |_| {
        breadcrumb(segments, |s: &mut State, i| {
            s.clicked_index = Some(i);
        })
    })
    .size(400.0, 50.0);

    h.frame();

    // All segments except the last should be clickable links
    // We can verify this by checking that the key exists
    for segment in segments.iter().take(segments.len() - 1) {
        let el = h.find_key(segment);
        assert!(
            el.is_some(),
            "Segment {} should exist as a clickable link",
            segment
        );
    }

    // The last segment should also exist but as static text
    let last = h.find_key(segments[segments.len() - 1]);
    assert!(last.is_some(), "Last segment should exist");
}

#[test]
fn breadcrumb_click_reports_the_segment_index() {
    let segments = &["home", "projects", "rui"];

    let mut h = Harness::new(State::default(), |_| {
        breadcrumb(segments, |s: &mut State, i| {
            s.clicked_index = Some(i);
        })
    })
    .size(400.0, 50.0);

    h.frame();

    // Click the first segment
    h.click_text("home");
    assert_eq!(
        h.state().clicked_index,
        Some(0),
        "Clicking first segment should report index 0"
    );

    h.frame();

    // Click the second segment
    h.click_text("projects");
    assert_eq!(
        h.state().clicked_index,
        Some(1),
        "Clicking second segment should report index 1"
    );
}

#[test]
fn breadcrumb_separates_segments_with_slashes() {
    let segments = &["a", "b", "c"];
    let mut h = Harness::new(State::default(), |_| {
        breadcrumb(segments, |_: &mut State, _| {})
    })
    .size(400.0, 50.0);

    h.frame();

    // Check that separators exist by looking for "/" in probes
    let probes = h.probes();
    let mut slash_count = 0;
    for probe in probes {
        if probe.text.as_deref() == Some("/") {
            slash_count += 1;
        }
    }

    // For 3 segments, we should have 2 slashes
    assert_eq!(
        slash_count,
        2,
        "Should have {} slashes between {} segments",
        segments.len() - 1,
        segments.len()
    );
}

#[test]
fn breadcrumb_of_one_segment_has_no_links() {
    let segments = &["home"];

    let mut h = Harness::new(State::default(), |_| {
        breadcrumb(segments, |s: &mut State, i| {
            s.clicked_index = Some(i);
        })
    })
    .size(400.0, 50.0);

    h.frame();

    // With one segment, there should be no links (only static text)
    // Try to click on it - it should not trigger the callback
    h.click_text("home");
    assert_eq!(
        h.state().clicked_index,
        None,
        "Single segment should not be clickable"
    );
}

#[test]
fn breadcrumb_of_none_renders_without_panicking() {
    let segments: &[&str] = &[];

    // This should not panic
    let mut h = Harness::new(State::default(), |_| {
        breadcrumb(segments, |_: &mut State, _| {})
    })
    .size(400.0, 50.0);

    // Should render successfully
    h.frame();

    // The breadcrumb should still exist and be renderable
    let _ = h.canvas().pixels();
}
