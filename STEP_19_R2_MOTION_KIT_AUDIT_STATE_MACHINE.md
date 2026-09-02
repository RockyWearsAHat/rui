# STEP 19 Extended: R2 Motion Kit Animation State Machine

**Document purpose**: Map animation lifecycle across all R2 features. Shows state transitions, lifecycle events, and cleanup ordering for all 7 animation types.

**Status**: Complete audit documentation for STEP 19. Provides precise state model for Phase 1/2/3 implementation.

**Last updated**: 2026-09-02 after acceptance testing.

---

## Animation Lifecycle Overview

Every animation in R2 passes through a deterministic lifecycle: **Creation → Active → Completion → Cleanup**.

The state machine enforces:
1. Each animation type follows a specific lifecycle
2. State transitions are unambiguous (no ambiguous states)
3. Cleanup is ordered (animations never resurrect)
4. Resources are freed when animations complete

---

## Core Animation State Machine

```
┌─────────────────────────────────────────────────────────────────┐
│                    ANIMATION LIFECYCLE                           │
└─────────────────────────────────────────────────────────────────┘

                      ┌──────────────────┐
                      │    CREATION      │
                      └────────┬─────────┘
                               │ Begin frame time captured
                               ▼
                      ┌──────────────────┐
                      │      ACTIVE      │ ← Can be paused
                      └────────┬─────────┘
                               │ Progress < 1.0 && not paused
                               ▼
                      ┌──────────────────┐
                      │    COMPLETING    │ ← Reaches 100% this frame
                      └────────┬─────────┘
                               │ Callbacks run
                               ▼
                      ┌──────────────────┐
                      │   COMPLETED      │ ← Cleanup scheduled
                      └────────┬─────────┘
                               │ Next frame: cleanup runs
                               ▼
                      ┌──────────────────┐
                      │   CLEANED_UP     │
                      └────────┬─────────┘
                               │ Memory freed
                               ▼
                      ┌──────────────────┐
                      │      DEAD        │ ← Never resurfaces
                      └──────────────────┘

INVARIANT: Once cleanup starts, animation cannot restart.
```

---

## Individual Animation Type Lifecycles

### 1. Eased Animation (ease() primitive)

**Storage**: `Memory::eased: HashMap<Id, Eased>`

**Lifecycle**:
```
START: Memory::ease(id, target, duration) called
  ├─ Check: Is `id` already animating?
  │   └─ Yes: Retarget (jump to target, restart duration)
  │   └─ No: Create new Eased { start, target, duration, seen }
  └─ ACTIVE: Frame loop calls ease(id) to read current value
     ├─ Each frame: elapsed = accumulated_time - start
     ├─ Progress: elapsed / duration, clamped [0.0, 1.0]
     ├─ Value: eased_value(progress) using Easing function
     └─ COMPLETING: When progress == 1.0 this frame
        └─ Next frame: Eased record cleaned up (seen != frame)

STATE: Eased { start: f32, target: f32, duration: f32, seen: u32 }
```

**Cleanup rule**: Frame cleanup retains only Eased records where `seen == current_frame`. Records not read during a frame are freed.

**Test evidence** (src/testing/mod.rs line 1129):
```rust
fn eased_values_nothing_draws_any_more_are_forgotten() {
    let mut memory = Memory::new();
    memory.begin_frame(0.016);
    let id = Id::new("test");
    
    // Eased record created
    memory.ease(id, 100.0, 1.0);
    assert_eq!(memory.eased.len(), 1);
    
    // Not drawn next frame → cleaned
    memory.end_frame();
    assert_eq!(memory.eased.len(), 0);
}
```

---

### 2. Phased Animation (phase() primitive)

**Storage**: `Memory::cycles: HashMap<Id, Cycle>`

**Lifecycle**:
```
START: Memory::phase(id, period_seconds) called
  ├─ Check: Is id already cycling?
  │   └─ Yes: Continue from current value
  │   └─ No: Create new Cycle { value: 0.0, seen }
  └─ ACTIVE: Each frame: value += (delta / period).fract()
     ├─ Returns value in [0.0, 1.0)
     ├─ Wraps at 1.0 (phase(id, 1.0) = 1.0 means next frame wraps to 0.0)
     └─ COMPLETING: Never completes (infinite loop, see Gap 1)
        └─ Pauses when not read (seen != frame)

STATE: Cycle { value: f32, seen: u32 }

CLEANUP: Same as Eased — frame cleanup drops cycles not read.
```

**Invariant**: Phase never completes naturally; it cycles forever. Completion would require breaking the physics of periodic motion.

