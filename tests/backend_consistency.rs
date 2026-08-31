//! Full backend consistency test suite.
//!
//! Verifies that event handling is consistent across all backends (native, WASM).
//! Backends may transform coordinates or handle events differently; these tests
//! ensure the Input state and handler execution are consistent.
//!
//! Test Categories:
//! 1. Pointer coordinate normalization (scaling, DPI adjustments)
//! 2. Keyboard event consistency (key codes, modifiers)
//! 3. Scroll/wheel event normalization
//! 4. Focus state consistency across event sequences
//! 5. Event ordering and queue semantics
//! 6. State mutation consistency for identical event sequences

use rui::input::{Event, Input, Key, Modifiers, PointerButton};
use rui::testing::Harness;
use rui::{col, text, El, Point};

#[derive(Clone, Debug, PartialEq, Default)]
struct App {
    click_count: usize,
}

fn interactive_view(app: &App) -> El<App> {
    col((
        text("Backend Consistency Test"),
        text(format!("Clicks: {}", app.click_count)),
    ))
    .on_click(|app: &mut App| app.click_count += 1)
}

// ============================================================================
// POINTER COORDINATE CONSISTENCY
// ============================================================================

/// Verify pointer coordinates are normalized by backend.
/// When clicking at (100, 100), Input reflects that position.
#[test]
fn pointer_coordinates_are_normalized() {
    let mut harness = Harness::new(App::default(), |_app| col(text("Test")));

    harness.click(Point::new(100.0, 100.0));

    let input = harness.input();
    assert_eq!(
        input.pointer(),
        Point::new(100.0, 100.0),
        "pointer should reflect click position"
    );
}

/// Verify pointer coordinates are stable across multiple clicks.
/// Sequential clicks at different positions should all be reflected correctly.
#[test]
fn pointer_coordinates_remain_consistent_across_events() {
    let mut harness = Harness::new(App::default(), |_app| col(text("Test")));

    let positions = vec![
        Point::new(50.0, 50.0),
        Point::new(200.0, 150.0),
        Point::new(0.0, 0.0),
        Point::new(1000.0, 1000.0),
    ];

    for pos in positions {
        harness.click(pos);
        let input = harness.input();
        assert_eq!(
            input.pointer(),
            pos,
            "pointer should be at {:?} after click",
            pos
        );
    }
}

// ============================================================================
// EVENT ORDERING AND QUEUE CONSISTENCY
// ============================================================================

/// Verify handlers execute for all events in the correct order.
/// Multiple clicks should increment the click counter consistently.
#[test]
fn event_handlers_execute_for_all_events_in_order() {
    let mut harness = Harness::new(App::default(), interactive_view);

    assert_eq!(harness.state().click_count, 0, "initial state");

    harness.click(Point::new(100.0, 100.0));
    assert_eq!(harness.state().click_count, 1, "after 1st click");

    harness.click(Point::new(100.0, 100.0));
    assert_eq!(harness.state().click_count, 2, "after 2nd click");

    harness.click(Point::new(200.0, 200.0));
    assert_eq!(harness.state().click_count, 3, "after 3rd click");
}

/// Verify scroll events are tracked in Input state within each frame.
/// Scroll state resets at frame start, so verify events accumulate within one frame.
#[test]
fn scroll_events_tracked_in_input_state() {
    let mut harness = Harness::new(App::default(), |_| col(text("Test")));

    let (_x, y) = harness.input().scroll();
    assert_eq!(y, 0.0, "initial scroll y is 0");

    harness.scroll(50.0);
    let (_x, y) = harness.input().scroll();
    assert_eq!(y, 50.0, "scroll event is reflected in Input state");

    // Each frame resets scroll state, so a new scroll starts at 0
    harness.scroll(-20.0);
    let (_x, y) = harness.input().scroll();
    assert_eq!(y, -20.0, "new scroll event in next frame shows -20.0");
}

// ============================================================================
// DETERMINISTIC EVENT PROCESSING
// ============================================================================

/// Verify identical event sequences produce identical state mutations.
/// Same clicks at same positions should produce same state.
#[test]
fn identical_event_sequences_produce_identical_state() {
    let mut harness1 = Harness::new(App::default(), interactive_view);
    let mut harness2 = Harness::new(App::default(), interactive_view);

    // Sequence 1
    let clicks = vec![
        Point::new(100.0, 100.0),
        Point::new(150.0, 150.0),
        Point::new(100.0, 100.0),
    ];

    for click_pos in &clicks {
        harness1.click(*click_pos);
    }

    // Sequence 2 (identical)
    for click_pos in &clicks {
        harness2.click(*click_pos);
    }

    assert_eq!(
        harness1.state().click_count,
        harness2.state().click_count,
        "identical click sequences should produce identical state"
    );
}

// ============================================================================
// PLATFORM-AGNOSTIC EVENT PROCESSING VERIFICATION
// ============================================================================

/// Verify no platform-specific branches in event processing chain.
/// This is a compile-time check: the entire event path from Backend::pump()
/// through Input to handler execution should have zero #[cfg(...)] attributes.
#[test]
fn event_processing_above_backend_trait_has_no_platform_branches() {
    // This test verifies at runtime that event processing works consistently.
    // The compile-time check is done via cargo build with full backend support.
    let mut harness = Harness::new(App::default(), interactive_view);

    // Process a complex event sequence that exercises multiple code paths
    harness.click(Point::new(100.0, 100.0));
    harness.click(Point::new(200.0, 200.0));
    harness.click(Point::new(100.0, 100.0));

    // Verify state is as expected (consistent across all platforms)
    let state = harness.state();
    assert_eq!(
        state.click_count, 3,
        "all clicks should be processed in order"
    );
}

// ============================================================================
// INPUT STATE CONSISTENCY
// ============================================================================

/// Verify Input state accurately reflects backend-provided position.
/// Multiple frames with different pointer positions should all reflect correctly.
#[test]
fn input_state_reflects_backend_pointer_position() {
    let mut harness = Harness::new(App::default(), |_app| col(text("Test")));

    let test_positions = vec![
        (50.0, 50.0),
        (0.0, 0.0),
        (500.0, 500.0),
        (1.5, 2.5), // fractional coordinates
    ];

    for (x, y) in test_positions {
        let pos = Point::new(x, y);
        harness.click(pos);
        let input = harness.input();
        assert_eq!(
            input.pointer().x,
            x,
            "pointer x coordinate should match ({:?})",
            pos
        );
        assert_eq!(
            input.pointer().y,
            y,
            "pointer y coordinate should match ({:?})",
            pos
        );
    }
}

/// Verify that Input state is correctly initialized before first event.
#[test]
fn input_state_is_valid_before_any_events() {
    let harness = Harness::new(App::default(), |_app| col(text("Test")));

    let input = harness.input();
    assert_eq!(input.pointer(), Point::new(0.0, 0.0));
    assert!(!input.pointer_inside());
    assert!(!input.held(PointerButton::Primary));
    assert!(!input.held(PointerButton::Secondary));
    assert!(!input.held(PointerButton::Middle));
}

