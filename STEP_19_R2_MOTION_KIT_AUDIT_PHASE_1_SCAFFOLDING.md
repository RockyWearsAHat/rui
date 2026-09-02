# STEP 19 Extended: R2 Motion Kit — Phase 1 Implementation Scaffolding

**Purpose**: Exact code locations, changes needed, and implementation order for Phase 1 (3 gaps). Bridge from audit to implementation with zero ambiguity.

**Phase 1 consists of**: 3 foundation gaps (very low risk, ~1 day, 3-5 commits)

---

## Phase 1 Overview

| Gap | File | Lines | Risk | Commits | Tests |
|-----|------|-------|------|---------|-------|
| Gap 5: Metrics.motion | src/theme.rs, src/memory.rs | ~12 | Very Low | 1 | 3 |
| Gap 6: Velocity type | src/memory.rs, src/paint.rs | ~35 | Very Low | 1 | 4 |
| Gap 4: 2-live-loop budget | src/shell/mod.rs, src/paint.rs | ~8 | Very Low | 1 | 3 |

**Total Phase 1**: ~55 lines of new code, 3 gaps, 3 acceptance tests activated, zero regressions expected.

---

## Gap 5: Metrics.motion Accessibility Check

**Status**: accessibility.audit() reports all animations, but Metrics has no motion configuration option.  
**Impact**: Apps can't collapse animations for `prefers-reduced-motion`.  
**Effort**: Very low (5 additions, 1 method).

### Current Code Location

**File**: src/theme.rs, struct `Metrics` (around line 45)

```rust
pub struct Metrics {
    pub spacing: ...,
    pub height: ...,
    pub corner: ...,
    // ← INSERT HERE
}
```

### Phase 1 Change

Add boolean flag to Metrics:

```rust
pub struct Metrics {
    pub spacing: ...,
    pub height: ...,
    pub corner: ...,
    pub motion: bool,  // ← ADD (default true)
}
```

**Constructor change** (around line 80):

```rust
impl Metrics {
    pub fn new() -> Self {
        Self {
            spacing: ...,
            height: ...,
            corner: ...,
            motion: true,  // ← ADD
        }
    }
}
```

**Getter method** (add after constructor):

```rust
pub fn is_motion_enabled(&self) -> bool {
    self.motion
}

pub fn with_motion(mut self, enabled: bool) -> Self {
    self.motion = enabled;
    self
}
```

### Integration Point

**File**: src/memory.rs, function `begin_frame()` (around line 220)

Add accessibility check:

```rust
pub fn begin_frame(&mut self, elapsed: Duration, theme: &Theme) {
    if !theme.metrics.is_motion_enabled() {
        // Collapse all animations on next frame
        self.eased.clear();
        self.cycles.clear();
        self.deferred.clear();
        self.transitions.clear();
    }
    // ... rest of begin_frame
}
```

### Acceptance Test (activate STEP_19_TEST_GAP_5)

Location: tests/recipes.rs (around line 890)

```rust
#[test]
fn r2_gap_5_metrics_motion_collapses_animations() {
    let mut theme_with_motion = Theme::default();
    let mut theme_no_motion = Theme::default().with_motion(false);
    
    let mut app = App { count: 0 };
    let mut h1 = Harness::new(app.clone(), view);
    let mut h2 = Harness::new(app.clone(), view);
    
    h1.set_theme(theme_with_motion);
    h2.set_theme(theme_no_motion);
    
    // Trigger animation that lasts 500ms
    h1.click_text("Increment");  // starts 500ms ease
    h2.click_text("Increment");  // should NOT start ease
    
    h1.frames(2);
    h2.frames(2);
    
    // h1 should have eased value != target
    // h2 should jump directly to target
    assert!(h1.state().animated_value != h1.state().target);
    assert_eq!(h2.state().animated_value, h2.state().target);
}
```

### Verification Checklist

- [ ] Metrics struct includes `motion: bool`
- [ ] Metrics::new() defaults to `motion: true`
- [ ] Metrics has `.with_motion(bool)` and `.is_motion_enabled()` methods
- [ ] Memory::begin_frame() checks theme.metrics.is_motion_enabled()
- [ ] When motion disabled, all animation containers are cleared
- [ ] Test passes: `cargo test r2_gap_5_metrics_motion_collapses_animations`
- [ ] No regressions: `cargo test --lib` (396 tests pass)

---

## Gap 6: Velocity Type

**Status**: Drag and spring retargeting need velocity inheritance, but no Velocity type exists.  
**Impact**: Springs can't smoothly continue momentum from drag; animations feel disconnected.  
**Effort**: Low (20 lines, 1 type, 2 methods).

### Current Code Location

**File**: src/memory.rs, after struct `Eased` (around line 100)

### Phase 1 Change

Add Velocity type:

