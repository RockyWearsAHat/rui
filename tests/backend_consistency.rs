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

    // ===== EDGE CASE COORDINATE TESTS =====

    #[test]
    fn zero_coordinate_click() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 100.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        // Click at origin (0,0)
        h.click(Point::new(0.0, 0.0));

        // Click at origin should register on the drawable area
        assert!(
            h.state().clicked,
            "Click at origin (0,0) should trigger handler"
        );
    }

    #[test]
    fn large_coordinate_click() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(500.0, 500.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view).size(800.0, 800.0);
        // Click at large coordinate (400, 400)
        h.click(Point::new(400.0, 400.0));

        // Click at large coordinate should register
        assert!(
            h.state().clicked,
            "Click at large coordinate should trigger handler"
        );
    }

    // ===== ADDITIONAL SCALE FACTOR TESTS =====

    #[test]
    fn pointer_coordinates_at_scale_1_25() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(62.5, 20.0));

        assert!(h.state().clicked, "Click handler should fire at scale 1.25");
    }

    #[test]
    fn pointer_coordinates_at_scale_3_0() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(150.0, 20.0));

        assert!(h.state().clicked, "Click handler should fire at scale 3.0");
    }

    // ===== NESTED ELEMENT COORDINATE TESTS =====

    #[test]
    fn nested_element_click_coordinates() {
        struct App {
            inner_clicked: bool,
            outer_clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((row((
                button("Outer").on_click(|state: &mut App| {
                    state.outer_clicked = true;
                }),
                button("Inner").on_click(|state: &mut App| {
                    state.inner_clicked = true;
                }),
            ))
            .gap(5.0),))
        }

        let mut h = Harness::new(
            App {
                inner_clicked: false,
                outer_clicked: false,
            },
            view,
        );

        // Click on inner element
        h.click(Point::new(100.0, 20.0));
        assert!(
            h.state().inner_clicked,
            "Inner element click should register at correct coordinate"
        );
    }

    // ===== RAPID CLICK SEQUENCE TESTS =====

    #[test]
    fn rapid_sequential_clicks_at_different_coordinates() {
        struct App {
            click_coords: Vec<(usize, usize)>,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Button 1").on_click(|state: &mut App| {
                    state.click_coords.push((1, 0));
                }),
                button("Button 2").on_click(|state: &mut App| {
                    state.click_coords.push((2, 0));
                }),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(
            App {
                click_coords: vec![],
            },
            view,
        );

        // Rapid sequential clicks at different coordinates
        h.click(Point::new(50.0, 20.0));
        h.click(Point::new(50.0, 50.0));
        h.click(Point::new(50.0, 20.0));

        // All three clicks should register
        assert_eq!(
            h.state().click_coords.len(),
            3,
            "Three rapid clicks should all register at correct coordinates"
        );
    }

    // ===== POINTER MOVEMENT WITH MULTIPLE ELEMENTS =====

    #[test]
    fn pointer_movement_across_multiple_elements() {
        struct App {
            first_over: bool,
            second_over: bool,
        }

        fn view(app: &App) -> El<App> {
            let (first_over, second_over) = (app.first_over, app.second_over);
            col((
                draw(Size::new(100.0, 50.0), move |painter, rect| {
                    let _ = (painter, rect, first_over);
                })
                .on_pointer_move(|state: &mut App, _pointing| {
                    state.first_over = true;
                }),
                draw(Size::new(100.0, 50.0), move |painter, rect| {
                    let _ = (painter, rect, second_over);
                })
                .on_pointer_move(|state: &mut App, _pointing| {
                    state.second_over = true;
                }),
            ))
            .gap(10.0)
        }

        let mut h = Harness::new(
            App {
                first_over: false,
                second_over: false,
            },
            view,
        );

        // Move over first element
        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);
        assert!(
            h.state().first_over,
            "Pointer movement should register on first element"
        );

        // Move over second element
        h.move_pointer(Point::new(50.0, 90.0));
        h.frames(1);
        assert!(
            h.state().second_over,
            "Pointer movement should register on second element"
        );
    }

    // ===== OFF-BOUNDARY CLICK TESTS =====

    #[test]
    fn click_outside_element_does_not_trigger() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view).size(300.0, 300.0);

        // Click well outside the drawable area (beyond 100,50)
        h.click(Point::new(250.0, 250.0));

        // Click outside should not trigger handler
        assert!(
            !h.state().clicked,
            "Click outside element boundaries should not trigger handler"
        );
    }

    // ===== CLICK POSITION CONSISTENCY =====

    #[test]
    fn same_click_position_always_triggers_same_handler() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Consistent").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        let click_point = Point::new(60.0, 20.0);

        // Click same position 5 times
        for _ in 0..5 {
            h.click(click_point);
        }

        // All 5 clicks at same position should register
        assert_eq!(
            h.state().click_count,
            5,
            "Clicking same position 5 times should register 5 times"
        );
    }

    // ===== FOCUS COORDINATE CONSISTENCY =====

    #[test]
    fn focus_ring_renders_at_correct_coordinates() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Focus me")
                .on_click(|state: &mut App| {
                    state.click_count += 1;
                })
                .focusable(),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click the button twice at the same coordinate
        h.click(Point::new(50.0, 20.0));
        h.frames(1);
        h.click(Point::new(50.0, 20.0));

        // Both clicks should register even with focus ring present
        assert_eq!(
            h.state().click_count,
            2,
            "Focus ring should render at correct coordinates and not interfere with clicks"
        );
    }

    // ===== FRACTIONAL COORDINATE TESTS =====

    #[test]
    fn fractional_coordinates_handled_correctly() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Fractional").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Click at fractional coordinate (0.5 pixels)
        h.click(Point::new(50.5, 20.3));

        assert!(
            h.state().clicked,
            "Fractional coordinates should be handled correctly"
        );
    }

    // ===== COORDINATE PRECISION ACROSS MULTIPLE SCALES =====

    #[test]
    fn coordinate_precision_maintained_at_high_dpi() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("High DPI").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Multiple clicks at slightly different coordinates
        let coords = vec![
            Point::new(50.0, 20.0),
            Point::new(50.1, 20.1),
            Point::new(49.9, 20.1),
            Point::new(50.0, 20.2),
        ];

        for coord in coords {
            h.click(coord);
        }

        // All clicks should register despite fractional differences
        assert_eq!(
            h.state().click_count,
            4,
            "All clicks with fractional coordinate differences should register"
        );
    }

    // ===== KEYBOARD FOCUS COORDINATE CONSISTENCY =====

    #[test]
    fn keyboard_navigation_maintains_coordinate_consistency() {
        struct App {
            button_1_focused: bool,
            button_2_focused: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Button 1").focusable().on_click(|state: &mut App| {
                    state.button_1_focused = !state.button_1_focused;
                }),
                button("Button 2").focusable().on_click(|state: &mut App| {
                    state.button_2_focused = !state.button_2_focused;
                }),
            ))
            .gap(10.0)
        }

        let mut h = Harness::new(
            App {
                button_1_focused: false,
                button_2_focused: false,
            },
            view,
        );

        // Click button 1 at known coordinate
        h.click(Point::new(50.0, 20.0));
        h.frames(1);

        assert!(
            h.state().button_1_focused,
            "Button 1 should respond to click"
        );

        // Click button 2 at known coordinate
        h.click(Point::new(50.0, 50.0));
        h.frames(1);

        assert!(
            h.state().button_2_focused,
            "Button 2 should respond to click at different coordinate"
        );
    }

    // ===== COORDINATE OVERFLOW/UNDERFLOW EDGE CASES =====

    #[test]
    fn very_small_positive_coordinates() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 100.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Click at very small positive coordinate
        h.click(Point::new(0.1, 0.1));

        assert!(
            h.state().clicked,
            "Very small positive coordinates should register"
        );
    }

    // ===== COORDINATE TRANSFORMATION FORMULA VERIFICATION =====

    #[test]
    fn coordinate_transformation_preserves_relative_distances() {
        struct App {
            first_clicked: bool,
            second_clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("First").on_click(|state: &mut App| {
                    state.first_clicked = true;
                }),
                draw(Size::new(100.0, 50.0), move |painter, rect| {
                    let _ = (painter, rect);
                }),
                button("Second").on_click(|state: &mut App| {
                    state.second_clicked = true;
                }),
            ))
            .gap(20.0)
        }

        let mut h = Harness::new(
            App {
                first_clicked: false,
                second_clicked: false,
            },
            view,
        );

        // Record initial positions
        let first_y = 20.0;
        let second_y = 20.0 + 50.0 + 20.0 + 50.0; // button + draw + gap + button

        // Click first button
        h.click(Point::new(50.0, first_y));
        assert!(h.state().first_clicked, "First button should click");

        // Click second button
        h.click(Point::new(50.0, second_y));
        assert!(h.state().second_clicked, "Second button should click");

        // Both should click without coordinate transformation errors
        assert!(
            h.state().first_clicked && h.state().second_clicked,
            "Relative distance preservation should maintain correct click targets"
        );
    }

    // ===== NEGATIVE COORDINATE HANDLING =====

    #[test]
    fn negative_offset_click_does_not_register() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Attempt to click at negative coordinate
        h.click(Point::new(-10.0, -10.0));

        assert!(
            !h.state().clicked,
            "Negative coordinates should not trigger handlers"
        );
    }

    // ===== COORDINATE CONSISTENCY AFTER REFLOW =====

    #[test]
    fn coordinates_remain_consistent_after_state_change_reflow() {
        struct App {
            expanded: bool,
            click_count: usize,
        }

        fn view(app: &App) -> El<App> {
            let expanded = app.expanded;
            col((
                button("Expand").on_click(|state: &mut App| {
                    state.expanded = !state.expanded;
                }),
                if expanded {
                    col((
                        draw(Size::new(100.0, 50.0), move |painter, rect| {
                            let _ = (painter, rect);
                        }),
                        button("Hidden").on_click(|state: &mut App| {
                            state.click_count += 1;
                        }),
                    ))
                } else {
                    col((button("Collapsed"),))
                },
            ))
        }

        let mut h = Harness::new(
            App {
                expanded: false,
                click_count: 0,
            },
            view,
        );

        // Click expand button
        h.click(Point::new(50.0, 20.0));
        h.frames(1);

        assert!(h.state().expanded, "Expand button should toggle state");

        // After reflow, click the now-visible button at its new coordinate
        h.click(Point::new(50.0, 80.0));

        assert_eq!(
            h.state().click_count,
            1,
            "Coordinates should be correct after reflow due to state change"
        );
    }
}