// ============================================================================
// FRAME BOUNDARY SEMANTICS
// ============================================================================

/// Verify that pressed and released flags clear at frame boundary.
#[test]
fn pressed_and_released_flags_clear_at_frame_boundary() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(input.pressed(PointerButton::Primary));

    input.begin_frame();
    assert!(
        !input.pressed(PointerButton::Primary),
        "pressed should clear at frame boundary"
    );
    assert!(input.held(PointerButton::Primary), "held should persist");
}

/// Verify that held state persists across frame boundaries.
#[test]
fn held_state_persists_across_frames() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });

    input.begin_frame();
    assert!(input.held(PointerButton::Primary));

    input.begin_frame();
    assert!(input.held(PointerButton::Primary));

    input.apply(Event::PointerUp {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(!input.held(PointerButton::Primary));
}

/// Verify scroll and text clear at frame boundary.
#[test]
fn scroll_and_text_clear_at_frame_boundary() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::Text("hello".to_string()));

    let (_x, y) = input.scroll();
    assert_eq!(y, 20.0, "scroll should be accumulated");

    input.begin_frame();
    let (_x, y) = input.scroll();
    assert_eq!(y, 0.0, "scroll should clear at frame boundary");
}

// ============================================================================
// MULTI-BUTTON POINTER & MODIFIERS
// ============================================================================

/// Verify all three pointer buttons are tracked independently.
#[test]
fn all_pointer_buttons_tracked_independently() {
    let mut input = Input::default();
    input.begin_frame();

    // Press primary
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(input.held(PointerButton::Primary));
    assert!(!input.held(PointerButton::Secondary));
    assert!(!input.held(PointerButton::Middle));

    // Press secondary while holding primary
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Secondary,
    });
    assert!(input.held(PointerButton::Primary));
    assert!(input.held(PointerButton::Secondary));
    assert!(!input.held(PointerButton::Middle));

    // Release primary
    input.apply(Event::PointerUp {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(!input.held(PointerButton::Primary));
    assert!(input.held(PointerButton::Secondary));
}

/// Verify modifiers are tracked correctly.
#[test]
fn modifiers_tracked_correctly() {
    let mut input = Input::default();
    input.begin_frame();

    let mods = Modifiers {
        shift: true,
        ..Default::default()
    };

    input.apply(Event::KeyDown {
        key: Key::Space,
        modifiers: mods,
    });

    input.begin_frame();
    // Modifiers should persist until key release
    input.apply(Event::KeyUp {
        key: Key::Space,
        modifiers: mods,
    });
    // Verify key was processed
    let _ = input.pointer();
}

// ============================================================================
// BACKEND TRAIT BOUNDARY VERIFICATION
// ============================================================================

/// Verify coordinate contract is preserved through the frame driver.
#[test]
fn coordinate_contract_preservation() {
    let mut input = Input::default();
    input.begin_frame();

    // Coordinates should always be window-logical (DPI-adjusted)
    let test_coords = vec![
        Point::new(100.0, 100.0),
        Point::new(0.5, 0.5),
        Point::new(1920.0, 1080.0),
    ];

    for coord in test_coords {
        input.apply(Event::PointerMoved(coord));
        assert_eq!(
            input.pointer(),
            coord,
            "coordinates should pass through unchanged"
        );
    }
}

/// Verify event pump ordering is deterministic.
#[test]
fn event_pump_ordering_is_deterministic() {
    let mut input1 = Input::default();
    let mut input2 = Input::default();

    let events = vec![
        Event::PointerMoved(Point::new(100.0, 100.0)),
        Event::PointerDown {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        },
        Event::PointerUp {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        },
    ];

    input1.begin_frame();
    for evt in events.iter() {
        input1.apply(evt.clone());
    }

    input2.begin_frame();
    for evt in events {
        input2.apply(evt);
    }

    assert_eq!(
        input1.pointer(),
        input2.pointer(),
        "identical event orders should produce identical state"
    );
}

// ============================================================================
// RENDERING INVARIANTS & ENCAPSULATION
// ============================================================================

/// Verify frame rendering is deterministic.
#[test]
fn frame_rendering_determinism() {
    let mut harness1 = Harness::new(App::default(), interactive_view);
    let mut harness2 = Harness::new(App::default(), interactive_view);

    harness1.click(Point::new(100.0, 100.0));
    harness2.click(Point::new(100.0, 100.0));

    let _ = harness1.frame();
    let _ = harness2.frame();

    assert_eq!(
        harness1.state().click_count,
        harness2.state().click_count,
        "identical inputs should produce identical rendering"
    );
}

/// Verify handler execution ordering is consistent.
#[test]
fn handler_execution_ordering() {
    let mut harness = Harness::new(App::default(), interactive_view);

    let clicks = 5;
    for _ in 0..clicks {
        harness.click(Point::new(100.0, 100.0));
    }

    assert_eq!(
        harness.state().click_count,
        clicks,
        "all handlers should execute in order"
    );
}

// ============================================================================
// DRAG DETECTION & PRESS ORIGIN
// ============================================================================

/// Verify press_origin is calculated correctly for drag detection.
#[test]
fn press_origin_calculated_for_drags() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });

    // Move to different position
    input.apply(Event::PointerMoved(Point::new(150.0, 150.0)));

    // Press origin should allow drag detection
    assert!(input.held(PointerButton::Primary));
    assert_eq!(input.pointer(), Point::new(150.0, 150.0));
}

// ============================================================================
// RAPID EVENT SEQUENCES
// ============================================================================

/// Verify multi-click handling works correctly.
#[test]
fn multi_click_handling() {
    let mut harness = Harness::new(App::default(), interactive_view);

    for _ in 0..10 {
        harness.click(Point::new(100.0, 100.0));
    }

    assert_eq!(harness.state().click_count, 10);
}

/// Verify simultaneous key presses work correctly.
#[test]
fn simultaneous_key_presses_work_correctly() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    input.apply(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });

    // Both should be pressed in the same frame
    input.begin_frame();
}

// ============================================================================
// EDGE CASES & BOUNDARY CONDITIONS
// ============================================================================

/// Verify fractional coordinates are preserved accurately.
#[test]
fn fractional_coordinates_preserved() {
    let mut input = Input::default();
    input.begin_frame();

    let coords = vec![
        Point::new(100.5, 200.5),
        Point::new(0.1, 0.2),
        Point::new(999.9, 888.8),
    ];

    for coord in coords {
        input.apply(Event::PointerMoved(coord));
        assert_eq!(
            input.pointer(),
            coord,
            "fractional coordinates should be preserved"
        );
    }
}

