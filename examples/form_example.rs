//! A comprehensive form demonstrating all form control widgets: text input, select, checkbox.
//! Shows how state flows through form controls and handlers update application state.

use rui_native::{col, text, widgets, El};

#[derive(Clone, Default)]
struct App {
    name: String,
    email: String,
    country: usize,
    subscribe: bool,
    terms_accepted: bool,
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("Form Example", App::default(), view)
}

fn view(app: &App) -> El<App> {
    let countries = &[
        "United States",
        "Canada",
        "United Kingdom",
        "Australia",
        "Other",
    ];

    col((
        text("Registration Form").text_size(24.0),
        col((
            text("Full Name:").text_size(12.0),
            widgets::text_input(&app.name).key("name"),
        ))
        .gap(4.0),
        col((
            text("Email Address:").text_size(12.0),
            widgets::text_input(&app.email).key("email"),
        ))
        .gap(4.0),
        col((
            text("Country:").text_size(12.0),
            widgets::select(countries, app.country, |app: &mut App, index| {
                app.country = index;
            }),
        ))
        .gap(4.0),
        col((widgets::checkbox_group(
            &[
                "Subscribe to newsletters",
                "I accept the terms and conditions",
            ],
            &[app.subscribe, app.terms_accepted],
            |app: &mut App, index| match index {
                0 => app.subscribe = !app.subscribe,
                1 => app.terms_accepted = !app.terms_accepted,
                _ => {}
            },
        ),))
        .gap(8.0),
        text(format_form_state(app)).text_size(11.0),
    ))
    .pad(16.0)
    .gap(12.0)
}

fn format_form_state(app: &App) -> String {
    let country_name = [
        "United States",
        "Canada",
        "United Kingdom",
        "Australia",
        "Other",
    ][app.country];
    format!(
        "Name: {}\nEmail: {}\nCountry: {}\nSubscribed: {}\nTerms: {}",
        if app.name.is_empty() {
            "(empty)"
        } else {
            &app.name
        },
        if app.email.is_empty() {
            "(empty)"
        } else {
            &app.email
        },
        country_name,
        app.subscribe,
        app.terms_accepted,
    )
}
