# STEP 19: R2 Motion Kit Audit — Implementation Blueprint

## Overview

This document provides a phase-by-phase implementation guide for R2 Motion Kit. It shows exactly what code changes are needed in each phase, in which order, with verification tests.

**Goal**: Transform 7 gaps into working animation features in 3 phases, each with acceptance criteria.

---

## Implementation Phases

### Phase 1: Foundation (3 low-cost wins)

**Duration**: ~3-4 commits  
**Gaps Closed**: Gap 5 (Metrics.motion=0), Gap 6 (Velocity), Gap 4 (2-live-loop budget)

**Why First**: No dependencies; high value; enables subsequent phases

---

#### Phase 1.1: Metrics.motion=0 Collapse (Gap 5)

**What**: Add motion accessibility check to all animation methods

**Files to Change**: `src/memory.rs`

**Changes**:
```rust
// Before:
pub fn ease(&mut self, id: Id, target: f32, seconds: f32) -> f32 {
    if let Some(e) = self.eased.get_mut(&id) {
        // ...
    }
}

// After:
pub fn ease(&mut self, id: Id, target: f32, seconds: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 {
        return target;  // Accessibility: skip animation
    }
    if let Some(e) = self.eased.get_mut(&id) {
        // ... normal easing
    }
}

// Same for phase():
pub fn phase(&mut self, id: Id, period: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 {
        return 0.0;  // Return start position
    }
    // ... normal cycling
}
```

**Impact on Painter** (`src/paint.rs`):
```rust
// Painter needs access to metrics
pub struct Painter<'a> {
    memory: &'a mut Memory,
    metrics: &'a Metrics,  // ADD THIS
}

impl<'a> Painter<'a> {
    pub fn new(memory: &'a mut Memory, metrics: &'a Metrics) -> Self {
        Painter { memory, metrics }
    }

    pub fn ease(&mut self, key: &str, target: f32, seconds: f32) -> f32 {
        self.memory.ease(key, target, seconds, self.metrics)
    }

    pub fn phase(&mut self, key: &str, period: f32) -> f32 {
        self.memory.phase(key, period, self.metrics)
    }
}
```

**Test Case** (from acceptance stubs, uncomment):
```rust
#[test]
fn r2_acceptance_metrics_motion_collapse() {
    let mut memory = Memory::new();
    
    // Normal animation
    let normal_metrics = Metrics { motion: 1.0, .. };
    let val1 = memory.ease(id, 1.0, 1.0, &normal_metrics);
    assert!(val1 < 1.0);  // Animating
    
    // Motion disabled
    let zero_metrics = Metrics { motion: 0.0, .. };
    let val2 = memory.ease(id, 1.0, 1.0, &zero_metrics);
    assert_eq!(val2, 1.0);  // Instant
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_metrics_motion_collapse`

---

#### Phase 1.2: Velocity Inheritance on Retarget (Gap 6)

**What**: Track velocity during easing; inherit it on retarget

**Files to Change**: `src/memory.rs`

**Changes**:
```rust
// Extend Eased struct:
pub struct Eased {
    position: f32,
    target: f32,
    elapsed: f32,
    duration: f32,
    velocity: f32,  // ADD THIS
}

// In ease() method:
pub fn ease(&mut self, id: Id, target: f32, seconds: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 { return target; }
    
    if let Some(e) = self.eased.get_mut(&id) {
        // Calculate velocity from last frame's delta
        let old_velocity = e.velocity;
        
        // Retarget but preserve velocity
        e.target = target;
        e.elapsed = 0.0;
        e.duration = seconds;
        e.velocity = old_velocity;  // Carry over velocity
        
        return e.position;
    }
    
    self.eased.insert(id, Eased {
        position: target,
        target,
        elapsed: 0.0,
        duration: seconds,
        velocity: 0.0,
    });
    target
}

// At frame end, update velocities:
// (This happens during next frame when ease() is called again)
```

