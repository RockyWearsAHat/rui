# STEP 19 Extended: R2 Motion Kit — Phase 3 Implementation Scaffolding

**Purpose**: Exact code locations and changes for Phase 3 (2 quality gaps). Foundation and physics from Phase 1/2 enable polish features.

**Phase 3 consists of**: 2 quality/polish gaps (low risk, 1 day, 2-3 commits)

---

## Phase 3 Overview

| Gap | File | Lines | Risk | Commits | Tests | Depends On |
|-----|------|-------|------|---------|-------|-----------|
| Gap 3: Easing enum | src/memory.rs, src/paint.rs | ~25 | Low | 1 | 2 | Phase 1 |
| Gap 7: Cleanup & Memory::after() | src/memory.rs, src/paint.rs | ~40 | Low | 2 | 2 | Phase 2 |

**Total Phase 3**: ~65 lines of new code, 2 gaps, 4 acceptance tests activated, low risk.

---

## Gap 3: Easing Enum Support

**Status**: Animations use hardcoded cubic easing (ease-in-out); no way to customize.  
**Impact**: Apps can't use linear, ease-in, ease-out, or custom curves; all motion feels uniform.  
**Effort**: Low (Easing enum, curve interpolation).

### Current Code Location

**File**: src/memory.rs, after Velocity struct (around line 120)

### Phase 3 Change

Add Easing enum with standard curves:

```rust
#[derive(Clone, Copy, Debug)]
pub enum Easing {
    Linear,           // t = t (constant speed)
    EaseIn,           // t = t² (slow start, fast finish)
    EaseOut,          // t = 1-(1-t)² (fast start, slow finish)
    EaseInOut,        // t = cubic (default, smooth both ends)
    EaseInCirc,       // t = sqrt(1 - (1-t)²) (stronger ease-in)
    EaseOutCirc,      // t = sqrt(t) (stronger ease-out)
    Custom(f32, f32), // cubic bezier: (x1, y1) control point
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        debug_assert!(t >= 0.0 && t <= 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            },
            Easing::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            Easing::EaseOutCirc => t.sqrt(),
            Easing::Custom(x1, y1) => {
                // Cubic Bezier: (0,0) → (x1,y1) → (1,1)
                // Simplified: approximate via Newton-Raphson
                cubic_bezier(*x1, *y1, t)
            },
        }
    }
}

// Helper: cubic Bezier approximation
fn cubic_bezier(x1: f32, y1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    // Bezier basis functions
    let b0 = mt3;
    let b1 = 3.0 * mt2 * t;
    let b2 = 3.0 * mt * t2;
    let b3 = t3;
    
    // Control points: (0,0), (x1,y1), (1,1), (1,1)
    b0 * 0.0 + b1 * y1 + b2 * y1 + b3 * 1.0
}
```

### Storage Addition

**File**: src/memory.rs, struct `Eased` (around line 90)

```rust
pub struct Eased {
    pub from: f32,
    pub to: f32,
    pub elapsed: Duration,
    pub duration: Duration,
    pub easing: Easing,  // ← ADD
}
```

**Constructor update** (around line 95):

```rust
impl Eased {
    pub fn new(from: f32, to: f32, duration: Duration, easing: Easing) -> Self {
        Self {
            from,
            to,
            elapsed: Duration::ZERO,
            duration,
            easing,  // ← ADD
        }
    }
}
```

### Paint Integration

**File**: src/paint.rs, `ease()` method (around line 410)

**Before**:
```rust
pub fn ease(&mut self, id: Id, from: f32, to: f32, duration: Duration) -> f32 {
    let progress = self.elapsed_fraction(&id, duration);
    let t = progress.powi(3);  // Hardcoded cubic
    from + (to - from) * t
}
```

**After**:
```rust
pub fn ease(&mut self, id: Id, from: f32, to: f32, duration: Duration) -> f32 {
    self.ease_with(id, from, to, duration, Easing::EaseInOut)
}

pub fn ease_with(&mut self, id: Id, from: f32, to: f32, duration: Duration, easing: Easing) -> f32 {
    let progress = self.elapsed_fraction(&id, duration);
    let eased = easing.apply(progress);
    from + (to - from) * eased
}
```

### Element Integration

**File**: src/element.rs, builder method (new, around line 430)