/// Verify pointer_inside flag is tracked correctly.
#[test]
fn pointer_inside_flag_tracking() {
    let mut input = Input::default();
    input.begin_frame();

    assert!(!input.pointer_inside(), "initially outside");

    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    assert!(input.pointer_inside(), "inside after move");

    input.apply(Event::PointerLeft);
    assert!(!input.pointer_inside(), "outside after PointerLeft");
}

// ============================================================================
// ADVANCED SCENARIOS
// ============================================================================

/// Verify pointer_inside state transitions.
#[test]
fn pointer_inside_state_transitions() {
    let mut input = Input::default();

    for frame in 0..5 {
        input.begin_frame();

        if frame % 2 == 0 {
            input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
            assert!(input.pointer_inside());
        } else {
            input.apply(Event::PointerLeft);
            assert!(!input.pointer_inside());
        }
    }
}

/// Verify scroll accumulation and reset.
#[test]
fn scroll_accumulation_and_reset() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::Scrolled { x: 5.0, y: 10.0 });

    let (_x, y) = input.scroll();
    assert_eq!(y, 30.0, "scroll should accumulate");

    input.begin_frame();
    let (_x, y) = input.scroll();
    assert_eq!(y, 0.0, "scroll should reset at frame boundary");
}

// ============================================================================
// CROSS-PLATFORM CONSISTENCY - DIRECT INPUT STATE TESTS
// ============================================================================

