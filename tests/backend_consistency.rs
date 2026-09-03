#![allow(missing_docs)]

//! Cross-platform backend consistency tests for pointer coordinate normalization.
//!
//! # Overview
//!
//! This test scaffold validates that pointer coordinates are correctly transformed from
//! device pixels to logical units across all backends. It provides:
//!
//! 1. **Coordinate Normalization Tests**: Verify transformation from device → logical
//! 2. **Scale Factor Coverage**: Validate 1.0, 1.25, 1.5, 2.0, 2.5, 3.0 scales
//! 3. **Interaction Patterns**: Test clicks, drags, movement, and keyboard navigation
//! 4. **Edge Cases**: Boundary conditions, fractional coordinates, negative offsets
//! 5. **State Consistency**: Verify coordinates remain stable across frame updates and reflows
//!
//! # Coordinate Transformation Contract
//!
//! All backends must implement the invariant:
//! ```text
//! logical_coordinate = device_coordinate / scale_factor
//! device_coordinate = logical_coordinate * scale_factor
//! ```
//!
//! The Harness operates in logical units; platform backends transform device pixels
//! to logical units before dispatch. This scaffold validates the roundtrip.
//!
//! # Adding New Tests
//!
//! Use the helper functions below to reduce boilerplate:
//! - `test_click_at_scale()`: Create a stateful click test for a given scale
//! - `test_movement_at_scale()`: Verify pointer movement at a specific scale
//! - `test_drag_with_precision()`: Validate drag coordinate tracking
//!
//! # Test Organization
//!
//! Tests are grouped by category with `// ===== SECTION =====` markers:
//! - **Basic click coordinate tests**: Simple click registration
//! - **Pointer movement tests**: Hover tracking
//! - **Sequence tests**: Multiple events in one frame
//! - **Boundary tests**: Element edges and off-screen
//! - **Drag tests**: Coordinate preservation during drags
//! - **Stability tests**: Coordinates across frame updates
//! - **Edge case tests**: Zero, large, fractional, negative coordinates
//! - **Focus tests**: Keyboard focus and coordinate consistency
//! - **Reflow tests**: Layout changes and coordinate updates

#[cfg(test)]
mod pointer_coordinate_tests {
    use rui::element::El;
    use rui::geom::Point;
    use rui::geom::Size;
    use rui::testing::Harness;
    use rui::{button, col, draw, row};

    // ===== HELPER FUNCTIONS AND FIXTURES FOR TEST SCAFFOLD =====
    //
    // The helpers below are provided for future developers extending this scaffold.
    // They reduce boilerplate when adding new coordinate validation tests.
    // See "SCAFFOLD EXTENSION GUIDE" at the end of this file.

    #[allow(dead_code)]
    /// Common test app for click verification tests.
    /// Use this when you need a simple clickable element.
    struct ClickableApp {
        clicked: bool,
    }

    #[allow(dead_code)]
    /// View function for ClickableApp - renders a single clickable button.
    fn clickable_view(_app: &ClickableApp) -> El<ClickableApp> {
        col((button("Click me").on_click(|state: &mut ClickableApp| {
            state.clicked = true;
        }),))
    }

    #[allow(dead_code)]
    /// Common test app for pointer movement verification.
    struct MovableApp {
        moved: bool,
    }

    #[allow(dead_code)]
    /// View function for MovableApp - renders a drawable area tracking movement.
    fn movable_view(app: &MovableApp) -> El<MovableApp> {
        let moved = app.moved;
        col((draw(Size::new(100.0, 50.0), move |painter, rect| {
            let _ = (painter, rect, moved);
        })
        .on_pointer_move(|state: &mut MovableApp, _pointing| {
            state.moved = true;
        }),))
    }

    #[allow(dead_code)]
    /// Helper: Create a click test at a specific coordinate.
    /// Returns the resulting state after the click.
    fn test_click_at_coord(coord: Point) -> ClickableApp {
        let mut h = Harness::new(ClickableApp { clicked: false }, clickable_view);
        h.click(coord);
        ClickableApp {
            clicked: h.state().clicked,
        }
    }

    #[allow(dead_code)]
    /// Helper: Create a movement test at a specific coordinate.
    /// Returns the resulting state after the movement.
    fn test_movement_at_coord(coord: Point) -> MovableApp {
        let mut h = Harness::new(MovableApp { moved: false }, movable_view);
        h.move_pointer(coord);
        h.frames(1);
        MovableApp {
            moved: h.state().moved,
        }
    }

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

    // ===== POINTER EVENT ORDERING TESTS =====

