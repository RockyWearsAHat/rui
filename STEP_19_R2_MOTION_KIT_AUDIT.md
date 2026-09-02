# STEP 19: R2 Motion Kit Audit

## Executive Summary

This document audits the current animation system in rui and identifies gaps that must be filled for R2 (Motion Kit) implementation. The audit reveals **4 fully working animation primitives**, **5 framework spots** that hold animation state, and **7 missing features** that block complete motion system implementation.

**Current State**: Animations work for simple easing and looping. No springs, no enter/exit transitions, no deferred callbacks, no 2-live-loop budget enforcement, no `Metrics.motion=0` collapse.

**Readiness for R2**: Foundation is solid; gaps are well-defined and isolated.

---

## Current Animation Primitives (4 working)

### 1. **Easing** (`Painter::ease()` → `Memory::ease()`)
- **Purpose**: Animate a value from current to target over N seconds
- **Code**: `src/paint.rs` lines 147–153 (Painter) → `src/memory.rs` lines 259–285 (Memory)
- **Behavior**: 
  - Reads/writes `Memory.eased: HashMap<Id, Eased>`
  - Uses quadratic in-out by default (no configurable easing)
  - Returns target immediately if outside a frame
  - Requests redraw while animating via `Memory.animating = true`
- **Example**: 
  ```rust
  let opacity = painter.ease("opacity", 0.5, 0.3);  // Animate to 0.5 over 0.3s
  ```
- **Test Coverage**: `tests/r2_motion_kit_audit.rs::primitives_ease_works()`

### 2. **Phase (Cycles)** (`Painter::phase()` → `Memory::phase()`)
- **Purpose**: Animate a value 0→1→0 repeatedly every N seconds
- **Code**: `src/paint.rs` lines 155–178 (Painter) → `src/memory.rs` lines 287–310 (Memory)
- **Behavior**:
  - Reads/writes `Memory.cycles: HashMap<Id, Cycle>`
  - Returns 0.0 outside a frame
  - Always requests redraw (no termination; cycles run forever)
- **Example**: 
  ```rust
  let pulse = painter.phase("pulse", 1.0);  // Pulse 0→1→0 every 1.0s
  ```
- **Test Coverage**: `tests/r2_motion_kit_audit.rs::primitives_phase_works()`

### 3. **Deferred Callbacks** (`Memory.deferred`)
- **Purpose**: Schedule a one-time action after N seconds
- **Code**: `src/memory.rs` lines 246–251 (field), lines 441–460 (implementation)
- **Behavior**:
  - Maps `Id → time_to_fire` (absolute time in seconds since app start)
  - Fired at end of frame if `accumulated_time >= fire_time`
  - Handlers invoked after all views drawn (deferred execution model)
  - **Limitation**: No callbacks; fires a flag, not a closure
- **Example**: 
  ```rust
  memory.defer(id, 2.0);  // Flag fires at absolute time 2.0s
  ```
- **Test Coverage**: `tests/r2_motion_kit_audit.rs::primitives_deferred_works()`

### 4. **Transitions** (`Memory.transitions`)
- **Purpose**: Animate between discrete states over a duration
- **Code**: `src/memory.rs` lines 250–251 (field)
- **Behavior**:
  - Maps `Id → (start_time, total_duration)`
  - Allows querying progress via `Memory.progress_of(id)` (lines 462–474)
  - No easing curve (linear only)
  - Works with any discrete-state enum via manual progress query
- **Example**: 
  ```rust
  memory.start_transition(id, 0.5);  // 0.5s transition
  let progress = memory.progress_of(id);  // Returns 0.0–1.0
  ```
- **Test Coverage**: `tests/r2_motion_kit_audit.rs::primitives_transitions_works()`

---

## Framework Spots Holding Animation State (5 locations)

### 1. `Memory.eased: HashMap<Id, Eased>` (line 213)
- Holds: Current position, target, remaining duration for easing animations
- Accessed by: `Memory::ease()`, frame loop end cleanup
- Lifecycle: Created on first call, removed when animation finishes

### 2. `Memory.cycles: HashMap<Id, Cycle>` (line 215)
- Holds: Current phase position (0.0–1.0), period for looping
- Accessed by: `Memory::phase()`, frame loop end
- Lifecycle: Created on first call, never removed (cycles run forever)

### 3. `Memory.deferred: HashMap<Id, f32>` (line 247)
- Holds: Absolute fire time for each deferred action
- Accessed by: `Memory::defer()`, frame loop end (checking fired times)
- Lifecycle: Created on defer call, removed after firing

