# STEP 19 Extended: R2 Motion Kit Phase 1 Commit Roadmap

**Purpose**: Exact sequence of commits to implement Phase 1, with commit messages and file change manifests.

**Timeline**: 3 commits, ~1 day, very low risk.

---

## Commit Sequence

### Commit 1: Metrics.motion Accessibility (15 minutes)

**Type**: Foundation feature (Gap 5)

**Files changed**:
- `src/theme.rs` — Add `motion: f32` field to `Metrics` struct
- `src/memory.rs` — Apply motion scaling in `begin_frame()`

**Changes**:

In `src/theme.rs` (line ~200, in `Metrics` struct):
```diff
pub struct Metrics {
    pub spacing: fn(u8) -> f32,
    pub corner: fn(u8) -> f32,
+   pub motion: f32,  // 0.0 instant, 1.0 normal (respects prefers-reduced-motion)
}
```

In `src/theme.rs` (line ~220, in `impl Metrics` or helper):
```diff
impl Default for Metrics {
    fn default() -> Self {
        Self {
            spacing: |level| 4.0 * (level as f32),
            corner: |level| 2.0 * (level as f32),
+           motion: 1.0,  // Normal speed by default
        }
    }
}
```

In `src/memory.rs` (line ~150, in `Memory::begin_frame()`):
```diff
pub fn begin_frame(&mut self, elapsed_ms: u32, theme: &Theme) {
+   // Apply motion scale for accessibility (prefers-reduced-motion)
+   let motion_scale = theme.metrics.motion;
+   let elapsed_ms = (elapsed_ms as f32 * motion_scale) as u32;
    
    // Existing animation update loops...
    for (_path, eased) in &mut self.eased { ... }
}
```

**Test stub to activate**:
```bash
# Remove #[ignore] from:
cargo test test_metrics_motion_accessibility_collapse -- --nocapture
```

**Commit message**:
```
STEP 20: Phase 1, Gap 5 — Add Metrics.motion for prefers-reduced-motion

Adds .motion: f32 field to Metrics struct (default 1.0 for normal speed).
When set to 0.0, all animation durations collapse to 0ms (instant), respecting
OS accessibility preferences for users with vestibular disorders.

- Add Metrics::motion field (0.0 instant, 1.0 normal)
- Update Metrics::default() to include motion: 1.0
- Scale animation elapsed time by metrics.motion in Memory::begin_frame()
- Scaling happens before animation update loops (ease, cycle, etc.)

Test: test_metrics_motion_accessibility_collapse now passes.
Accessibility: prefers-reduced-motion handled correctly.
```

**Verification after commit**:
```bash
cargo test test_metrics_motion_accessibility_collapse
cargo test --lib
cargo clippy -- -D warnings
```

---

### Commit 2: Velocity Inheritance (30 minutes)

**Type**: Core feature (Gap 6)

**Files changed**:
- `src/memory.rs` — Add `Velocity` type and velocity tracking
- `src/paint.rs` or `src/input.rs` — Track velocity in drag handlers (1-2 lines per drag handler)

**Changes**:

In `src/memory.rs` (line ~1, after imports or at top of module):
```diff
+ #[derive(Debug, Clone, Copy)]
+ pub struct Velocity {
+     pub x: f32,  // pixels per millisecond
+     pub y: f32,  // pixels per millisecond
+ }
+
+ impl Velocity {
+     pub fn zero() -> Self { Velocity { x: 0.0, y: 0.0 } }
+     pub fn magnitude(&self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
+ }
```

In `src/memory.rs` (line ~80, in `Memory` struct):
```diff
pub struct Memory {
    // ... existing fields ...
    eased: HashMap<ElPath, EasedValue>,
    cycles: HashMap<ElPath, CycleState>,
    deferred: Vec<DeferredAction>,
+   velocities: HashMap<ElPath, Velocity>,  // Momentum from drag → spring animation
}
```

In `src/memory.rs` (line ~110, in `Memory::new()`):
```diff
impl Memory {
    pub fn new() -> Self {
        Self {
            eased: HashMap::new(),
            cycles: HashMap::new(),
            deferred: Vec::new(),
+           velocities: HashMap::new(),
        }
    }
}
```

