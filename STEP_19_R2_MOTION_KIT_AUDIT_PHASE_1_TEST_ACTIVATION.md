# STEP 19 Extended: R2 Motion Kit Phase 1 Test Activation Guide

**Purpose**: Exact instructions for activating Phase 1 acceptance tests and linking them to implementation commits.

**Timeline**: 3-5 commits, ~1 day of implementation.

---

## Test Activation Strategy

Phase 1 has 3 acceptance test stubs (one per gap) currently marked `#[ignore]`. These tests:
- Show the expected R2 API after Phase 1
- Guide implementation step-by-step
- Block merge if Phase 1 isn't complete

### Activation Order

Activate tests in this order, implementing each gap before its test:

| Commit # | Gap | Test File | Test Name | Status |
|----------|-----|-----------|-----------|--------|
| 1 | Gap 5 (Metrics.motion) | `tests/recipes.rs` | `test_metrics_motion_accessibility_collapse` | `#[ignore]` → activate |
| 2 | Gap 6 (Velocity) | `tests/recipes.rs` | `test_velocity_inheritance_smooth_spring_retarget` | `#[ignore]` → activate |
| 3 | Gap 4 (2-live-loop) | `tests/recipes.rs` | `test_animation_budget_two_live_loops_maximum` | `#[ignore]` → activate |

---

## Commit 1: Metrics.motion Accessibility

### Pre-Implementation Checklist

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_PHASE_1_SCAFFOLDING.md (Metrics.motion section)
- [ ] Check test location: `tests/recipes.rs` line XXX (find test named `test_metrics_motion_accessibility_collapse`)
- [ ] Verify gap: `src/theme.rs` Metrics struct missing `.motion: f32` field

### Implementation Steps

**Step 1**: Add `motion` field to `Metrics` struct
```rust
pub struct Metrics {
    pub spacing: fn(u8) -> f32,    // existing
    pub corner: fn(u8) -> f32,     // existing
    pub motion: f32,                // NEW: 1.0 normal, 0.0 instant (prefers-reduced-motion)
}
```

**Step 2**: Update `with_metrics()` builder method in `Theme`
```rust
pub fn with_metrics(mut self, metrics: Metrics) -> Self {
    self.metrics = metrics;
    self  // returns Theme with new metrics including .motion
}
```

**Step 3**: Use in `Memory::begin_frame()` - scale animation velocity by `theme.metrics.motion`
```rust
// In Memory::begin_frame(), when processing eased animations:
let motion_scale = theme.metrics.motion;  // 0.0 = instant, 1.0 = normal
elapsed_ms = (elapsed_ms as f32 * motion_scale) as u32;
```

### Test Activation

1. Open `tests/recipes.rs`
2. Find: `#[ignore]` on line `test_metrics_motion_accessibility_collapse`
3. Delete the `#[ignore]` attribute
4. Run: `cargo test test_metrics_motion_accessibility_collapse -- --nocapture`
5. Expected: **PASS**

### Verification

```bash
# Test passes
cargo test test_metrics_motion_accessibility_collapse

# Accessibility audit passes
cargo test accessibility::tests::the_theme_respects_prefers_reduced_motion
```

### Commit Message

```
STEP 20: Phase 1, Gap 5 — Add Metrics.motion for prefers-reduced-motion accessibility

Adds .motion: f32 field to Metrics struct. When set to 0.0, all animation
durations collapse to 0ms (instant), respecting OS accessibility preferences
for users with vestibular disorders or cognitive overload.

- Add Metrics::motion field (0.0 instant, 1.0 normal)
- Update Theme::with_metrics() to accept new field
- Scale animation elapsed time by metrics.motion in Memory::begin_frame()
- Add accessibility audit check: theme.metrics.motion >= 0.0

Test: test_metrics_motion_accessibility_collapse now passes.
```

---

## Commit 2: Velocity Inheritance