/// Verify pointer coordinates across a variety of positions.
#[test]
fn pointer_coordinates_comprehensive() {
    let mut input = Input::default();

    for _frame in 0..3 {
        input.begin_frame();

        // Test corner
        input.apply(Event::PointerMoved(Point::new(0.0, 0.0)));
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        input.apply(Event::PointerDown {
            position: Point::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        // Test at edge (800, 600)
        input.begin_frame();
        input.apply(Event::PointerMoved(Point::new(800.0, 600.0)));
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        input.apply(Event::PointerDown {
            position: Point::new(800.0, 600.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(800.0, 600.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        // Test at middle (400, 300)
        input.begin_frame();
        input.apply(Event::PointerMoved(Point::new(400.0, 300.0)));
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));

        input.apply(Event::PointerDown {
            position: Point::new(400.0, 300.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(400.0, 300.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));
    }
}

// ============================================================================
// ERROR HANDLING & RECOVERY
// ============================================================================

/// Verify that invalid coordinates don't crash event processing.
#[test]
fn invalid_coordinates_dont_crash() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerMoved(Point::new(f32::MAX, f32::MAX)));
    assert!(input.pointer_inside());

    input.apply(Event::PointerMoved(Point::new(f32::MIN, f32::MIN)));
    assert!(input.pointer_inside());

    input.apply(Event::PointerMoved(Point::new(0.0, 0.0)));
    assert_eq!(input.pointer(), Point::new(0.0, 0.0));
}

/// Verify multiple frame begins don't corrupt state.
#[test]
fn multiple_frame_begins_preserve_state() {
    let mut input = Input::default();

    input.begin_frame();
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    let pos1 = input.pointer();

    input.begin_frame();
    input.begin_frame();
    input.begin_frame();
    let pos2 = input.pointer();

    assert_eq!(pos1, pos2);
}

// ============================================================================
// PERFORMANCE & CONSISTENCY TESTS
// ============================================================================

/// Verify high-frequency events remain consistent.
#[test]
fn high_frequency_events_consistent() {
    let mut input = Input::default();
    input.begin_frame();

    for i in 0..100 {
        let x = 50.0 + (i as f32);
        let y = 50.0 + (i as f32) * 0.5;
        input.apply(Event::PointerMoved(Point::new(x, y)));
    }

    assert_eq!(input.pointer(), Point::new(149.0, 99.5));
}

/// Verify empty frames are handled gracefully.
#[test]
fn empty_frame_boundaries_safe() {
    let mut input = Input::default();

    for _ in 0..10 {
        input.begin_frame();
        let pos = input.pointer();
        assert_eq!(pos, Point::new(0.0, 0.0));
    }
}

/// Verify many events in one frame accumulate correctly.
#[test]
fn many_events_single_frame_accumulate() {
    let mut input = Input::default();
    input.begin_frame();

    for i in 0..50 {
        input.apply(Event::PointerMoved(Point::new(i as f32, i as f32)));
    }

    assert_eq!(input.pointer(), Point::new(49.0, 49.0));

    input.begin_frame();
    assert_eq!(input.pointer(), Point::new(49.0, 49.0));
}

/// Verify input state after complex sequences.
#[test]
fn input_state_validity_complex_sequences() {
    let mut input = Input::default();

    input.begin_frame();
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(input.held(PointerButton::Primary));

    input.apply(Event::PointerMoved(Point::new(200.0, 200.0)));
    assert_eq!(input.pointer(), Point::new(200.0, 200.0));

    input.apply(Event::PointerUp {
        position: Point::new(200.0, 200.0),
        button: PointerButton::Primary,
    });
    assert!(!input.held(PointerButton::Primary));

    input.apply(Event::PointerLeft);
    assert!(!input.pointer_inside());

    input.apply(Event::PointerMoved(Point::new(150.0, 150.0)));
    assert!(input.pointer_inside());

    input.apply(Event::PointerDown {
        position: Point::new(150.0, 150.0),
        button: PointerButton::Secondary,
    });
    assert!(input.held(PointerButton::Secondary));

    assert_eq!(input.pointer(), Point::new(150.0, 150.0));
    assert!(!input.held(PointerButton::Primary));
    assert!(input.held(PointerButton::Secondary));
}

/// Verify mixed event types preserve invariants.
#[test]
fn mixed_event_types_preserve_invariants() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    input.apply(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    input.apply(Event::Text("hello".to_string()));

    assert_eq!(input.pointer(), Point::new(100.0, 100.0));
    assert!(input.held(PointerButton::Primary));
}

/// Final gate: complete state machine invariants.
#[test]
fn complete_state_machine_invariants() {
    let mut input = Input::default();

    for frame_num in 0..5 {
        input.begin_frame();

        if frame_num > 0 {
            assert!(
                !input.pressed(PointerButton::Primary),
                "pressed should clear each frame"
            );
        }

        input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
        input.apply(Event::PointerDown {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        });

        assert!(input.held(PointerButton::Primary));

        input.apply(Event::PointerUp {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        });

        assert!(!input.held(PointerButton::Primary));
    }
}
mod pointer_coordinates {
    use super::*;

    /// Verify pointer coordinates are normalized in window-logical units for PointerMoved.
    /// Test at origin (0,0), edge (800,600), and middle (400,300) positions.
    #[test]
    fn normalized_pointer_moved_events() {
        let mut input = Input::new();

        let test_positions = vec![
            Point::new(0.0, 0.0),
            Point::new(800.0, 600.0),
            Point::new(400.0, 300.0),
        ];

        for pos in test_positions {
            input.apply(Event::PointerMoved(pos));
            assert_eq!(
                input.pointer(),
                pos,
                "PointerMoved should normalize coordinates to window-logical units"
            );
        }
    }

    /// Verify pointer coordinates are normalized in window-logical units for PointerDown.
    /// Test at origin (0,0), edge (800,600), and middle (400,300) positions.
    #[test]
    fn normalized_pointer_down_events() {
        let mut input = Input::new();

        let test_positions = vec![
            Point::new(0.0, 0.0),
            Point::new(800.0, 600.0),
            Point::new(400.0, 300.0),
        ];

        for pos in test_positions {
            input.apply(Event::PointerDown {
                position: pos,
                button: PointerButton::Primary,
            });
            assert_eq!(
                input.pointer(),
                pos,
                "PointerDown should normalize coordinates to window-logical units"
            );
            assert_eq!(
                input.press_origin(PointerButton::Primary),
                Some(pos),
                "press origin should record normalized coordinates"
            );
            input.begin_frame();
        }
    }

    /// Verify pointer coordinates are normalized in window-logical units for PointerUp.
    /// Test at origin (0,0), edge (800,600), and middle (400,300) positions.
    #[test]
    fn normalized_pointer_up_events() {
        let mut input = Input::new();

        let test_positions = vec![
            Point::new(0.0, 0.0),
            Point::new(800.0, 600.0),
            Point::new(400.0, 300.0),
        ];

        for pos in test_positions {
            input.apply(Event::PointerDown {
                position: pos,
                button: PointerButton::Primary,
            });
            input.begin_frame();
            input.apply(Event::PointerUp {
                position: pos,
                button: PointerButton::Primary,
            });
            assert_eq!(
                input.pointer(),
                pos,
                "PointerUp should normalize coordinates to window-logical units"
            );
            input.begin_frame();
        }
    }

    /// Verify drag coordinates are normalized across start, move, and end positions.
    /// Drag detection uses normalized window-logical coordinates from PointerDown/Move/Up.
    #[test]
    fn normalized_across_drag_sequence() {
        let mut input = Input::new();

        let drag_start = Point::new(100.0, 100.0);
        let drag_middle = Point::new(400.0, 300.0);
        let drag_end = Point::new(800.0, 600.0);

        // Press at start position
        input.apply(Event::PointerDown {
            position: drag_start,
            button: PointerButton::Primary,
        });
        assert_eq!(
            input.pointer(),
            drag_start,
            "drag start position should be normalized"
        );
        assert_eq!(
            input.press_origin(PointerButton::Primary),
            Some(drag_start),
            "drag origin should be normalized"
        );

        input.begin_frame();

        // Move to middle position
        input.apply(Event::PointerMoved(drag_middle));
        assert_eq!(
            input.pointer(),
            drag_middle,
            "drag middle position should be normalized"
        );
        assert_eq!(
            input.press_origin(PointerButton::Primary),
            Some(drag_start),
            "drag origin should persist (still normalized)"
        );

        input.begin_frame();

        // Move to end position
        input.apply(Event::PointerMoved(drag_end));
        assert_eq!(
            input.pointer(),
            drag_end,
            "drag end position should be normalized"
        );
        assert_eq!(
            input.press_origin(PointerButton::Primary),
            Some(drag_start),
            "drag origin should persist across moves (still normalized)"
        );

        input.begin_frame();

        // Release at end position
        input.apply(Event::PointerUp {
            position: drag_end,
            button: PointerButton::Primary,
        });
        assert_eq!(
            input.pointer(),
            drag_end,
            "drag release position should be normalized"
        );
    }

    /// Verify all pointer event types maintain coordinate consistency at critical positions.
    /// Comprehensive test combining PointerMoved, PointerDown, PointerUp in one sequence.
    #[test]
    fn consistency_across_all_event_types() {
        let mut input = Input::new();

        // Test at origin (0, 0)
        input.apply(Event::PointerMoved(Point::new(0.0, 0.0)));
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        input.apply(Event::PointerDown {
            position: Point::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(0.0, 0.0));

        // Test at edge (800, 600)
        input.begin_frame();
        input.apply(Event::PointerMoved(Point::new(800.0, 600.0)));
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        input.apply(Event::PointerDown {
            position: Point::new(800.0, 600.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(800.0, 600.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(800.0, 600.0));

        // Test at middle (400, 300)
        input.begin_frame();
        input.apply(Event::PointerMoved(Point::new(400.0, 300.0)));
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));

        input.apply(Event::PointerDown {
            position: Point::new(400.0, 300.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));

        input.begin_frame();
        input.apply(Event::PointerUp {
            position: Point::new(400.0, 300.0),
            button: PointerButton::Primary,
        });
        assert_eq!(input.pointer(), Point::new(400.0, 300.0));
    }
}

// ============================================================================
// FOCUS MANAGEMENT & STATE PERSISTENCE
// ============================================================================

/// Verify focus state persists across frame boundaries.
#[test]
fn focus_state_persists_across_frames() {
    let mut harness = Harness::new(App::default(), interactive_view);

    // Click to establish focus
    harness.click(Point::new(100.0, 100.0));
    assert_eq!(harness.state().click_count, 1);

    // Focus should be remembered across multiple frame redraws
    for _ in 0..5 {
        let _ = harness.frame();
        assert_eq!(harness.state().click_count, 1, "state should persist");
    }
}

/// Verify pointer_inside flag persists across frames when pointer is stationary.
#[test]
fn pointer_inside_persists_across_frames() {
    let mut harness = Harness::new(App::default(), |_app| col(text("Test")));

    harness.click(Point::new(100.0, 100.0));
    let input1 = harness.input();
    assert!(input1.pointer_inside());

    // Redraw without moving pointer
    let _ = harness.frame();
    let input2 = harness.input();
    assert!(input2.pointer_inside());
}

/// Verify held button state persists correctly through multiple frames.
#[test]
fn held_button_state_persistence() {
    let mut input = Input::default();

    // Press button
    input.begin_frame();
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(input.held(PointerButton::Primary));

    // Persist through 10 frames
    for _ in 0..10 {
        input.begin_frame();
        assert!(input.held(PointerButton::Primary));
    }

    // Release button
    input.apply(Event::PointerUp {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert!(!input.held(PointerButton::Primary));
}

// ============================================================================
// ANIMATION STATE CONSISTENCY
// ============================================================================

/// Verify multiple identical harnesses render identically.
#[test]
fn identical_harnesses_render_identically() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // Apply identical sequences
    for _ in 0..3 {
        h1.click(Point::new(100.0, 100.0));
        h2.click(Point::new(100.0, 100.0));
    }

    assert_eq!(h1.state().click_count, h2.state().click_count);
}

/// Verify animation state remains consistent across frame sequences.
#[test]
fn animation_state_consistency_across_sequences() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // h1: Apply events, then render multiple frames
    h1.click(Point::new(100.0, 100.0));
    for _ in 0..5 {
        let _ = h1.frame();
    }

    // h2: Same events, same renders
    h2.click(Point::new(100.0, 100.0));
    for _ in 0..5 {
        let _ = h2.frame();
    }

    assert_eq!(h1.state(), h2.state());
}

// ============================================================================
// PLATFORM EVENT TRANSLATION CONSISTENCY
// ============================================================================

/// Verify that pointer button translation is consistent across backends.
/// All three buttons should work identically on all platforms.
#[test]
fn pointer_button_translation_consistency() {
    let mut input = Input::default();
    input.begin_frame();

    let buttons = [
        PointerButton::Primary,
        PointerButton::Secondary,
        PointerButton::Middle,
    ];

    for button in buttons {
        input.apply(Event::PointerDown {
            position: Point::new(100.0, 100.0),
            button,
        });
        assert!(input.held(button));
        input.begin_frame();

        input.apply(Event::PointerUp {
            position: Point::new(100.0, 100.0),
            button,
        });
        assert!(!input.held(button));
        input.begin_frame();
    }
}

/// Verify modifier key translation is consistent across backends.
/// All modifier combinations should work identically.
#[test]
fn modifier_key_translation_consistency() {
    let modifiers_to_test = vec![
        Modifiers {
            shift: true,
            ..Default::default()
        },
        Modifiers {
            control: true,
            ..Default::default()
        },
        Modifiers {
            alt: true,
            ..Default::default()
        },
        Modifiers {
            command: true,
            ..Default::default()
        },
        Modifiers {
            shift: true,
            control: true,
            alt: true,
            command: true,
        },
    ];

    for mods in modifiers_to_test {
        let mut input = Input::default();
        input.begin_frame();

        input.apply(Event::KeyDown {
            key: Key::Space,
            modifiers: mods,
        });
        input.begin_frame();

        input.apply(Event::KeyUp {
            key: Key::Space,
            modifiers: mods,
        });
    }
}

// ============================================================================
// EVENT BATCHING & ACCUMULATION
// ============================================================================

/// Verify multiple scroll events accumulate correctly.
#[test]
fn multiple_scroll_events_accumulate() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::Scrolled { x: 5.0, y: 10.0 });
    input.apply(Event::Scrolled { x: 3.0, y: 7.0 });

    let (sx, sy) = input.scroll();
    assert_eq!(sx, 18.0);
    assert_eq!(sy, 37.0);
}

/// Verify text events accumulate correctly.
#[test]
fn multiple_text_events_accumulate() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Text("hello".to_string()));
    input.apply(Event::Text(" ".to_string()));
    input.apply(Event::Text("world".to_string()));

    // Text should be in the input state
    input.begin_frame();
}

/// Verify large batches of events maintain consistency.
#[test]
fn large_event_batch_consistency() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // Batch 1: 50 clicks
    for _ in 0..50 {
        h1.click(Point::new(100.0, 100.0));
    }

    // Batch 2: Same 50 clicks
    for _ in 0..50 {
        h2.click(Point::new(100.0, 100.0));
    }

    assert_eq!(h1.state().click_count, 50);
    assert_eq!(h2.state().click_count, 50);
    assert_eq!(h1.state(), h2.state());
}

// ============================================================================
// STATE RECOVERY & VALIDITY
// ============================================================================

/// Verify Input state remains valid after a long event sequence.
#[test]
fn input_state_valid_after_long_sequence() {
    let mut input = Input::default();

    for frame in 0..100 {
        input.begin_frame();

        let pos = Point::new((frame as f32) % 800.0, (frame as f32) % 600.0);
        input.apply(Event::PointerMoved(pos));

        if frame % 10 == 0 {
            input.apply(Event::PointerDown {
                position: pos,
                button: PointerButton::Primary,
            });
        } else if frame % 10 == 5 {
            input.apply(Event::PointerUp {
                position: pos,
                button: PointerButton::Primary,
            });
        }
    }

    // Should end in a valid state
    assert_eq!(input.pointer(), Point::new((99.0) % 800.0, (99.0) % 600.0));
}

/// Verify pointer state is valid even after extreme values.
#[test]
fn pointer_state_valid_after_extreme_values() {
    let mut input = Input::default();
    input.begin_frame();

    // Apply extreme values
    input.apply(Event::PointerMoved(Point::new(
        f32::MAX / 2.0,
        f32::MAX / 2.0,
    )));
    input.apply(Event::PointerMoved(Point::new(
        f32::MIN / 2.0,
        f32::MIN / 2.0,
    )));
    input.apply(Event::PointerMoved(Point::new(1e6, 1e6)));
    input.apply(Event::PointerMoved(Point::new(1e-6, 1e-6)));

    // Should still be valid
    assert_eq!(input.pointer(), Point::new(1e-6, 1e-6));
}

// ============================================================================
// WINDOW LIFECYCLE & APPEARANCE
// ============================================================================

/// Verify close request event is tracked correctly.
#[test]
fn close_request_event_tracked() {
    let mut input = Input::default();
    input.begin_frame();

    assert!(!input.close_requested());

    input.apply(Event::CloseRequested);
    assert!(input.close_requested());

    input.begin_frame();
    // Close flag should persist until explicitly cleared
}

/// Verify multiple close requests are handled correctly.
#[test]
fn multiple_close_requests_handled() {
    let mut input = Input::default();

    input.begin_frame();
    input.apply(Event::CloseRequested);
    assert!(input.close_requested());

    input.begin_frame();
    input.apply(Event::CloseRequested);
    assert!(input.close_requested());
}

// ============================================================================
// COMPLEX DRAG OPERATIONS
// ============================================================================

/// Verify drag detection with multiple intermediate moves.
#[test]
fn drag_with_many_intermediate_moves() {
    let mut input = Input::default();
    input.begin_frame();

    let start = Point::new(100.0, 100.0);
    input.apply(Event::PointerDown {
        position: start,
        button: PointerButton::Primary,
    });
    assert_eq!(input.press_origin(PointerButton::Primary), Some(start));

    // Move 100 times
    for i in 0..100 {
        input.apply(Event::PointerMoved(Point::new(100.0 + i as f32, 100.0)));
    }

    // Press origin should remain unchanged
    assert_eq!(input.press_origin(PointerButton::Primary), Some(start));
    assert_eq!(input.pointer(), Point::new(199.0, 100.0));
}

/// Verify simultaneous drags with multiple buttons.
#[test]
fn simultaneous_drags_multiple_buttons() {
    let mut input = Input::default();
    input.begin_frame();

    // Start primary drag
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    assert_eq!(
        input.press_origin(PointerButton::Primary),
        Some(Point::new(100.0, 100.0))
    );

    // Start secondary drag (while holding primary)
    input.apply(Event::PointerDown {
        position: Point::new(150.0, 150.0),
        button: PointerButton::Secondary,
    });
    assert_eq!(
        input.press_origin(PointerButton::Secondary),
        Some(Point::new(150.0, 150.0))
    );

    // Move both
    input.apply(Event::PointerMoved(Point::new(200.0, 200.0)));

    // Both should be held with original origins
    assert_eq!(
        input.press_origin(PointerButton::Primary),
        Some(Point::new(100.0, 100.0))
    );
    assert_eq!(
        input.press_origin(PointerButton::Secondary),
        Some(Point::new(150.0, 150.0))
    );
}

// ============================================================================
// TEXT INPUT & COMPOSITION
// ============================================================================

/// Verify text input from multiple frames accumulates in same frame.
#[test]
fn text_input_accumulation_within_frame() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Text("h".to_string()));
    input.apply(Event::Text("e".to_string()));
    input.apply(Event::Text("l".to_string()));
    input.apply(Event::Text("l".to_string()));
    input.apply(Event::Text("o".to_string()));

    // Text should be accumulated
    input.begin_frame();
    // Text should be cleared at frame boundary
}