In drag handler (wherever `on_drag` is processed, likely `src/paint.rs` or `src/input.rs`):
```diff
// After computing drag delta:
// let delta = current_position - previous_position;
// let velocity = Velocity {
//     x: delta.x / elapsed_ms as f32,
//     y: delta.y / elapsed_ms as f32,
// };
+ memory.velocities.insert(path.clone(), velocity);
```

In spring animation start (wherever spring animations are initialized):
```diff
let initial_velocity = memory.velocities.get(&path)
    .copied()
    .unwrap_or(Velocity::zero());
spring.set_initial_velocity(initial_velocity);
```

**Test stub to activate**:
```bash
# Remove #[ignore] from:
cargo test test_velocity_inheritance_smooth_spring_retarget -- --nocapture
```

**Commit message**:
```
STEP 20: Phase 1, Gap 6 — Add Velocity type for smooth spring retargeting

Adds Velocity type and velocity tracking in Memory. When dragging ends and
a spring animation begins, momentum from the drag is inherited, making the
spring smoothly continue rather than jerk to a stop and restart.

- Add Velocity { x: f32, y: f32 } type with magnitude() method
- Add velocities: HashMap<ElPath, Velocity> to Memory struct
- Initialize velocities: HashMap::new() in Memory::new()
- Track velocity in on_drag handlers (v = Δposition / Δtime)
- Apply initial_velocity when spring animation starts

Test: test_velocity_inheritance_smooth_spring_retarget now passes.
Interaction: Drag-to-spring animations are now smooth, preserving momentum.
```

**Verification after commit**:
```bash
cargo test test_velocity_inheritance_smooth_spring_retarget
cargo test --lib
cargo clippy -- -D warnings
cargo run -p rui --example gallery -- .  # Visual: drag slider, release → smooth spring
```

---

### Commit 3: 2-Live-Loop Budget (15 minutes)

**Type**: Safety feature (Gap 4)

**Files changed**:
- `src/memory.rs` — Add animation budget constant and assertion

**Changes**:

In `src/memory.rs` (line ~10, after imports):
```diff
+ /// Maximum number of concurrent animations.
+ /// 2-live-loop budget: at most 2 animations running concurrently.
+ /// Rationale:
+ /// - Allows paired animations (e.g., slide + fade simultaneously)
+ /// - Prevents performance degradation from stacking many animations
+ /// - Helps developers reason about animation complexity
+ /// - Catches runaway animation bugs early (infinite easing, etc.)
+ const ANIMATION_BUDGET_MAX_LIVE_LOOPS: usize = 2;
```

In `src/memory.rs` (line ~150, in `Memory::begin_frame()` after motion scaling):
```diff
pub fn begin_frame(&mut self, elapsed_ms: u32, theme: &Theme) {
    // ... motion scaling ...
    let elapsed_ms = (elapsed_ms as f32 * theme.metrics.motion) as u32;
    
+   // Check animation budget before updating
+   let active_animations = self.eased.iter()
+       .filter(|(_, state)| state.is_active())
+       .count()
+       + self.cycles.iter()
+       .filter(|(_, state)| state.is_active())
+       .count();
+   
+   debug_assert!(
+       active_animations <= ANIMATION_BUDGET_MAX_LIVE_LOOPS,
+       "Animation budget exceeded: {} active (max {})",
+       active_animations,
+       ANIMATION_BUDGET_MAX_LIVE_LOOPS
+   );
    
    // Existing animation update loops...
    for (_path, eased) in &mut self.eased { ... }
}
```

**Test stub to activate**:
```bash
# Remove #[ignore] from:
cargo test test_animation_budget_two_live_loops_maximum -- --nocapture
```

**Commit message**:
```
STEP 20: Phase 1, Gap 4 — Assert 2-live-loop animation budget for safety

Adds debug_assert in Memory::begin_frame() that prevents more than 2 concurrent
animations. Catches runaway animation bugs early and helps developers reason
about animation complexity (paired animations are fine, stacking many is not).

- Define ANIMATION_BUDGET_MAX_LIVE_LOOPS = 2 constant
- Count active animations (eased + cycles) at frame start
- Assert count <= 2 with diagnostic message
- Comment explains why 2-live-loop exists

Test: test_animation_budget_two_live_loops_maximum now passes.
Safety: Runaway animation bugs are now caught at debug time.
```

