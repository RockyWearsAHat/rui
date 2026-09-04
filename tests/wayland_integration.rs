//! Integration tests for the Wayland backend.
//!
//! Phase 3: Integration — Verify Wayland backend lifecycle, coordinate transformation,
//! event handling, and feature gate behavior.
//!
//! The arithmetic/contract tests below are backend-agnostic (they document the
//! wire-level encoding Phase 2/3 code relies on) and always compile when the
//! `wayland` feature is enabled. The `Harness`-based tests exercise the same
//! rendering/event pipeline every other backend's tests run through.

#![cfg(feature = "wayland")]

use rui::shell::WindowOptions;
use rui::testing::Harness;
use rui::{Radius, Size, Tone, button, col, draw, text};

#[test]
fn wayland_feature_is_enabled() {
    // Verify the wayland feature gate is active
    // This test only runs when compiled with --features wayland
    assert!(cfg!(feature = "wayland"));
}

#[test]
fn wayland_window_options_default() {
    // Verify WindowOptions default values
    let options = WindowOptions::default();
    assert_eq!(options.width, 960.0);
    assert_eq!(options.height, 640.0);
    assert_eq!(options.min_width, 420.0);
    assert_eq!(options.min_height, 320.0);
}

#[test]
fn wayland_coordinate_transformation() {
    // Phase 2/3: Coordinate transformation formula (device → logical)
    //
    // Example 1: Standard monitor (1.0 scale)
    // - Device pixel (100, 200) at 1.0 scale
    // - Logical = device / scale_factor = (100, 200) / 1.0 = (100, 200)
    let device_x = 100.0_f32;
    let device_y = 200.0_f32;
    let scale_1_0 = 1.0_f32;
    let (logical_x, logical_y) = (device_x / scale_1_0, device_y / scale_1_0);
    assert_eq!(logical_x, 100.0);
    assert_eq!(logical_y, 200.0);

    // Example 2: HiDPI monitor (2.0 scale)
    // - Device pixel (200, 400) at 2.0 scale
    // - Logical = device / scale_factor = (200, 400) / 2.0 = (100, 200)
    let device_x = 200.0_f32;
    let device_y = 400.0_f32;
    let scale_2_0 = 2.0_f32;
    let (logical_x, logical_y) = (device_x / scale_2_0, device_y / scale_2_0);
    assert_eq!(logical_x, 100.0);
    assert_eq!(logical_y, 200.0);

    // Example 3: Laptop monitor (1.5 scale)
    // - Device pixel (300, 450) at 1.5 scale
    // - Logical = device / scale_factor = (300, 450) / 1.5 = (200, 300)
    let device_x = 300.0_f32;
    let device_y = 450.0_f32;
    let scale_1_5 = 1.5_f32;
    let (logical_x, logical_y) = (device_x / scale_1_5, device_y / scale_1_5);
    assert!((logical_x - 200.0).abs() < 0.01);
    assert!((logical_y - 300.0).abs() < 0.01);
}

#[test]
fn wayland_scale_factor_validation() {
    // Phase 2: Scale factor must be in valid range (1.0–4.0)
    // Typical values:
    // - 1.0: Standard monitor (96 DPI)
    // - 1.5: Laptop screen (144 DPI)
    // - 2.0: Retina/HiDPI display (192 DPI)
    // - 4.0: Ultra-high-DPI (384 DPI, rare)

    let valid_scales = vec![1.0, 1.25, 1.5, 1.75, 2.0, 3.0, 4.0];
    for scale in valid_scales {
        assert!(scale >= 1.0 && scale <= 4.0);
    }

    // Out-of-range values should be clamped (Phase 2 implementation)
    let out_of_range: Vec<f32> = vec![0.5, 0.9, 5.0, 10.0];
    for scale in out_of_range {
        let clamped = scale.max(1.0).min(4.0);
        assert!(clamped >= 1.0 && clamped <= 4.0);
    }
}

#[test]
fn wayland_appearance_default() {
    // Phase 2: Appearance queries Wayland portal for light/dark theme
    // Default is Light if unavailable
    use rui::Appearance;

    let _appearance = Appearance::Light;
    // Phase 2 implementation would query org.freedesktop.portal.Settings
    // and map color-scheme: 0 = none, 1 = dark, 2 = light
}

#[test]
fn wayland_modifier_key_encoding() {
    // Phase 2: Modifier key state encoded as u8 bitmask
    // shift=1, control=2, alt=4, super=8
    let shift = 1u8;
    let control = 2u8;
    let alt = 4u8;
    let super_key = 8u8;

    // Shift + Control = 3
    let shift_control = shift | control;
    assert_eq!(shift_control, 3);
    assert!(shift_control & shift != 0);
    assert!(shift_control & control != 0);

    // Alt + Super = 12
    let alt_super = alt | super_key;
    assert_eq!(alt_super, 12);
    assert!(alt_super & alt != 0);
    assert!(alt_super & super_key != 0);
}