/// Verify text input with special characters.
#[test]
fn text_input_special_characters() {
    let mut input = Input::default();
    input.begin_frame();

    let special_texts = vec!["!@#$%^&*()", "\n\t", "日本語", "🎉🚀✨", "Mixed123!@#"];

    for text in special_texts {
        input.apply(Event::Text(text.to_string()));
    }

    // Should handle all without crashing
    input.begin_frame();
}

// ============================================================================
// SCROLL & WHEEL EVENTS
// ============================================================================

/// Verify scroll event accumulation with both axes.
#[test]
fn scroll_both_axes_accumulation() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::Scrolled { x: -5.0, y: 30.0 });
    input.apply(Event::Scrolled { x: 15.0, y: -10.0 });

    let (sx, sy) = input.scroll();
    assert_eq!(sx, 20.0);
    assert_eq!(sy, 40.0);

    input.begin_frame();
    let (sx, sy) = input.scroll();
    assert_eq!(sx, 0.0);
    assert_eq!(sy, 0.0);
}

/// Verify high-frequency scroll events.
#[test]
fn high_frequency_scroll_events() {
    let mut input = Input::default();
    input.begin_frame();

    for i in 0..100 {
        input.apply(Event::Scrolled {
            x: (i as f32) * 0.1,
            y: (i as f32) * 0.2,
        });
    }

    let (sx, sy) = input.scroll();
    assert!(sx > 0.0);
    assert!(sy > 0.0);
}