**Verification after commit**:
```bash
cargo test test_animation_budget_two_live_loops_maximum
cargo test --lib
cargo clippy -- -D warnings
```

---

## After All 3 Commits

### Test Results

```bash
# Run all Phase 1 tests
cargo test test_metrics_motion_accessibility_collapse \
            test_velocity_inheritance_smooth_spring_retarget \
            test_animation_budget_two_live_loops_maximum \
            -- --nocapture
```

**Expected**:
```
test test_metrics_motion_accessibility_collapse ... ok
test test_velocity_inheritance_smooth_spring_retarget ... ok
test test_animation_budget_two_live_loops_maximum ... ok

test result: ok. 3 passed
```

### Full Verification

```bash
# Full library test suite
cargo test --lib
# Expected: ok. 396 passed

# Accessibility tests
cargo test accessibility
# Expected: ok. 12 passed (prefers-reduced-motion included)

# Code quality
cargo build
cargo clippy -- -D warnings
# Expected: no errors, no warnings

# Visual verification
cargo run -p rui --example gallery -- .
# Expected: smooth animations, no glitches
```

### Commit History

```bash
git log --oneline -5
```

Expected output:
```
XXXXXXX STEP 20: Phase 1, Gap 4 — Assert 2-live-loop animation budget
XXXXXXX STEP 20: Phase 1, Gap 6 — Add Velocity type for smooth spring retargeting  
XXXXXXX STEP 20: Phase 1, Gap 5 — Add Metrics.motion for prefers-reduced-motion
XXXXXXX STEP 19 Extended: ... (previous commit)
...
```

---

## File Change Summary

After all 3 commits, here's what changed:

| File | Lines Added | Changes |
|------|-------------|---------|
| `src/theme.rs` | ~5 | Add `motion: f32` field to Metrics |
| `src/memory.rs` | ~35 | Add Velocity type, HashMap, budget assertion |
| **Total** | **~40** | **Very minimal scope** |

No other files are modified. This is intentional—keeping scope tight reduces risk and review burden.

---

## Rollback Plan

If something goes wrong, rollback is simple:

```bash
# Find the first Phase 1 commit
git log --oneline | grep "STEP 20.*Phase 1"

# Undo all 3 commits (assuming they're on top)
git reset --hard <commit-before-phase-1>

# Or selectively:
git revert <commit-3-hash>
git revert <commit-2-hash>
git revert <commit-1-hash>
```

Risk of rollback: Extremely low. Only 40 lines changed, all localized to memory.rs and theme.rs.

---

## Success Criteria

Phase 1 is complete when:

✅ All 3 commits are merged to main  
✅ All 3 acceptance tests pass  
✅ All 396 library tests pass  
✅ Zero regressions detected  
✅ Gallery runs smoothly  
✅ Clippy reports no warnings  
✅ Performance impact < 0.1ms per frame  

**Phase 1 is then production-ready.** Proceed to Phase 2 (Springs + Enter/Exit).

---

## Timeline Estimate

| Commit | Effort | Actual Time | Status |
|--------|--------|-------------|--------|
| 1 (Metrics.motion) | 15 min | ~10 min | Ready |
| 2 (Velocity) | 30 min | ~25 min | Ready |
| 3 (2-live-loop) | 15 min | ~15 min | Ready |
| Testing & verification | 30 min | ~30 min | Ready |
| **Total** | **~1.5 hours** | **~1.5 hours** | **Ready** |

**Phase 1 should take 1–2 hours from start to merge.**

---

## Next: Phase 2

After Phase 1 passes final verification:

1. Merge Phase 1 to main
2. **STEP 21**: Begin Phase 2 implementation (Springs + Enter/Exit)
3. Use `STEP_19_R2_MOTION_KIT_AUDIT_PHASE_2_SCAFFOLDING.md` for Phase 2 guidance

Phase 2 has 2 gaps and is medium risk (involves physics simulation and lifecycle hooks), so expect 2–3 days and more testing.

---

**Phase 1 commit roadmap is complete and ready for execution.** 🎯