### 4. `Memory.transitions: HashMap<Id, (f32, f32)>` (line 251)
- Holds: Start time and total duration for state transitions
- Accessed by: `Memory::start_transition()`, `Memory::progress_of()`
- Lifecycle: Created on start_transition call, removed when elapsed

### 5. `Memory.accumulated_time: f32` (line 249)
- Holds: Total elapsed time since application start (in seconds)
- Updated by: `Memory::begin_frame(delta)` each frame
- Read by: Deferred and transition handlers for absolute time calculations

---

## Animation System Architecture

### Frame Loop Time Injection
```
App::run() loop
  ├─ calculate delta (elapsed since last frame)
  ├─ memory.begin_frame(delta)  // Updates accumulated_time
  │  └─ deferred/transition handlers check accumulated_time
  ├─ draw(view)  // Painter.ease/phase called here
  │  └─ reads eased/cycles, writes to memory if no match
  └─ if memory.animating: request_redraw()
```

### Key Invariants
1. **Time is injected, never read** — No `Instant::now()` calls; all animation driven by `delta`
2. **Eased values clean up when done** — HashMap entries removed after animation target reached
3. **Cycles never terminate** — HashMap entries persist forever (intended for continuous loops)
4. **Deferred fires at absolute time** — Calculated as `accumulated_time >= fire_time`
5. **Transitions are linear** — No easing curve; progress is simple `elapsed / duration`

---

## Missing Features (7 gaps blocking R2)

### 1. **Springs with Configurable Bounce**
- **Current**: None
- **Needed**: `Painter::spring(key, target, stiffness, damping)` → `f32`
- **Impact**: Bouncy overshot animations (rubber band, elastic snap) not possible
- **R2 Scope**: Add `Spring` struct to `Memory`, implement physics
- **Budget**: Moderate; requires new HashMap in Memory + spring solver in ease loop

### 2. **Enter/Exit Transitions**
- **Current**: Easing works mid-animation; no dedicated enter/exit phases
- **Needed**: `Memory::enter(id, duration)` and `Memory::exit(id, duration)` + phase tracking
- **Impact**: UI animations with start/end states (fade in, slide in) require manual ease management
- **R2 Scope**: Phase-based transitions with automatic cleanup on completion
- **Budget**: Low; 3 phases (enter/live/exit) in state, phase detection in draw

### 3. **Memory::after() Sugar**
- **Current**: Deferred exists; no sugar API
- **Needed**: `Memory::after(id, seconds, closure)` to schedule callbacks cleanly
- **Impact**: Delays, auto-dismiss, staggered animations require manual time tracking
- **R2 Scope**: Add callback storage and invocation at frame end
- **Budget**: Low; new HashMap<Id, Fn()> + late-frame dispatch

### 4. **2-Live-Loop Budget Enforcement**
- **Current**: Cycles request redraw; no budget tracked
- **Needed**: Mechanical assertion that at most 2 `phase()` calls per frame
- **Impact**: Runaway animations (3+ loops) not caught; silent perf drain
- **R2 Scope**: Debug assert in `Memory::phase()` counting loops per frame
- **Budget**: Very low; counter + frame boundary reset

### 5. **Metrics.motion=0 Collapse**
- **Current**: No check for motion=0; animations run anyway
- **Needed**: Check `theme.metrics.motion` at each animation call; skip if 0
- **Impact**: Accessibility (reduced motion) not respected; animation budget not disabled
- **R2 Scope**: Add `if metrics.motion == 0 { return target }` to ease/phase/spring
- **Budget**: Very low; early return in 3 methods

### 6. **Velocity Inheritance on Retarget**
- **Current**: Easing resets to zero velocity on retarget
- **Needed**: Track velocity from previous animation; carry over on new target
- **Impact**: Smooth interruption of animations (e.g., dragging a slider) feels jerky
- **R2 Scope**: Extend `Eased` struct to store velocity; inherit on retarget
- **Budget**: Low; math in ease solver + retarget handler

### 7. **Cleanup Policy and Memory Leaks**
- **Current**: Cycles never removed; eased values persist if ID reused
- **Needed**: Explicit lifecycle policy: when are eased/cycles/deferred/transitions cleaned?
- **Impact**: UI trees with dynamic children could leak animation state if IDs collide
- **R2 Scope**: Document + enforce cleanup on frame 1 after animation finishes
- **Budget**: Moderate; ID tracking + removal heuristics + tests

---

## Test Baseline (10 comprehensive tests)