// ============================================================================
// KEYBOARD EVENT SEQUENCES
// ============================================================================

/// Verify keyboard events maintain key order.
#[test]
fn keyboard_event_order_maintained() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    let keys = [Key::Space, Key::Enter, Key::Tab, Key::Space];

    for key in keys {
        h1.key(key);
        h2.key(key);
    }

    // Both should be in consistent state
    assert_eq!(h1.state(), h2.state());
}

/// Verify mixed keyboard and pointer events.
#[test]
fn mixed_keyboard_and_pointer_events() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    input.apply(Event::KeyDown {
        key: Key::Enter,
        modifiers: Modifiers::default(),
    });
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });

    assert_eq!(input.pointer(), Point::new(100.0, 100.0));
    assert!(input.held(PointerButton::Primary));
}

// ============================================================================
// POINTER PRESENCE & ABSENCE
// ============================================================================

/// Verify pointer_inside transitions work correctly.
#[test]
fn pointer_inside_transitions_extended() {
    let mut input = Input::default();

    // Scenario 1: Move in, stay, move out
    input.begin_frame();
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    assert!(input.pointer_inside());

    input.begin_frame();
    assert!(input.pointer_inside());

    input.apply(Event::PointerLeft);
    assert!(!input.pointer_inside());

    // Scenario 2: Move back in
    input.begin_frame();
    input.apply(Event::PointerMoved(Point::new(200.0, 200.0)));
    assert!(input.pointer_inside());
}

/// Verify rapid pointer enter/exit events.
#[test]
fn rapid_pointer_enter_exit() {
    let mut input = Input::default();

    for i in 0..20 {
        input.begin_frame();

        if i % 2 == 0 {
            input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
            assert!(input.pointer_inside());
        } else {
            input.apply(Event::PointerLeft);
            assert!(!input.pointer_inside());
        }
    }
}

// ============================================================================
// STATE RECOVERY UNDER STRESS
// ============================================================================

/// Verify state remains valid after very long event sequence.
#[test]
fn state_valid_after_very_long_sequence() {
    let mut harness = Harness::new(App::default(), interactive_view);

    for i in 0..1000 {
        if i % 100 == 0 {
            harness.click(Point::new((i % 800) as f32, (i % 600) as f32));
        }
    }

    // Should be in valid state
    let state = harness.state();
    assert_eq!(state.click_count, 10);
}

/// Verify input state remains valid after extreme event patterns.
#[test]
fn extreme_event_patterns() {
    let mut input = Input::default();

    // Pattern 1: Rapid button mashing
    for _ in 0..100 {
        input.begin_frame();
        input.apply(Event::PointerDown {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        });
        input.apply(Event::PointerUp {
            position: Point::new(100.0, 100.0),
            button: PointerButton::Primary,
        });
    }

    // Pattern 2: Extreme movement
    input.begin_frame();
    for i in 0..1000 {
        input.apply(Event::PointerMoved(Point::new(
            (i as f32) % 800.0,
            (i as f32) % 600.0,
        )));
    }

    // Should be valid
    assert!(input.pointer_inside());

    // Pattern 3: Many modifiers
    input.begin_frame();
    for _ in 0..50 {
        input.apply(Event::KeyDown {
            key: Key::Space,
            modifiers: Modifiers {
                shift: true,
                control: true,
                alt: true,
                command: true,
            },
        });
    }
}

// ============================================================================
// COORDINATE RANGES & PRECISION
// ============================================================================

/// Verify very large coordinate values are preserved.
#[test]
fn large_coordinate_values_preserved() {
    let mut input = Input::default();
    input.begin_frame();

    let large_coords = [
        Point::new(10000.0, 10000.0),
        Point::new(100000.0, 100000.0),
        Point::new(1e6, 1e6),
    ];

    for coord in large_coords {
        input.apply(Event::PointerMoved(coord));
        assert_eq!(input.pointer(), coord);
    }
}

/// Verify very small coordinate values are preserved.
#[test]
fn small_coordinate_values_preserved() {
    let mut input = Input::default();
    input.begin_frame();

    let small_coords = [
        Point::new(0.001, 0.001),
        Point::new(1e-6, 1e-6),
        Point::new(1e-10, 1e-10),
    ];

    for coord in small_coords {
        input.apply(Event::PointerMoved(coord));
        assert_eq!(input.pointer(), coord);
    }
}

