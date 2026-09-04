//! A simple calculator application demonstrating numeric input handling,
//! stateful computation, and button grid layouts.
//!
//! Run with: `cargo run -p rui-native --example calculator`

use rui_native::{col, heading, panel, row, text, widgets, El, Tone};

struct State {
    display: String,
    accumulator: f64,
    operation: Option<char>,
    new_number: bool,
}

fn view(state: &State) -> El<State> {
    let display_text = if state.display.is_empty() {
        "0".to_string()
    } else {
        state.display.clone()
    };

    col((
        heading("🔢 Calculator"),
        panel(
            col((text(display_text).h(40.0),))
                .pad(16.0)
                .fill(Tone::Sunken),
        ),
        col((
            row((
                button_char(state, "7"),
                button_char(state, "8"),
                button_char(state, "9"),
                button_op(state, "/"),
            ))
            .gap(4.0),
            row((
                button_char(state, "4"),
                button_char(state, "5"),
                button_char(state, "6"),
                button_op(state, "*"),
            ))
            .gap(4.0),
            row((
                button_char(state, "1"),
                button_char(state, "2"),
                button_char(state, "3"),
                button_op(state, "-"),
            ))
            .gap(4.0),
            row((
                button_char(state, "0").grow(),
                button_char(state, "."),
                button_op(state, "+"),
            ))
            .gap(4.0),
            row((
                widgets::button("Clear")
                    .on_click(|state: &mut State| {
                        state.display.clear();
                        state.accumulator = 0.0;
                        state.operation = None;
                        state.new_number = true;
                    })
                    .grow(),
                widgets::button("=")
                    .on_click(|state: &mut State| {
                        if let Ok(current) = state.display.parse::<f64>() {
                            let result = match state.operation {
                                Some('+') => state.accumulator + current,
                                Some('-') => state.accumulator - current,
                                Some('*') => state.accumulator * current,
                                Some('/') => state.accumulator / current,
                                _ => current,
                            };
                            state.display = result.to_string();
                            state.accumulator = result;
                            state.operation = None;
                            state.new_number = true;
                        }
                    })
                    .grow(),
            ))
            .gap(4.0),
        ))
        .gap(4.0),
    ))
    .gap(16.0)
    .pad(16.0)
}

fn button_char(_state: &State, ch: &str) -> El<State> {
    let ch_clone = ch.to_string();
    widgets::button(ch)
        .on_click(move |state: &mut State| {
            if state.new_number {
                state.display.clear();
                state.new_number = false;
            }
            if ch_clone != "." || !state.display.contains('.') {
                state.display.push_str(&ch_clone);
            }
        })
        .grow()
}

fn button_op(_state: &State, op: &str) -> El<State> {
    let op_char = op.chars().next().unwrap();
    widgets::button(op)
        .on_click(move |state: &mut State| {
            if let Ok(current) = state.display.parse::<f64>() {
                if let Some(prev_op) = state.operation {
                    let result = match prev_op {
                        '+' => state.accumulator + current,
                        '-' => state.accumulator - current,
                        '*' => state.accumulator * current,
                        '/' => state.accumulator / current,
                        _ => current,
                    };
                    state.display = result.to_string();
                    state.accumulator = result;
                } else {
                    state.accumulator = current;
                }
            }
            state.operation = Some(op_char);
            state.new_number = true;
        })
        .grow()
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run(
        "Calculator",
        State {
            display: String::new(),
            accumulator: 0.0,
            operation: None,
            new_number: true,
        },
        view,
    )
}
