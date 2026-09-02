# STEP 19: R2 Motion Kit Audit — Detailed Analysis

## Overview

This document provides a phase-by-phase breakdown of the R2 Motion Kit audit, identifying the current animation system architecture, documenting what works, precisely locating what's missing, and providing a roadmap for implementation.

**Audit Result**: 4 working animation primitives + 5 framework storage spots + 7 identified gaps. Foundation is solid; gaps are isolated and sequenceable.

---

## Phase 1: Current State Inventory

### Animation Primitives (4 working, fully functional)

#### 1. **Easing** — Smooth value interpolation
- **Location**: `src/paint.rs` lines 147–153 (Painter), `src/memory.rs` lines 259–285 (Memory)
- **Mechanism**: 
  ```rust
  // In paint frame:
  let opacity = painter.ease("opacity", 0.5, 0.3);  // → 0.0 to 0.5 over 0.3s
  
  // In Memory:
  pub fn ease(&mut self, id: Id, target: f32, seconds: f32) -> f32 {
      if let Some(e) = self.eased.get_mut(&id) {
          e.target = target;  // Retarget mid-animation
          return e.position;  // Return current position
      }
      self.eased.insert(id, Eased { position: target, target, elapsed: 0.0 });
      target
  }
  ```
- **Storage**: `Memory.eased: HashMap<Id, Eased>` (line 213)
  - Holds: current position, target value, elapsed time
  - Lifecycle: Created on first call, **removed when animation finishes**
  - Cleanup: Frame loop checks if elapsed ≥ duration, deletes entry
- **Behavior**:
  - Returns current position each frame (interpolated toward target)
  - Accepts retargeting mid-animation (smoothly changes direction)
  - Uses quadratic in-out easing (hardcoded, not configurable)
  - Requests redraw via `Memory.animating = true` while active
- **Tests**: `primitives_ease_works()`, `animation_retarget_during_easing()`, `animation_cleanup_after_easing_finishes()`
- **Verification**: ✅ Working; only limitation is non-configurable easing curve

#### 2. **Phase (Cycles)** — Looping 0→1→0 animation
- **Location**: `src/paint.rs` lines 155–178 (Painter), `src/memory.rs` lines 287–310 (Memory)
- **Mechanism**:
  ```rust
  // In paint frame:
  let pulse = painter.phase("pulse", 1.0);  // → 0→1→0 every 1.0s
  
  // In Memory:
  pub fn phase(&mut self, id: Id, period: f32) -> f32 {
      let t = self.accumulated_time % period / period;  // 0.0 to 1.0 over period
      if t < 0.5 {
          2.0 * t        // 0→1 first half
      } else {
          2.0 * (1.0 - t)  // 1→0 second half
      }
  }
  ```
- **Storage**: `Memory.cycles: HashMap<Id, Cycle>` (line 215)
  - Holds: period, current phase position
  - Lifecycle: Created on first call, **never removed** (runs forever)
  - No cleanup: Cycles are intended to repeat indefinitely
- **Behavior**:
  - Returns triangle wave 0→1→0 every `period` seconds
  - Always requests redraw (continuous animation)
  - No termination condition (by design for spinners, pulsing)
  - No decay or fade (permanent until ID reused)
- **Tests**: `primitives_phase_works()`, `combined_ease_phase_defer()`
- **Verification**: ✅ Working; note that it never self-terminates

#### 3. **Deferred Callbacks** — One-time actions after delay
- **Location**: `src/memory.rs` lines 246–251 (field), lines 441–450 (implementation)
- **Mechanism**:
  ```rust
  // Schedule:
  memory.defer(id, 2.0);  // Fire at accumulated_time = 2.0s (absolute)
  
  // Check at frame end:
  for (id, fire_time) in &self.deferred {
      if self.accumulated_time >= fire_time {
          // Handler fires here (framework-specific)
          self.deferred.remove(id);
      }
  }
  ```
- **Storage**: `Memory.deferred: HashMap<Id, f32>` (line 247)
  - Holds: fire time (absolute seconds since app start)
  - Lifecycle: Created on defer call, **removed after firing**
  - Cleanup: Automatic once condition is met
- **Behavior**:
  - Fires once at absolute time (e.g., 2.5 seconds after app start)
  - No closure support (current limitation; only sets a flag)
  - Survives retargeting (can reschedule by calling defer() again)
  - Deferred execution (handler runs after frame drawn)
- **Tests**: `primitives_deferred_works()`, `combined_ease_phase_defer()`
- **Verification**: ✅ Working; limited by lack of closure support (no callback payload)