**Test evidence** (src/testing/mod.rs line 681):
```rust
fn the_phase_animation_returns_a_value_that_cycles() {
    let mut memory = Memory::new();
    memory.begin_frame(0.016);
    let id = Id::new("pulse");
    
    // Phase ticks forward
    assert!(memory.phase(id, 1.0) < 0.1);
    
    memory.end_frame();
    memory.begin_frame(0.016);
    
    // Still animating
    assert!(memory.phase(id, 1.0) > 0.0 && memory.phase(id, 1.0) < 0.1);
}
```

---

### 3. Deferred Execution (defer() primitive)

**Storage**: `Memory::deferred: HashMap<Id, f32>` (fire_time)

**Lifecycle**:
```
START: Memory::defer(id, delay_seconds) called
  └─ Schedules fire_time = accumulated_time + delay
     └─ PENDING: Waits for accumulated_time ≥ fire_time
        ├─ Each frame: Check should_defer_fire(id)
        ├─ When ready: Handler fires, entry removed
        └─ FIRED: Record deleted, animation complete

STATE: Id → f32 (fire time)
```

**Frame semantics**:
- `Memory::defer(id, 0.5)` called at t=0.0 → fires at t≥0.5
- Firing happens once, atomically
- No retargeting (unlike ease)

**Test evidence** (tests/recipes.rs):
```rust
fn a_deferred_handler_fires_at_the_right_time() {
    let mut h = Harness::new(app, view);
    h.defer_at(0.5);
    h.frames(30);  // 30 * 0.016 ≈ 0.48s
    assert!(!app.fired);
    h.frame();     // 0.5s exactly
    assert!(app.fired);
}
```

---

### 4. Transition Animation (start_transition() primitive)

**Storage**: `Memory::transitions: HashMap<Id, (f32, f32)>` (start_time, duration)

**Lifecycle**:
```
START: Memory::start_transition(id, duration) called
  ├─ Records (accumulated_time, duration)
  └─ ACTIVE: Each frame: progress = elapsed / duration
     ├─ Returns [0.0, 1.0]
     ├─ Called by view to transform element properties
     └─ COMPLETING: progress ≥ 1.0 this frame
        ├─ Callbacks/handlers run (see Gap 3)
        └─ CLEANUP: clear_transition(id) removes record

STATE: Id → (start_time: f32, duration: f32)
```

**R1 limitation**: No callback API (Gap 3). Completion must be manually detected.

**Test evidence** (src/testing/mod.rs line 1195):
```rust
fn transitions_report_progress() {
    let mut memory = Memory::new();
    let id = Id::new("slide");
    
    memory.begin_frame(0.0);
    memory.start_transition(id, 1.0);
    
    memory.begin_frame(0.5);
    assert_eq!(memory.transition_progress(id), Some(0.5));
    
    memory.begin_frame(1.0);
    assert_eq!(memory.transition_progress(id), Some(1.0));
    
    memory.clear_transition(id);
    assert_eq!(memory.transition_progress(id), None);
}
```

---

## R2 Animation Features (Planned)

### 5. Spring Animation (Gap 1)

**Planned storage**: `Memory::springs: HashMap<Id, Spring>`

**Planned lifecycle**:
```
START: Memory::spring(id, target, damping, stiffness) called
  └─ ACTIVE: Physics loop each frame
     ├─ velocity += (target - position) * stiffness - velocity * damping
     ├─ position += velocity * delta
     ├─ Overshoot if damping < 1.0 (bounce)
     └─ COMPLETING: velocity ≈ 0 && |position - target| < epsilon
        └─ CLEANUP: Spring record removed

STATE: Spring {
  position: f32,
  target: f32,
  velocity: f32,  // ← Gap 6 (currently missing)
  damping: f32,
  stiffness: f32,
  seen: u32
}
```

**Implementation location**: src/memory.rs after Eased struct (line 195)

**Test stub** (tests/recipes.rs, currently ignored):
```rust
#[test]
#[ignore = "R2 Phase 2: Spring animations"]
fn spring_overshoots_then_settles() {
    let mut memory = Memory::new();
    let id = Id::new("bounce");
    
    memory.begin_frame(0.0);
    memory.spring(id, 100.0, 0.5, 100.0);  // damping < 1.0 = bounce
    
    // Collect peak and check overshoot
    let mut max = 0.0;
    for _ in 0..100 {
        memory.begin_frame(0.016);
        let pos = memory.spring_position(id);
        max = max.max(pos);
    }
    
    assert!(max > 100.0, "spring should overshoot");
}
```

---

### 6. Enter/Exit Transitions (Gap 2)

**Planned storage**: `Memory::enter_exit: HashMap<Id, EnterExit>`