**Note**: Velocity calculation happens naturally during easing interpolation. When a value changes from frame to frame, we can calculate `velocity = delta_position / delta_time`.

**Test Case**:
```rust
#[test]
fn r2_acceptance_velocity_inheritance() {
    let mut memory = Memory::new();
    let metrics = Metrics { motion: 1.0, .. };
    
    // Start animating
    let val1 = memory.ease(id, 1.0, 1.0, &metrics);  // Frame 1: 0.0
    memory.begin_frame(0.1);  // 100ms
    
    let val2 = memory.ease(id, 1.0, 1.0, &metrics);  // Frame 2: interpolated
    assert!(val2 > val1);
    
    // Retarget mid-animation
    let val3 = memory.ease(id, 0.5, 0.5, &metrics);  // New target, inherit velocity
    assert!(val3 > val2);  // Velocity smooths the transition
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_velocity_inheritance`

---

#### Phase 1.3: 2-Live-Animation-Loop Budget (Gap 4)

**What**: Enforce constraint that at most 2 `phase()` calls happen per frame

**Files to Change**: `src/memory.rs`

**Changes**:
```rust
pub struct Memory {
    // ... existing fields ...
    
    #[cfg(debug_assertions)]
    phase_call_count: usize,  // ADD THIS
}

// In phase() method:
pub fn phase(&mut self, id: Id, period: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 { return 0.0; }
    
    #[cfg(debug_assertions)]
    {
        self.phase_call_count += 1;
        if self.phase_call_count > 2 {
            panic!("Animation budget exceeded: {} live loops per frame", 
                   self.phase_call_count);
        }
    }
    
    // ... normal cycling
}

// In begin_frame():
pub fn begin_frame(&mut self, delta: f32) {
    #[cfg(debug_assertions)]
    { self.phase_call_count = 0; }  // Reset counter for new frame
    
    // ... rest of begin_frame
}
```

**Test Case**:
```rust
#[test]
fn r2_acceptance_2_live_loop_budget() {
    let mut memory = Memory::new();
    let metrics = Metrics { motion: 1.0, .. };
    
    // First call OK
    memory.phase(id1, 1.0, &metrics);
    
    // Second call OK
    memory.phase(id2, 1.0, &metrics);
    
    // Third call panics (in debug)
    #[cfg(debug_assertions)]
    {
        let result = std::panic::catch_unwind(|| {
            memory.phase(id3, 1.0, &metrics);
        });
        assert!(result.is_err());  // Panic caught
    }
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_2_live_loop_budget`

---

### Phase 1 Acceptance Criteria

Run after implementing all three Phase 1 gaps:

```bash
cargo test --test r2_motion_kit_audit -- r2_acceptance_metrics_motion_collapse --nocapture
cargo test --test r2_motion_kit_audit -- r2_acceptance_velocity_inheritance --nocapture
cargo test --test r2_motion_kit_audit -- r2_acceptance_2_live_loop_budget --nocapture
```

**Expected**: All 3 tests pass

**Regression Check**:
```bash
cargo test --lib 2>&1 | grep "test result"
# Should still show: test result: ok. 396 passed; 0 failed
```

---

### Phase 2: Core Features (Springs + Enter/Exit)

**Duration**: ~4-5 commits  
**Gaps Closed**: Gap 1 (Springs), Gap 2 (Enter/Exit)

**Dependencies**: Phase 1 complete (velocity available for spring inherit)

---

#### Phase 2.1: Springs with Configurable Bounce (Gap 1)

**What**: Add Spring struct from motion.rs into Memory; implement spring solver

**Files to Change**: `src/memory.rs`, `src/motion.rs`