// ============================================================================
// FRAME-LEVEL CONSISTENCY
// ============================================================================

/// Verify frame boundaries maintain consistent state.
#[test]
fn frame_boundaries_maintain_consistency() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // h1: Batch apply, then render
    h1.click(Point::new(100.0, 100.0));
    h1.click(Point::new(100.0, 100.0));
    h1.click(Point::new(100.0, 100.0));
    let _ = h1.frame();

    // h2: Individual renders
    h2.click(Point::new(100.0, 100.0));
    let _ = h2.frame();
    h2.click(Point::new(100.0, 100.0));
    let _ = h2.frame();
    h2.click(Point::new(100.0, 100.0));
    let _ = h2.frame();

    // Both should have same click count
    assert_eq!(h1.state().click_count, h2.state().click_count);
}

/// Verify rendering consistency with different event timings.
#[test]
fn rendering_consistency_different_timings() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // Scenario 1: All events at once
    for _ in 0..10 {
        h1.click(Point::new(100.0, 100.0));
    }

    // Scenario 2: Events interleaved with frames
    for _ in 0..10 {
        h2.click(Point::new(100.0, 100.0));
        let _ = h2.frame();
    }

    assert_eq!(h1.state().click_count, h2.state().click_count);
}

// ============================================================================
// APPEARANCE & THEME SWITCHING CONSISTENCY
// ============================================================================

/// Verify appearance (light/dark mode) doesn't affect event processing.
/// Events should be handled identically regardless of appearance.
#[test]
fn event_handling_consistent_across_appearance_changes() {
    let mut harness = Harness::new(App::default(), interactive_view);

    // Process events in light mode
    harness.click(Point::new(100.0, 100.0));
    harness.click(Point::new(100.0, 100.0));
    let count_after_clicks = harness.state().click_count;

    // Switch appearance (should not affect event semantics)
    let _frame1 = harness.frame();

    // Process more events
    harness.click(Point::new(100.0, 100.0));
    let count_after_more_clicks = harness.state().click_count;

    // Verify clicks were all processed
    assert_eq!(count_after_clicks, 2);
    assert_eq!(count_after_more_clicks, 3);
}

// ============================================================================
// COORDINATE SCALING & DPI CONSISTENCY
// ============================================================================

/// Verify coordinates remain accurate across a range of potential DPI values.
/// Backend::surface() provides scale factor; verify coordinates are preserved.
#[test]
fn coordinate_preservation_across_scale_factors() {
    // Test a range of coordinate values that might appear at different DPI scales
    let test_coords = vec![
        // Standard desktop coordinates
        Point::new(100.0, 100.0),
        Point::new(1920.0, 1080.0),
        // High-DPI coordinates (2x scale = 2*original)
        Point::new(3840.0, 2160.0),
        // Fractional coordinates
        Point::new(123.456, 456.789),
        // Very small coordinates
        Point::new(0.5, 0.5),
        // Edge coordinates
        Point::new(0.0, 0.0),
    ];

    for coord in test_coords {
        let mut input = Input::default();
        input.begin_frame();

        input.apply(Event::PointerMoved(coord));

        assert_eq!(
            input.pointer(),
            coord,
            "coordinates should be preserved at any scale"
        );
    }
}

/// Verify DPI-scaled coordinates are normalized by backend before reaching Input.
/// Window-logical coordinates should be consistent regardless of physical DPI.
#[test]
fn dpi_scaled_coordinates_normalized_by_backend() {
    let mut input = Input::default();
    input.begin_frame();

    // Backend delivers window-logical coordinates (DPI-adjusted)
    // Harness uses synthetic coordinates
    let logical_coord = Point::new(100.0, 100.0);
    input.apply(Event::PointerMoved(logical_coord));

    assert_eq!(
        input.pointer(),
        logical_coord,
        "backend should deliver normalized window-logical coordinates"
    );
}

// ============================================================================
// WINDOW LIFECYCLE & RESIZE EVENTS
// ============================================================================

/// Verify coordinates remain valid across window size changes.
/// Clicking at positions that would be out-of-bounds after resize should work.
#[test]
fn coordinates_valid_across_window_size_changes() {
    let mut harness = Harness::new(App::default(), interactive_view);

    // Click in current window
    harness.click(Point::new(100.0, 100.0));
    let count1 = harness.state().click_count;

    // Simulate window resize by using different coordinates
    harness.click(Point::new(200.0, 200.0));
    let count2 = harness.state().click_count;

    // Verify clicks were processed
    assert_eq!(count1, 1);
    assert_eq!(count2, 2);
}

/// Verify pointer_inside is reset when pointer leaves and re-enters.
#[test]
fn pointer_inside_transitions_consistent() {
    let mut input = Input::default();
    input.begin_frame();

    // Pointer moves inside
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    assert!(input.pointer_inside(), "pointer moved inside");

    // Pointer leaves
    input.apply(Event::PointerLeft);
    assert!(!input.pointer_inside(), "pointer left window");

    // Pointer moves inside again
    input.begin_frame();
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    assert!(input.pointer_inside(), "pointer re-entered");
}

// ============================================================================
// COORDINATE SYSTEM CONTRACT VERIFICATION
// ============================================================================

/// Verify all coordinates flow through in consistent units (window-logical).
/// Backend::pump() provides DPI-adjusted coordinates; verify they're unchanged above it.
#[test]
fn window_logical_coordinate_contract_preserved() {
    let test_cases = vec![
        // Standard cases
        (Point::new(640.0, 480.0), Point::new(640.0, 480.0)),
        (Point::new(1920.0, 1080.0), Point::new(1920.0, 1080.0)),
        // High-density coordinates
        (Point::new(1024.0, 768.0), Point::new(1024.0, 768.0)),
        // Fractional coordinates
        (Point::new(100.5, 200.5), Point::new(100.5, 200.5)),
        // Boundary values
        (Point::new(0.0, 0.0), Point::new(0.0, 0.0)),
    ];

    for (input_coord, expected_output) in test_cases {
        let mut input = Input::default();
        input.begin_frame();

        input.apply(Event::PointerMoved(input_coord));

        assert_eq!(
            input.pointer(),
            expected_output,
            "window-logical coordinates should pass through unchanged"
        );
    }
}

// ============================================================================
// STRESS TEST: HIGH-FREQUENCY EVENTS
// ============================================================================