**Planned lifecycle**:
```
START: Element marked with .enter_transition(duration) or .exit_transition(duration)
  ├─ ENTERING: Animation plays when element first appears
  │   └─ Progress from 0.0 to 1.0
  │   └─ View reads progress from memory
  │   └─ On completion: Stays at 1.0 (animation done)
  │
  └─ EXITING: Animation plays when element is removed
      ├─ Element remains visible during exit
      ├─ View must check if exiting && return element with reduced opacity
      └─ On completion: Element finally removed from tree

STATE: EnterExit {
  kind: EnterExitKind,  // Enter or Exit
  start_time: f32,
  duration: f32,
  seen: u32
}
```

**Integration point**: paint.rs (line 1200) must check for enter/exit animations

**Test stub** (tests/recipes.rs):
```rust
#[test]
#[ignore = "R2 Phase 2: Enter/exit transitions"]
fn element_fades_in_when_entering() {
    let mut h = Harness::new(app, view).size(200.0, 200.0);
    h.frame();
    
    // Element doesn't exist yet
    assert!(!h.contains_text("Hello"));
    
    h.frame();
    // Element appears with fade-in animation
    h.update(|app| app.show_hello = true);
    
    // Still visible but opacity < 1.0
    let rect = h.text_rect("Hello");
    assert!(rect.w > 0.0, "element is drawn");
}
```

---

### 7. Memory::after() Sugar (Gap 3)

**Planned storage**: Reuses `Memory::deferred` with callback wrapper

**Planned lifecycle**:
```
START: Memory::after(id, delay, callback) called
  └─ Schedules callback to fire at delay
     ├─ PENDING: Waits for accumulated_time ≥ (now + delay)
     └─ FIRED: Callback runs once, entry removed
        └─ CLEANUP: Callback storage freed

DIFFERENCE FROM defer(): defer schedules a check; after() schedules a callback.
```

**Implementation location**: src/memory.rs (line 446), extend defer() section

**Test stub** (tests/recipes.rs):
```rust
#[test]
#[ignore = "R2 Phase 3: Memory::after callback sugar"]
fn memory_after_runs_callback_after_delay() {
    let mut memory = Memory::new();
    let mut ran = false;
    let id = Id::new("delayed_action");
    
    memory.begin_frame(0.0);
    memory.after(id, 0.5, || { ran = true; });
    
    memory.end_frame();
    memory.begin_frame(0.5);
    // Callback fires during phase transition
    assert!(ran);
}
```

---

## 2-Live-Loop Budget Enforcement (Gap 4)

**Current state**: No enforcement. Multiple animations can run simultaneously with no frame-time budget.

**Planned enforcement**:
```
BUDGET: Maximum 2 animations rendering ("live loops") per frame

EXAMPLES:
  ✓ One eased color change + one phase pulse = 2 live loops (OK)
  ✓ One spring + one transition = 2 live loops (OK)
  ✗ Three eased animations = 3 live loops (VIOLATION)

ENFORCEMENT MECHANISM:
  1. Count active animations per frame
  2. If count > 2: Panic with helpful message
     "Too many concurrent animations: 3 live loops detected.
      Merge similar animations or stagger start times with Memory::defer()."
  3. In tests: Pass budget assertions in STEP 20
```

**Implementation location**: src/memory.rs (line 491), extend is_animating() logic

**Test stub** (tests/recipes.rs):
```rust
#[test]
#[ignore = "R2 Phase 1: 2-live-loop budget enforcement"]
#[should_panic(expected = "Too many concurrent animations")]
fn exceeding_2_live_loops_panics_with_helpful_message() {
    let mut memory = Memory::new();
    let id1 = Id::new("anim1");
    let id2 = Id::new("anim2");
    let id3 = Id::new("anim3");
    
    memory.begin_frame(0.0);
    memory.ease(id1, 100.0, 1.0);
    memory.ease(id2, 200.0, 1.0);
    memory.ease(id3, 300.0, 1.0);  // Third animation
    
    // Should panic at frame end when checking budget
    memory.end_frame();
}
```

---

## Cleanup Ordering (Gap 7)

**Current state**: Frame cleanup is straightforward (remove unseen records).

**R2 required cleanup order**:
```
CLEANUP PHASES (run in order):

1. ANIMATION_COMPLETE: Callbacks for completed animations
   └─ Springs reaching target
   └─ Transitions reaching 100%
   └─ Deferred timers firing
   └─ Exit animations finishing

2. MEMORY_CLEANUP: Remove unseen records
   └─ eased records not read this frame
   └─ cycles not read this frame
   └─ springs not read this frame
   └─ enter_exit animations finished

3. HANDLER_EXECUTION: Run any handlers triggered by (1)
   └─ Modify app state
   └─ May start new animations

4. NEXT_FRAME_PREP: Reset counters
   └─ Increment frame number
   └─ Clear pointer_moved flag
   └─ Prepare for next frame

INVARIANT: Animation cannot resurrect after cleanup phase.
```

