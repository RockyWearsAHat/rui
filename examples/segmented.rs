//! A segmented control: a series of mutually-exclusive choices in a row.
//!
//! `cargo run -p rui --example segmented` runs it.
//!
//! This is the minimal exemplar for an interactive widget: state in, description
//! out, one handler. The segmented control is a good starting point because it
//! demonstrates the full pattern in ~26 lines, and you can copy it as-is or
//! modify immediately.
//!
//! **State:** A usize index of the selected choice.
//! **View:** The segmented() widget receives the labels and selected index.
//! **Interaction:** Clicking a choice updates state via the handler callback.
//!
//! Everything above the state definition is the reusable widget;
//! everything below is specific to this application.

use rui::{col, row, text, widgets, El};

struct App {
    selected: usize,
}

fn main() -> Result<(), rui::Error> {
    rui::run("Segmented Control", App { selected: 1 }, view)
}

fn view(app: &App) -> El<App> {
    let choices = ["Small", "Medium", "Large"];

    col((
        text("Pick a size:"),
        widgets::segmented(&choices, app.selected, |app: &mut App, index| {
            app.selected = index;
        }),
        row((text("Your choice: "), text(choices[app.selected]))),
    ))
    .gap(16.0)
    .pad(16.0)
}
