//! Interactive form with text input and validation.
//!
//! Demonstrates:
//! - Text input fields with .on_input() handlers
//! - Form submission and validation
//! - Error message display
//!
//! Run with: `cargo run -p rui --example form_example`

use rui::{button, caption, col, field_row, heading, row, text, App, El, Tone};

#[derive(Clone)]
struct Form {
    email: String,
    password: String,
    name: String,
    has_error: bool,
}

impl Form {
    fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            name: String::new(),
            has_error: false,
        }
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && !self.email.is_empty() && self.password.len() >= 8
    }
}

fn view(form: &Form) -> El<Form> {
    col((
        heading("Sign Up Form"),
        col((
            field_row(
                "NAME",
                field(&form.name).on_input(|f: &mut Form, name| f.name = name),
            ),
            field_row(
                "EMAIL",
                field(&form.email).on_input(|f: &mut Form, email| f.email = email),
            ),
            field_row(
                "PASSWORD",
                field(&form.password).on_input(|f: &mut Form, password| f.password = password),
            ),
        ))
        .gap(16.0),
        if form.has_error {
            col((
                text("Please fill out all fields:"),
                if form.name.is_empty() {
                    caption("• Name is required")
                } else {
                    caption("")
                },
                if form.email.is_empty() {
                    caption("• Email is required")
                } else {
                    caption("")
                },
                if form.password.len() < 8 {
                    caption("• Password must be at least 8 characters")
                } else {
                    caption("")
                },
            ))
            .gap(4.0)
            .color(Tone::Bad)
        } else {
            col(text(""))
        },
        row((
            button("Submit").on_click(|f: &mut Form| {
                f.has_error = !f.is_valid();
            }),
            button("Clear").on_click(|f: &mut Form| {
                f.name.clear();
                f.email.clear();
                f.password.clear();
                f.has_error = false;
            }),
        ))
        .gap(12.0),
    ))
    .gap(20.0)
    .pad(32.0)
    .max_w(400.0)
}

fn field<S: 'static>(value: impl Into<String>) -> El<S> {
    rui::field(value)
}

fn main() -> Result<(), rui::Error> {
    let app = App::new("Form Example", Form::new(), view);
    app.run()
}
