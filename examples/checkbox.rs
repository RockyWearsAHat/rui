//! A checkbox: a binary choice with a label.
//!
//! `cargo run -p rui --example checkbox` runs it.
//!
//! This is the minimal exemplar for a binary interactive widget. The pattern shown here
//! is the foundation for all toggle controls in rui:
//!
//! 1. **State** — Plain Rust struct (no `Rc`, no `RefCell`). Just data.
//! 2. **View** — Pure function of state: `fn view(app: &App) -> El<App>`. Rebuilds every frame.
//! 3. **Handler** — Closure receiving mutable state: `|app: &mut App| { app.notify = !app.notify }`.
//!
//! The checkbox demonstrates this pattern in ~26 lines, and you can copy it
//! as-is or modify immediately. Everything above the state definition is the reusable
//! widget; everything below is specific to this application.
//!
//! **To customize:**
//! - Change the label text in the view function.
//! - Modify the handler to update other fields or trigger actions.
//! - Copy this pattern to build multiple checkboxes, radio buttons, or toggles.
//! - See `src/widgets.rs` line 259–283 for the checkbox() implementation.
//! - See `tests/recipes.rs` for more examples: segmented, switch, slider, radio.

use rui_native::{col, row, text, widgets, El};

// === STATE ===
// This struct holds all the application's interactive state.
// In this example, we track a simple boolean preference.
// Add more fields here for more complex interactions.
struct App {
    notify: bool,
}

fn main() -> Result<(), rui_native::Error> {
    // Initialize the app with default state and pass the view function.
    // The event loop calls view() once per frame, after processing input.
    rui_native::run("Checkbox", App { notify: true }, view)
}

// === VIEW ===
// Pure function of state: given the current app state,
// return a description of what to draw.
// Called every frame; the description is rebuilt from scratch each time.
fn view(app: &App) -> El<App> {
    col((
        text("Your preferences:"),
        // The handler (the closure) is called when the checkbox is clicked.
        // It receives mutable state and toggles the value.
        // Change app.notify = !app.notify to update state. The view will rebuild next frame.
        widgets::checkbox("Enable notifications", app.notify, |app: &mut App| {
            app.notify = !app.notify;
        }),
        // Echo the selection back to the user.
        row((
            text("Notifications: "),
            text(if app.notify { "ON" } else { "OFF" }),
        )),
    ))
    .gap(16.0)
    .pad(16.0)
}