The file `tests/r2_motion_kit_audit.rs` establishes baseline tests for all current primitives and edge cases:

1. **`primitives_ease_works()`** — Basic easing from 0 to 1 over 1 second
2. **`primitives_phase_works()`** — Looping phase from 0→1→0
3. **`primitives_deferred_works()`** — Deferred actions fire at correct time
4. **`primitives_transitions_works()`** — Manual progress tracking for state transitions
5. **`animation_id_collision_ease_vs_phase()`** — Same ID used for ease + phase (should be independent)
6. **`animation_retarget_during_easing()`** — Retargeting an in-flight ease changes target without reset
7. **`animation_cleanup_after_easing_finishes()`** — Eased HashMap entry removed when animation done
8. **`combined_ease_phase_defer()`** — Multiple animations on same element co-exist
9. **`constraint_audit_existing_primitives()`** — Documents what IS mechanically asserted
10. **`constraint_audit_missing_features()`** — Documents what IS NOT asserted

**Run with**:
```bash
cargo test --test r2_motion_kit_audit -- --nocapture
```

**Expected output**: All 10 pass; "CURRENT STATE" output shows 4 primitives + 5 framework spots + 7 gaps.

---

## Impact Analysis: What Breaks Without R2

### Without Springs
- Elastic interactions (overshot buttons, rubber-band scrolls) → stiff, uninviting
- Visual feedback on collision/snap feels mechanical, not organic

### Without Enter/Exit Transitions
- Page/modal/overlay animations require manual easing wrapper code
- Choreography (staggered children) impossible; each animation is isolated

### Without memory::after()
- Auto-dismiss (toasts, snackbars) requires app-level timer management
- Cascading animations (A ends → start B) require manual deferred handlers

### Without 2-Live-Loop Budget
- Runaway animations (spinner + pulse + shimmer) not caught; smooth 60fps degrades silently
- Accessibility (motion=0) can be violated without warning

### Without Metrics.motion=0 Collapse
- Users with vestibular disorders forced to watch animations
- Animations run even on zero-motion themes (accessibility violation)

### Without Velocity Inheritance
- Interrupting a dragged slider feels like jumping; no smoothness
- UI interactions feel sluggish and unresponsive

### Without Cleanup Policy
- Long-running UIs with dynamic children leak animation state
- Memory overhead grows with time; cycling-through-IDs can cause collisions

---

## Implementation Order for R2

Based on impact and dependencies:

1. **Velocity Inheritance** (low cost, high feel improvement)
2. **Metrics.motion=0 Collapse** (accessibility; very low cost)
3. **2-Live-Loop Budget** (perf safety; very low cost)
4. **Springs** (high impact; medium cost; no dependencies)
5. **Enter/Exit Transitions** (choreography; low cost; depends on phase tracking)
6. **Memory::after()** (auto-dismiss; low cost; simple callback storage)
7. **Cleanup Policy** (robustness; document + enforce frame-end cleanup)

---

## Acceptance Criteria for This Audit

- [x] `tests/r2_motion_kit_audit.rs` exists with 10 comprehensive tests
- [x] All tests pass and document current state
- [x] `cargo test --test r2_motion_kit_audit -- --nocapture` outputs "CURRENT STATE"
- [x] Test output enumerates:
  - 4 existing primitives: ease, phase, defer, transitions
  - 5 framework spots: eased, cycles, deferred, transitions, accumulated_time
  - 7 missing features: springs, enter/exit, memory::after, 2-live-loop budget, metrics.motion=0, velocity inheritance, cleanup policy
- [x] This audit document provides analysis and implementation guidance
- [x] All 396 library tests + 10 audit tests pass (406 total)

---

## Next Steps

The audit is complete and baseline tests are established. Subsequent steps will implement R2 features in order of impact and dependency:

**STEP 20**: Velocity Inheritance + Metrics.motion=0 Collapse + 2-Live-Loop Budget (trifecta of low-cost, high-value wins)

**STEP 21**: Springs with Configurable Bounce

**STEP 22**: Enter/Exit Transitions with Phase Tracking

**STEP 23**: Memory::after() Sugar and Auto-Dismiss Recipes

**STEP 24**: Cleanup Policy and Memory Safety Tests

---

## References

- **Current Code**: `src/memory.rs` (lines 200–252, 259–310, 441–474)
- **Painter Interface**: `src/paint.rs` (lines 147–178)
- **Test Baseline**: `tests/r2_motion_kit_audit.rs` (10 tests, 100% passing)
- **Roadmap**: `rui.dx` (R2 Motion Kit entry)
