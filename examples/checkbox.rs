//! A checkbox: binary toggle with a label (minimal exemplar ~26 lines).
//! Demonstrates the state-view-handler pattern for interactive controls.
//! Run: `cargo run -p rui --example checkbox`

use rui_native::{col, row, text, widgets};

struct App {
    notify: bool,
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("Checkbox", App { notify: true }, view)
}

fn view(app: &App) -> rui_native::El<App> {
    col((
        text("Your preferences:"),
        widgets::checkbox("Enable notifications", app.notify, |app: &mut App| {
            app.notify = !app.notify;
        }),
        row((
            text("Notifications: "),
            text(if app.notify { "ON" } else { "OFF" }),
        )),
    ))
    .gap(16.0)
    .pad(16.0)
}