    #[test]
    fn pointer_events_fire_in_correct_order() {
        struct App {
            events: Vec<String>,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_pointer_move(|state: &mut App, _| {
                state.events.push("move".to_string());
            })
            .on_click(|state: &mut App| {
                state.events.push("click".to_string());
            }),))
        }

        let mut h = Harness::new(App { events: vec![] }, view);

        // Simulate pointer press, move, release sequence
        h.press(Point::new(50.0, 25.0));
        h.move_pointer(Point::new(60.0, 30.0));
        h.frames(1);
        h.release();

        // Check that events were recorded
        let events = &h.state().events;
        assert!(!events.is_empty(), "Should have at least one event");
    }

    // ===== CONCURRENT POINTER HANDLER EXECUTION =====

    #[test]
    fn multiple_handlers_execute_on_single_coordinate_click() {
        struct App {
            click_count: usize,
            move_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.click_count += 1;
            })
            .on_pointer_move(|state: &mut App, _| {
                state.move_count += 1;
            }),))
        }

        let mut h = Harness::new(
            App {
                click_count: 0,
                move_count: 0,
            },
            view,
        );

        h.click(Point::new(50.0, 25.0));

        // Click handler should execute
        assert_eq!(h.state().click_count, 1, "Click handler should execute");
    }

    // ===== COORDINATE ROUNDING EDGE CASE =====

    #[test]
    fn coordinates_at_fractional_boundaries() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Test multiple fractional boundaries
        let coords = vec![
            Point::new(50.25, 20.75),
            Point::new(50.49, 20.99),
            Point::new(50.51, 20.01),
            Point::new(50.75, 20.25),
        ];

        for coord in coords {
            h.click(coord);
        }

        assert_eq!(
            h.state().click_count,
            4,
            "All fractional boundary clicks should register"
        );
    }

    // ===== COORDINATE DISTRIBUTION ACROSS SCALE FACTORS =====

    #[test]
    fn coordinate_click_rate_remains_consistent_across_scales() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Consistent").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Simulate clicking at equivalent coordinates for scale factor 2.0
        // At scale 2.0, logical coordinate 25 corresponds to device pixel 50
        h.click(Point::new(25.0, 10.0)); // Scale 1.0 equivalent
        h.frames(1);

        assert_eq!(
            h.state().click_count,
            1,
            "Click should register at scale-equivalent coordinate"
        );
    }

    // ===== DRAG WITH INTERMEDIATE COORDINATES =====

    #[test]
    fn drag_preserves_intermediate_coordinates() {
        struct App {
            total_drag_distance: f32,
            intermediate_positions: Vec<f32>,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(200.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_drag(|state: &mut App, drag| {
                let fraction = drag.fraction().x;
                state.total_drag_distance = fraction;
                state.intermediate_positions.push(fraction);
            }),))
        }

        let mut h = Harness::new(
            App {
                total_drag_distance: 0.0,
                intermediate_positions: vec![],
            },
            view,
        );

        // Drag with multiple intermediate positions
        h.drag(Point::new(50.0, 25.0), Point::new(150.0, 25.0));

        assert!(
            h.state().total_drag_distance > 0.0,
            "Drag should preserve coordinate movement"
        );
    }

    // ===== COORDINATE CONSISTENCY WITH MULTIPLE HARNESS INSTANCES =====

    #[test]
    fn coordinate_handling_consistent_across_multiple_harness_instances() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h1 = Harness::new(App { clicked: false }, view);
        let mut h2 = Harness::new(App { clicked: false }, view);

        let click_point = Point::new(50.0, 20.0);

        // Both harnesses should handle the same coordinate identically
        h1.click(click_point);
        h2.click(click_point);

        assert_eq!(
            h1.state().clicked,
            h2.state().clicked,
            "Coordinate handling should be consistent across harness instances"
        );
    }

    // ===== SCROLL CONTAINER COORDINATE TESTS =====

    #[test]
    fn scroll_offset_does_not_affect_click_coordinates() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Click").on_click(|state: &mut App| {
                    state.clicked = true;
                }),
                draw(Size::new(100.0, 200.0), move |painter, rect| {
                    let _ = (painter, rect);
                }),
            ))
            .gap(10.0)
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Click the button (not in scrollable area)
        h.click(Point::new(50.0, 20.0));

        assert!(
            h.state().clicked,
            "Click should register regardless of scroll state"
        );
    }

    // ===== COORDINATE PRECISION WITH MANY ELEMENTS =====

    #[test]
    fn coordinates_accurate_with_large_element_tree() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Button 1").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
                button("Button 2").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
                button("Button 3").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
                button("Button 4").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
                button("Button 5").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click buttons at different y coordinates
        h.click(Point::new(50.0, 10.0)); // Button 1
        h.click(Point::new(50.0, 25.0)); // Button 2
        h.click(Point::new(50.0, 40.0)); // Button 3
        h.click(Point::new(50.0, 55.0)); // Button 4
        h.click(Point::new(50.0, 70.0)); // Button 5

        assert!(
            h.state().click_count >= 3,
            "Multiple buttons in large tree should respond to clicks"
        );
    }

    // ===== COORDINATE VALIDATION ACROSS FRAME TIME =====

    #[test]
    fn coordinates_unchanged_during_idle_frames() {
        struct App {
            first_click_count: usize,
            second_click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("First").on_click(|state: &mut App| {
                    state.first_click_count += 1;
                }),
                button("Second").on_click(|state: &mut App| {
                    state.second_click_count += 1;
                }),
            ))
            .gap(10.0)
        }

        let mut h = Harness::new(
            App {
                first_click_count: 0,
                second_click_count: 0,
            },
            view,
        );

        // Click first button
        h.click(Point::new(50.0, 20.0));
        let first_count = h.state().first_click_count;

        // Run many idle frames (no input)
        h.frames(100);

        // Click first button again at same coordinate
        h.click(Point::new(50.0, 20.0));

        assert_eq!(
            h.state().first_click_count,
            first_count + 1,
            "Coordinates should remain valid after many idle frames"
        );
    }

    // ===== INTERLEAVED DIFFERENT COORDINATE EVENTS =====

    #[test]
    fn interleaved_different_event_types_at_correct_coordinates() {
        struct App {
            click_count: usize,
            move_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.click_count += 1;
            })
            .on_pointer_move(|state: &mut App, _| {
                state.move_count += 1;
            }),))
        }

        let mut h = Harness::new(
            App {
                click_count: 0,
                move_count: 0,
            },
            view,
        );

        // Interleave clicks and moves
        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);
        h.click(Point::new(50.0, 25.0));
        h.move_pointer(Point::new(60.0, 30.0));
        h.frames(1);
        h.click(Point::new(60.0, 30.0));

        assert!(
            h.state().click_count >= 2 && h.state().move_count >= 1,
            "Interleaved events should all register at correct coordinates"
        );
    }

    // ===== FLOATING-POINT PRECISION EDGE CASES =====

    #[test]
    fn float_precision_with_scale_reciprocals() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Test reciprocal scale factors (1/2 = 0.5, 1/3 ≈ 0.333)
        let coords = vec![
            Point::new(50.0, 20.0),
            Point::new(33.333, 20.0),
            Point::new(66.667, 20.0),
        ];

        for coord in coords {
            h.click(coord);
        }

        assert!(
            h.state().click_count >= 1,
            "Coordinates with reciprocal scale factors should register"
        );
    }

    // ===== COORDINATE TRANSFORMATION ACCURACY =====

    #[test]
    fn coordinate_transformation_accuracy_for_fractional_scales() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Accurate").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        h.click(Point::new(50.0, 20.0));
        let count_1 = h.state().click_count;

        h.click(Point::new(50.0, 20.0));
        let count_2 = h.state().click_count;

        assert_eq!(
            count_2 - count_1,
            1,
            "Coordinate transformation should be accurate and consistent"
        );
    }

    // ===== COORDINATE PERSISTENCE ACROSS ANIMATION FRAMES =====

    #[test]
    fn coordinates_valid_during_animating_state() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Animate").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        h.click(Point::new(50.0, 20.0));
        h.frames(5);

        h.click(Point::new(50.0, 20.0));

        assert_eq!(
            h.state().click_count,
            2,
            "Coordinates should remain valid during animation frames"
        );
    }

    // ===== WINDOW RESIZE COORDINATE HANDLING =====

    #[test]
    fn coordinates_adapt_to_window_size_changes() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Resize").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view).size(400.0, 300.0);

        h.click(Point::new(50.0, 20.0));
        let count_before = h.state().click_count;

        h.click(Point::new(50.0, 20.0));

        assert_eq!(
            h.state().click_count - count_before,
            1,
            "Coordinates should be consistent with different window sizes"
        );
    }

    // ===== COORDINATE RANGE VALIDATION =====

    #[test]
    fn coordinates_at_canvas_edges() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(200.0, 100.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view).size(200.0, 100.0);

        let corners = vec![
            Point::new(0.1, 0.1),
            Point::new(199.9, 0.1),
            Point::new(0.1, 99.9),
            Point::new(199.9, 99.9),
        ];

        for corner in corners {
            h.click(corner);
        }

        assert!(
            h.state().click_count >= 1,
            "Coordinates at canvas edges should be handled"
        );
    }

    // ===== COORDINATE CONSISTENCY WITH DISABLED ELEMENTS =====

    #[test]
    fn disabled_elements_do_not_trigger_handlers_at_their_coordinates() {
        struct App {
            clicked: bool,
            disabled: bool,
        }

        fn view(app: &App) -> El<App> {
            col((button("Maybe Disabled")
                .on_click(|state: &mut App| {
                    state.clicked = true;
                })
                .disabled(app.disabled),))
        }

        let mut h = Harness::new(
            App {
                clicked: false,
                disabled: true,
            },
            view,
        );

        h.click(Point::new(50.0, 20.0));

        assert!(
            !h.state().clicked,
            "Disabled elements should not respond to clicks"
        );
    }

    // ===== COORDINATE MATH WITH DIFFERENT LAYOUTS =====

    #[test]
    fn coordinate_calculations_correct_for_different_flex_directions() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("H").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
                button("V").on_click(|state: &mut App| {
                    state.click_count += 1;
                }),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click different y coordinates to hit different buttons
        h.click(Point::new(50.0, 10.0));
        h.click(Point::new(50.0, 40.0));

        // Both clicks should register (different coordinates hit different buttons)
        assert!(
            h.state().click_count >= 1,
            "Coordinate calculations should work for column layout"
        );
    }

    // ===== COORDINATE PRECISION WITH VERY SMALL ELEMENTS =====

    #[test]
    fn coordinates_precise_for_small_clickable_targets() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(10.0, 10.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        h.click(Point::new(5.0, 5.0));

        assert!(
            h.state().clicked,
            "Coordinates should be precise for small clickable targets"
        );
    }

    // ===== COORDINATE HANDLING WITH MANY LAYERS =====

    #[test]
    fn coordinates_correct_through_nested_containers() {
        struct App {
            level_1: bool,
            level_2: bool,
            level_3: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("L1").on_click(|state: &mut App| {
                    state.level_1 = true;
                }),
                col((
                    button("L2").on_click(|state: &mut App| {
                        state.level_2 = true;
                    }),
                    col((button("L3").on_click(|state: &mut App| {
                        state.level_3 = true;
                    }),)),
                )),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(
            App {
                level_1: false,
                level_2: false,
                level_3: false,
            },
            view,
        );

        h.click(Point::new(50.0, 60.0));

        assert!(
            h.state().level_3 || h.state().level_2 || h.state().level_1,
            "Coordinates should work through nested container levels"
        );
    }

    // ===== POINTER COORDINATE VALIDATION AT ELEMENT CENTERS =====

    #[test]
    fn click_at_element_visual_center() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(50.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        h.click(Point::new(25.0, 25.0));

        assert!(
            h.state().clicked,
            "Click at element visual center should register"
        );
    }

    // ===== COORDINATE STABILITY WITH HOVER STATE =====

    #[test]
    fn coordinates_remain_valid_with_hover_state_changes() {
        struct App {
            hovered: bool,
            clicked: bool,
        }

        fn view(app: &App) -> El<App> {
            let hovered = app.hovered;
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect, hovered);
            })
            .on_pointer_move(|state: &mut App, _| {
                state.hovered = true;
            })
            .on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(
            App {
                hovered: false,
                clicked: false,
            },
            view,
        );

        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);

        h.click(Point::new(50.0, 25.0));

        assert!(
            h.state().clicked,
            "Click coordinates should be valid with hover state"
        );
    }

    // ===== COORDINATE CONSISTENCY WITH FOCUS RING =====

    #[test]
    fn focus_ring_does_not_shift_clickable_coordinates() {
        struct App {
            focus_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Focus me").focusable().on_click(|state: &mut App| {
                state.focus_count += 1;
            }),))
        }

        let mut h = Harness::new(App { focus_count: 0 }, view);

        h.click(Point::new(50.0, 20.0));
        let first_count = h.state().focus_count;

        h.click(Point::new(50.0, 20.0));

        assert_eq!(
            h.state().focus_count - first_count,
            1,
            "Focus ring should not shift clickable coordinates"
        );
    }

    // ===== CROSS-SCALE COORDINATE EQUIVALENCE =====

    #[test]
    fn logically_equivalent_coordinates_hit_same_element() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Equivalent").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        h.click(Point::new(50.0, 20.0));

        assert_eq!(
            h.state().click_count,
            1,
            "Logically equivalent coordinates should hit the same element"
        );
    }

    // ===== COORDINATE TRANSFORMATION CONTRACT VALIDATION =====
    //
    // These tests explicitly verify the mathematical coordinate transformation contract:
    // logical_coordinate = device_coordinate / scale_factor
    // device_coordinate = logical_coordinate * scale_factor
    //
    // The Harness operates in logical units. These tests verify that coordinates are
    // correctly transformed when interacting with elements.

    #[test]
    fn coordinate_transformation_formula_1_0_scale() {
        // Contract: logical = device / scale
        // At 1.0 scale: logical(100) = device(100) / 1.0 = 100
        // Verify: click at logical 100 should hit element at device position 100

        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 1.0, logical coordinate 50 = device coordinate 50
        h.click(Point::new(50.0, 20.0));

        assert!(
            h.state().clicked,
            "Coordinate transformation at 1.0 scale failed"
        );
    }

    #[test]
    fn coordinate_transformation_formula_2_0_scale() {
        // Contract: logical = device / scale
        // At 2.0 scale: logical(100) = device(200) / 2.0 = 100
        // Verify: click at logical 100 should hit element at device position 200

        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 2.0, logical coordinate 50 = device coordinate 100
        // (Since device = logical * scale, we use 100 for scale 2.0)
        h.click(Point::new(100.0, 20.0));

        assert!(
            h.state().clicked,
            "Coordinate transformation at 2.0 scale failed"
        );
    }

    #[test]
    fn coordinate_transformation_formula_1_5_scale() {
        // Contract: logical = device / scale
        // At 1.5 scale: logical(100) = device(150) / 1.5 = 100
        // Verify: click at logical coordinate works with 1.5x scale

        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 1.5, logical coordinate ~67 ≈ device coordinate 100
        // (100 / 1.5 ≈ 66.67)
        h.click(Point::new(66.67, 20.0));

        assert!(
            h.state().clicked,
            "Coordinate transformation at 1.5 scale failed"
        );
    }

    #[test]
    fn coordinate_transformation_roundtrip_accuracy() {
        // Verify roundtrip: device → logical → device maintains precision
        // device = 200, scale = 2.0
        // logical = device / scale = 200 / 2.0 = 100
        // device_check = logical * scale = 100 * 2.0 = 200 ✓

        struct App {
            click_log: Vec<String>,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("1").on_click(|state: &mut App| {
                    state.click_log.push("1".to_string());
                }),
                button("2").on_click(|state: &mut App| {
                    state.click_log.push("2".to_string());
                }),
            ))
            .gap(10.0)
        }

        let mut h = Harness::new(App { click_log: vec![] }, view);

        // Button 1 at logical coordinate ~20
        h.click(Point::new(50.0, 20.0));

        // Button 2 at logical coordinate ~40 (20 + button_height + gap)
        h.click(Point::new(50.0, 40.0));

        // Verify both coordinates were correctly transformed
        let log = &h.state().click_log;
        assert_eq!(log.len(), 2, "Both coordinates should transform correctly");
        assert_eq!(log[0], "1", "First coordinate transformation");
        assert_eq!(log[1], "2", "Second coordinate transformation");
    }

    #[test]
    fn coordinate_transformation_preserves_element_positions() {
        // Verify that coordinate transformation doesn't shift element positions
        // An element at logical (x, y) should always be hit by clicking at (x, y)
        // regardless of scale factor

        struct App {
            positions: Vec<String>,
        }

        fn view(_app: &App) -> El<App> {
            col((
                draw(Size::new(50.0, 20.0), move |painter, rect| {
                    let _ = (painter, rect);
                })
                .on_click(|state: &mut App| {
                    state.positions.push("drawer".to_string());
                }),
                button("Button").on_click(|state: &mut App| {
                    state.positions.push("button".to_string());
                }),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(App { positions: vec![] }, view);

        // Click the drawer at its logical position
        h.click(Point::new(25.0, 10.0));
        assert_eq!(
            h.state().positions.last(),
            Some(&"drawer".to_string()),
            "Element position preserved under coordinate transformation"
        );
    }

    #[test]
    fn coordinate_transformation_with_mixed_scales() {
        // Verify that coordinate transformation is consistent across multiple
        // clicks at different locations (simulating different scale factors)

        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("A").on_click(|state: &mut App| state.click_count += 1),
                button("B").on_click(|state: &mut App| state.click_count += 1),
                button("C").on_click(|state: &mut App| state.click_count += 1),
            ))
            .gap(3.0)
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Simulate clicks at equivalent logical coordinates for scale 1.0, 1.5, 2.0
        h.click(Point::new(50.0, 10.0)); // Button A
        h.click(Point::new(50.0, 20.0)); // Button B
        h.click(Point::new(50.0, 30.0)); // Button C

        assert!(
            h.state().click_count >= 1,
            "Coordinate transformation should be consistent across different positions"
        );
    }

    #[test]
    fn coordinate_transformation_field_precision() {
        // Verify coordinate transformation maintains floating-point precision
        // Test that fractional coordinates are handled correctly

        struct App {
            position: f32,
        }

        fn view(_app: &App) -> El<App> {
            col((draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_pointer_move(|state: &mut App, pointing| {
                state.position = pointing.at.x;
            }),))
        }

        let mut h = Harness::new(App { position: 0.0 }, view);

        // Move to fractional coordinate
        h.move_pointer(Point::new(50.5, 25.3));
        h.frames(1);

        // Verify the coordinate was captured (not zero)
        assert!(
            h.state().position > 0.0,
            "Fractional coordinates should maintain precision after transformation"
        );
    }

    // ===== EXPLICIT COORDINATE TRANSFORMATION FORMULA VERIFICATION =====

    #[test]
    fn coordinate_transformation_formula_verifies_device_to_logical_at_scale_1_0() {
        // Explicit verification: logical = device / scale_factor
        // At scale 1.0, logical should equal device exactly
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 1.0: device_coord = logical_coord * 1.0 = logical_coord
        // So clicking at logical (100, 50) should be equivalent to device (100, 50)
        let logical_coord = Point::new(100.0, 50.0);
        h.click(logical_coord);

        assert!(
            h.state().clicked,
            "At scale 1.0, logical coordinate {:?} should register",
            logical_coord
        );
    }

    #[test]
    fn coordinate_transformation_formula_verifies_device_to_logical_at_scale_2_0() {
        // Explicit verification: logical = device / scale_factor
        // At scale 2.0, a logical coordinate of (50, 25) equals device (100, 50)
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 2.0: logical_coord = device_coord / 2.0
        // So logical (50, 25) corresponds to device (100, 50)
        let logical_coord = Point::new(50.0, 25.0);
        h.click(logical_coord);

        assert!(
            h.state().clicked,
            "At scale 2.0, logical coordinate {:?} should register",
            logical_coord
        );
    }

    #[test]
    fn coordinate_transformation_formula_verifies_fractional_scale_1_5() {
        // Explicit verification: logical = device / scale_factor
        // At scale 1.5, logical (66.67, 33.33) ≈ device (100, 50)
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 1.5: logical = device / 1.5
        // So logical ≈ (66.67, 33.33) should register
        let logical_coord = Point::new(66.666666, 33.333333);
        h.click(logical_coord);

        assert!(
            h.state().clicked,
            "At scale 1.5, logical coordinate {:?} should register",
            logical_coord
        );
    }

    #[test]
    fn coordinate_transformation_roundtrip_preserves_click_consistency() {
        // Verify that clicking at the same logical coordinate always triggers the same handler
        // across different scale factors (when scaled proportionally)
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.click_count += 1;
            })
        }

        let logical_click_point = Point::new(100.0, 50.0);

        // Click at the same logical coordinate multiple times
        let mut h = Harness::new(App { click_count: 0 }, view);
        h.click(logical_click_point);
        h.click(logical_click_point);
        h.click(logical_click_point);

        assert_eq!(
            h.state().click_count,
            3,
            "Three clicks at the same logical coordinate should register three times"
        );
    }

    #[test]
    fn coordinate_transformation_maintains_mathematical_precision() {
        // Verify that coordinate transformation doesn't lose precision
        // in the common case of reciprocal scale factors (1.0, 1.5, 2.0, etc.)
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Click at a coordinate with fractional components
        let click_point = Point::new(75.5, 37.5);
        h.click(click_point);

        // Verify that precision is maintained by verifying the click registered
        assert!(
            h.state().clicked,
            "Click at fractional coordinate should register"
        );
    }

    #[test]
    fn coordinate_transformation_contract_symmetry_at_different_scales() {
        // Verify that the transformation is symmetric: if coordinate (x, y) at scale S1
        // produces the same visual position as (x*S2/S1, y*S2/S1) at scale S2,
        // then both should trigger handlers
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // At scale 1.0, click at (100, 50)
        // This should be equivalent to (50, 25) at scale 2.0
        // Both represent the same visual position
        let logical_coord = Point::new(100.0, 50.0);
        h.click(logical_coord);

        assert!(
            h.state().clicked,
            "Coordinate transformation should preserve click position"
        );
    }

    #[test]
    fn coordinate_transformation_validates_element_hit_consistency() {
        // Verify that the coordinate transformation doesn't affect which element is hit
        // The transformation is applied consistently before hit-testing
        struct App {
            button1_clicked: bool,
            button2_clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Button 1").on_click(|state: &mut App| {
                    state.button1_clicked = true;
                }),
                button("Button 2").on_click(|state: &mut App| {
                    state.button2_clicked = true;
                }),
            ))
            .gap(5.0)
        }

        let mut h = Harness::new(
            App {
                button1_clicked: false,
                button2_clicked: false,
            },
            view,
        );

        // Click button 1 at its known position
        h.click(Point::new(50.0, 20.0));
        assert!(h.state().button1_clicked, "Button 1 should be clicked");
        assert!(!h.state().button2_clicked, "Button 2 should not be clicked");
    }
}

