//! A segmented control: a series of mutually-exclusive choices in a row.
//!
//! `cargo run -p rui --example segmented` runs it.
//!
//! This is the minimal exemplar for an interactive widget. The pattern shown here
//! is the foundation for all custom controls in rui:
//!
//! 1. **State** — Plain Rust struct (no `Rc`, no `RefCell`). Just data.
//! 2. **View** — Pure function of state: `fn view(app: &App) -> El<App>`. Rebuilds every frame.
//! 3. **Handler** — Closure receiving mutable state: `|app: &mut App, index| { app.selected = index }`.
//!
//! The segmented control demonstrates this pattern in ~26 lines, and you can copy it
//! as-is or modify immediately. Everything above the state definition is the reusable
//! widget; everything below is specific to this application.
//!
//! **To customize:**
//! - Change `["Small", "Medium", "Large"]` to your own labels.
//! - Modify the handler to update other fields or trigger actions.
//! - Copy this pattern to build checkboxes, radio buttons, tabs, or sliders.
//! - See `src/widgets.rs` line 333–365 for the segmented() implementation.
//! - See `tests/recipes.rs` for more examples: checkbox, switch, slider, radio.

use rui::{El, col, row, text, widgets};

// === STATE ===
// This struct holds all the application's interactive state.
// In this example, we only track which choice is selected (by index).
// Add more fields here for more complex interactions.
struct App {
    selected: usize,
}

fn main() -> Result<(), rui::Error> {
    // Initialize the app with default state and pass the view function.
    // The event loop calls view() once per frame, after processing input.
    rui::run("Segmented Control", App { selected: 1 }, view)
}

// === VIEW ===
// Pure function of state: given the current app state,
// return a description of what to draw.
// Called every frame; the description is rebuilt from scratch each time.
fn view(app: &App) -> El<App> {
    let choices = ["Small", "Medium", "Large"];

    col((
        text("Pick a size:"),
        // The handler (the closure) is called when a button is clicked.
        // It receives mutable state and the clicked index.
        // Change app.selected = index to update state. The view will rebuild next frame.
        widgets::segmented(&choices, app.selected, |app: &mut App, index| {
            app.selected = index;
        }),
        // Echo the selection back to the user.
        row((text("Your choice: "), text(choices[app.selected]))),
    ))
    .gap(16.0)
    .pad(16.0)
}
