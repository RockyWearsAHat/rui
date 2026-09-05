//! Widget-specific visual regression tests for the gallery.
//!
//! These tests verify that individual gallery widgets render correctly
//! and consistently across frames and appearance variants.

use rui::testing::Harness;
use rui::{
    button, caption, col, dot, field, figure, heading, meter, micro, row, segmented, spacer, tabs,
    tag, text, title, Appearance, Status, Theme, Tone,
};

#[derive(Default, Clone)]
struct WidgetState {
    tab: usize,
    mode: usize,
    name: String,
}

// ---- Title and Text Widgets ----

#[test]
fn title_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| title("rui").bold()).size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "title should render identically across frames"
    );
}

#[test]
fn heading_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| heading("RUNNING")).size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "heading should render identically across frames"
    );
}

#[test]
fn caption_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| {
        caption("a declarative interface library")
    })
    .size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "caption should render identically across frames"
    );
}

#[test]
fn micro_text_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| micro("127.0.0.1:9191")).size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "micro text should render identically across frames"
    );
}

// ---- Button Widgets ----

#[test]
fn button_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| {
        button("Click me").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "button should render identically when not interacted with"
    );
}

#[test]
fn button_disabled_renders_differently() {
    let mut h_enabled = Harness::new(WidgetState::default(), |_| {
        button("Click me").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 50.0);

    let mut h_disabled = Harness::new(WidgetState::default(), |_| {
        button("Click me")
            .on_click(|_: &mut WidgetState| {})
            .disabled(true)
    })
    .size(200.0, 50.0);

    h_enabled.frame();
    h_disabled.frame();

    assert_ne!(
        h_enabled.canvas().pixels(),
        h_disabled.canvas().pixels(),
        "disabled button should render differently than enabled"
    );
}

// ---- Tabs Widget ----

#[test]
fn tabs_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |state| {
        tabs(
            &["Overview", "Definition", "Output"],
            state.tab,
            |s: &mut WidgetState, t| {
                s.tab = t;
            },
        )
    })
    .size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "tabs should render identically when not interacted with"
    );
}

#[test]
fn tabs_different_selections_render_differently() {
    let mut h_tab0 = Harness::new(
        WidgetState {
            tab: 0,
            ..Default::default()
        },
        |state| {
            tabs(
                &["Overview", "Definition", "Output"],
                state.tab,
                |s: &mut WidgetState, t| {
                    s.tab = t;
                },
            )
        },
    )
    .size(400.0, 50.0);

    let mut h_tab1 = Harness::new(
        WidgetState {
            tab: 1,
            ..Default::default()
        },
        |state| {
            tabs(
                &["Overview", "Definition", "Output"],
                state.tab,
                |s: &mut WidgetState, t| {
                    s.tab = t;
                },
            )
        },
    )
    .size(400.0, 50.0);

    h_tab0.frame();
    h_tab1.frame();

    assert_ne!(
        h_tab0.canvas().pixels(),
        h_tab1.canvas().pixels(),
        "tabs with different selections should render differently"
    );
}

// ---- Segmented Control ----

#[test]
fn segmented_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |state| {
        segmented(
            &["Manual", "At boot", "On demand"],
            state.mode,
            |s: &mut WidgetState, m| {
                s.mode = m;
            },
        )
    })
    .size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "segmented should render identically when not interacted with"
    );
}

#[test]
fn segmented_different_selections_render_differently() {
    let mut h_mode0 = Harness::new(
        WidgetState {
            mode: 0,
            ..Default::default()
        },
        |state| {
            segmented(
                &["Manual", "At boot", "On demand"],
                state.mode,
                |s: &mut WidgetState, m| {
                    s.mode = m;
                },
            )
        },
    )
    .size(400.0, 50.0);

    let mut h_mode1 = Harness::new(
        WidgetState {
            mode: 1,
            ..Default::default()
        },
        |state| {
            segmented(
                &["Manual", "At boot", "On demand"],
                state.mode,
                |s: &mut WidgetState, m| {
                    s.mode = m;
                },
            )
        },
    )
    .size(400.0, 50.0);

    h_mode0.frame();
    h_mode1.frame();

    assert_ne!(
        h_mode0.canvas().pixels(),
        h_mode1.canvas().pixels(),
        "segmented with different selections should render differently"
    );
}

// ---- Field (Input) Widget ----

#[test]
fn field_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |state| field(&state.name)).size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "field should render identically across frames"
    );
}

#[test]
fn field_different_values_render_differently() {
    let mut h_empty = Harness::new(
        WidgetState {
            name: String::new(),
            ..Default::default()
        },
        |state| field(&state.name),
    )
    .size(400.0, 50.0);

    let mut h_filled = Harness::new(
        WidgetState {
            name: "mongod".into(),
            ..Default::default()
        },
        |state| field(&state.name),
    )
    .size(400.0, 50.0);

    h_empty.frame();
    h_filled.frame();

    assert_ne!(
        h_empty.canvas().pixels(),
        h_filled.canvas().pixels(),
        "field with different values should render differently"
    );
}