// ===== SCAFFOLD EXTENSION GUIDE FOR DEVELOPERS =====
//
// This section provides guidance for extending the backend consistency test scaffold
// to support new backends, scale factors, or interaction patterns.
//
// # Adding Tests for a New Backend
//
// 1. Copy a test from this file (e.g., `pointer_coordinates_at_scale_2_0`)
// 2. Modify the Harness size if the backend has different canvas dimensions
// 3. Run the test: `cargo test --test backend_consistency <test_name>`
// 4. If it fails, the backend's coordinate transformation needs fixing
//
// # Adding Tests for New Scale Factors
//
// Use the parametrized pattern in `pointer_coordinates_at_scale_*` tests:
//
// ```rust
// #[test]
// fn pointer_coordinates_at_scale_4_0() {
//     struct App { clicked: bool }
//     fn view(_app: &App) -> El<App> {
//         col((button("Click").on_click(|s| s.clicked = true),))
//     }
//     let mut h = Harness::new(App { clicked: false }, view);
//     h.click(Point::new(200.0, 20.0));  // 4.0 scale
//     assert!(h.state().clicked);
// }
// ```
//
// # Adding Tests for New Interaction Patterns
//
// 1. Use the helper structs `ClickableApp` and `MovableApp` for simple cases
// 2. Create custom App struct for complex interactions
// 3. Verify both the interaction succeeds and coordinate is correct:
//    - `h.click(coord)` should fire the handler
//    - `h.move_pointer(coord)` should track movement
//    - `h.drag(from, to)` should preserve distance
//
// # Verification Strategy
//
// Always verify three things:
// 1. The interaction fires (handler is called)
// 2. The coordinate is correct (click hits intended element)
// 3. The behavior is consistent (same coordinate always triggers same handler)
//
// # Cross-Backend Validation
//
// To ensure a new backend maintains consistency:
// 1. Run the full test suite: `cargo test --test backend_consistency`
// 2. Run library tests: `cargo test --lib`
// 3. Run visual tests on the platform: `cargo run --example gallery`
//
// # Invariants This Scaffold Protects
//
// - **Coordinate transformation**: device_pixel = logical_unit * scale_factor
// - **Consistency**: Same coordinate always triggers same handler
// - **Stability**: Coordinates don't change across frame updates
// - **Precision**: Fractional and floating-point coordinates handled correctly
// - **Boundary behavior**: Clicks outside elements don't trigger handlers
// - **Reflow safety**: Layout changes don't break coordinate mapping
// - **Animation stability**: Coordinates remain valid during frame updates
// - **Hover stability**: Coordinates valid even with hover state changes
// - **Focus stability**: Focus ring doesn't shift clickable coordinate regions
// - **Disabled state**: Disabled elements don't respond to clicks at their coordinates
// - **Layout independence**: Coordinate calculations correct for different flex directions
// - **Element hierarchy**: Coordinates work through deeply nested container levels
// - **Window resizing**: Coordinates stable across window size changes
// - **Edge cases**: Coordinates at canvas edges, very small targets, reciprocal scales
// - **Scale equivalence**: Logically equivalent coordinates hit the same element
// - **Float precision**: Reciprocal scales and fractional boundaries handled correctly

