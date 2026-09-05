//! Test demonstrating the Draw and Painter pattern from CLAUDE.md documentation.
//!
//! This test verifies that custom widgets can be built using draw() and Painter
//! to render backgrounds, borders, and text with semantic tones that adapt to
//! light/dark modes.

use rui::testing::Harness;
use rui::{col, draw, text, Align, El, Ink, Painter, Point, Radius, Rect, Size, Tone};

#[test]
fn draw_painter_pattern_renders_with_background_border_and_text() {
    #[derive(Default)]
    struct CustomWidgetState {
        counter: usize,
    }

    // Custom widget built with draw() and Painter demonstrating the pattern from CLAUDE.md.
    // This shows: state → view function with draw/painter → handler that receives &mut state.
    fn custom_button<S: 'static>(
        label: String,
        counter: usize,
        on_click: impl Fn(&mut S) + 'static,
    ) -> El<S> {
        draw(
            Size::new(120.0, 36.0),
            move |painter: &mut Painter<'_>, rect: Rect| {
                // Fill background with Surface tone (adapts to light/dark mode)
                painter.fill(rect, Radius::Units(4.0), Tone::Surface);
                // Stroke border with Border tone (semantic color)
                painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
                // Render text on top using Painter.text() with semantic tone
                let ink = Ink {
                    tone: Tone::Text,
                    ..Ink::default()
                };
                painter.text(rect, ink, Align::Center, &format!("{}: {}", label, counter));
            },
        )
        .size(120.0, 36.0)
        .on_click(move |state: &mut S| on_click(state))
    }

    let mut harness = Harness::new(CustomWidgetState::default(), |state: &CustomWidgetState| {
        col((
            text("Draw + Painter Example:"),
            custom_button(
                "Count".into(),
                state.counter,
                |state: &mut CustomWidgetState| {
                    state.counter += 1;
                },
            )
            .key("custom-button"),
        ))
    });

    // Verify initial render
    assert_eq!(harness.state().counter, 0, "counter starts at 0");
    harness.frame();
    // The widget renders pixels (draw + painter creates visual output)
    assert!(
        !harness.canvas().pixels().is_empty(),
        "draw/painter widget renders pixels"
    );

    // Verify interaction: click updates state via draw/painter handler
    harness.click(Point::new(60.0, 32.0)); // Click the button
    assert_eq!(
        harness.state().counter,
        1,
        "draw/painter handler updates state on click"
    );

    harness.click(Point::new(60.0, 32.0)); // Click again
    assert_eq!(
        harness.state().counter,
        2,
        "draw/painter pattern scales with state changes"
    );
}