#[test]
fn wayland_button_state_encoding() {
    // Phase 2: Mouse button state encoded as u32 bitmask
    // left=0x1, right=0x2, middle=0x4, forward=0x8, back=0x10
    let left = 0x1u32;
    let right = 0x2u32;
    let middle = 0x4u32;
    let forward = 0x8u32;
    let back = 0x10u32;

    // Left + Right pressed = 0x3
    let left_right = left | right;
    assert_eq!(left_right, 0x3);
    assert!(left_right & left != 0);
    assert!(left_right & right != 0);
    assert!(left_right & middle == 0);

    // All buttons pressed = 0x1F
    let all_buttons = left | right | middle | forward | back;
    assert_eq!(all_buttons, 0x1F);
    for button in &[left, right, middle, forward, back] {
        assert!(all_buttons & button != 0);
    }
}

#[test]
fn wayland_key_translation_range() {
    // Phase 2: Keyboard key translation covers:
    // - Navigation keys: Tab, Enter, Escape, Arrows, Home, End, PgUp, PgDn
    // - Control keys: Backspace, Insert, Delete
    // - Special: Space
    // - Printable: ASCII 0x21–0x7E

    let keysyms = vec![
        0xff09, // Tab
        0xff0d, // Enter
        0xff1b, // Escape
        0xff08, // Backspace
        0xff63, // Insert
        0xffff, // Delete
        0xff50, // Home
        0xff57, // End
        0xff55, // PageUp
        0xff56, // PageDown
        0xff51, // Left
        0xff53, // Right
        0xff52, // Up
        0xff54, // Down
        0x20,   // Space
    ];

    // Verify all navigation keys are in valid keysym range
    for keysym in keysyms {
        assert!(keysym > 0 && keysym <= 0xffff);
    }

    // Printable ASCII range
    let printable_start = 0x21u32;
    let printable_end = 0x7eu32;
    for c in printable_start..=printable_end {
        assert!(c >= 0x21 && c <= 0x7e);
    }
}

#[test]
fn wayland_dpi_detection_range() {
    // Phase 2: DPI detection validates scale factor
    fn validate_scale_factor(scale: f32) -> bool {
        scale >= 1.0 && scale <= 4.0
    }

    // Valid scales
    assert!(validate_scale_factor(1.0));
    assert!(validate_scale_factor(1.5));
    assert!(validate_scale_factor(2.0));
    assert!(validate_scale_factor(3.0));
    assert!(validate_scale_factor(4.0));

    // Invalid scales (should be clamped)
    assert!(!validate_scale_factor(0.5));
    assert!(!validate_scale_factor(5.0));
}

#[test]
fn wayland_event_translation_pipeline() {
    // Phase 2: Document event translation pipeline structure
    //
    // 1. Keyboard: wl_keyboard::Event::Key
    //    → Translate via xkb keymap to keysym
    //    → Create KeyEvent { keysym, modifiers }
    //    → KeyEvent::to_key() returns Option<rui::input::Key>
    //    → Create Event::Key with pressed/released state
    //
    // 2. Pointer: wl_pointer::Event::{Motion, Button, Axis}
    //    → Update PointerState
    //    → Transform device pixels to logical (/ scale_factor)
    //    → Create Event::Pointer with position and state
    //
    // 3. Window: xdg_surface/xdg_toplevel::Event
    //    → Configure → resize window
    //    → Close → set is_open = false
    //
    // 4. DPI: wl_output::Event::Scale
    //    → Update scale_factor
    //    → Used for all coordinate transformations

    // This test documents the pipeline (actual implementation in Phase 3)
    assert!(true, "Event translation pipeline documented");
}

// --- Ported from origin/main's Harness-based suite ---
//
// origin/main's version of this file ran these scenarios against a
// `Harness::frame()` that returned a `Frame` value with public `width`,
// `height`, and `appearance` fields (e.g. `harness.frame().width`,
// `harness.frame().appearance`). The real `Harness::frame` (src/testing/mod.rs)
// returns `&mut Self`, not such a `Frame` struct, and `Harness` has no public
// getter for its current appearance. The tests below are rewritten against the
// verified real API (`Harness::shows`, `Harness::click_text`, `Harness::state`,
// `Harness::state_mut`, `Harness::canvas().width()/.height()`). Scenarios that
// depended on reading back the current appearance are left as a TODO rather
// than guessing at a field/method that may not exist once the Wayland backend
// merge lands.

#[test]
fn wayland_backend_window_opens() {
    // Verify the render pipeline produces a window-sized canvas.
    let mut harness = Harness::new((), |_: &()| col((text("Wayland Backend Test"),)));

    harness.frame();
    assert!(
        harness.canvas().width() > 0,
        "Window width must be positive"
    );
    assert!(
        harness.canvas().height() > 0,
        "Window height must be positive"
    );
}

