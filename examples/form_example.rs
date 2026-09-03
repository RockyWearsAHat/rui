//! A comprehensive form demonstrating all form control widgets: text input, select, checkbox.
//! Shows how state flows through form controls and handlers update application state.

use rui::prelude::*;

#[derive(Clone, Default)]
struct App {
    name: String,
    email: String,
    country: usize, // Index into countries list
    subscribe: bool,
    terms_accepted: bool,
}

fn main() {
    rui::run(App::default(), view)
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
            widgets::text_input(&app.name, "Enter your full name", |app: &mut App, value| {
                app.name = value
            }),
        ))
        .gap(4.0),
        col((
            text("Email Address:").text_size(12.0),
            widgets::text_input(&app.email, "Enter your email", |app: &mut App, value| {
                app.email = value
            }),
        ))
        .gap(4.0),
        col((
            text("Country:").text_size(12.0),
            widgets::select(countries, app.country, |app: &mut App, index| {
                app.country = index;
            }),
        ))
        .gap(4.0),
        col((
            widgets::checkbox(
                "Subscribe to newsletters",
                app.subscribe,
                |app: &mut App| {
                    app.subscribe = !app.subscribe;
                },
            ),
            widgets::checkbox(
                "I accept the terms and conditions",
                app.terms_accepted,
                |app: &mut App| {
                    app.terms_accepted = !app.terms_accepted;
                },
            ),
        ))
        .gap(8.0),
        text(format_form_state(app))
            .text_size(11.0)
            .fill(Tone::Muted),
    ))
    .pad(16.0)
    .gap(12.0)
    .max_width(400.0)
}

fn format_form_state(app: &App) -> String {
    format!(
        "Name: {}\nEmail: {}\nCountry: {}\nSubscribed: {}\nTerms accepted: {}",
        if app.name.is_empty() {
            "(empty)"
        } else {
            &app.name
        },
        if app.email.is_