// ---- Meter (Progress Bar) ----

#[test]
fn meter_widget_renders_consistently() {
    let mut h =
        Harness::new(WidgetState::default(), |_| meter(0.62, Tone::Accent)).size(300.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "meter should render identically across frames"
    );
}

#[test]
fn meter_different_values_render_differently() {
    let mut h_low =
        Harness::new(WidgetState::default(), |_| meter(0.25, Tone::Accent)).size(300.0, 50.0);

    let mut h_high =
        Harness::new(WidgetState::default(), |_| meter(0.75, Tone::Accent)).size(300.0, 50.0);

    h_low.frame();
    h_high.frame();

    assert_ne!(
        h_low.canvas().pixels(),
        h_high.canvas().pixels(),
        "meter with different values should render differently"
    );
}

// ---- Status Indicator (Dot) ----

#[test]
fn dot_widget_ok_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| dot(Status::Ok, 4.0)).size(100.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "dot should render identically across frames"
    );
}

#[test]
fn dot_different_statuses_render_differently() {
    let mut h_ok = Harness::new(WidgetState::default(), |_| dot(Status::Ok, 4.0)).size(100.0, 50.0);

    let mut h_bad =
        Harness::new(WidgetState::default(), |_| dot(Status::Bad, 4.0)).size(100.0, 50.0);

    h_ok.frame();
    h_bad.frame();

    assert_ne!(
        h_ok.canvas().pixels(),
        h_bad.canvas().pixels(),
        "dots with different statuses should render differently"
    );
}

// ---- Tag Widget ----

#[test]
fn tag_widget_renders_consistently() {
    let mut h =
        Harness::new(WidgetState::default(), |_| tag(Status::Ok, "running")).size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "tag should render identically across frames"
    );
}

// ---- Row/Col Layout ----

#[test]
fn row_layout_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| {
        row((caption("Label 1"), caption("Label 2"), caption("Label 3"))).gap(8.0)
    })
    .size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "row layout should render identically across frames"
    );
}

#[test]
fn col_layout_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| {
        col((heading("Title"), caption("Description"), text("Detail"))).gap(8.0)
    })
    .size(300.0, 150.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "col layout should render identically across frames"
    );
}

// ---- Appearance Variants ----

#[test]
fn light_and_dark_appearances_render_differently() {
    let mut h_light = Harness::new(WidgetState::default(), |_| {
        button("Click me").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 50.0)
    .appearance(Appearance::Light);

    let mut h_dark = Harness::new(WidgetState::default(), |_| {
        button("Click me").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 50.0)
    .appearance(Appearance::Dark);

    h_light.frame();
    h_dark.frame();

    assert_ne!(
        h_light.canvas().pixels(),
        h_dark.canvas().pixels(),
        "light and dark appearances should render differently"
    );
}

#[test]
fn theme_corner_style_affects_rendering() {
    let round_theme = |appearance: rui::Appearance, ui: rui::FontId, mono: rui::FontId| {
        Theme::new(appearance, ui, mono)
    };

    let cut_theme = |appearance: rui::Appearance, ui: rui::FontId, mono: rui::FontId| {
        Theme::new(appearance, ui, mono).with_corners(rui::CornerStyle::Cut)
    };

    let mut h_round = Harness::new(WidgetState::default(), |_| {
        // Use button which has visible corners and rounded by default
        button("Click").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 200.0)
    .theme(round_theme);

    let mut h_cut = Harness::new(WidgetState::default(), |_| {
        // Same button with cut corners from theme
        button("Click").on_click(|_: &mut WidgetState| {})
    })
    .size(200.0, 200.0)
    .theme(cut_theme);

    h_round.frame();
    h_cut.frame();

    assert_ne!(
        h_round.canvas().pixels(),
        h_cut.canvas().pixels(),
        "round and cut corner styles should render differently"
    );
}

// ---- Spacer Widget ----

#[test]
fn spacer_grow_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| {
        row((caption("Start"), spacer().grow(), caption("End"))).gap(8.0)
    })
    .size(400.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "spacer with grow should render identically across frames"
    );
}

// ---- Figure Widget ----

#[test]
fn figure_widget_renders_consistently() {
    let mut h = Harness::new(WidgetState::default(), |_| figure("123").bold()).size(200.0, 50.0);
    h.frame();
    let first = h.canvas().pixels().to_vec();

    h.frame();
    assert_eq!(
        first,
        h.canvas().pixels().to_vec(),
        "figure should render identically across frames"
    );
}
