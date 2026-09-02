# STEP 19 Extended: R2 Motion Kit Phase 1 Acceptance Verification

**Purpose**: Checklist to verify Phase 1 is complete and ready for merge.

**When to use**: After all 3 Phase 1 commits are made, before opening a PR.

---

## Pre-Verification: Code Review

Before running tests, review the code changes:

### Commit 1: Metrics.motion

**Files changed**: `src/theme.rs`

- [ ] `Metrics` struct has new field: `motion: f32`
- [ ] Field is documented: `/// Scale factor for animation durations (0.0 instant, 1.0 normal). Respects prefers-reduced-motion.`
- [ ] `Theme::default().metrics.motion == 1.0` (normal speed by default)
- [ ] `Theme::with_metrics()` accepts and sets `motion` field
- [ ] All `Metrics { ... }` constructions include `motion: value`

**Files changed**: `src/memory.rs`

- [ ] `Memory::begin_frame()` reads `theme.metrics.motion`
- [ ] Elapsed time is scaled: `elapsed_ms = (elapsed_ms as f32 * motion_scale) as u32`
- [ ] Scaling happens *before* animation update loops (ease, cycles, etc.)
- [ ] Animation curves preserve shape (scaling time, not progress)

### Commit 2: Velocity

**Files changed**: `src/memory.rs`

- [ ] `Velocity` type defined: `pub struct Velocity { pub x: f32, pub y: f32 }`
- [ ] `Velocity::zero()` constructor exists
- [ ] `Velocity::magnitude()` method exists: `sqrt(x² + y²)`
- [ ] `Memory` struct has new field: `velocities: HashMap<ElPath, Velocity>`
- [ ] `Memory::new()` initializes: `velocities: HashMap::new()`
- [ ] Drag handlers track velocity: `velocity = (current - previous) / elapsed_ms`
- [ ] Velocity is stored in memory after drag ends: `memory.velocities.insert(path, velocity)`

**Files changed**: `src/paint.rs` or `src/input.rs` (wherever drag is handled)

- [ ] After drag completes, velocity is available for next animation
- [ ] If spring animation starts with stored velocity, it begins with momentum
- [ ] Velocity is cleared when the animated element changes position (to prevent stale data)

### Commit 3: 2-Live-Loop Budget

**Files changed**: `src/memory.rs`

- [ ] Constant defined: `const ANIMATION_BUDGET_MAX_LIVE_LOOPS: usize = 2;`
- [ ] Comment above explains: "2-live-loop budget: at most 2 animations running concurrently"
- [ ] `Memory::begin_frame()` counts active animations:
  - [ ] Counts `eased` animations that are not Completed/Dead
  - [ ] Counts `cycles` animations that are not Completed/Dead
  - [ ] Total = eased_active + cycles_active
- [ ] `debug_assert!` fires if count > 2 with message: `"Animation budget exceeded: {} active (max {})"`

---

## Test Verification

Run these test commands in order:

### Step 1: Run Phase 1 Tests

```bash
cargo test test_metrics_motion_accessibility_collapse \
            test_velocity_inheritance_smooth_spring_retarget \
            test_animation_budget_two_live_loops_maximum \
            -- --nocapture
```

**Expected output**:
```
test test_metrics_motion_accessibility_collapse ... ok
test test_velocity_inheritance_smooth_spring_retarget ... ok
test test_animation_budget_two_live_loops_maximum ... ok

test result: ok. 3 passed
```

- [ ] All 3 tests pass
- [ ] No `FAILED` or `IGNORED` tests
- [ ] All tests run in < 5 seconds total

### Step 2: Run Full Library Test Suite

```bash
cargo test --lib
```

**Expected output**:
```
test result: ok. 396 passed; 0 failed
```

- [ ] All 396 tests pass
- [ ] Zero regressions
- [ ] No new `FAILED` tests
- [ ] Execution time is similar to before Phase 1 (within 5% variance)

### Step 3: Run Accessibility Tests

```bash
cargo test accessibility::tests::
```

**Expected output**:
```
test result: ok. 12 passed; 0 failed
```

- [ ] All accessibility tests pass (especially prefers-reduced-motion)
- [ ] No new accessibility violations
- [ ] `the_theme_respects_prefers_reduced_motion` passes

### Step 4: Compile and Lint

```bash
cargo build
cargo clippy -- -D warnings
```

**Expected output**:
```
Finished dev [unoptimized + debuginfo] target(s)
error: could not compile `rui` (or success if no warnings)
```

- [ ] Compilation succeeds
- [ ] No clippy warnings
- [ ] No clippy errors
- [ ] Code is formatted: `cargo fmt --check` (or auto-format: `cargo fmt`)

### Step 5: Visual Verification

```bash
cargo run -p rui --example gallery -- .
```

**Expected behavior**:
- [ ] Gallery starts and displays correctly
- [ ] All animations still work (slide, fade, etc.)
- [ ] No visual glitches or jank
- [ ] Smooth interaction (no dropped frames)
- [ ] On slower machines or with "Reduce Motion" OS setting enabled:
  - [ ] Animations still work but are instant (or very fast)
  - [ ] No broken UI or layout shifts

---

## Code Quality Verification

### Documentation

- [ ] All new types have doc comments: `/// ...`
- [ ] All new fields have doc comments explaining the range (e.g., "0.0 to 1.0")
- [ ] `Metrics::motion` doc explains prefers-reduced-motion accessibility
- [ ] `Velocity` doc explains momentum inheritance
- [ ] Animation budget doc explains why 2 is the limit

### Naming

- [ ] `motion` is clear (not `animation_scale` or `speed_factor`)
- [ ] `Velocity` matches physics terminology (vs `Momentum` or `Drag`)
- [ ] `ANIMATION_BUDGET_MAX_LIVE_LOOPS` is explicit and self-documenting