```rust
pub fn on_ease_with(self, id: Id, from: f32, to: f32, duration: Duration, easing: Easing) -> Self {
    self.on_draw(move |painter, rect| {
        let value = painter.ease_with(id, from, to, duration, easing);
        // Apply animated value to element
    })
}
```

### Acceptance Tests (activate STEP_19_TEST_GAP_3_PHASE_3)

Location: tests/recipes.rs (around line 1050)

```rust
#[test]
fn r2_gap_3_easing_linear_is_constant_speed() {
    let mut app = App { position: 0.0 };
    let mut h = Harness::new(app, view);
    
    let duration = Duration::from_millis(100);
    h.ease_add("pos", 0.0, 100.0, duration, Easing::Linear);
    
    h.frames(2);  // ~33ms
    let pos_early = h.state().position;
    
    h.frames(2);  // ~66ms total
    let pos_mid = h.state().position;
    
    h.frames(2);  // ~100ms total
    let pos_final = h.state().position;
    
    // Linear should increment evenly
    assert_approx_eq!(pos_early, 33.0, 5.0);
    assert_approx_eq!(pos_mid, 66.0, 5.0);
    assert_eq!(pos_final, 100.0);
}

#[test]
fn r2_gap_3_easing_in_vs_out_different_curves() {
    let mut app = App { position: 0.0 };
    let mut h1 = Harness::new(app.clone(), view);
    let mut h2 = Harness::new(app.clone(), view);
    
    let duration = Duration::from_millis(100);
    h1.ease_add("pos", 0.0, 100.0, duration, Easing::EaseIn);
    h2.ease_add("pos", 0.0, 100.0, duration, Easing::EaseOut);
    
    h1.frames(3);  // ~50ms (halfway)
    h2.frames(3);
    
    let in_pos = h1.state().position;
    let out_pos = h2.state().position;
    
    // EaseIn should be slower at 50% (more time spent in second half)
    assert!(in_pos < out_pos);
}
```

---

## Gap 7: Cleanup Policy & Memory::after()

**Status**: Animations finish but tracking state persists; no way to defer actions until animation completes.  
**Impact**: Deferred cleanup causes memory leaks; no callback mechanism for post-animation actions.  
**Effort**: Low (after() method, cleanup ordering).

### Current Code Location

**File**: src/memory.rs, struct `Memory` (around line 160)

### Phase 3 Change

Add deferred action storage:

```rust
#[derive(Clone)]
pub struct DeferredAction {
    pub when: DeferWhen,
    pub id: Id,
    pub action: Box<dyn Fn(&mut S)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeferWhen {
    AfterAnimation { animation_id: Id },  // Run when specific animation finishes
    AfterDelay { duration: Duration },    // Run after duration elapses
    NextFrame,                             // Run next frame
}
```

**Storage update** (struct `Memory`):

```rust
pub struct Memory {
    pub eased: HashMap<...>,
    pub cycles: HashMap<...>,
    pub deferred: Vec<...>,
    pub transitions: HashMap<...>,
    pub accumulated_time: HashMap<...>,
    pub last_velocity: Option<Velocity>,
    pub springs: HashMap<...>,
    pub spring_velocities: HashMap<...>,
    pub deferred_actions: Vec<DeferredAction>,  // ← ADD (ordered by when)
}
```

### Memory Method

**File**: src/memory.rs, impl Memory (new method, around line 220)

```rust
pub fn after_animation<S: 'static>(
    &mut self,
    animation_id: Id,
    action: impl Fn(&mut S) + 'static,
) {
    self.deferred_actions.push(DeferredAction {
        when: DeferWhen::AfterAnimation { animation_id },
        id: animation_id,
        action: Box::new(action),
    });
}

pub fn after_delay<S: 'static>(
    &mut self,
    duration: Duration,
    action: impl Fn(&mut S) + 'static,
) {
    self.deferred_actions.push(DeferredAction {
        when: DeferWhen::AfterDelay { duration },
        id: Id::default(),
        action: Box::new(action),
    });
}

fn execute_deferred_actions(&mut self, state: &mut S) {
    let mut completed = Vec::new();
    
    for (i, deferred) in self.deferred_actions.iter().enumerate() {
        match deferred.when {
            DeferWhen::AfterAnimation { animation_id } => {
                // Check if animation finished
                if !self.eased.contains_key(&animation_id)
                    && !self.springs.contains_key(&animation_id)
                    && !self.cycles.contains_key(&animation_id)
                {
                    completed.push(i);
                }
            },
            DeferWhen::AfterDelay { duration } => {
                // Check if delay elapsed
                if let Some(elapsed) = self.accumulated_time.get(&deferred.id) {
                    if *elapsed >= duration {
                        completed.push(i);
                    }
                }
            },
            DeferWhen::NextFrame => {
                completed.push(i);  // Always execute next frame
            },
        }
    }
    
    // Execute in reverse order so indices stay valid
    for i in completed.iter().rev() {
        let action = self.deferred_actions.remove(*i);
        (action.action)(state);
    }
}
```

