//! A simple todo list application demonstrating state management,
//! list rendering, and user interaction patterns in rui-native.
//!
//! Run with: `cargo run -p rui-native --example todo_app`

use rui_native::{
    col, draw, heading, panel, row, text, widgets, El, Key, Modifiers, Painter, Radius, Rect, Size,
    Tone,
};

#[derive(Clone)]
struct Todo {
    text: String,
    completed: bool,
}

struct State {
    todos: Vec<Todo>,
    input: String,
}

fn checkbox<S: 'static>(checked: bool, toggle: impl Fn(&mut S) + 'static) -> El<S> {
    draw(
        Size::new(15.0, 15.0),
        move |painter: &mut Painter<'_>, rect: Rect| {
            let fill = if checked { Tone::Accent } else { Tone::Sunken };
            painter.fill(rect, Radius::Units(4.0), fill);
            painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
            if checked {
                let inset = rect.w * 0.28;
                let mark = Rect::new(
                    rect.x + inset,
                    rect.y + inset,
                    rect.w - inset * 2.0,
                    rect.h - inset * 2.0,
                );
                painter.fill(mark, Radius::Units(1.0), Tone::OnAccent);
            }
        },
    )
    .h(15.0)
    .w(15.0)
    .on_click(move |state: &mut S| toggle(state))
}

fn view(state: &State) -> El<State> {
    col((
        heading("📝 Todo List"),
        row((
            widgets::field(&state.input).grow(),
            widgets::button("Add")
                .on_click(|state: &mut State| {
                    if !state.input.trim().is_empty() {
                        state.todos.push(Todo {
                            text: state.input.clone(),
                            completed: false,
                        });
                        state.input.clear();
                    }
                })
                .w(60.0),
        ))
        .gap(8.0),
        panel(
            col(state
                .todos
                .iter()
                .enumerate()
                .map(|(i, todo)| {
                    row((
                        checkbox(todo.completed, move |state: &mut State| {
                            if i < state.todos.len() {
                                state.todos[i].completed = !state.todos[i].completed;
                            }
                        }),
                        text(if todo.completed {
                            format!("✓ {}", todo.text)
                        } else {
                            todo.text.clone()
                        })
                        .grow(),
                        widgets::button("×")
                            .on_click(move |state: &mut State| {
                                if i < state.todos.len() {
                                    state.todos.remove(i);
                                }
                            })
                            .w(28.0),
                    ))
                    .gap(8.0)
                    .h(28.0)
                })
                .collect::<Vec<_>>())
            .gap(4.0)
            .pad(12.0),
        ),
    ))
    .gap(16.0)
    .pad(16.0)
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run(
        "Todo App",
        State {
            todos: vec![
                Todo {
                    text: "Learn rui-native".to_string(),
                    completed: true,
                },
                Todo {
                    text: "Build an app".to_string(),
                    completed: false,
                },
                Todo {
                    text: "Deploy to production".to_string(),
                    completed: false,
                },
            ],
            input: String::new(),
        },
        view,
    )
}