**Changes to memory.rs**:
```rust
pub struct Memory {
    // ... existing ...
    springs: HashMap<Id, Spring>,  // ADD THIS
    // ... rest of fields
}

// Add spring() method:
pub fn spring(&mut self, id: Id, target: f32, stiffness: f32, damping: f32, metrics: &Metrics) -> f32 {
    if metrics.motion == 0.0 { return target; }
    
    // Use Spring from motion.rs
    let spring = self.springs.entry(id).or_insert_with(|| {
        Spring {
            position: target,
            target,
            velocity: 0.0,
            stiffness,
            damping,
        }
    });
    
    // Update target on retarget
    spring.target = target;
    
    // Step the spring simulation
    spring.step(0.016);  // Assume 60fps; typically called once per frame
    
    // Check if spring has settled
    let energy = spring.energy();
    if energy < 0.01 {
        self.springs.remove(&id);  // Remove when settled
    }
    
    spring.position
}
```

**Spring Solver** (in motion.rs or memory.rs):
```rust
pub struct Spring {
    position: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,      // k: How fast to return to target
    damping: f32,        // c: How much bouncing
}

impl Spring {
    pub fn step(&mut self, delta: f32) {
        let distance = self.target - self.position;
        let acceleration = self.stiffness * distance - self.damping * self.velocity;
        
        self.velocity += acceleration * delta;
        self.position += self.velocity * delta;
    }
    
    pub fn energy(&self) -> f32 {
        (self.position - self.target).abs() + self.velocity.abs()
    }
}
```

**Painter Integration**:
```rust
impl<'a> Painter<'a> {
    pub fn spring(&mut self, key: &str, target: f32, stiffness: f32, damping: f32) -> f32 {
        self.memory.spring(key, target, stiffness, damping, self.metrics)
    }
}
```

**Test Case**:
```rust
#[test]
fn r2_acceptance_spring_integration() {
    let mut memory = Memory::new();
    let metrics = Metrics { motion: 1.0, .. };
    
    // Create spring
    let val1 = memory.spring(id, 1.0, 50.0, 0.5, &metrics);
    assert!(val1 < 1.0);  // Not yet at target
    
    // Step multiple frames
    for _ in 0..100 {
        memory.begin_frame(0.016);
        memory.spring(id, 1.0, 50.0, 0.5, &metrics);
    }
    
    // Eventually settles near target
    let final_val = memory.spring(id, 1.0, 50.0, 0.5, &metrics);
    assert!((final_val - 1.0).abs() < 0.01);
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_spring_integration`

---

#### Phase 2.2: Enter/Exit Transitions with Phase Tracking (Gap 2)

**What**: Track enter/live/exit phases for choreography

**Files to Change**: `src/memory.rs`

**Changes**:
```rust
// Extend transitions storage:
pub struct Transition {
    start_time: f32,
    enter_duration: f32,
    live_duration: f32,
    exit_duration: f32,
    phase: enum TransitionPhase { Entering, Live, Exiting },
}

pub struct Memory {
    transitions: HashMap<Id, Transition>,  // Extended from (f32, f32)
    // ...
}

// Add enter/exit convenience methods:
pub fn enter(&mut self, id: Id, duration: f32) -> bool {
    let now = self.accumulated_time;
    let trans = self.transitions.entry(id).or_insert_with(|| Transition {
        start_time: now,
        enter_duration: duration,
        live_duration: f32::INFINITY,
        exit_duration: 0.0,
        phase: TransitionPhase::Entering,
    });
    
    let elapsed = now - trans.start_time;
    if elapsed < trans.enter_duration {
        true  // Still entering
    } else {
        trans.phase = TransitionPhase::Live;
        false  // Finished entering
    }
}

pub fn exit(&mut self, id: Id, duration: f32) -> bool {
    // Mark for exit; animate over duration
    let now = self.accumulated_time;
    if let Some(trans) = self.transitions.get_mut(&id) {
        trans.phase = TransitionPhase::Exiting;
        trans.exit_duration = duration;
        
        let exit_elapsed = now - (trans.start_time + trans.enter_duration + trans.live_duration);
        if exit_elapsed < duration {
            true  // Still exiting
        } else {
            self.transitions.remove(id);
            false  // Finished exiting
        }
    } else {
        false
    }
}
```