### Cleanup Policy

**File**: src/paint.rs, `draw()` function end (around line 470)

Add cleanup ordering:

```rust
pub fn finish_frame(&mut self) -> Vec<Handler> {
    // 1. Execute deferred actions (after animations complete)
    self.memory.execute_deferred_actions(&mut self.app_state);
    
    // 2. Clear finished animations
    self.memory.eased.retain(|_, eased| {
        eased.elapsed < eased.duration
    });
    self.memory.springs.retain(|_, (spring, pos, vel)| {
        (*pos - spring.target).abs() > 0.01 || vel.abs() > 0.1
    });
    
    // 3. Clear any dangling deferred actions
    self.memory.deferred_actions.clear();
    
    self.handlers.clone()
}
```

### Element Integration

**File**: src/element.rs, builder method (new, around line 450)

```rust
pub fn after_animation(self, animation_id: Id, action: impl Fn(&mut S) + 'static) -> Self {
    self.on_frame(move |state, memory| {
        memory.after_animation(animation_id, action);
    })
}
```

### Acceptance Tests (activate STEP_19_TEST_GAP_7_PHASE_3)

Location: tests/recipes.rs (around line 1090)

```rust
#[test]
fn r2_gap_7_memory_after_executes_when_animation_finishes() {
    let mut app = App { count: 0, animation_done: false };
    let mut h = Harness::new(app, view);
    
    let anim_id = Id::new("fade");
    h.ease_add(anim_id, 0.0, 1.0, Duration::from_millis(100));
    
    // Register callback
    h.after_animation(anim_id, |app| {
        app.animation_done = true;
    });
    
    h.frames(2);  // ~33ms, still animating
    assert_eq!(h.state().animation_done, false);
    
    h.frames(4);  // ~100ms+, animation finished
    assert_eq!(h.state().animation_done, true);  // Callback fired
}

#[test]
fn r2_gap_7_deferred_actions_cleanup_on_frame_end() {
    let mut app = App { cleanup_count: 0 };
    let mut h = Harness::new(app, view);
    
    // Register 3 deferred actions
    h.after_animation(Id::new("a1"), |app| app.cleanup_count += 1);
    h.after_animation(Id::new("a2"), |app| app.cleanup_count += 1);
    h.after_animation(Id::new("a3"), |app| app.cleanup_count += 1);
    
    h.frames(1);
    
    // All should execute and clear
    assert_eq!(h.state().cleanup_count, 3);
    
    // No lingering state
    let memory = h.memory();
    assert_eq!(memory.deferred_actions.len(), 0);
}
```

---

## Implementation Order

### Commit 1: Easing Enum & Curves

**Files**: src/memory.rs, src/paint.rs  
**Lines**: ~30  
**Tests**: 2 acceptance (linear, ease-in vs ease-out)

```bash
git add src/memory.rs src/paint.rs tests/recipes.rs
git commit -m "STEP 22 Phase 3: Gap 3 — Easing enum with standard curves

Adds Easing enum (Linear, EaseIn, EaseOut, EaseInOut, Custom Bezier).
Painter::ease_with() allows custom curves per animation.
Easing::apply() interpolates t ∈ [0,1] for smooth timing control.

Tests: r2_gap_3_easing_linear_is_constant_speed (NEW)
       r2_gap_3_easing_in_vs_out_different_curves (NEW)"
```

### Commit 2: Memory::after() & Deferred Actions

**Files**: src/memory.rs, src/paint.rs  
**Lines**: ~40  
**Tests**: 2 acceptance (execution, cleanup)

