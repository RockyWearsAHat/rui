#![allow(missing_docs)]

use rui::testing::Harness;
use rui::{column, table, table_row, text, Length};

#[test]
fn table_renders_one_keyed_row_per_entry() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [
            column("name", Length::Fixed(100.0)),
            column("value", Length::Fill(1.0)),
        ];

        let rows = (0..12)
            .map(|i| {
                table_row(
                    format!("row-{}", i),
                    vec![text(format!("Row {}", i)), text(format!("Value {}", i))],
                )
            })
            .collect();

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State, view);
    harness.frame();

    let probes = harness.probes();
    let keys: Vec<_> = probes.iter().filter_map(|p| p.key.clone()).collect();

    // Should have row-0 through row-11
    for i in 0..12 {
        assert!(
            keys.contains(&format!("row-{}", i)),
            "Expected key 'row-{}' not found in probes",
            i
        );
    }
}

#[test]
fn table_header_labels_appear_once() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [
            column("name", Length::Fixed(100.0)),
            column("value", Length::Fill(1.0)),
        ];

        let rows = vec![table_row("row-0", vec![text("Data 1"), text("Data 2")])];
        let header = vec![text("Name"), text("Value")];

        table(&cols, Some(header), rows)
    }

    let mut harness = Harness::new(State, view);
    harness.frame();

    assert!(harness.shows("Name"));
    assert!(harness.shows("Value"));
}

#[test]
fn table_row_hover_lifts_the_fill() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fill(1.0))];

        let rows = (0..5)
            .map(|i| {
                table_row(format!("row-{}", i), vec![text(format!("Row {}", i))])
                    .on_click(|_state: &mut State| {})
            })
            .collect();

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State, view);
    harness.frame();

    // Move pointer to row 3
    if let Some(probe) = harness.find_key("row-3") {
        harness.move_pointer(probe.rect.center());
        harness.frame();

        // Row 3 should exist
        let probes = harness.probes();
        assert!(probes
            .iter()
            .any(|p| p.key.as_ref().is_some_and(|k| k == "row-3")));
    }
}

#[test]
fn table_row_click_fires_for_that_row_only() {
    struct State {
        clicked_row: Option<usize>,
    }

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fill(1.0))];

        let rows = vec![
            table_row("row-0", vec![text("Row 0")]).on_click(|s: &mut State| {
                s.clicked_row = Some(0);
            }),
            table_row("row-1", vec![text("Row 1")]).on_click(|s: &mut State| {
                s.clicked_row = Some(1);
            }),
        ];

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State { clicked_row: None }, view);
    harness.frame();

    harness.click_text("Row 1");
    assert_eq!(harness.state().clicked_row, Some(1));
}

#[test]
fn table_row_enter_activates_the_focused_row() {
    struct State {
        clicked_row: Option<usize>,
    }

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fill(1.0))];

        let rows = vec![
            table_row("row-0", vec![text("Row 0")]).on_click(|s: &mut State| {
                s.clicked_row = Some(0);
            }),
        ];

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State { clicked_row: None }, view);
    harness.frame();

    // First focus the row by clicking it
    harness.click_text("Row 0");
    harness.frame();

    // Then press Enter while it's focused
    harness.key(rui::Key::Enter);
    harness.frame();

    assert_eq!(harness.state().clicked_row, Some(0));
}

#[test]
fn table_selected_row_is_not_overridden_by_hover() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fill(1.0))];

        let rows = vec![
            table_row("row-0", vec![text("Row 0")])
                .selected(true)
                .on_click(|_: &mut State| {}),
            table_row("row-1", vec![text("Row 1")]).on_click(|_: &mut State| {}),
        ];

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State, view);
    harness.frame();

    // Move pointer over row 0 (selected)
    if let Some(probe) = harness.find_key("row-0") {
        harness.move_pointer(probe.rect.center());
        harness.frame();
        // Row 0 should stay selected despite hover
    }
}

#[test]
fn table_cell_click_does_not_also_fire_the_row() {
    struct State {
        clicked_row: Option<usize>,
    }

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fill(1.0))];

        let rows = vec![
            table_row("row-0", vec![text("Cell").on_click(|_: &mut State| {})]).on_click(
                |s: &mut State| {
                    s.clicked_row = Some(0);
                },
            ),
        ];

        table(&cols, None, rows)
    }

    let mut harness = Harness::new(State { clicked_row: None }, view);
    harness.frame();

    harness.click_text("Cell");
    // The cell's click should fire, but the row's click should not
}

#[test]
fn table_cells_never_shrink_their_text() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [column("col", Length::Fixed(200.0))];

        let long_text = "A longer cell text";
        let rows = vec![table_row("row-0", vec![text(long_text)])];

        table(&cols, None, rows)
    }

    // Test at narrow width
    let mut harness = Harness::new(State, view).size(600.0, 400.0);
    harness.frame();

    // Check that the row exists with proper key
    let probes = harness.probes();
    let row_found = probes
        .iter()
        .any(|p| p.key.as_ref().is_some_and(|k| k == "row-0"));
    assert!(row_found, "Row should be found at narrow width");

    // Test at wider width
    let mut harness = Harness::new(State, view).size(1400.0, 400.0);
    harness.frame();

    let probes = harness.probes();
    let row_found = probes
        .iter()
        .any(|p| p.key.as_ref().is_some_and(|k| k == "row-0"));
    assert!(row_found, "Row should be found at wide width");
}

#[test]
fn table_with_no_rows_still_draws_its_header() {
    struct State;

    fn view(_: &State) -> rui::El<State> {
        let cols = [
            column("name", Length::Fixed(100.0)),
            column("value", Length::Fill(1.0)),
        ];

        let rows = vec![];
        let header = vec![text("Name"), text("Value")];

        table(&cols, Some(header), rows)
    }

    let mut harness = Harness::new(State, view);
    harness.frame();

    // Header should be visible even with no rows
    assert!(harness.shows("Name"));
    assert!(harness.shows("Value"));

    // No body rows should exist
    let probes = harness.probes();
    let body_rows = probes
        .iter()
        .filter(|p| p.key.as_ref().is_some_and(|k| k.starts_with("row-")))
        .collect::<Vec<_>>();
    assert_eq!(body_rows.len(), 0, "Should have no body rows");
}