```rust
#[derive(Clone, Copy, Debug)]
pub struct Velocity {
    pub value: f32,         // pixels/frame (inject frame time to convert)
    pub direction: f32,     // 1.0 = positive, -1.0 = negative
}

impl Velocity {
    pub fn new(value: f32, direction: f32) -> Self {
        Self { 
            value: value.abs(),
            direction: direction.signum(),
        }
    }
    
    pub fn magnitude(&self) -> f32 {
        self.value * self.direction
    }
    
    pub fn zero() -> Self {
        Self { value: 0.0, direction: 1.0 }
    }
}
```

### Storage Addition

**File**: src/memory.rs, struct `Memory` (around line 140)

```rust
pub struct Memory {
    pub eased: HashMap<...>,
    pub cycles: HashMap<...>,
    pub deferred: Vec<...>,
    pub transitions: HashMap<...>,
    pub accumulated_time: HashMap<...>,
    pub last_velocity: Option<Velocity>,  // ← ADD
}
```

**Constructor** (around line 170):

```rust
impl Memory {
    pub fn new() -> Self {
        Self {
            eased: HashMap::new(),
            cycles: HashMap::new(),
            deferred: Vec::new(),
            transitions: HashMap::new(),
            accumulated_time: HashMap::new(),
            last_velocity: None,  // ← ADD
        }
    }
}
```

### Integration Point

**File**: src/paint.rs, function `on_drag()` handler (around line 520)

```rust
pub fn on_drag(&mut self, drag: Drag) {
    let magnitude = (drag.current - drag.previous).abs();
    let direction = if drag.current > drag.previous { 1.0 } else { -1.0 };
    
    // Store velocity for spring retargeting
    self.memory.last_velocity = Some(Velocity::new(magnitude, direction));
}
```

### Acceptance Test (activate STEP_19_TEST_GAP_6)

Location: tests/recipes.rs (around line 910)

```rust
#[test]
fn r2_gap_6_velocity_inheritance_from_drag() {
    let mut app = App { count: 0, position: 0.0 };
    let mut h = Harness::new(app, view);
    
    // Drag slider quickly (should build velocity)
    h.drag_slider("Position", 100.0);
    
    // Velocity should be captured
    assert!(h.memory().last_velocity.is_some());
    let vel = h.memory().last_velocity.unwrap();
    assert!(vel.value > 0.0);
    assert!(vel.magnitude() != 0.0);
}
```

### Verification Checklist

- [ ] Velocity type defined with value/direction fields
- [ ] Velocity::new() normalizes and signs correctly
- [ ] Velocity::magnitude() returns signed value
- [ ] Memory struct includes `last_velocity: Option<Velocity>`
- [ ] Memory::new() initializes to None
- [ ] on_drag() captures and stores velocity
- [ ] Velocity preserved across frame boundary
- [ ] Test passes: `cargo test r2_gap_6_velocity_inheritance_from_drag`
- [ ] No regressions: `cargo test --lib` (396 tests pass)

---

## Gap 4: 2-Live-Loop Budget

**Status**: No safety limit on concurrent animations; apps can create unlimited simultaneous animations.  
**Impact**: Resource exhaustion, memory bloat, frame rate collapse on large lists.  
**Effort**: Very low (5 assertions, 1 check function).

### Current Code Location

**File**: src/paint.rs, function `draw()` (around line 310)

### Phase 1 Change

Add budget check function:

```rust
fn assert_animation_budget(memory: &Memory) {
    let active_animations = memory.eased.len() 
        + memory.cycles.len()
        + memory.deferred.len()
        + memory.transitions.len();
    
    assert!(
        active_animations <= 2,
        "Animation budget exceeded: {} active (max 2 allowed). \
         This typically means a loop is creating animations per-item. \
         Use .key() for list reordering or defer animation setup.",
        active_animations
    );
}
```

### Integration Point

**File**: src/shell/mod.rs, function `draw()` main loop (around line 305)

```rust
pub fn draw(...) {
    // After painting
    assert_animation_budget(&memory);
    
    // Clear animation state
    memory.eased.clear();
    // ...
}
```

**Alternative Location**: src/paint.rs, function `Painter::finish()` (around line 450)

```rust
pub fn finish(self) -> Vec<Handler> {
    assert_animation_budget(&self.memory);
    self.handlers
}
```

### Acceptance Test (activate STEP_19_TEST_GAP_4)

Location: tests/recipes.rs (around line 930)

```rust
#[test]
#[should_panic(expected = "Animation budget exceeded")]
fn r2_gap_4_budget_panics_on_third_concurrent_animation() {
    let mut app = App::default();
    let mut h = Harness::new(app, view);
    
    // Create 2 animations (OK)
    h.eased_add("anim1", 0.0, 1.0, Duration::from_millis(500));
    h.eased_add("anim2", 0.0, 1.0, Duration::from_millis(500));
    h.frames(1);
    
    // Create 3rd animation (should panic)
    h.eased_add("anim3", 0.0, 1.0, Duration::from_millis(500));
    h.frames(1);  // ← Panic happens here
}
```

### Verification Checklist

