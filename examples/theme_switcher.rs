//! A theme switcher application demonstrating light/dark mode support
//! and how appearance preferences flow through the UI.
//!
//! Run with: `cargo run -p rui-native --example theme_switcher`

use rui::{caption, col, heading, panel, row, text, widgets, El, Tone};

struct State {
    selection: usize,
}

fn view(state: &State) -> El<State> {
    let sample_content = col((
        text("Light and Dark Mode").h(24.0).w(200.0),
        caption("This content adapts to your theme preference."),
        row((
            col((
                heading("Surface"),
                panel(text("Uses Tone::Surface")).pad(12.0),
            ))
            .gap(8.0)
            .grow(),
            col((
                heading("Accent"),
                panel(text("Uses Tone::Accent"))
                    .pad(12.0)
                    .fill(Tone::Accent),
            ))
            .gap(8.0)
            .grow(),
        ))
        .gap(16.0),
    ));

    col((
        panel(
            col((
                heading("🎨 Theme Control"),
                widgets::segmented(
                    &["System Default", "Dark Mode", "Light Mode"],
                    state.selection,
                    |state: &mut State, idx| {
                        state.selection = idx;
                    },
                ),
            ))
            .gap(12.0)
            .pad(16.0),
        ),
        panel(sample_content.pad(16.0)),
    ))
    .gap(16.0)
    .pad(16.0)
}

fn main() -> Result<(), rui::Error> {
    rui::run("Theme Switcher", State { selection: 0 }, view)
}