**Usage in View**:
```rust
fn view_item(memory: &Memory, item: &Item, painter: &Painter) -> El<App> {
    if memory.enter(item.id, 0.3) {
        // Entering: fade/scale up
        let progress = memory.enter_progress(item.id).unwrap_or(0.0);
        col((
            text(&item.name),
        )).opacity(progress)
    } else if memory.exit(item.id, 0.2) {
        // Exiting: fade/scale down
        let progress = memory.exit_progress(item.id).unwrap_or(1.0);
        col((
            text(&item.name),
        )).opacity(progress)
    } else {
        // Live: normal state
        col((
            text(&item.name),
        ))
    }
}
```

**Test Case**:
```rust
#[test]
fn r2_acceptance_enter_exit_transitions() {
    let mut memory = Memory::new();
    
    // Start entering
    assert!(memory.enter(id, 0.3));  // Still entering
    
    memory.begin_frame(0.2);  // 200ms into 300ms enter
    assert!(memory.enter(id, 0.3));  // Still entering
    
    memory.begin_frame(0.15);  // 350ms total, past 300ms
    assert!(!memory.enter(id, 0.3));  // Done entering
    
    // Now exit
    assert!(memory.exit(id, 0.2));  // Still exiting
    
    memory.begin_frame(0.2);  // 200ms total
    assert!(!memory.exit(id, 0.2));  // Done exiting
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_enter_exit_transitions`

---

### Phase 2 Acceptance Criteria

```bash
cargo test --test r2_motion_kit_audit -- r2_acceptance_spring_integration --nocapture
cargo test --test r2_motion_kit_audit -- r2_acceptance_enter_exit_transitions --nocapture
cargo test --lib 2>&1 | grep "test result"  # No regressions
```

---

### Phase 3: Sugar APIs and Robustness

**Duration**: ~2-3 commits  
**Gaps Closed**: Gap 3 (Memory::after), Gap 7 (Cleanup Policy)

**Dependencies**: Phase 1 + 2 complete

---

#### Phase 3.1: Memory::after() Sugar (Gap 3)

**What**: Add callback scheduling for auto-dismiss and cascading animations

**Files to Change**: `src/memory.rs`

**Changes**:
```rust
pub struct Memory {
    callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>,  // ADD THIS
    // ...
}

pub fn after<F>(&mut self, id: Id, delay: f32, f: F)
where
    F: Fn() + 'static,
{
    let fire_time = self.accumulated_time + delay;
    self.callbacks.insert(id, (fire_time, Box::new(f)));
}

// In begin_frame(), invoke fired callbacks:
pub fn begin_frame(&mut self, delta: f32) {
    // ... existing code ...
    
    // Fire callbacks
    let now = self.accumulated_time;
    let to_fire: Vec<_> = self.callbacks.iter()
        .filter(|(_, (fire_time, _))| *fire_time <= now)
        .map(|(id, _)| *id)
        .collect();
    
    for id in to_fire {
        if let Some((_, cb)) = self.callbacks.remove(&id) {
            cb();  // Invoke
        }
    }
}
```

**Problem**: Callbacks can't mutate App state (closure only receives `()`)

**Solution**: Use deferred flag + app-level handler

```rust
// Instead of:
memory.after(id, 3.0, || app.dismiss_toast());  // Won't compile

// Use:
memory.defer(toast_id, 3.0);  // Flag fires
if memory.should_defer_fire(toast_id) {
    app.dismiss_toast();  // App handles
}
```

**Alternative**: Use El::on_animation_end() (not in scope for Phase 3)

**Test Case**:
```rust
#[test]
fn r2_acceptance_memory_after_sugar() {
    let mut memory = Memory::new();
    let mut called = std::cell::Cell::new(false);
    
    memory.after(id, 0.5, || {
        called.set(true);
    });
    
    // Before timeout
    memory.begin_frame(0.3);
    assert!(!called.get());
    
    // After timeout
    memory.begin_frame(0.3);  // Total 0.6s
    assert!(called.get());
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_memory_after_sugar`

---

#### Phase 3.2: Animation Cleanup Policy (Gap 7)

