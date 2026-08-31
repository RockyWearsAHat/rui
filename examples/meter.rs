//! A meter (progress bar) showing how to display values in real time.
//!
//! The meter widget is a read-only indicator—it displays a value (0.0–1.0)
//! as a filled bar. Unlike interactive controls (like segmented), meters
//! do not handle input; they simply visualize data.
//!
//! This example simulates uploading a file and updates the progress bar
//! in real time. The meter responds to the app state, not to user clicks.

use rui_native::{col, meter, text, Align, El};

#[derive(Default)]
struct App {
    progress: f32, // current progress: 0.0 = start, 1.0 = complete
}

fn view(app: &App) -> El<App> {
    col((
        text("Download progress:"),
        // The meter widget takes two arguments:
        // 1. progress: the fraction (0.0–1.0) displayed as a filled bar
        // 2. tone: the color role (here: Accent, the primary color)
        //
        // The meter is passive—it has no handler. It just visualizes state.
        // To change progress, you must update app.progress in the event loop.
        meter(app.progress, rui_native::Tone::Accent),
        text(format!("{:.0}%", app.progress * 100.0)),
    ))
    .gap(16.0)
    .pad(16.0)
    .align(Align::Center)
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("meter", App::default(), view)
}