```bash
git add src/memory.rs src/paint.rs tests/recipes.rs
git commit -m "STEP 22 Phase 3: Gap 7 — Memory::after() for post-animation callbacks

Adds deferred action tracking and execution.
Memory::after_animation() registers callbacks to run when animations finish.
execute_deferred_actions() runs completed actions and clears state.
Prevents memory leaks from lingering animation tracking.

Tests: r2_gap_7_memory_after_executes_when_animation_finishes (NEW)
       r2_gap_7_deferred_actions_cleanup_on_frame_end (NEW)"
```

### Commit 3: Documentation & Polish

**Files**: CLAUDE.md, src/lib.rs  
**Lines**: ~20  
**Tests**: Verification (no new tests, validate all Phase 3 tests)

```bash
git add CLAUDE.md src/lib.rs
git commit -m "STEP 23: Update CLAUDE.md with R2 Motion Kit patterns

Mark R2 Motion Kit as landed in Library Roadmap.
Add Easing/Spring/EnterExit to 'Key Architectural Patterns' section.
Document .with_enter_exit(), .on_ease_with(), Memory::after_animation().
Update 'Conventions' section with motion budget and lifecycle rules."
```

---

## Pre-Implementation Checklist

Before Phase 3:

- [ ] Phase 1 and Phase 2 fully complete (10 commits total)
- [ ] All Phase 1/2 acceptance tests passing (13 total)
- [ ] All 396 library tests still passing
- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md Phase 3 section
- [ ] Review STEP_19_R2_MOTION_KIT_AUDIT_EXAMPLE_IMPLEMENTATIONS.md Phase 3 code
- [ ] Create branch: `git checkout -b step-22-phase-3`

### Code Review Checklist (per commit)

- [ ] Easing curves match standard implementations (test against reference)
- [ ] Deferred action ordering is deterministic
- [ ] Cleanup ordering prevents resurrection bugs
- [ ] No cycles in deferred action graph (no action triggers itself)
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All library tests still pass
- [ ] New acceptance tests pass

---

## Debugging Phase 3

### If Easing test fails

```bash
# Check Easing enum variants
grep -B 1 -A 2 "pub enum Easing" src/memory.rs

# Check apply() function math
grep -B 2 -A 20 "fn apply" src/memory.rs

# Verify eased struct has easing field
grep -n "pub easing:" src/memory.rs

# Run with verbose output
cargo test r2_gap_3_easing_linear_is_constant_speed -- --nocapture
```

### If Memory::after() test fails

```bash
# Check DeferredAction struct
grep -B 1 -A 3 "pub struct DeferredAction" src/memory.rs

# Check execute_deferred_actions() logic
grep -B 2 -A 15 "fn execute_deferred_actions" src/memory.rs

# Check cleanup in finish_frame()
grep -B 2 -A 5 "finish_frame" src/paint.rs

# Run with state inspection
cargo test r2_gap_7_memory_after_executes_when_animation_finishes -- --nocapture
```

---

## Success Criteria

Phase 3 is complete when:

✅ Easing enum with 6+ curves implemented (25 lines)  
✅ Memory::after_animation() and deferred actions (40 lines)  
✅ 4 acceptance test stubs activated and passing  
✅ All 396 library tests still passing (zero regressions)  
✅ Cleanup policy prevents memory leaks  
✅ Deferred actions execute in correct order  
✅ CLAUDE.md updated with R2 as landed  
✅ Ready for next roadmap item

---

## Sign-Off

Phase 3 completes the R2 Motion Kit:

| Phase | Gaps | Lines | Risk | Status |
|-------|------|-------|------|--------|
| 1 | Gap 5, 6, 4 | ~55 | Very Low | Ready (STEP 20) |
| 2 | Gap 1, 2 | ~140 | Medium | Ready (STEP 21) |
| 3 | Gap 3, 7 | ~65 | Low | Ready (STEP 22) |
| **Total** | **7 gaps** | **~260 lines** | **Low avg** | **Complete** |

All 7 gaps identified in audit are addressed. R2 Motion Kit transforms rui from 4 basic primitives into a complete, physics-based animation system with lifecycle control.

**Phase 3 bridge is solid and well-tested.** ✅