### Pre-Implementation Checklist

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_PHASE_1_SCAFFOLDING.md (Velocity section)
- [ ] Check test location: `tests/recipes.rs` line XXX (find test named `test_velocity_inheritance_smooth_spring_retarget`)
- [ ] Verify gap: `src/memory.rs` missing `Velocity` type and velocity storage

### Implementation Steps

**Step 1**: Add `Velocity` type to `src/memory.rs`
```rust
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub x: f32,  // pixels per millisecond
    pub y: f32,  // pixels per millisecond
}

impl Velocity {
    pub fn zero() -> Self { Velocity { x: 0.0, y: 0.0 } }
    pub fn magnitude(&self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
}
```

**Step 2**: Add velocity storage to `Memory` struct
```rust
pub struct Memory {
    // existing fields...
    eased: HashMap<ElPath, EasedValue>,
    cycles: HashMap<ElPath, CycleState>,
    deferred: Vec<DeferredAction>,
    // NEW:
    velocities: HashMap<ElPath, Velocity>,  // Tracks momentum for spring animations
}
```

**Step 3**: Track velocity in drag handlers
```rust
// In on_drag handlers, compute velocity as dx/dt:
// velocity = (current_position - previous_position) / elapsed_ms
// Store in memory.velocities[path]
```

**Step 4**: Apply velocity when transitioning to spring animation
```rust
// When starting a spring animation with stored velocity:
let initial_velocity = memory.velocities.get(&path).copied().unwrap_or(Velocity::zero());
spring.set_initial_velocity(initial_velocity);
```

### Test Activation

1. Open `tests/recipes.rs`
2. Find: `#[ignore]` on line `test_velocity_inheritance_smooth_spring_retarget`
3. Delete the `#[ignore]` attribute
4. Run: `cargo test test_velocity_inheritance_smooth_spring_retarget -- --nocapture`
5. Expected: **PASS**

### Verification

```bash
# Test passes
cargo test test_velocity_inheritance_smooth_spring_retarget

# Drag-to-spring animation is smooth (no jerk at transition)
cargo run -p rui --example gallery -- .
# Manually: drag a slider, release, watch spring smoothly continue momentum
```

### Commit Message

```
STEP 20: Phase 1, Gap 6 — Add Velocity type for smooth spring retargeting

Adds Velocity type and velocity tracking in Memory. When dragging ends and
a spring animation begins, momentum from the drag is inherited, making the
spring smoothly continue rather than jerk to a stop and restart.

- Add Velocity { x: f32, y: f32 } type with magnitude() method
- Add velocities: HashMap<ElPath, Velocity> to Memory
- Track velocity in on_drag handlers (v = Δposition / Δtime)
- Apply initial_velocity when spring animation starts

Test: test_velocity_inheritance_smooth_spring_retarget now passes.
```

---

## Commit 3: 2-Live-Loop Budget

### Pre-Implementation Checklist

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_PHASE_1_SCAFFOLDING.md (2-live-loop section)
- [ ] Check test location: `tests/recipes.rs` line XXX (find test named `test_animation_budget_two_live_loops_maximum`)
- [ ] Verify gap: `src/memory.rs` missing animation budget assertion

### Implementation Steps

**Step 1**: Define animation budget constant
```rust
// In src/memory.rs at module level
const ANIMATION_BUDGET_MAX_LIVE_LOOPS: usize = 2;
// "2-live-loop budget": at most 2 animations running concurrently
```

**Step 2**: Count active animations in `Memory::begin_frame()`
```rust
// Count animations that are currently Active (not Completed/Dead)
let active_animations = self.eased.iter().filter(|(_, state)| state.is_active()).count()
                      + self.cycles.iter().filter(|(_, state)| state.is_active()).count();

debug_assert!(
    active_animations <= ANIMATION_BUDGET_MAX_LIVE_LOOPS,
    "Animation budget exceeded: {} active (max {})",
    active_animations,
    ANIMATION_BUDGET_MAX_LIVE_LOOPS
);
```