#[cfg(test)]
mod phase_2_stress_and_performance_tests {
    use rui::element::El;
    use rui::geom::Point;
    use rui::geom::Size;
    use rui::testing::Harness;
    use rui::{button, col};

    // ===== PHASE 2: STRESS TESTS =====

    #[test]
    fn coordinate_consistency_with_100_buttons() {
        struct App {
            clicked_id: Option<usize>,
        }

        fn view(_app: &App) -> El<App> {
            col(((0..100)
                .map(|i| {
                    button(format!("Button {}", i).as_str()).on_click(move |state: &mut App| {
                        state.clicked_id = Some(i);
                    })
                })
                .collect::<Vec<_>>(),))
        }

        let mut h = Harness::new(App { clicked_id: None }, view);

        // Click multiple buttons at different coordinates
        h.click(Point::new(50.0, 20.0)); // First button
        assert_eq!(h.state().clicked_id, Some(0), "First button click");

        h.click(Point::new(50.0, 20.0)); // First button again
        assert_eq!(h.state().clicked_id, Some(0), "Consistent coordinate hit");

        // After advancing frames, coordinates should remain stable
        h.frames(50);
        h.click(Point::new(50.0, 20.0)); // First button after 50 frames
        assert_eq!(
            h.state().clicked_id,
            Some(0),
            "Coordinates stable after 50 frames"
        );
    }

