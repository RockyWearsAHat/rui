use rui::{
    button, caption, col, heading, row, section, segmented, spacer, tag, App, El, Status, Tone,
};
use wasm_bindgen::prelude::*;

/// Demo state.
pub struct Showcase {
    count: i32,
    selected: usize,
}

pub fn demo() -> Showcase {
    Showcase {
        count: 0,
        selected: 0,
    }
}

pub fn view(s: &Showcase) -> El<Showcase> {
    col((
        row((
            heading("RUI Showcase"),
            spacer().grow(),
            tag(Status::Ok, "Live"),
        ))
        .gap(16.0)
        .pad(16.0),
        col((
            section("Buttons", None),
            row((
                button("Click me")
                    .primary()
                    .on_click(|s: &mut Showcase| s.count += 1),
                button("Reset")
                    .on_click(|s: &mut Showcase| s.count = 0),
                spacer().grow(),
                caption(format!("Count: {}", s.count)),
            ))
            .gap(8.0),
            section("Segmented Control", None),
            segmented(
                &["Option A", "Option B", "Option C"],
                s.selected,
                |s: &mut Showcase, idx| s.selected = idx,
            ),
            section("Status", None),
            row((
                row((rui::dot(Status::Ok, 5.0), caption("Online"))).gap(4.0),
                row((rui::dot(Status::Warn, 5.0), caption("Degraded"))).gap(4.0),
                row((rui::dot(Status::Bad, 5.0), caption("Error"))).gap(4.0),
            ))
            .gap(12.0),
        ))
        .gap(8.0)
        .pad(16.0),
    ))
    .gap(0.0)
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let state = demo();
    let app = App::new("RUI", state, view);
    let fonts = rui::shell::load_system_fonts()
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    app.run_with_fonts(fonts)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
