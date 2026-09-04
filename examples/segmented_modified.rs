//! Testing the "How to modify" guide from CLAUDE.md.
//! This example follows the documented modification: changing ["Small", "Medium", "Large"]
//! to different choices. This shows that the learning path is complete and working.

use rui::{El, col, text, widgets};

struct App {
    selected: usize,
}

fn main() -> Result<(), rui::Error> {
    rui::run("Segmented: Modified Example", App { selected: 0 }, view)
}

fn view(app: &App) -> El<App> {
    let choices = ["Beginner", "Intermediate", "Advanced"];

    col((
        text("Select difficulty level:"),
        widgets::segmented(&choices, app.selected, |app: &mut App, index| {
            app.selected = index;
        }),
        text(format!("You selected: {}", choices[app.selected])),
    ))
}