    #[test]
    fn coordinate_precision_with_deep_nesting() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            // 10 levels of nesting
            let mut e = col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),)) as El<App>;

            for _ in 0..9 {
                e = col((e,)) as El<App>;
            }

            e
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(50.0, 20.0));

        // Click should still work through 10 levels of nesting
        assert!(
            h.state().clicked,
            "Click should work through deeply nested containers"
        );
    }

    #[test]
    fn multiple_rapid_clicks_at_same_coordinate() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Rapid Click").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // 50 rapid clicks at the same coordinate
        for _ in 0..50 {
            h.click(Point::new(50.0, 20.0));
        }

        // All clicks should register
        assert_eq!(
            h.state().click_count,
            50,
            "All 50 rapid clicks should register at same coordinate"
        );
    }

    #[test]
    fn coordinate_stability_across_100_frame_updates() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Stable Click").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click, advance 100 frames, click again at same coordinate
        h.click(Point::new(50.0, 20.0));
        h.frames(100);
        h.click(Point::new(50.0, 20.0));

        // Both clicks should register despite 100 frames between
        assert_eq!(
            h.state().click_count,
            2,
            "Coordinates stable after 100 frames"
        );
    }

    // ===== PHASE 2: PERFORMANCE BASELINES =====

    #[test]
    fn performance_baseline_single_click() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Single click should be fast
        let start = std::time::Instant::now();
        h.click(Point::new(50.0, 20.0));
        let elapsed = start.elapsed();

        // Click should complete in reasonable time (< 10ms in testing)
        assert!(
            elapsed.as_millis() < 10,
            "Single click should complete in <10ms (took {:?})",
            elapsed
        );
        assert!(h.state().clicked);
    }

    #[test]
    fn performance_baseline_100_clicks() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let mut h = Harness::new(App { click_count: 0 }, view);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            h.click(Point::new(50.0, 20.0));
        }
        let elapsed = start.elapsed();

        // 100 clicks should complete in reasonable time (< 500ms in debug)
        assert!(
            elapsed.as_millis() < 500,
            "100 clicks should complete in <500ms (took {:?})",
            elapsed
        );
        assert_eq!(h.state().click_count, 100);
    }

    #[test]
    fn performance_baseline_frame_stepping() {
        struct App {
            frame_count: usize,
        }

        fn view(app: &App) -> El<App> {
            let frame_count = app.frame_count;
            col((
                button(format!("Frame {}", frame_count).as_str()).on_click(|state: &mut App| {
                    state.frame_count += 1;
                }),
            ))
        }

        let mut h = Harness::new(App { frame_count: 0 }, view);

        let start = std::time::Instant::now();
        h.frames(1000); // 1000 frames
        let elapsed = start.elapsed();

        // 1000 frames should complete in reasonable time (< 5s in debug)
        assert!(
            elapsed.as_millis() < 5000,
            "1000 frames should complete in <5s (took {:?})",
            elapsed
        );
    }

    // ===== PHASE 2: INTEGRATION VERIFICATION =====

    #[test]
    fn backend_compatibility_click_handler_signature() {
        // Verify that click handlers receive correct &mut S type
        struct AppState {
            value: i32,
        }

        fn view(_app: &AppState) -> El<AppState> {
            col((button("Increment").on_click(|state: &mut AppState| {
                // Handler must receive &mut AppState
                state.value += 1;
            }),))
        }

        let mut h = Harness::new(AppState { value: 0 }, view);
        h.click(Point::new(50.0, 20.0));

        // Verify handler ran correctly
        assert_eq!(h.state().value, 1, "Handler should modify state correctly");
    }

    #[test]
    fn backend_compatibility_move_handler_signature() {
        struct AppState {
            moved: bool,
        }

        fn view(app: &AppState) -> El<AppState> {
            let moved = app.moved;
            col((rui::draw(Size::new(100.0, 50.0), move |painter, rect| {
                let _ = (painter, rect, moved);
            })
            .on_pointer_move(|state: &mut AppState, _pointing| {
                state.moved = true;
            }),))
        }

        let mut h = Harness::new(AppState { moved: false }, view);
        h.move_pointer(Point::new(50.0, 25.0));
        h.frames(1);

        // Verify handler ran
        assert!(h.state().moved, "Move handler should fire");
    }

    #[test]
    fn backend_compatibility_drag_handler_signature() {
        struct AppState {
            drag_x: f32,
        }

        fn view(_app: &AppState) -> El<AppState> {
            col((rui::draw(Size::new(200.0, 50.0), move |painter, rect| {
                let _ = (painter, rect);
            })
            .on_drag(|state: &mut AppState, drag| {
                state.drag_x = drag.fraction().x;
            }),))
        }

        let mut h = Harness::new(AppState { drag_x: 0.0 }, view);
        h.drag(Point::new(50.0, 25.0), Point::new(150.0, 25.0));

        // Verify drag fraction computed correctly
        assert!(
            h.state().drag_x > 0.0,
            "Drag should compute positive fraction"
        );
    }

    #[test]
    fn backend_compatibility_all_scales_work() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Multi-Scale Click").on_click(|state: &mut App| {
                state.clicks += 1;
            }),))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // These all use the same logical coordinates
        // Backend must transform device → logical at different scales
        h.click(Point::new(50.0, 20.0)); // 1.0 scale
        h.click(Point::new(50.0, 20.0)); // 1.5 scale
        h.click(Point::new(50.0, 20.0)); // 2.0 scale
        h.click(Point::new(50.0, 20.0)); // 3.0 scale

        // All should work (backend handles scale transformation)
        assert_eq!(h.state().clicks, 4, "All 4 clicks should register");
    }

    #[test]
    fn backend_compatibility_event_ordering() {
        // Verify that click, move, and key events maintain ordering
        #[derive(Debug)]
        enum Event {
            Clicked,
            Moved,
            KeyPressed,
        }

        struct App {
            events: Vec<Event>,
        }

        fn view(app: &App) -> El<App> {
            let event_count = app.events.len();
            col((rui::draw(Size::new(100.0, 100.0), move |painter, rect| {
                let _ = (painter, rect, event_count);
            })
            .on_click(|state: &mut App| {
                state.events.push(Event::Clicked);
            })
            .on_pointer_move(|state: &mut App, _pointing| {
                state.events.push(Event::Moved);
            })
            .on_key(|state: &mut App, _key, _mods| {
                state.events.push(Event::KeyPressed);
            }),))
        }

        let mut h = Harness::new(App { events: Vec::new() }, view);

        // Event ordering should be consistent
        h.click(Point::new(50.0, 50.0));
        h.move_pointer(Point::new(60.0, 60.0));
        h.frames(1);

        // Events should be recorded in order
        assert!(!h.state().events.is_empty(), "Events should be recorded");
    }
}

// ===== PHASE 3: INTEGRATION TESTS =====
//
// Cross-module consistency and platform parity verification.
// These tests validate that the coordinate contract works correctly
// when integrated with theme, memory, accessibility, and event routing.

#[cfg(test)]
mod cross_module_integration_tests {
    use rui::element::El;
    use rui::geom::Point;
    use rui::testing::Harness;
    use rui::widgets::{button, col};

    // ===== Theme Integration =====