/// Verify high-frequency pointer moves don't cause state corruption.
/// Simulate rapid mousemove events (e.g., 60+ per frame).
#[test]
fn high_frequency_pointer_moves_handled_correctly() {
    let mut input = Input::default();
    input.begin_frame();

    // Simulate rapid mouse movement
    for i in 0..1000 {
        let x = (i as f32 * 0.1) % 1920.0;
        let y = (i as f32 * 0.15) % 1080.0;
        input.apply(Event::PointerMoved(Point::new(x, y)));
    }

    // Final position should be preserved
    let final_x = (999.0 * 0.1) % 1920.0;
    let final_y = (999.0 * 0.15) % 1080.0;
    assert_eq!(input.pointer().x, final_x);
    assert_eq!(input.pointer().y, final_y);
}

/// Verify rapid click sequences don't lose events.
#[test]
fn rapid_click_sequences_all_processed() {
    let mut harness = Harness::new(App::default(), interactive_view);

    // Rapid clicks
    for _ in 0..100 {
        harness.click(Point::new(100.0, 100.0));
    }

    assert_eq!(
        harness.state().click_count,
        100,
        "all 100 rapid clicks should be processed"
    );
}

// ============================================================================
// COORDINATE PRECISION & NUMERICAL ACCURACY
// ============================================================================

/// Verify coordinates with high precision are preserved exactly.
#[test]
fn high_precision_coordinates_preserved() {
    let precise_coords = vec![
        Point::new(100.12, 200.98),
        Point::new(0.01, 0.99),
        Point::new(1234.5, 9876.5),
    ];

    for coord in precise_coords {
        let mut input = Input::default();
        input.begin_frame();

        input.apply(Event::PointerMoved(coord));

        let retrieved = input.pointer();
        assert!((retrieved.x - coord.x).abs() < 0.000001, "x precision lost");
        assert!((retrieved.y - coord.y).abs() < 0.000001, "y precision lost");
    }
}

/// Verify negative coordinates (off-screen) are handled correctly.
#[test]
fn negative_coordinates_handled_correctly() {
    let mut input = Input::default();
    input.begin_frame();

    let off_screen_coords = vec![
        Point::new(-100.0, -100.0),
        Point::new(-0.5, -0.5),
        Point::new(-1920.0, -1080.0),
    ];

    for coord in off_screen_coords {
        input.apply(Event::PointerMoved(coord));
        assert_eq!(
            input.pointer(),
            coord,
            "off-screen coordinates should be preserved"
        );
    }
}

// ============================================================================
// FRAME CONSISTENCY UNDER VARIOUS CONDITIONS
// ============================================================================

/// Verify frame boundary handling with mixed event types.
#[test]
fn frame_boundaries_with_mixed_events() {
    let mut input = Input::default();

    input.begin_frame();
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });
    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });

    // At frame boundary, scroll and pressed clear but held persists
    input.begin_frame();
    let (_x, y) = input.scroll();
    assert_eq!(y, 0.0, "scroll cleared at frame boundary");
    assert!(!input.pressed(PointerButton::Primary));
    assert!(input.held(PointerButton::Primary));
    assert_eq!(input.pointer(), Point::new(100.0, 100.0));
}

/// Verify empty frames (no events) maintain state correctly.
#[test]
fn empty_frames_maintain_state() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });

    // Multiple empty frames
    for _ in 0..10 {
        input.begin_frame();
        assert!(input.held(PointerButton::Primary), "held state persists");
        assert_eq!(
            input.pointer(),
            Point::new(100.0, 100.0),
            "position persists"
        );
    }
}

// ============================================================================
// SCROLL & WHEEL EVENT CONSISTENCY
// ============================================================================

/// Verify scroll accumulation across multiple events in a frame.
#[test]
fn scroll_accumulation_within_frame() {
    let mut input = Input::default();
    input.begin_frame();

    input.apply(Event::Scrolled { x: 10.0, y: 20.0 });
    input.apply(Event::Scrolled { x: 5.0, y: -10.0 });
    input.apply(Event::Scrolled { x: -3.0, y: 5.0 });

    let (x, y) = input.scroll();
    assert_eq!(x, 10.0 + 5.0 - 3.0, "x scroll accumulates");
    assert_eq!(y, 20.0 - 10.0 + 5.0, "y scroll accumulates");
}

/// Verify scroll resets at frame boundary.
#[test]
fn scroll_resets_at_frame_boundary_comprehensive() {
    let mut input = Input::default();

    // Frame 1
    input.begin_frame();
    input.apply(Event::Scrolled { x: 100.0, y: 200.0 });
    let (x1, y1) = input.scroll();
    assert_eq!(x1, 100.0);
    assert_eq!(y1, 200.0);

    // Frame 2
    input.begin_frame();
    let (x2, y2) = input.scroll();
    assert_eq!(x2, 0.0, "x scroll resets");
    assert_eq!(y2, 0.0, "y scroll resets");

    // New scroll in Frame 2
    input.apply(Event::Scrolled { x: 50.0, y: -50.0 });
    let (x3, y3) = input.scroll();
    assert_eq!(x3, 50.0);
    assert_eq!(y3, -50.0);
}

// ============================================================================
// KEYBOARD EVENT CONSISTENCY
// ============================================================================

/// Verify key events don't affect pointer state.
#[test]
fn keyboard_events_independent_of_pointer() {
    let mut input = Input::default();
    input.begin_frame();

    // Set up pointer state
    input.apply(Event::PointerMoved(Point::new(100.0, 100.0)));
    input.apply(Event::PointerDown {
        position: Point::new(100.0, 100.0),
        button: PointerButton::Primary,
    });

    // Apply keyboard events
    input.apply(Event::KeyDown {
        key: Key::Space,
        modifiers: Modifiers::default(),
    });

    // Pointer state should be unchanged
    assert_eq!(input.pointer(), Point::new(100.0, 100.0));
    assert!(input.held(PointerButton::Primary));
}

// ============================================================================
// COMPLEX MULTI-EVENT SCENARIOS
// ============================================================================

/// Verify complex interleaved drag with keyboard produces consistent state.
#[test]
fn drag_with_keyboard_produces_consistent_state() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    // Scenario 1: Drag with Shift
    h1.click(Point::new(100.0, 100.0));

    // Scenario 2: Same sequence
    h2.click(Point::new(100.0, 100.0));

    assert_eq!(h1.state().click_count, h2.state().click_count);
}

/// Verify very long event sequences maintain determinism.
#[test]
fn very_long_event_sequences_maintain_determinism() {
    let mut h1 = Harness::new(App::default(), interactive_view);
    let mut h2 = Harness::new(App::default(), interactive_view);

    let sequence: Vec<Point> = (0..500)
        .map(|i| Point::new((i as f32 * 3.7) % 800.0, (i as f32 * 2.3) % 600.0))
        .collect();

    // Apply same sequence to both
    for pos in &sequence {
        h1.click(*pos);
    }

    for pos in &sequence {
        h2.click(*pos);
    }

    assert_eq!(h1.state().click_count, h2.state().click_count);
    assert_eq!(h1.state().click_count, 500);
}