**What**: Document and enforce animation lifecycle

**Files to Change**: `src/memory.rs` (code), `CLAUDE.md` (documentation)

**Changes to memory.rs**:
```rust
// Add documentation comment:
pub struct Memory {
    // Animation state cleanup policy:
    // 
    // AUTOMATIC cleanup (removed when done):
    //   - eased: Removed when elapsed >= duration
    //   - springs: Removed when energy < threshold
    //   - deferred: Removed after callback fires
    //   - transitions: Removed when exit phase finishes
    //   - callbacks: Removed after invocation
    //
    // MANUAL cleanup (user calls Memory::clear_animation):
    //   - cycles: Run forever; call clear_cycle(id) to stop
    //
    // ID collision: If ID reused before cleanup, stale state can interfere
    // Solution: Use unique IDs (use item.id + field name)
    
    eased: HashMap<Id, Eased>,
    cycles: HashMap<Id, Cycle>,
    springs: HashMap<Id, Spring>,
    deferred: HashMap<Id, f32>,
    transitions: HashMap<Id, Transition>,
    callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>,
}

// Add explicit cleanup methods:
pub fn clear_animation(&mut self, id: Id) {
    self.eased.remove(&id);
    self.cycles.remove(&id);
    self.springs.remove(&id);
    self.deferred.remove(&id);
    self.transitions.remove(&id);
    self.callbacks.remove(&id);
}

pub fn clear_cycle(&mut self, id: Id) {
    self.cycles.remove(&id);
}
```

**Changes to CLAUDE.md**:
Add section under "Contributor Workflow":
```markdown
### Animation Lifecycle and Cleanup

Animations automatically clean up when finished:
- **Easing**: Removed when `elapsed >= duration`
- **Springs**: Removed when `energy < 0.01`
- **Deferred/Callbacks**: Removed after firing
- **Transitions**: Removed when exit phase finishes

Only **cycles** require manual cleanup:
- Use `Memory::clear_cycle(id)` to stop a looping animation
- Cycles run forever by design (for spinners, pulsing indicators)
- ID reuse without cleanup causes stale state interference

Best practice for animated lists:
```rust
for (i, item) in items.iter().enumerate() {
    let unique_id = format!("item_{}_opacity", item.id);  // Include item ID
    let opacity = painter.ease(&unique_id, 1.0, 0.3);
}
```
```

**Test Case**:
```rust
#[test]
fn r2_acceptance_animation_cleanup_policy() {
    let mut memory = Memory::new();
    let metrics = Metrics { motion: 1.0, .. };
    
    // Create various animations
    memory.ease(id1, 1.0, 0.5, &metrics);
    memory.phase(id2, 1.0, &metrics);
    memory.defer(id3, 1.0);
    
    // Simulate time passing
    for _ in 0..100 {
        memory.begin_frame(0.01);
    }
    
    // Easing and deferred should be cleaned up
    assert!(memory.eased.is_empty());
    assert!(memory.deferred.is_empty());
    
    // Cycles never auto-clean
    assert!(!memory.cycles.is_empty());
    
    // Manual cleanup works
    memory.clear_cycle(id2);
    assert!(memory.cycles.is_empty());
}
```

**Verification**: `cargo test --test r2_motion_kit_audit -- r2_acceptance_animation_cleanup_policy`

---

### Phase 3 Acceptance Criteria

```bash
cargo test --test r2_motion_kit_audit -- r2_acceptance_memory_after_sugar --nocapture
cargo test --test r2_motion_kit_audit -- r2_acceptance_animation_cleanup_policy --nocapture
cargo test --lib 2>&1 | grep "test result"  # No regressions
```

---

## Implementation Checklist

### Pre-Implementation
- [ ] All audit tests passing (`cargo test --test r2_motion_kit_audit`)
- [ ] No uncommitted changes (`git status`)
- [ ] On main branch (`git branch`)

