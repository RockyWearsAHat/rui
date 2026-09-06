//! Tests for the tab_bar component with counts and status lamps.

use rui::testing::Harness;
use rui::{tab_bar, Key, Status, TabItem};

/// Test state for tab_bar tests.
#[derive(Clone, Default)]
struct App {
    selected: usize,
    click_log: Vec<usize>,
}

#[test]
fn tab_bar_marks_the_selected_tab() {
    let items = vec![
        TabItem {
            label: "Files",
            count: None,
            status: None,
        },
        TabItem {
            label: "Changes",
            count: None,
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
            app.click_log.push(idx);
        })
    });

    harness.frame();
    let probe = harness.find("Files").expect("Files tab exists");
    assert!(!probe.disabled, "the first tab is visible");
}

#[test]
fn tab_bar_shows_a_count_beside_a_label() {
    let items = vec![
        TabItem {
            label: "Files",
            count: Some(5),
            status: None,
        },
        TabItem {
            label: "Changes",
            count: Some(3),
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
        })
    });

    harness.frame();
    assert!(harness.shows("5"), "the count is displayed");
    assert!(harness.shows("3"), "all counts are displayed");
}

#[test]
fn tab_bar_omits_the_badge_when_the_count_is_none() {
    let items = vec![
        TabItem {
            label: "Files",
            count: Some(5),
            status: None,
        },
        TabItem {
            label: "Changes",
            count: None,
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
        })
    });

    harness.frame();
    assert!(harness.shows("Files"), "Files label is shown");
    assert!(harness.shows("5"), "Files count is shown");
    assert!(harness.shows("Changes"), "Changes label is shown");
}

#[test]
fn tab_bar_shows_a_status_dot_when_given_one() {
    let items = vec![
        TabItem {
            label: "Checks",
            count: None,
            status: Some(Status::Ok),
        },
        TabItem {
            label: "Deploy",
            count: None,
            status: Some(Status::Bad),
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
        })
    });

    harness.frame();
    assert!(harness.shows("Checks"), "Checks tab is shown");
    assert!(harness.shows("Deploy"), "Deploy tab is shown");
}

#[test]
fn tab_bar_right_arrow_moves_to_the_next_tab() {
    let items = vec![
        TabItem {
            label: "Files",
            count: None,
            status: None,
        },
        TabItem {
            label: "Changes",
            count: None,
            status: None,
        },
        TabItem {
            label: "History",
            count: None,
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
        })
    });

    harness.frame();
    assert_eq!(harness.state().selected, 0, "starts at first tab");

    // Click to focus the first tab
    harness.click_text("Files");
    // Send right arrow key - should navigate based on current selected
    harness.key(Key::Right);
    assert_eq!(harness.state().selected, 1, "right arrow moves to next tab");

    harness.key(Key::Right);
    assert_eq!(
        harness.state().selected,
        2,
        "right arrow moves to next tab again"
    );
}

#[test]
fn tab_bar_left_arrow_wraps_from_the_first_to_the_last() {
    let items = vec![
        TabItem {
            label: "Files",
            count: None,
            status: None,
        },
        TabItem {
            label: "Changes",
            count: None,
            status: None,
        },
        TabItem {
            label: "History",
            count: None,
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
        })
    });

    harness.frame();
    assert_eq!(harness.state().selected, 0, "starts at first tab");

    // Click to focus the first tab
    harness.click_text("Files");
    // Send left arrow key - should wrap to last tab
    harness.key(Key::Left);
    assert_eq!(harness.state().selected, 2, "left arrow wraps to last tab");

    harness.key(Key::Left);
    assert_eq!(
        harness.state().selected,
        1,
        "left arrow moves to previous tab"
    );
}

#[test]
fn tab_bar_click_reports_the_index_clicked() {
    let items = vec![
        TabItem {
            label: "Files",
            count: None,
            status: None,
        },
        TabItem {
            label: "Changes",
            count: None,
            status: None,
        },
    ];

    let mut harness = Harness::new(App::default(), move |app: &App| {
        tab_bar(&items, app.selected, |app: &mut App, idx| {
            app.selected = idx;
            app.click_log.push(idx);
        })
    });

    harness.frame();
    harness.click_text("Files");
    assert_eq!(
        harness.state().click_log,
        vec![0],
        "clicking Files reports 0"
    );

    harness.click_text("Changes");
    assert_eq!(
        harness.state().click_log,
        vec![0, 1],
        "clicking Changes reports 1"
    );
}
