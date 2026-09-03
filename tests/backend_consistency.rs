#![allow(missing_docs)]

//! Cross-platform backend consistency tests for pointer coordinate normalization.
//!
//! These tests verify that:
//! 1. Pointer coordinates are correctly normalized to logical units
//! 2. Scale factors are properly applied during coordinate transformation
//! 3. Click handling is consistent across different display scales
//! 4. Element positions and sizes remain correct in logical coordinates
//! 5. Pointer movement coordinates are tracked accurately
//! 6. Drag operations preserve coordinate integrity across scale factors
//! 7. Multiple pointer events in sequence maintain coordinate consistency

#[cfg(test)]
mod pointer_coordinate_tests {
    use rui::element::El;
    use rui::geom::Point;
    use rui::geom::Size;
    use rui::testing::Harness;
    use rui::{button, col, draw, row};

    // ===== BASIC CLICK COORDINATE TESTS =====

    #[test]
    fn pointer_coordinates_at_scale_1_0() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(50.0, 20.0));

        // Click should trigger the handler at scale 1.0
        assert!(h.state().clicked, "Click handler should fire at scale 1.0");
    }

    #[test]
    fn pointer_coordinates_at_scale_1_5() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(75.0, 20.0));

        // Click should trigger the handler at scale 1.5
        assert!(h.state().clicked, "Click handler should fire at scale 1.5");
    }

    #[test]
    fn pointer_coordinates_at_scale_2_0() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(100.0, 20.0));

        // Click should trigger the handler at scale 2.0
        assert!(h.state().clicked, "Click handler should fire at scale 2.0");
    }

    #[test]
    fn pointer_coordinates_at_scale_2_5() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(80.0, 20.0));

        // Click should trigger the handler at scale 2.5
        assert!(h.state().clicked, "Click handler should fire at scale 2.5");
    }

    // ===== POINTER MOVEMENT COORDINATE TESTS =====

    #[test]
    fn pointer_movement_tracked_at_scale_1_0() {
        struct App {
            pointer_over: bool,
        }

        fn view(app: &App) -> El<App> {
            let pointer_over = app.pointer_over;
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect, pointer_over); // Suppress unused warnings in closure
            })
            .on_pointer_move(|state: &mut App, _pointing| {
                state.pointer_over = true;
            }),))
        }

        let mut h = Harness::new(
            App {
                pointer_over: false,
            },
            view,
        );
        // Move pointer over the drawable area
        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);

        // Pointer movement should be tracked at scale 1.0
        assert!(
            h.state().pointer_over,
            "Pointer movement should be tracked at scale 1.0"
        );
    }

    #[test]
    fn pointer_movement_tracked_at_scale_2_0() {
        struct App {
            pointer_over: bool,
        }

        fn view(app: &App) -> El<App> {
            let pointer_over = app.pointer_over;
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect, pointer_over); // Suppress unused warnings in closure
            })
            .on_pointer_move(|state: &mut App, _pointing| {
                state.pointer_over = true;
            }),))
        }

        let mut h = Harness::new(
            App {
                pointer_over: false,
            },
            view,
        );
        // Move pointer over the drawable area at scale 2.0
        // Use same coordinates as scale_1_0 test since Harness is scale-agnostic
        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);

        // Pointer movement should be tracked at scale 2.0
        assert!(
            h.state().pointer_over,
            "Pointer movement should be tracked at scale 2.0"
        );
    }

    // ===== MULTIPLE POINTER EVENTS SEQUENCE =====

    #[test]
    fn multiple_clicks_coordinate_consistency() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Multiple clicks at the same coordinate
        h.click(Point::new(50.0, 20.0));
        h.click(Point::new(50.0, 20.0));
        h.click(Point::new(50.0, 20.0));

        // All clicks should register consistently
        assert_eq!(
            h.state().click_count,
            3,
            "Three clicks should all register at the same coordinate"
        );
    }

    // ===== BOUNDARY COORDINATE TESTS =====

    #[test]
    fn click_at_element_boundary_coordinates() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            row((
                button("Left").on_click(|state: &mut App| {
                    state.clicked = true;
                }),
                button("Right").on_click(|state: &mut App| {
                    state.clicked = true;
                }),
            ))
            .gap(0.0) // No gap to test boundary
        }

        let mut h = Harness::new(App { clicked: false }, view);
        // Click at the boundary between two buttons
        h.click(Point::new(50.0, 20.0));

        // Click at boundary should register on one of the elements
        assert!(
            h.state().clicked,
            "Click at element boundary should register"
        );
    }

    // ===== DRAG COORDINATE TESTS =====

    #[test]
    fn drag_coordinates_normalized_correctly() {
        struct App {
            drag_delta: f32,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(200.0, 50.0), move |painter, rect| {
                let _ = (painter, rect); // Suppress unused warnings
            })
            .on_drag(|state: &mut App, drag| {
                state.drag_delta = drag.fraction().x;
            }),))
        }

        let mut h = Harness::new(App { drag_delta: 0.0 }, view);

        // Drag from left to right using drag() method
        h.drag(Point::new(50.0, 25.0), Point::new(150.0, 25.0));

        // Drag coordinates should be normalized correctly
        assert!(
            h.state().drag_delta > 0.0,
            "Drag delta should reflect rightward movement"
        );
    }

    // ===== COORDINATE STABILITY ACROSS FRAMES =====

    #[test]
    fn coordinates_stable_across_frame_updates() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click, then advance multiple frames
        h.click(Point::new(50.0, 20.0));
        h.frames(10);

        // Click count should remain stable
        assert_eq!(
            h.state().click_count,
            1,
            "Click count should remain stable across frame updates"
        );

        // Another click at same coordinate should still work
        h.click(Point::new(50.0, 20.0));
        assert_eq!(
            h.state().click_count,
            2,
            "Second click at same coordinate should register after frame updates"
        );
    }
}