**Step 3**: Document why 2-live-loop exists
```rust
// Comment above the assertion:
// Why 2-live-loop budget?
// - Allows paired animations (e.g., slide + fade simultaneously)
// - Prevents performance degradation from stacking many animations
// - Helps developers reason about animation complexity
// - Catches runaway animation bugs early (infinite easing, etc.)
```

### Test Activation

1. Open `tests/recipes.rs`
2. Find: `#[ignore]` on line `test_animation_budget_two_live_loops_maximum`
3. Delete the `#[ignore]` attribute
4. Run: `cargo test test_animation_budget_two_live_loops_maximum -- --nocapture`
5. Expected: **PASS**

### Verification

```bash
# Test passes
cargo test test_animation_budget_two_live_loops_maximum

# Library tests still pass (no false positives)
cargo test --lib

# Run the gallery with no animation budget violations
cargo run -p rui --example gallery -- .
```

### Commit Message

```
STEP 20: Phase 1, Gap 4 — Assert 2-live-loop animation budget for safety

Adds debug_assert in Memory::begin_frame() that prevents more than 2 concurrent
animations. Catches runaway animation bugs early and helps developers reason
about animation complexity (paired animations are fine, stacking many is not).

- Define ANIMATION_BUDGET_MAX_LIVE_LOOPS = 2 constant
- Count active animations at frame start
- Assert count <= 2 with diagnostic message
- Document why 2-live-loop exists (perf, reasoning, paired animations)

Test: test_animation_budget_two_live_loops_maximum now passes.
```

---

## Phase 1 Verification Checklist

After all 3 commits are complete, verify:

- [ ] All 3 acceptance tests pass: `cargo test test_metrics_motion_accessibility_collapse test_velocity_inheritance_smooth_spring_retarget test_animation_budget_two_live_loops_maximum`
- [ ] No regressions: `cargo test --lib` (all 396 tests pass)
- [ ] Code compiles: `cargo build`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Accessibility audit passes: `cargo test accessibility`
- [ ] Gallery runs smoothly: `cargo run -p rui --example gallery -- .`
- [ ] All 3 commits have meaningful messages following project convention
- [ ] All changes are in src/memory.rs and src/theme.rs only (no extraneous files)

---

## If a Test Fails

**Common issue: Test expects `Metrics { ..., motion: 1.0 }` but struct doesn't have field**

Fix:
1. Add `motion: f32` field to `Metrics` struct in `src/theme.rs`
2. Update all `Metrics { ... }` constructions to include `motion: 1.0` (for normal motion)
3. Re-run: `cargo test test_metrics_motion_accessibility_collapse`

**Common issue: Velocity test expects `memory.velocities` HashMap but it doesn't exist**

Fix:
1. Add `velocities: HashMap<ElPath, Velocity>` to `Memory` struct
2. Initialize in `Memory::new()`: `velocities: HashMap::new()`
3. Re-run: `cargo test test_velocity_inheritance_smooth_spring_retarget`

**Common issue: Budget test fails with "active_animations > 2"**

Fix:
1. Check if test is starting 3+ animations (should only be 2)
2. Verify eased/cycles are being cleaned up after completion
3. Check that test is stepping time forward (calls `memory.begin_frame(elapsed)`)
4. Re-run: `cargo test test_animation_budget_two_live_loops_maximum`

---

## Phase 1 Summary

| Commit | Gap | Effort | Risk | Time |
|--------|-----|--------|------|------|
| 1 | Metrics.motion | 15 min | Very Low | 15 min |
| 2 | Velocity | 30 min | Very Low | 30 min |
| 3 | 2-live-loop | 15 min | Very Low | 15 min |
| **Total** | **3 gaps** | **Low** | **Very Low** | **~1 hour** |

**Expected outcome after Phase 1**: 3 acceptance tests pass, 396 library tests still pass, zero regressions, animation foundation is solid.

Ready for Phase 2 (springs + enter/exit transitions).
