//! A caller that owns its own surface and its own loop — not `run()`, and not
//! `Harness` — driving `App` directly through the public surface only.
//!
//! This is the shape a page with an async fetch needs: init once, keep the
//! `App` around across many frames, and fold newly-arrived data into it from
//! outside a click handler between one frame and the next. `App::run` cannot
//! do this (it owns state for the life of the loop) and neither could
//! `Harness` until now (`state()` is read-only) — `App::state_mut` and the
//! newly-public `App::frame` are what this test proves actually works.

use rui::{col, testing, text, App, Appearance, Canvas, El, Input, Memory, Theme};

#[derive(Default)]
struct Counter {
    count: i32,
}

fn view(counter: &Counter) -> El<Counter> {
    col(text(format!("count: {}", counter.count)))
}

#[test]
fn state_mut_between_frames_drives_the_next_frame() {
    let mut app = App::new("test", Counter::default(), view);

    let loaded = testing::test_fonts();
    let theme = Theme::new(Appearance::Light, loaded.ui_font, loaded.mono_font);
    let mut canvas = Canvas::new(800, 600, 1.0);
    let input = Input::new();
    let mut memory = Memory::new();

    app.frame(&mut canvas, &loaded.fonts, &input, &mut memory, &theme);
    assert_eq!(
        app.state().count,
        0,
        "no click happened, nothing changed it"
    );

    // The whole point: mutate state from outside a click handler — the way an
    // async fetch response arriving between frames would — with no window and
    // no `Harness`, then drive one more frame with the same `App`.
    app.state_mut().count = 5;
    assert_eq!(
        app.state().count,
        5,
        "visible immediately, before any frame runs"
    );

    app.frame(&mut canvas, &loaded.fonts, &input, &mut memory, &theme);
    assert_eq!(
        app.state().count,
        5,
        "survives a frame it did not come from"
    );
}