- [ ] Budget check function added
- [ ] Check runs in draw() loop or Painter::finish()
- [ ] Asserts active animations <= 2
- [ ] Error message explains the budget constraint
- [ ] Test passes (panics as expected): `cargo test --test recipes -- r2_gap_4_budget_panics --should-panic`
- [ ] Normal operation unaffected: `cargo test --lib` (396 tests pass)

---

## Implementation Order

### Commit 1: Gap 5 (Metrics.motion)

**Files**: src/theme.rs, src/memory.rs  
**Lines**: ~15  
**Tests**: 3 activation

```bash
git add src/theme.rs src/memory.rs tests/recipes.rs
git commit -m "STEP 20 Phase 1: Gap 5 — Metrics.motion accessibility option

Adds Metrics::motion boolean flag to control animation enables.
When disabled (e.g., prefers-reduced-motion), Memory::begin_frame()
collapses all active animations immediately, jumping to targets.

Tests: r2_gap_5_metrics_motion_collapses_animations (NEW)"
```

### Commit 2: Gap 6 (Velocity)

**Files**: src/memory.rs, src/paint.rs  
**Lines**: ~35  
**Tests**: 4 activation

```bash
git add src/memory.rs src/paint.rs tests/recipes.rs
git commit -m "STEP 20 Phase 1: Gap 6 — Velocity type for momentum inheritance

Adds Velocity struct (value, direction) to track drag momentum.
on_drag() captures and stores velocity for spring retargeting.
Enables smooth continuation of momentum from user input into
spring animations (needed for Phase 2).

Tests: r2_gap_6_velocity_inheritance_from_drag (NEW)"
```

### Commit 3: Gap 4 (2-Live-Loop Budget)

**Files**: src/paint.rs, src/shell/mod.rs  
**Lines**: ~8  
**Tests**: 3 activation

```bash
git add src/paint.rs src/shell/mod.rs tests/recipes.rs
git commit -m "STEP 20 Phase 1: Gap 4 — 2-live-loop animation budget

Adds assert_animation_budget() check in draw() loop.
Panics if more than 2 concurrent animations are active,
preventing resource exhaustion and frame rate collapse.
Guides developers toward proper use of .key() for list items.

Tests: r2_gap_4_budget_panics_on_third_concurrent_animation (NEW)"
```

---

## Pre-Implementation Checklist

Before starting Phase 1 implementation:

- [ ] Read this document (you are here ✓)
- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md Phase 1 section
- [ ] Review STEP_19_R2_MOTION_KIT_AUDIT_EXAMPLE_IMPLEMENTATIONS.md for code examples
- [ ] Clone current branch: `git checkout -b step-20-phase-1`
- [ ] Verify baseline tests pass: `cargo test --lib` (expect 396)
- [ ] Verify audit tests pass: `cargo test --test recipes -- r2_gap` (expect 27)

### Code Review Checklist (before committing each gap)

For each commit:
- [ ] Code added has no `unsafe` blocks
- [ ] Error messages are actionable
- [ ] Tests use Harness (no manual time tracking)
- [ ] No new dependencies added
- [ ] Formatting matches codebase: `cargo fmt --check`
- [ ] Clippy passes: `cargo clippy -- -D warnings`
- [ ] All 396 library tests still pass
- [ ] New acceptance tests pass

---

## Debugging Phase 1

### If Metrics.motion test fails

```bash
# Check Metrics struct has motion field
grep -n "pub motion:" src/theme.rs

# Check Memory::begin_frame() clears animations
grep -A 5 "is_motion_enabled" src/memory.rs

# Run with verbose output
cargo test r2_gap_5_metrics_motion_collapses_animations -- --nocapture
```

### If Velocity test fails

```bash
# Check Velocity type definition
grep -B 2 -A 5 "pub struct Velocity" src/memory.rs

# Check Memory has last_velocity field
grep -n "last_velocity:" src/memory.rs

# Check on_drag() updates velocity
grep -A 3 "on_drag" src/paint.rs
```

### If Budget test fails

```bash
# Check assert_animation_budget() exists and is called
grep -n "assert_animation_budget" src/paint.rs src/shell/mod.rs

# Check error message is clear
cargo test r2_gap_4_budget_panics_on_third_concurrent_animation --nocapture
```

---

## Success Criteria

Phase 1 is complete when:

✅ All 3 gaps implemented (55 lines of code)  
✅ All 3 acceptance test stubs activated and passing  
✅ All 396 library tests still passing (zero regressions)  
✅ All 3 commits created with clear, focused messages  
✅ Code reviewed for safety, style, and clarity  
✅ Ready to proceed to Phase 2 (springs, enter/exit)

---

## Next: Phase 2

When Phase 1 is done:
1. Read STEP_19_R2_MOTION_KIT_AUDIT_PHASE_2_SCAFFOLDING.md
2. Implement Spring struct and Enter/Exit enum
3. Activate 2 additional acceptance tests
4. Expected: 2-3 days, 5-7 commits, medium risk

**Phase 1 bridge is complete. Ready to implement.** ✅