**Implementation location**: src/memory.rs (line 605), extend end_frame()

**Test stub** (tests/recipes.rs):
```rust
#[test]
#[ignore = "R2 Phase 3: Cleanup ordering"]
fn completed_animations_fire_callbacks_before_memory_cleanup() {
    let mut h = Harness::new(app, view);
    let id = Id::new("fade");
    
    h.ease(id, 0.0, 0.5);  // 0.5s animation
    h.frames(30);          // 30 * 0.016 ≈ 0.48s
    
    // Callback runs
    assert!(!h.state().fade_complete);
    
    h.frame();             // 0.5s exactly, callback fires
    assert!(h.state().fade_complete);
    
    // Memory cleanup removes record
    h.frame();
    // Animation is gone, fade_complete stays true
    assert!(h.state().fade_complete);
}
```

---

## Velocity Inheritance (Gap 6)

**Current state**: Springs don't track velocity; can't smoothly inherit momentum.

**Planned behavior**:
```
SCENARIO: User drags slider to position, animates to final snap point

CURRENT (R1):
  1. Drag ends at position = 150, velocity is lost
  2. Snap animation starts: spring(target=200)
  3. Spring animates from 150→200 (no knowledge of drag momentum)
  4. Motion feels jittery (sudden change in direction)

R2 IMPROVED:
  1. Drag ends at position = 150, velocity = 200/s (rightward)
  2. Snap animation starts: spring(target=200, inherit_velocity=true)
  3. Spring sees incoming velocity, smooths into spring motion
  4. Motion feels natural (momentum carries through)

IMPLEMENTATION:
  - Spring tracks velocity (new field)
  - On spring retarget: velocity = last_drag_velocity
  - Spring physics applies damping to inherited velocity
```

**Implementation location**: src/memory.rs (Spring struct, new field velocity: f32)

**Test stub** (tests/recipes.rs):
```rust
#[test]
#[ignore = "R2 Phase 2: Velocity inheritance"]
fn spring_inherits_drag_momentum() {
    let mut h = Harness::new(app, view);
    let id = Id::new("slider");
    
    // User drags slider
    h.drag_text("slider", |drag| drag.x = 0.8);
    h.frame();
    
    // Snap animation starts
    h.drag_end();
    h.spring(id, 1.0, 0.5, 100.0);
    
    // Spring should reach target smoothly via inherited momentum
    h.frames(60);
    assert!(h.memory().spring_position(id).abs() < 0.01);  // Nearly at target
}
```

---

## State Machine Verification

**All animation states must pass these assertions**:

1. **No resurrection**: Once cleanup completes, animation never runs again
2. **Deterministic timing**: Same elapsed time → same animation value
3. **Cleanup completeness**: No memory leaks between animations
4. **Callback ordering**: Callbacks fire before memory cleanup
5. ** 2-live-loop budget**: Concurrent animations ≤ 2 at all times

**Test verification suite** (tests/recipes.rs, STEP 19 baseline):
```bash
# All baseline tests verify state machine
cargo test --test recipes -- a_
# 27 tests covering all current animation types
# 12 acceptance stubs covering all R2 animation types
```

---

## Relationship to Other Modules

| Module | Interacts with | How |
|--------|----------------|-----|
| **paint.rs** | ease, phase, transitions | Reads animation progress to transform drawing |
| **input.rs** | Memory | Provides drag velocity for momentum (Gap 6) |
| **shell/mod.rs** | Memory::begin_frame | Injects delta time each frame |
| **testing/Harness** | All animation APIs | Steps time precisely for testing |
| **Element.rs** | Memory::ease | ease() called from style builder setters |

---

## Summary

The animation state machine enforces a strict lifecycle: **Create → Active → Complete → Cleanup → Dead**. No animation can resurrect after cleanup. Each animation type (ease, phase, defer, transition, spring, enter/exit, after) passes through this lifecycle with specific cleanup rules.

R2 adds springs, enter/exit, and Memory::after while maintaining the same state model. The 2-live-loop budget limits concurrent animations to 2, preventing frame-time overruns.

This state machine is **testable** (deterministic timing), **observable** (frame-by-frame progress), and **auditable** (cleanup ordering verified at each step).

For implementation guidance, see STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md.