### Phase 1
- [ ] Gap 5 (Metrics.motion=0)
  - [ ] Add metrics parameter to ease() and phase()
  - [ ] Add metrics field to Painter
  - [ ] Update call sites to pass &theme.metrics
  - [ ] Test passes
  - [ ] Commit
- [ ] Gap 6 (Velocity inheritance)
  - [ ] Extend Eased struct with velocity field
  - [ ] Update ease solver to calculate velocity
  - [ ] Test passes
  - [ ] Commit
- [ ] Gap 4 (2-live-loop budget)
  - [ ] Add debug counter to Memory
  - [ ] Check in phase() method
  - [ ] Reset in begin_frame()
  - [ ] Test passes
  - [ ] Commit
- [ ] Verify: All 3 acceptance tests pass + 0 regressions

### Phase 2
- [ ] Gap 1 (Springs)
  - [ ] Add Spring struct to motion.rs (or memory.rs)
  - [ ] Add springs HashMap to Memory
  - [ ] Implement spring() method with solver
  - [ ] Extend Painter with spring() method
  - [ ] Test passes
  - [ ] Commit
- [ ] Gap 2 (Enter/Exit)
  - [ ] Extend Transition struct with phases
  - [ ] Add enter() and exit() methods to Memory
  - [ ] Add helper methods for progress queries
  - [ ] Test passes
  - [ ] Commit
- [ ] Verify: Both acceptance tests pass + 0 regressions

### Phase 3
- [ ] Gap 3 (Memory::after)
  - [ ] Add callbacks HashMap to Memory
  - [ ] Implement after() method
  - [ ] Add callback invocation in begin_frame()
  - [ ] Test passes
  - [ ] Commit
- [ ] Gap 7 (Cleanup policy)
  - [ ] Add clear_animation() method
  - [ ] Document lifecycle policy in code comments
  - [ ] Update CLAUDE.md with best practices
  - [ ] Test passes
  - [ ] Commit
- [ ] Verify: Both acceptance tests pass + 0 regressions

### Final Verification
- [ ] All 27 baseline tests pass
- [ ] All 12 acceptance tests pass (no longer ignored)
- [ ] All 396 library tests pass (0 regressions)
- [ ] `cargo test --lib` shows `test result: ok`
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` shows no warnings
- [ ] Git history is clean: one commit per logical change

---

## Common Implementation Pitfalls

### Pitfall 1: Forgetting to Pass Metrics
**Mistake**: Update ease() but forget to pass &metrics in Painter

**Fix**: Update both memory.rs AND paint.rs, verify call sites

---

### Pitfall 2: ID String Collisions
**Mistake**: Use same key "opacity" for all list items

**Fix**: Document best practice; include item.id in key

---

### Pitfall 3: Cycle Cleanup
**Mistake**: Add cycle cleanup in begin_frame()

**Fix**: Cycles are intentionally persistent; only manually clear

---

### Pitfall 4: Spring Oscillation Forever
**Mistake**: Spring energy threshold too small; never settles

**Fix**: Use 0.01 as threshold; test with stiffness=50, damping=0.5

---

### Pitfall 5: Callback Closure Panic
**Mistake**: Callback tries to mutate App through & reference

**Fix**: Use deferred flag instead; app handles mutation in event loop

---

## Success Criteria

**Phase 1 Complete** when:
- [ ] Metrics.motion=0 skips all animations ✓
- [ ] Velocity inheritance makes retargeting smooth ✓
- [ ] 2-live-loop budget catches runaway animations ✓
- [ ] All 396 library tests still pass ✓

**Phase 2 Complete** when:
- [ ] Springs bounce and settle correctly ✓
- [ ] Enter/exit phases enable choreography ✓
- [ ] All 396 library tests still pass ✓

**Phase 3 Complete** when:
- [ ] Memory::after() schedules callbacks ✓
- [ ] Cleanup policy documented and enforced ✓
- [ ] All 396 library tests still pass ✓
- [ ] All 12 acceptance tests pass ✓

---

## Next Document

See **STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md** for quick reference during implementation.