### Error Messages

- [ ] Animation budget assertion message includes actual count and max: `"Animation budget exceeded: 3 active (max 2)"`
- [ ] No generic panics or assertions
- [ ] Errors are debuggable (include context like ElPath if applicable)

---

## Integration Verification

### Metrics.motion Integration

- [ ] Theme can be constructed with custom motion: `Theme::default().with_metrics(Metrics { motion: 0.5, ... })`
- [ ] Motion value affects animation speed correctly (0.0 = instant, 0.5 = half speed, 1.0 = normal)
- [ ] All animation types respect motion: ease, phase, cycles, deferred, transitions
- [ ] When motion=0.0, animations complete in 0ms (instant state changes)
- [ ] Motion value doesn't affect animation curve shape, only timing

### Velocity Integration

- [ ] Velocity is captured after every drag gesture
- [ ] Velocity is available when spring animation starts
- [ ] Spring animation begins with captured velocity (not zero)
- [ ] Releasing a drag mid-spring doesn't duplicate velocity
- [ ] Velocity is cleared when element is removed from tree (no memory leaks)

### Animation Budget Integration

- [ ] Assertion doesn't fire during normal gallery usage
- [ ] Assertion could fire if you try to start 3 concurrent animations (test this intentionally)
- [ ] Animation budget is per-frame, not per-element
- [ ] Starting animation 2, completing animation 1, starting animation 3 works fine (budget is 2 *concurrent*)

---

## Performance Verification

### Frame Time Impact

Run performance benchmarks:

```bash
cargo run -p rui --release --example cost
```

**Expected output**: Frame time and memory metrics

- [ ] Frame time is unchanged (< 0.1ms added per frame)
- [ ] Memory usage is similar (HashMap lookups are O(1))
- [ ] No allocations in hot path (animation update loop)

### Memory Impact

- [ ] `Velocity` type is small (2 × f32 = 8 bytes)
- [ ] `velocities` HashMap holds at most ~100 entries (one per interactive element)
- [ ] No memory leaks when animations complete (HashMap entries cleared)
- [ ] No memory fragmentation from HashMap resizing

---

## Regression Detection

### Animation Curves

- [ ] Easing animations still reach their target values
- [ ] Cycle animations still loop correctly
- [ ] Deferred animations still fire at the right time
- [ ] Transitions still complete in the expected duration

### Accessibility

- [ ] Focus ring animations respect motion setting
- [ ] Hover effects respect motion setting
- [ ] No violations on `cargo test accessibility`

### Interaction

- [ ] Click handlers still fire
- [ ] Drag handlers still update state
- [ ] Keyboard handlers still work
- [ ] Focus navigation still works

---

## Merge Readiness Checklist

Before creating a PR:

- [ ] All Phase 1 tests pass (3/3)
- [ ] All library tests pass (396/396)
- [ ] All accessibility tests pass
- [ ] Code compiles with zero clippy warnings
- [ ] Gallery runs smoothly with no visual glitches
- [ ] Performance impact is negligible (< 0.1ms per frame)
- [ ] All code changes are in `src/theme.rs` and `src/memory.rs` only
- [ ] All new types and fields are documented
- [ ] All commit messages follow project convention
- [ ] All changes are intentional (no accidental formatting, reordering, or cleanup)

---

## If Verification Fails

### Animation Budget Assertion Fires in Tests

**Symptom**: Test fails with `Animation budget exceeded: 3 active (max 2)`

**Root cause**: Test is starting 3+ concurrent animations

**Fix**:
1. Find which test is failing (grep for the test name)
2. Check test setup: is it intentionally starting 3+ animations?
3. If intentional (testing budget overflow), expect the assertion and wrap in `#[should_panic]`
4. If unintentional, separate animations into different frames or reduce animation count

### Velocity Isn't Inherited by Spring Animation

**Symptom**: Spring animation starts from velocity 0 instead of inherited drag velocity

**Root cause**: Velocity not being stored or retrieved correctly

**Debug steps**:
1. Add logging to drag handler: `println!("Velocity stored: {:?}", velocity);`
2. Add logging to spring start: `println!("Spring velocity: {:?}", initial_velocity);`
3. Verify `memory.velocities.insert()` is being called
4. Verify `memory.velocities.get()` is being called with same ElPath

**Fix**:
```bash
# Search for all places velocity is handled:
grep -n "velocities" src/memory.rs
grep -n "Velocity" src/memory.rs
```

### Motion Scaling Isn't Working

**Symptom**: Animations still run at full speed even with `motion: 0.0`

**Root cause**: Time scaling not applied before animation updates

**Debug steps**:
```rust
// In Memory::begin_frame(), add logging:
eprintln!("Motion scale: {}, elapsed_ms: {} → {}", 
    theme.metrics.motion, 
    elapsed_ms, 
    (elapsed_ms as f32 * theme.metrics.motion) as u32);
```

**Fix**: Ensure time scaling happens *before* eased/cycles update loops, not after.

---

## Sign-Off

When all verification passes:

- [ ] You have reviewed all code changes
- [ ] You have run all test suites
- [ ] You have visually verified the gallery
- [ ] You have verified performance impact is negligible
- [ ] You are confident Phase 1 is production-ready

**Phase 1 is approved for merge.** Ready to proceed to Phase 2.

---

## Next Steps

After Phase 1 merge:

1. Merge to `main`
2. Mark STEP 19 as complete in memory
3. Proceed to **STEP 21: Phase 2 Implementation** (Springs + Enter/Exit)
4. Use `STEP_19_R2_MOTION_KIT_AUDIT_PHASE_2_SCAFFOLDING.md` for Phase 2 guidance

**Phase 1 acceptance verification complete.** 🎯
