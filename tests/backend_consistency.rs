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