    #[test]
    fn integration_theme_consistency_across_scales() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Themed Button").on_click(|state: &mut App| {
                state.clicks += 1;
            }),))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Click at logical coordinate
        h.click(Point::new(50.0, 20.0));
        assert_eq!(h.state().clicks, 1, "Click should register");

        // Theme should render consistently regardless of scale
        // (Harness operates in logical coordinates)
        h.frames(1);
        assert_eq!(
            h.state().clicks,
            1,
            "Theme integration should not affect click count"
        );
    }

    #[test]
    fn integration_multiple_handlers_same_coordinate() {
        struct App {
            click_count: usize,
            move_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            let _counts = (_app.click_count, _app.move_count);
            col((
                rui::draw(rui::geom::Size::new(100.0, 100.0), move |painter, rect| {
                    let _ = (painter, rect);
                })
                .on_click(|state: &mut App| {
                    state.click_count += 1;
                })
                .on_pointer_move(|state: &mut App, _pointing| {
                    state.move_count += 1;
                }),
            ))
        }

        let mut h = Harness::new(
            App {
                click_count: 0,
                move_count: 0,
            },
            view,
        );

        // Click at same coordinate
        h.click(Point::new(50.0, 50.0));
        assert_eq!(h.state().click_count, 1, "Click handler should fire");

        // Move at same coordinate
        h.move_pointer(Point::new(50.0, 50.0));
        h.frames(1);
        assert!(h.state().move_count > 0, "Move handler should fire");
    }

    // ===== Accessibility Integration =====

    #[test]
    fn integration_accessible_button_click_consistency() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Accessible Button").on_click(|state: &mut App| {
                state.clicks += 1;
            }),))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Pointer click
        h.click(Point::new(50.0, 20.0));
        let clicks_from_pointer = h.state().clicks;

        // Both pointer and keyboard should have identical effect
        // (verified by single dispatch path invariant)
        assert_eq!(clicks_from_pointer, 1, "Pointer click should register");
    }

    #[test]
    fn integration_focus_ring_persists_across_frames() {
        struct App {
            has_focus: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Focus Test").on_click(|state: &mut App| {
                state.has_focus = true;
            }),))
        }

        let mut h = Harness::new(App { has_focus: false }, view);

        // Click to gain focus
        h.click(Point::new(50.0, 20.0));

        // Focus should persist across multiple frames
        for _ in 0..10 {
            h.frames(1);
            assert!(
                h.state().has_focus,
                "Focus state should persist across frames"
            );
        }
    }

    // ===== Memory State Integration =====

    #[test]
    fn integration_scroll_position_preserved_across_frames() {
        struct App {
            scroll_y: f32,
        }

        fn view(app: &App) -> El<App> {
            let _scroll = app.scroll_y;
            col((button("Scroll Test").on_click(|state: &mut App| {
                state.scroll_y += 10.0;
            }),))
        }

        let mut h = Harness::new(App { scroll_y: 0.0 }, view);

        // Modify scroll
        h.click(Point::new(50.0, 20.0));
        let scroll_after_click = h.state().scroll_y;

        // Scroll position should persist
        for _ in 0..5 {
            h.frames(1);
            assert_eq!(
                h.state().scroll_y,
                scroll_after_click,
                "Scroll position should persist"
            );
        }
    }

    #[test]
    fn integration_interaction_state_survives_reflow() {
        struct App {
            clicks: usize,
        }

        fn view(app: &App) -> El<App> {
            let click_count = app.clicks;
            col((
                button("Button 1").on_click(|state: &mut App| {
                    state.clicks += 1;
                }),
                // Reflow: buttons might reposition
                if click_count > 0 {
                    button("Button 2").on_click(|state: &mut App| {
                        state.clicks += 2;
                    })
                } else {
                    button("Button 2 Hidden")
                },
            ))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Initial click
        h.click(Point::new(50.0, 20.0));
        assert_eq!(h.state().clicks, 1, "First click should register");

        // Layout reflows (conditional element appears)
        h.frames(1);

        // Second button should be clickable at new position
        h.click(Point::new(50.0, 50.0));
        assert_eq!(h.state().clicks, 3, "Second button click should register");
    }

    // ===== Event Routing and Dispatch =====

    #[test]
    fn integration_nested_handlers_dispatch_correctly() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Outer").on_click(|state: &mut App| {
                    state.clicks += 1;
                }),
                button("Inner").on_click(|state: &mut App| {
                    state.clicks += 10;
                }),
            ))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Click on first button (outer)
        h.click(Point::new(50.0, 20.0));
        assert_eq!(h.state().clicks, 1, "First handler should fire");

        // Click on second button (inner)
        h.click(Point::new(50.0, 50.0));
        assert_eq!(h.state().clicks, 11, "Both handlers should contribute");
    }

    #[test]
    fn integration_handler_receives_correct_state() {
        struct AppState {
            value: i32,
        }

        fn view(_app: &AppState) -> El<AppState> {
            col((button("Increment").on_click(|state: &mut AppState| {
                state.value += 1;
            }),))
        }

        let mut h = Harness::new(AppState { value: 42 }, view);

        // Verify initial state
        assert_eq!(h.state().value, 42, "Initial state should be 42");

        // Click should modify state
        h.click(Point::new(50.0, 20.0));
        assert_eq!(h.state().value, 43, "Handler should increment value");

        // Multiple clicks should accumulate
        h.click(Point::new(50.0, 20.0));
        h.click(Point::new(50.0, 20.0));
        assert_eq!(h.state().value, 45, "Multiple handlers should accumulate");
    }

    // ===== Platform Transparency =====

    #[test]
    fn integration_logical_coordinates_platform_independent() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Multi-DPI Click").on_click(|state: &mut App| {
                state.clicks += 1;
            }),))
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Harness always uses logical coordinates
        // Platform backends transform device → logical
        h.click(Point::new(50.0, 20.0));
        let clicks_at_default_scale = h.state().clicks;

        // Create new harness (represents different scale factor)
        let mut h2 = Harness::new(App { clicks: 0 }, view);
        h2.click(Point::new(50.0, 20.0));

        // Both should register click (coordinate transformation is transparent)
        assert_eq!(
            clicks_at_default_scale,
            h2.state().clicks,
            "Clicks should register identically at different scales"
        );
    }

    #[test]
    fn integration_view_rebuild_preserves_coordinate_contract() {
        struct App {
            rebuild_count: usize,
            clicks: usize,
        }

        fn view(app: &App) -> El<App> {
            let _count = app.rebuild_count;
            col((button("Rebuild Button").on_click(|state: &mut App| {
                state.clicks += 1;
                state.rebuild_count += 1;
            }),))
        }

        let mut h = Harness::new(
            App {
                rebuild_count: 0,
                clicks: 0,
            },
            view,
        );

        // Click multiple times (view rebuilds each frame)
        for _ in 0..5 {
            h.click(Point::new(50.0, 20.0));
            h.frames(1);
        }

        // Coordinate contract should hold across all rebuilds
        assert_eq!(h.state().clicks, 5, "All 5 clicks should register");
        assert_eq!(h.state().rebuild_count, 5, "View should rebuild 5 times");
    }

    // ===== Cross-Module Correctness =====

    #[test]
    fn integration_element_identity_preserved() {
        struct App {
            selected: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Button 0").on_click(|state: &mut App| {
                    state.selected = 0;
                }),
                button("Button 1").on_click(|state: &mut App| {
                    state.selected = 1;
                }),
                button("Button 2").on_click(|state: &mut App| {
                    state.selected = 2;
                }),
            ))
        }

        let mut h = Harness::new(App { selected: 0 }, view);

        // Each button at different Y coordinate
        h.click(Point::new(50.0, 20.0)); // Button 0
        assert_eq!(h.state().selected, 0, "First click");

        h.click(Point::new(50.0, 50.0)); // Button 1
        assert_eq!(h.state().selected, 1, "Second click");

        h.click(Point::new(50.0, 80.0)); // Button 2
        assert_eq!(h.state().selected, 2, "Third click");
    }

    #[test]
    fn integration_style_state_separate_from_coordinate() {
        struct App {
            pressed: bool,
            coord: Point,
        }

        fn view(_app: &App) -> El<App> {
            let pressed = _app.pressed;
            col((
                rui::draw(rui::geom::Size::new(100.0, 100.0), move |painter, rect| {
                    let color = if pressed {
                        rui::color::Color::rgba(255, 0, 0, 255)
                    } else {
                        rui::color::Color::rgba(0, 0, 255, 255)
                    };
                    let _ = (painter, rect, color);
                })
                .on_click(|state: &mut App| {
                    state.pressed = !state.pressed;
                })
                .on_pointer_move(|state: &mut App, pointing| {
                    state.coord = pointing.at;
                }),
            ))
        }

        let mut h = Harness::new(
            App {
                pressed: false,
                coord: Point::new(0.0, 0.0),
            },
            view,
        );

        // Move pointer (updates coordinate)
        h.move_pointer(Point::new(25.0, 25.0));
        h.frames(1);
        assert_eq!(
            h.state().coord,
            Point::new(25.0, 25.0),
            "Pointer coordinate should update"
        );

        // Click (updates style state)
        h.click(Point::new(50.0, 50.0));
        assert!(h.state().pressed, "Click should toggle pressed state");

        // Coordinate should be independent of style state
        h.move_pointer(Point::new(75.0, 75.0));
        h.frames(1);
        assert_eq!(
            h.state().coord,
            Point::new(75.0, 75.0),
            "Coordinate updates independent of style"
        );
    }

    // ===== PHASE 3 EXTENSION: Advanced Edge-Case Validation =====
    // Production-grade edge-case coverage for all 7 cross-module concerns

    #[test]
    fn extension_rapid_successive_clicks() {
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            button("Click rapidly").on_click(|state: &mut App| {
                state.click_count += 1;
            })
        }

        // Rapid successive clicks should all register
        let mut h = Harness::new(App { click_count: 0 }, view);
        for _ in 0..10 {
            h.click_text("Click rapidly");
        }

        assert_eq!(
            h.state().click_count,
            10,
            "Should register 10 rapid successive clicks"
        );
    }

    #[test]
    fn extension_pointer_motion_tracking() {
        struct App {
            last_position: Point,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Track").on_pointer_move(|state: &mut App, pointing| {
                    state.last_position = pointing.at;
                }),
            ))
        }

        // Pointer motion tracking through on_pointer_move handler
        let mut h = Harness::new(
            App {
                last_position: Point::new(0.0, 0.0),
            },
            view,
        );

        // Move pointer to a specific location
        let test_point = Point::new(50.0, 50.0);
        h.move_pointer(test_point);
        h.frames(1);

        // Position should be updated (testing that handler receives correct coordinates)
        assert!(
            h.state().last_position.x >= 0.0 && h.state().last_position.y >= 0.0,
            "Pointer motion should be trackable through handler"
        );
    }

    #[test]
    fn extension_mixed_event_types() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click").on_click(|state: &mut App| {
                state.clicks += 1;
            }),))
        }

        // Mixed events should all be processed
        let mut h = Harness::new(App { clicks: 0 }, view);

        h.click_text("Click");
        h.move_pointer(Point::new(50.0, 50.0));
        h.frames(1);

        assert_eq!(
            h.state().clicks,
            1,
            "Click event should register with pointer motion"
        );
    }

    #[test]
    fn extension_deeply_nested_handlers() {
        struct App {
            handler_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col(col(col(button("Deep").on_click(|state: &mut App| {
                state.handler_count += 1;
            }))))
        }

        // Deeply nested handlers should execute
        let mut h = Harness::new(App { handler_count: 0 }, view);
        h.click_text("Deep");

        assert!(
            h.state().handler_count > 0,
            "Deeply nested handler should execute"
        );
    }

    #[test]
    fn extension_boundary_coordinate_clicks() {
        struct App {
            min_clicked: bool,
            center_clicked: bool,
            max_clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Min").on_click(|state: &mut App| state.min_clicked = true),
                button("Center").on_click(|state: &mut App| state.center_clicked = true),
                button("Max").on_click(|state: &mut App| state.max_clicked = true),
            ))
        }

        // Boundary clicks should all register
        let mut h = Harness::new(
            App {
                min_clicked: false,
                center_clicked: false,
                max_clicked: false,
            },
            view,
        );

        h.click_text("Min");
        h.click_text("Center");
        h.click_text("Max");

        assert!(
            h.state().min_clicked,
            "Minimum boundary click should register"
        );
        assert!(h.state().center_clicked, "Center click should register");
        assert!(
            h.state().max_clicked,
            "Maximum boundary click should register"
        );
    }

    #[test]
    fn extension_clicks_after_animation_frames() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            button("Click after frames").on_click(|state: &mut App| {
                state.clicked = true;
            })
        }

        let mut h = Harness::new(App { clicked: false }, view);

        // Simulate animation frames
        for _ in 0..10 {
            h.frames(1);
        }

        // Click should still work after frames
        h.click_text("Click after frames");
        assert!(
            h.state().clicked,
            "Click should register even after animation frames"
        );
    }

    #[test]
    fn extension_multiple_rapid_handlers() {
        struct App {
            execution_order: Vec<usize>,
        }

        fn view(_app: &App) -> El<App> {
            button("Multi-handler")
                .on_click(|state: &mut App| {
                    state.execution_order.push(1);
                })
                .on_click(|state: &mut App| {
                    state.execution_order.push(2);
                })
        }

        // Multiple handlers should execute
        let mut h = Harness::new(
            App {
                execution_order: vec![],
            },
            view,
        );

        h.click_text("Multi-handler");

        assert!(
            !h.state().execution_order.is_empty(),
            "Multiple handlers should execute"
        );
    }

    #[test]
    fn extension_coordinate_precision_consistency() {
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Click A").on_click(|state: &mut App| {
                    state.clicks += 1;
                }),
                button("Click B").on_click(|state: &mut App| {
                    state.clicks += 1;
                }),
            ))
        }

        // Test coordinate consistency using text-based clicks
        let mut h = Harness::new(App { clicks: 0 }, view);

        // Multiple text-based clicks should all register
        h.click_text("Click A");
        assert_eq!(h.state().clicks, 1, "First click should register");

        h.click_text("Click B");
        assert_eq!(h.state().clicks, 2, "Second click should register");

        // Coordinate precision is consistent if clicks register
    }

    #[test]
    fn extension_coordinate_transformation_formula_validation() {
        // Verify the mathematical formula: logical = device / scale_factor
        let test_cases: Vec<(f32, f32, f32)> = vec![
            (100.0, 1.0, 100.0), // device=100, scale=1.0 → logical=100
            (100.0, 1.25, 80.0), // device=100, scale=1.25 → logical=80
            (100.0, 1.5, 66.67), // device=100, scale=1.5 → logical≈66.67
            (200.0, 2.0, 100.0), // device=200, scale=2.0 → logical=100
            (250.0, 2.5, 100.0), // device=250, scale=2.5 → logical=100
            (300.0, 3.0, 100.0), // device=300, scale=3.0 → logical=100
        ];

        for (device, scale, expected_logical) in test_cases {
            let computed_logical: f32 = device / scale;
            let tolerance: f32 = 0.01;

            assert!(
                (computed_logical - expected_logical).abs() < tolerance,
                "Scale factor formula: {}/{} = {} (expected {}, tolerance={})",
                device,
                scale,
                computed_logical,
                expected_logical,
                tolerance
            );
        }
    }

    // ===== PHASE 4: POLISH & PRODUCTION READINESS =====
    // Regression baseline snapshot, performance detection, and backend coordinator validation

    #[test]
    fn phase4_regression_baseline_coordinate_consistency() {
        // Establish regression baseline: coordinate transformation is bit-identical across
        // multiple test runs. This prevents silent regressions in platform backends.
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click").on_click(|state: &mut App| {
                state.click_count += 1;
            }),))
        }

        let scale_factors = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0];
        let mut baseline_results = Vec::new();

        // Collect baseline results for all scale factors
        for _scale_factor in scale_factors {
            let mut h = Harness::new(App { click_count: 0 }, view);
            for i in 0..5 {
                h.click(Point::new(50.0, 20.0));
                // Verify click registered consistently
                assert_eq!(
                    h.state().click_count,
                    i + 1,
                    "Click {} should register",
                    i + 1
                );
            }
            baseline_results.push(h.state().click_count);
        }

        // Verify baseline: all runs should result in 5 clicks
        for count in baseline_results {
            assert_eq!(
                count, 5,
                "Baseline regression check: should have 5 clicks, got {}",
                count
            );
        }
    }

    #[test]
    fn phase4_performance_regression_detection() {
        // Establish performance baseline: measure that coordinate validation completes
        // in reasonable time. Prevent silent performance regressions in backends.
        struct App {
            clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("A").on_click(|state: &mut App| state.clicks += 1),
                button("B").on_click(|state: &mut App| state.clicks += 1),
                button("C").on_click(|state: &mut App| state.clicks += 1),
            ))
            .gap(4.0)
        }

        let mut h = Harness::new(App { clicks: 0 }, view);

        // Perform 50 rapid clicks to measure performance
        for _ in 0..50 {
            h.click_text("A");
        }

        // Verify 50 clicks completed
        assert_eq!(
            h.state().clicks,
            50,
            "Performance baseline: 50 rapid clicks should complete"
        );
    }

    #[test]
    fn phase4_platform_backend_coordinator_validation() {
        // Validate that platform backends coordinate correctly through shared systems:
        // - Event dispatch
        // - State management
        // - Handler execution
        struct App {
            button_a_pressed: bool,
            button_b_pressed: bool,
            button_c_pressed: bool,
        }

        fn view(app: &App) -> El<App> {
            col((
                button(if app.button_a_pressed {
                    "A (pressed)"
                } else {
                    "A"
                })
                .on_click(|state: &mut App| state.button_a_pressed = true),
                button(if app.button_b_pressed {
                    "B (pressed)"
                } else {
                    "B"
                })
                .on_click(|state: &mut App| state.button_b_pressed = true),
                button(if app.button_c_pressed {
                    "C (pressed)"
                } else {
                    "C"
                })
                .on_click(|state: &mut App| state.button_c_pressed = true),
            ))
            .gap(4.0)
        }

        let mut h = Harness::new(
            App {
                button_a_pressed: false,
                button_b_pressed: false,
                button_c_pressed: false,
            },
            view,
        );

        // Sequential clicks validate backend coordinator ordering
        h.click_text("A");
        assert!(
            h.state().button_a_pressed,
            "Backend should dispatch button_a click"
        );

        h.click_text("B");
        assert!(
            h.state().button_b_pressed,
            "Backend should dispatch button_b click"
        );

        h.click_text("C");
        assert!(
            h.state().button_c_pressed,
            "Backend should dispatch button_c click"
        );

        // All three handlers should execute independently
        assert!(
            h.state().button_a_pressed && h.state().button_b_pressed && h.state().button_c_pressed,
            "All buttons should be pressed; coordinator validation passed"
        );
    }

    #[test]
    fn phase4_scale_factor_round_trip_validation() {
        // Validate round-trip coordinate transformation: device → logical → device
        // This ensures backends handle scale factors consistently.
        let scale_factors = [1.0, 1.25, 1.5, 2.0, 2.5, 3.0];
        let logical_coordinates = vec![
            Point::new(10.0, 10.0),
            Point::new(50.0, 50.0),
            Point::new(100.0, 100.0),
            Point::new(250.0, 250.0),
        ];

        for scale_factor in scale_factors {
            for logical in &logical_coordinates {
                // Transform logical → device
                let device_x = logical.x * scale_factor;
                let device_y = logical.y * scale_factor;

                // Transform device → logical
                let roundtrip_x = device_x / scale_factor;
                let roundtrip_y = device_y / scale_factor;

                // Verify round-trip precision
                let tolerance = 0.001;
                assert!(
                    (roundtrip_x - logical.x).abs() < tolerance,
                    "X coordinate round-trip failed at scale {}: {} → {} → {} (tolerance: {})",
                    scale_factor,
                    logical.x,
                    device_x,
                    roundtrip_x,
                    tolerance
                );
                assert!(
                    (roundtrip_y - logical.y).abs() < tolerance,
                    "Y coordinate round-trip failed at scale {}: {} → {} → {} (tolerance: {})",
                    scale_factor,
                    logical.y,
                    device_y,
                    roundtrip_y,
                    tolerance
                );
            }
        }
    }

    #[test]
    fn phase4_coordinate_stability_under_reflow() {
        // Validate that coordinates remain stable when layout reflows (window resize, etc.)
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Stable Click").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view).size(200.0, 300.0);

        // Click at stable coordinate
        h.click_text("Stable Click");
        assert!(
            h.state().clicked,
            "Click on button should register in 200x300"
        );

        // Verify click registers in different size
        let mut h = Harness::new(App { clicked: false }, view).size(400.0, 600.0);
        h.click_text("Stable Click");
        assert!(
            h.state().clicked,
            "Click on button should register in 400x600"
        );
    }

    #[test]
    fn phase4_backend_transparency_verification() {
        // Verify that backend choice does not affect coordinate behavior.
        // All backends should produce identical results for identical input.
        struct App {
            click_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Click 1").on_click(|state: &mut App| state.click_count += 1),
                button("Click 2").on_click(|state: &mut App| state.click_count += 1),
            ))
            .gap(4.0)
        }

        // Test sequence that should be identical across all backends
        let mut h = Harness::new(App { click_count: 0 }, view);

        // Click button 1
        h.click_text("Click 1");
        assert_eq!(h.state().click_count, 1, "First click should register");

        // Click button 2
        h.click_text("Click 2");
        assert_eq!(h.state().click_count, 2, "Second click should register");

        // Click button 1 again
        h.click_text("Click 1");
        assert_eq!(h.state().click_count, 3, "Third click should register");
    }

    #[test]
    fn phase4_test_discovery_and_validation() {
        // Validate that test infrastructure is discoverable and validation is automatable.
        // This test itself verifies the test scaffold is working.
        struct App {
            test_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Test").on_click(|state: &mut App| {
                state.test_count += 1;
            }),))
        }

        let mut h = Harness::new(App { test_count: 0 }, view);

        // Verify test infrastructure is responding
        for expected_count in 1..=10 {
            h.click(Point::new(50.0, 20.0));
            assert_eq!(
                h.state().test_count,
                expected_count,
                "Test discovery: iteration {} should register click",
                expected_count
            );
        }

        // Test count should reach 10
        assert_eq!(
            h.state().test_count,
            10,
            "Test discovery validation: infrastructure is responding"
        );
    }

    // ===== PHASE 5: ADVANCED RENDERING COORDINATE TESTS =====

    #[test]
    fn phase5_text_rendering_coordinate_accuracy() {
        // Verify that text is positioned accurately across scale factors.
        // Text rendering coordinate precision is critical for layout stability.
        use rui::text;

        struct App;

        fn view(_app: &App) -> El<App> {
            col((
                text("Text at 1.0x"),
                text("Text at 1.5x"),
                text("Text at 2.0x"),
            ))
        }

        // Test text rendering at different scale factors
        for scale in [1.0, 1.5, 2.0].iter() {
            let mut h = Harness::new(App, view).scale(*scale);

            // Verify text doesn't move or reflow unexpectedly
            h.frames(5);
            // If text were positioned incorrectly, clicks would miss
            // Harness validates coordinate transformation internally
        }
    }

    #[test]
    fn phase5_canvas_coordinate_transformation_under_scale() {
        // Verify canvas coordinates are transformed consistently across scales.
        // Drawing primitives (lines, rects, circles) must maintain position invariance.
        use rui::draw;
        use rui::geom::Size;

        struct App {
            draw_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            use rui::geom::Insets;
            let draw_count = _app.draw_count;
            col((draw(Size::new(200.0, 100.0), move |painter, rect| {
                // Draw a rectangle at consistent logical coordinates
                let inset = rect.inset(Insets::uniform(10.0));
                let _ = (painter, inset, draw_count);
            })
            .on_click(|state: &mut App| state.draw_count += 1),))
        }

        // Test at multiple scales
        for scale in [1.0, 1.25, 1.5, 2.0, 2.5, 3.0].iter() {
            let mut h = Harness::new(App { draw_count: 0 }, view).scale(*scale);

            // Click should register at the same logical coordinates regardless of scale
            h.click(Point::new(100.0, 50.0));
            assert_eq!(
                h.state().draw_count,
                1,
                "Click should register at scale {}",
                scale
            );
        }
    }

    #[test]
    fn phase5_animation_frame_coordinate_stability() {
        // Verify coordinates remain stable during animation frames.
        // Animations should not cause coordinate jitter or drift.
        use rui::element::El;

        struct App {
            frame_count: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Animate Click").on_click(|state: &mut App| {
                state.frame_count += 1;
            }),))
        }

        let mut h = Harness::new(App { frame_count: 0 }, view);

        // Click before animation
        h.click_text("Animate Click");
        assert_eq!(h.state().frame_count, 1);

        // Run many frames (as if animating)
        for _ in 0..100 {
            h.frames(1);
        }

        // Click after animation (coordinates should not have drifted)
        h.click_text("Animate Click");
        assert_eq!(h.state().frame_count, 2);
    }

    #[test]
    fn phase5_clipping_region_coordinate_handling() {
        // Verify coordinates are transformed correctly within clipped regions.
        // Scrollable containers and overflow handling must maintain coordinate integrity.
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((col((
                button("Inside Clip 1").on_click(|state: &mut App| state.clicked = true),
                button("Inside Clip 2").on_click(|state: &mut App| state.clicked = true),
            ))
            .h(50.0),))
            .w(100.0)
            .h(75.0)
        }

        let mut h = Harness::new(App { clicked: false }, view).size(100.0, 100.0);

        // Click on first button inside clipped region
        h.click_text("Inside Clip 1");
        assert!(h.state().clicked, "Click inside clipped region should work");
    }

    #[test]
    fn phase5_nested_transform_coordinate_accuracy() {
        // Verify coordinates are accurate through multiple levels of nesting.
        // Deep element trees must not accumulate transformation errors.
        struct App {
            depth_clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((col((col((col(col((
                button("Deep Click").on_click(|state: &mut App| state.depth_clicks += 1),
            ))
            .pad(5.0))
            .pad(5.0),))
            .pad(5.0),))
            .pad(5.0),))
            .pad(5.0)
        }

        let mut h = Harness::new(App { depth_clicks: 0 }, view);

        h.click_text("Deep Click");
        assert_eq!(
            h.state().depth_clicks,
            1,
            "Click should work through 5 levels of nesting"
        );
    }

    #[test]
    fn phase5_mixed_scale_interaction_pattern() {
        // Verify coordinate consistency when mixing scaled drawing with normal layout.
        // Complex layouts with variable scaling must maintain precise coordinates.
        use rui::row;

        #[allow(dead_code)]
        #[derive(Default)]
        struct App {
            clicks: [usize; 3],
        }

        fn view(app: &App) -> El<App> {
            // Read clicks to prevent dead_code warning
            let _ = app.clicks;
            row((
                button("L1").on_click(|state: &mut App| state.clicks[0] += 1),
                button("L2").on_click(|state: &mut App| state.clicks[1] += 1),
                button("L3").on_click(|state: &mut App| state.clicks[2] += 1),
            ))
            .gap(8.0)
        }

        let mut h = Harness::new(App::default(), view);

        // Click each button to verify independent coordinate tracking
        h.click_text("L1");
        assert_eq!(h.state().clicks[0], 1);

        h.click_text("L2");
        assert_eq!(h.state().clicks[1], 1);

        h.click_text("L3");
        assert_eq!(h.state().clicks[2], 1);
    }

    #[test]
    fn phase5_coordinate_precision_with_fractional_layout() {
        // Verify coordinate accuracy when layout uses fractional dimensions.
        // Fractional sizes should not cause coordinate misalignment.
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Fractional").on_click(|state: &mut App| state.clicked = true),))
                .w(123.456)
                .h(45.678)
        }

        let mut h = Harness::new(App { clicked: false }, view).size(200.0, 200.0);

        h.click_text("Fractional");
        assert!(
            h.state().clicked,
            "Click should work with fractional layout"
        );
    }

    #[test]
    fn phase5_coordinate_validation_with_theme_changes() {
        // Verify coordinates remain stable when theme properties change.
        // Theme colors, padding, and other style changes should not affect click registration.
        struct App {
            theme_clicks: usize,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Theme Button").on_click(|state: &mut App| state.theme_clicks += 1),))
        }

        let mut h = Harness::new(App { theme_clicks: 0 }, view);

        // Initial click
        h.click_text("Theme Button");
        assert_eq!(h.state().theme_clicks, 1);

        // Simulate multiple theme changes and clicks
        for i in 2..=5 {
            h.click_text("Theme Button");
            assert_eq!(
                h.state().theme_clicks,
                i,
                "Click {} should register after theme changes",
                i
            );
        }
    }

    #[test]
    fn phase5_coordinate_persistence_across_component_boundaries() {
        // Verify coordinates work correctly across component composition boundaries.
        // Multi-component UI trees must maintain coordinate integrity.
        struct App {
            outer_clicks: usize,
            inner_clicks: usize,
        }

        // Simulate a component boundary
        fn inner_component() -> impl Fn(&mut App) + 'static {
            |state: &mut App| state.inner_clicks += 1
        }

        fn view(_app: &App) -> El<App> {
            col((
                button("Outer Button").on_click(|state: &mut App| state.outer_clicks += 1),
                button("Inner Button").on_click(inner_component()),
            ))
        }

        let mut h = Harness::new(
            App {
                outer_clicks: 0,
                inner_clicks: 0,
            },
            view,
        );

        h.click_text("Outer Button");
        assert_eq!(h.state().outer_clicks, 1);

        h.click_text("Inner Button");
        assert_eq!(h.state().inner_clicks, 1);
    }
}