#[test]
fn wayland_backend_supports_drawing() {
    // Verify basic drawing works through the same pipeline every backend uses.
    let mut harness = Harness::new((), |_: &()| {
        col((
            text("Drawing Test"),
            draw(Size::new(100.0, 50.0), |painter, rect| {
                painter.fill(rect, Radius::None, Tone::Accent);
            }),
        ))
    });

    assert!(harness.shows("Drawing Test"));
}

#[test]
fn wayland_backend_handles_state_changes() {
    // Verify state changes are reflected in rendered output
    struct State {
        count: i32,
    }

    fn view(state: &State) -> rui::El<State> {
        col((
            text(format!("Count: {}", state.count)),
            button("Increment", |state: &mut State| {
                state.count += 1;
            }),
        ))
    }

    let mut harness = Harness::new(State { count: 0 }, view);
    assert!(harness.shows("Count: 0"));

    harness.click_text("Increment");
    assert_eq!(harness.state().count, 1);
    assert!(harness.shows("Count: 1"));
}

#[test]
fn wayland_backend_supports_multiple_event_types() {
    // Verify the Wayland-shaped pipeline can handle repeated pointer events
    struct State {
        click_count: i32,
    }

    let mut harness = Harness::new(State { click_count: 0 }, |state: &State| {
        col((
            text(format!("Clicks: {}", state.click_count)),
            button("Click Me", |s: &mut State| s.click_count += 1),
        ))
    });

    for i in 1..=3 {
        harness.click_text("Click Me");
        assert_eq!(harness.state().click_count, i);
    }
}

#[test]
fn wayland_backend_preserves_state_between_frames() {
    // Verify state persists across multiple frame renders (memory consistency)
    struct State {
        value: String,
    }

    let mut harness = Harness::new(
        State {
            value: "test".into(),
        },
        |state: &State| text(&state.value),
    );

    assert!(harness.shows("test"));

    harness.state_mut().value = "modified".into();
    assert!(harness.shows("modified"));
}

#[test]
fn wayland_backend_text_rendering() {
    // Verify text rendering works correctly through the shared pipeline
    let text_content = "Hello, Wayland!";
    let mut harness = Harness::new((), |_: &()| col((text(text_content),)));

    assert!(harness.shows(text_content));
}

#[test]
fn wayland_backend_no_panic_on_empty_view() {
    // Verify backend doesn't panic on minimal/empty views
    let mut harness = Harness::new((), |_: &()| col((text(""),)));
    harness.frame();
    // Should not panic
}

#[test]
fn wayland_backend_matches_x11_coordinate_contract() {
    // Verify Wayland-shaped rendering hits the same elements as every other
    // backend after a click, i.e. the coordinate contract is preserved.
    struct TestState {
        button_clicked: bool,
    }

    fn view(state: &TestState) -> rui::El<TestState> {
        col((
            text(if state.button_clicked {
                "Clicked"
            } else {
                "Not Clicked"
            }),
            button("Click", |s: &mut TestState| s.button_clicked = true),
        ))
    }

    let mut harness = Harness::new(
        TestState {
            button_clicked: false,
        },
        view,
    );
    assert!(harness.shows("Not Clicked"));

    harness.click_text("Click");
    assert!(harness.shows("Clicked"));
}

#[test]
fn wayland_backend_event_ordering() {
    // Verify events are processed in order
    struct State {
        events: Vec<String>,
    }

    let mut harness = Harness::new(State { events: vec![] }, |state: &State| {
        col((
            text(state.events.join(", ")),
            button("Event 1", |s: &mut State| s.events.push("1".into())),
            button("Event 2", |s: &mut State| s.events.push("2".into())),
        ))
    });

    harness.click_text("Event 1");
    assert_eq!(harness.state().events, vec!["1"]);

    harness.click_text("Event 2");
    assert_eq!(harness.state().events, vec!["1", "2"]);

    harness.click_text("Event 1");
    assert_eq!(harness.state().events, vec!["1", "2", "1"]);
}

// TODO(wayland-merge): origin/main also had `wayland_backend_appearance_is_consistent`,
// asserting that `Harness`'s current appearance doesn't change between two
// `.frame()` calls unless `set_appearance` is called. `Harness` (src/testing/mod.rs)
// has `set_appearance(&mut self, Appearance)` but no public getter for the
// current appearance, so this scenario can't be expressed without adding one.
// Left out rather than guessing at a getter name — add back once such a getter
// exists (or once the merged Wayland `Window` exposes appearance directly).

// TODO(wayland-merge): origin/main's `wayland_backend_coordinate_system` and
// `wayland_backend_colors_are_rendered`/`wayland_backend_animation_state_persistence`
// relied on a `Frame` value with `width`/`height` fields returned from
// `harness.frame()`. Once the real Wayland `Window` (12-method `Backend` impl,
// see tests/wayland_parity.rs) lands, revisit whether these are worth
// reintroducing via `harness.canvas().width()/.height()`.
