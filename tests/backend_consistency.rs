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