#### 4. **Transitions** — Linear state progression
- **Location**: `src/memory.rs` lines 250–251 (field), lines 462–474 (query)
- **Mechanism**:
  ```rust
  // Start:
  memory.start_transition(id, 0.5);  // 0.5s transition
  
  // Query progress:
  let progress = memory.progress_of(id);  // → 0.0 to 1.0
  
  // In view:
  let color = if let Some(progress) = memory.progress_of(id) {
      blend(from_color, to_color, progress)
  } else {
      to_color  // Transition finished
  };
  ```
- **Storage**: `Memory.transitions: HashMap<Id, (f32, f32)>` (line 251)
  - Holds: start time (absolute), duration
  - Lifecycle: Created on start_transition call, **removed when elapsed ≥ duration**
  - Cleanup: Automatic; progress_of() returns None after finish
- **Behavior**:
  - Returns progress 0.0→1.0 over duration (linear only, no easing curve)
  - Works with any discrete enum (manually blend in view code)
  - No choreography (each transition is independent)
  - No phase tracking (can't distinguish enter/live/exit)
- **Tests**: `primitives_transitions_works()`, `combined_ease_phase_defer()`
- **Verification**: ✅ Working; intentionally minimal (no curves, no phases)

---

## Phase 2: Framework Storage Inventory

The animation system uses 5 locations in Memory to hold state:

### 1. `Memory.eased: HashMap<Id, Eased>` (line 213)
- **Reads**: `Memory::ease()` (line 259)
- **Writes**: `Memory::ease()` inserts on first call, removes on finish (line 268)
- **Frame lifecycle**: Checked and updated each frame by painter.ease()
- **Cleanup trigger**: Automatic when `elapsed >= duration`
- **Max entries**: Unbounded (one per unique animated property)
- **Memory impact**: One Eased struct per animation (24 bytes: position, target, elapsed)

### 2. `Memory.cycles: HashMap<Id, Cycle>` (line 215)
- **Reads**: `Memory::phase()` (line 287)
- **Writes**: `Memory::phase()` inserts on first call (line 291)
- **Frame lifecycle**: Checked each frame, never explicitly removed
- **Cleanup trigger**: NEVER (cycles run forever by design)
- **Max entries**: Unbounded; grows with unique phase() IDs
- **Memory impact**: One Cycle struct per looping animation (16 bytes: period, phase)
- **Risk**: ID collision → reuse of old period; no reset

### 3. `Memory.deferred: HashMap<Id, f32>` (line 247)
- **Reads**: Frame loop checks accumulated_time (line 442)
- **Writes**: `Memory::defer()` inserts (line 446), frame loop removes on fire (line 450)
- **Frame lifecycle**: Checked at frame end, entry removed after firing
- **Cleanup trigger**: Automatic when fire condition met
- **Max entries**: Unbounded (one per deferred action)
- **Memory impact**: One f32 per deferred action (8 bytes)

### 4. `Memory.transitions: HashMap<Id, (f32, f32)>` (line 251)
- **Reads**: `Memory::progress_of()` (line 462)
- **Writes**: `Memory::start_transition()` inserts (line 467)
- **Frame lifecycle**: Checked each call, entry removed on finish (line 471)
- **Cleanup trigger**: Automatic when `elapsed >= duration`
- **Max entries**: Unbounded (one per transition)
- **Memory impact**: One tuple per transition (16 bytes: start_time, duration)

### 5. `Memory.accumulated_time: f32` (line 249)
- **Updated**: `Memory::begin_frame(delta)` each frame (line 325)
- **Read by**: Deferred, transitions, phase calculations
- **Type**: Running total in seconds
- **Precision**: f32 (sufficient for typical UI animations)
- **Range**: Unbounded; grows over lifetime of app

---

## Phase 3: Gap Analysis — What's Missing

### Gap 1: Springs with Configurable Bounce

**Current Limitation**: No spring physics; easing is always quadratic in-out.

**What's Needed**:
```rust
// API:
pub fn spring(&mut self, id: Id, target: f32, stiffness: f32, damping: f32) -> f32

// Example:
let bounciness = painter.spring("toggle", 1.0, 50.0, 0.5);  // Rubber-band effect
```

**Why It Matters**:
- Elastic interactions (button press, rubber-band scroll, snap-to-grid)
- Springy feedback feels more organic and playful
- Accessibility: Reduced-motion users shouldn't experience bounces

**Implementation Footprint**:
- Add `Spring` struct to `motion.rs` (physics state: position, velocity, target, k, c)
- Add `Memory.springs: HashMap<Id, Spring>` (new field)
- Add spring solver in ease loop (Hooke's law + damping)
- Budget: ~80 lines of new code + 1 HashMap field

**R2 Scope**: High priority; moderate cost

---

### Gap 2: Enter/Exit Transitions with Phase Tracking

**Current Limitation**: Transitions are a single linear progression; no enter/live/exit phases.

**What's Needed**:
```rust
// API:
pub fn enter(&mut self, id: Id, duration: f32) -> bool  // true while entering
pub fn exit(&mut self, id: Id, duration: f32) -> bool   // true while exiting

// In view:
if memory.enter(id, 0.3) {
    // Entering: fade in, slide up, etc.
} else if memory.exit(id, 0.2) {
    // Exiting: fade out, slide down
    // Remove from DOM when false
}
```

**Why It Matters**:
- Choreography: Enter animation for whole list, staggered children
- Modal/overlay animations: Fade in (enter) → live → fade out (exit)
- Choreography impossible without phase tracking

**Implementation Footprint**:
- Extend `Transition` struct: add `phase: enum { Enter, Live, Exit }`
- Extend transition tracking with phase detection
- Budget: ~40 lines of logic + field addition

**R2 Scope**: Medium priority; low cost; depends on phase tracking

---

### Gap 3: Memory::after() Sugar

**Current Limitation**: `defer()` exists but no closure support; no sugar API for common patterns.

**What's Needed**:
```rust
// API:
pub fn after<F>(&mut self, id: Id, delay: f32, f: F)
where
    F: Fn(&mut S) + 'static

// Example:
memory.after(toast_id, 3.0, |app| app.dismiss_toast());
```

**Why It Matters**:
- Auto-dismiss (toasts disappear after 3s)
- Cascading animations (A ends → start B)
- Staggered reveals (item 1 shows now, item 2 shows 100ms later, etc.)

**Implementation Footprint**:
- Add `Memory.callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>` (fire_time + closure)
- Add invocation loop at frame end
- Budget: ~30 lines of code + 1 HashMap field

**R2 Scope**: Medium priority; low cost; enables recipes

---

### Gap 4: 2-Live-Animation-Loop Budget

**Current Limitation**: No constraint on how many phase() calls happen per frame; silent perf risk.

**What's Needed**:
```rust
// In Memory::phase():
#[cfg(debug_assertions)]
{
    self.phase_calls_this_frame += 1;
    if self.phase_calls_this_frame > 2 {
        panic!("Animation budget exceeded: {} live loops", self.phase_calls_this_frame);
    }
}

// Frame reset:
pub fn begin_frame(&mut self, delta: f32) {
    #[cfg(debug_assertions)]
    { self.phase_calls_this_frame = 0; }
    // ...
}
```

**Why It Matters**:
- Mechanical safety: Runaway animations caught early in development
- Accessibility: Metrics.motion=0 can't be checked without knowing animation count
- Performance: Prevents silent perf degradation (3+ simultaneous loops)

**Implementation Footprint**:
- Add `debug_phase_calls: usize` field to Memory
- Add counter increment in `phase()` method
- Budget: ~5 lines of code; debug-only

**R2 Scope**: Low priority; very low cost; safety catch

---

### Gap 5: Metrics.motion=0 Collapse

**Current Limitation**: Animations run even when theme.metrics.motion=0 (accessibility violation).

**What's Needed**:
```rust
// In each animation method:
pub fn ease(&mut self, id: Id, target: f32, seconds: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 {
        return target;  // Skip animation entirely
    }
    // ... normal easing
}
```

**Why It Matters**:
- Accessibility: Users with vestibular disorders can disable motion
- WCAG compliance: Motion must be controllable
- Graceful degradation: Instant transitions when motion is disabled

**Implementation Footprint**:
- Add `metrics: &Metrics` parameter to ease/phase/spring methods
- Add early-return check in each method
- Budget: ~5 lines per method × 3 methods = 15 lines total

**R2 Scope**: Low priority; very low cost; accessibility

---

### Gap 6: Velocity Inheritance on Retarget

**Current Limitation**: When retargeting an in-flight easing animation, velocity resets to zero (jerky).

**What's Needed**:
```rust
// Extend Eased struct:
pub struct Eased {
    position: f32,
    target: f32,
    elapsed: f32,
    velocity: f32,  // ← NEW: track previous velocity
}

// On retarget:
pub fn ease(&mut self, id: Id, target: f32, seconds: f32) -> f32 {
    if let Some(e) = self.eased.get_mut(&id) {
        let old_velocity = e.velocity;  // Preserve velocity
        e.target = target;
        e.velocity = old_velocity;  // Inherit velocity
        return e.position;
    }
    // ...
}
```

**Why It Matters**:
- Smooth interruption: Dragging a slider feels natural (no jump)
- Responsive UI: Interactions feel reactive, not stuck
- Physics: Velocity-preserving animations match real-world motion

**Implementation Footprint**:
- Extend `Eased` struct with velocity field
- Update ease solver to calculate velocity from delta
- Update retarget logic to inherit velocity
- Budget: ~30 lines of solver math

**R2 Scope**: Medium priority; low cost; high feel improvement

---

### Gap 7: Animation Cleanup Policy and Memory Safety

**Current Limitation**: No explicit lifecycle policy for animations; cycles never terminate; ID collision risk.

**What's Needed**:
```rust
// Explicit policy:
// 1. Easing: Removed when elapsed >= duration (AUTOMATIC)
// 2. Cycles: Removed explicitly via Memory::clear_cycle(id) (MANUAL)
// 3. Deferred: Removed after firing (AUTOMATIC)
// 4. Transitions: Removed when elapsed >= duration (AUTOMATIC)
// 5. Springs: Removed when velocity ≈ 0 (AUTOMATIC)

// Document and enforce:
pub fn clear_animation(&mut self, id: Id) {
    self.eased.remove(&id);
    self.cycles.remove(&id);
    self.deferred.remove(&id);
    self.transitions.remove(&id);
    // springs.remove when added in R2
}
```

**Why It Matters**:
- Memory safety: Long-running UIs with dynamic children don't leak state
- Debugging: Clear ownership model for animation lifetime
- ID collision: Reusing IDs doesn't pull stale animation state

**Implementation Footprint**:
- Add cleanup documentation
- Add `clear_animation()` method
- Add lifecycle assertions in tests
- Budget: ~40 lines of documentation + 10 lines of code

**R2 Scope**: High priority; moderate cost; robustness

---

## Phase 4: Implementation Dependencies

### Dependency Graph

```
Gap 5 (Metrics.motion=0)
  ↓
Gap 6 (Velocity inheritance)
  ↓
Gap 1 (Springs)  ← Depends on velocity
  ↓
Gap 2 (Enter/exit) ← Can use spring for choreography
  ↓
Gap 3 (Memory::after) ← Pairs with enter/exit for staggered reveals
  ↓
Gap 7 (Cleanup) ← Documents lifecycle established by above
  ↓
Gap 4 (2-live-loop budget) ← Safety catch for all of above
```

### Recommended Order

1. **Metrics.motion=0** (Gap 5) — No dependencies; enable all subsequent animations to respect motion setting
2. **Velocity inheritance** (Gap 6) — Low cost; improves feel of all easing
3. **Springs** (Gap 1) — Medium cost; enables elastic interactions
4. **Enter/exit transitions** (Gap 2) — Low cost; enables choreography patterns
5. **Memory::after()** (Gap 3) — Low cost; enables auto-dismiss and cascading animations
6. **Cleanup policy** (Gap 7) — Moderate cost; essential for long-running UIs
7. **2-live-loop budget** (Gap 4) — Very low cost; performance safety catch

---

## Verification Strategy

### Unit Tests
- Each animation primitive: basic interpolation, retargeting, cleanup
- Each gap: test that shows expected R2 API working (currently fails)
- Edge cases: ID collision, HashMap overflow, time discontinuities

### Integration Tests
- Multiple animations on same element
- Animations across frames and lifecycle
- Memory cleanup and leak prevention
- Metrics.motion=0 disables all animation

### Regression Tests
- Existing animations unchanged after R2 implementation
- Performance: animation loop cost ≤ 1ms
- No new unsafe code (physics solvers are safe Rust)

### Acceptance Gates
- All 7 gaps have passing tests showing R2 API
- All 4 existing primitives remain unchanged and passing
- No regressions in 396 existing library tests
- `cargo test --lib` shows 0 failures

---

## Code Locations Summary

| Primitive | Lines | File | Status |
|-----------|-------|------|--------|
| **Painter::ease()** | 147–153 | paint.rs | ✅ Working |
| **Memory::ease()** | 259–285 | memory.rs | ✅ Working |
| **Painter::phase()** | 155–178 | paint.rs | ✅ Working |
| **Memory::phase()** | 287–310 | memory.rs | ✅ Working |
| **Memory.eased field** | 213 | memory.rs | ✅ Working |
| **Memory.cycles field** | 215 | memory.rs | ✅ Working |
| **Memory.deferred field** | 247 | memory.rs | ✅ Working |
| **Memory.transitions field** | 251 | memory.rs | ✅ Working |
| **Memory.accumulated_time field** | 249 | memory.rs | ✅ Working |

---

## Next Document

See **STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md** for detailed test procedures, verification checklist, and acceptance criteria.

