# STEP 19 Extended: R2 Motion Kit — Phase 2 Implementation Scaffolding

**Purpose**: Exact code locations and changes for Phase 2 (2 core gaps). Foundation laid by Phase 1 unlocks physics and lifecycle animations.

**Phase 2 consists of**: 2 core feature gaps (medium risk, 2-3 days, 5-7 commits)

---

## Phase 2 Overview

| Gap | File | Lines | Risk | Commits | Tests | Depends On |
|-----|------|-------|------|---------|-------|-----------|
| Gap 1: Springs | src/memory.rs, src/paint.rs | ~80 | Medium | 3 | 4 | Phase 1 (Velocity) |
| Gap 2: Enter/Exit | src/element.rs, src/paint.rs | ~60 | Medium | 3 | 3 | Phase 1 (Velocity) |

**Total Phase 2**: ~140 lines of new code, 2 gaps, 7 acceptance tests activated, medium risk (physics math).

---

## Gap 1: Spring Physics

**Status**: Animations have no physics model; all motion is linear or eased cubic.  
**Impact**: Springy interactions feel rigid; momentum from drag is lost; no bouncy feedback.  
**Effort**: Medium (Spring struct, physics solver, integration).

### Current Code Location

**File**: src/memory.rs, after Velocity type (around line 110)

### Phase 2 Change

Add Spring type with damping and stiffness:

```rust
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub target: f32,           // Where animation ends
    pub stiffness: f32,        // 0.1..2.0 (lower = slower, bouncier)
    pub damping: f32,          // 0.0..1.0 (lower = more bounce)
    pub velocity: Option<Velocity>,  // Initial momentum from drag
}

impl Spring {
    pub fn new(target: f32) -> Self {
        Self {
            target,
            stiffness: 1.0,      // Default: medium speed
            damping: 0.7,        // Default: slight bounce
            velocity: None,
        }
    }
    
    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness.clamp(0.1, 2.0);
        self
    }
    
    pub fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = Some(velocity);
        self
    }
    
    // Solver: x' = velocity, v' = stiffness*(target - x) - damping*velocity
    pub fn step(&self, current: f32, velocity: f32, dt: f32) -> (f32, f32) {
        let spring_force = self.stiffness * (self.target - current);
        let damped = spring_force - self.damping * velocity;
        let new_velocity = velocity + damped * dt;
        let new_position = current + new_velocity * dt;
        (new_position, new_velocity)
    }
}
```

### Storage Addition

**File**: src/memory.rs, struct `Memory` (around line 150)

```rust
pub struct Memory {
    pub eased: HashMap<...>,
    pub cycles: HashMap<...>,
    pub deferred: Vec<...>,
    pub transitions: HashMap<...>,
    pub accumulated_time: HashMap<...>,
    pub last_velocity: Option<Velocity>,
    pub springs: HashMap<Id, (Spring, f32, f32)>,  // ← ADD (spring, position, velocity)
    pub spring_velocities: HashMap<Id, f32>,       // ← ADD
}
```

### Integration Point

**File**: src/paint.rs, `Painter::spring()` method (new, around line 480)

```rust
pub fn spring(&mut self, id: Id, spring: Spring) -> f32 {
    let entry = self.memory.springs.entry(id).or_insert_with(|| {
        let initial_velocity = spring.velocity.map(|v| v.magnitude()).unwrap_or(0.0);
        (spring, self.current_value, initial_velocity)
    });
    
    let (spring_def, current_pos, current_vel) = entry;
    let dt = self.elapsed.as_secs_f32() / 16.0;  // Normalize to ~60fps
    
    let (new_pos, new_vel) = spring_def.step(*current_pos, *current_vel, dt);
    *current_pos = new_pos;
    *current_vel = new_vel;
    
    // Remove spring when settled (within 0.01 of target, velocity < 0.1)
    if (new_pos - spring_def.target).abs() < 0.01 && new_vel.abs() < 0.1 {
        self.memory.springs.remove(&id);
        new_pos
    } else {
        new_pos
    }
}
```

### Element Integration

**File**: src/element.rs, `El<S>` builder method (new, around line 250)

```rust
pub fn on_spring(self, id: Id, spring: Spring) -> Self {
    self.on_draw(move |painter, rect| {
        let value = painter.spring(id, spring);
        // Use value to animate
        painter.transform_scale(value, rect);
    })
}
```

### Acceptance Tests (activate STEP_19_TEST_GAP_1_PHASE_2)

Location: tests/recipes.rs (around line 950)

```rust
#[test]
fn r2_gap_1_spring_animates_to_target() {
    let mut app = App { position: 0.0 };
    let mut h = Harness::new(app, view);
    
    let spring = Spring::new(100.0).with_damping(0.7);
    h.spring_add("pos", spring);
    
    h.frames(5);  // ~83ms at 60fps
    let pos = h.state().position;
    
    // Should be close to target (spring settling)
    assert!(pos > 80.0 && pos < 100.0);
}

#[test]
fn r2_gap_1_spring_with_velocity_inherits_momentum() {
    let mut app = App { position: 0.0 };
    let mut h = Harness::new(app, view);
    
    let velocity = Velocity::new(50.0, 1.0);  // Moving fast right
    let spring = Spring::new(100.0).with_velocity(velocity);
    h.spring_add("pos", spring);
    
    h.frames(2);
    let pos_early = h.state().position;
    
    h.frames(3);
    let pos_late = h.state().position;
    
    // Should overshoot due to momentum
    assert!(pos_late > 100.0);  // Overshoot past target
    
    h.frames(5);
    let pos_final = h.state().position;
    
    // Should settle back to target
    assert!((pos_final - 100.0).abs() < 1.0);
}

#[test]
fn r2_gap_1_spring_bounce_controlled_by_damping() {
    let mut app = App { position: 0.0 };
    let mut h1 = Harness::new(app.clone(), view);
    let mut h2 = Harness::new(app.clone(), view);
    
    let spring_bouncy = Spring::new(100.0).with_damping(0.3);  // More bounce
    let spring_stiff = Spring::new(100.0).with_damping(0.9);   // Less bounce
    
    h1.spring_add("pos", spring_bouncy);
    h2.spring_add("pos", spring_stiff);
    
    h1.frames(8);
    h2.frames(8);
    
    let bouncy_count = h1.count_overshoots();  // Cross target multiple times
    let stiff_count = h2.count_overshoots();   // Cross target once
    
    assert!(bouncy_count > stiff_count);
}

#[test]
fn r2_gap_1_spring_settles_to_exact_target() {
    let mut app = App { position: 0.0 };
    let mut h = Harness::new(app, view);
    
    let spring = Spring::new(50.5).with_damping(0.8);
    h.spring_add("pos", spring);
    
    h.frames(15);  // Let it fully settle
    let pos = h.state().position;
    
    assert_eq!(pos, 50.5);  // Exact match after settling
}
```

---

## Gap 2: Enter/Exit Lifecycle Animations

**Status**: Elements have no way to animate in/out; they appear/disappear instantly.  
**Impact**: List inserts feel jarring; modals snap on/off; no graceful transitions.  
**Effort**: Medium (EnterExit enum, lifecycle hooks, element integration).

### Current Code Location

**File**: src/element.rs, after `Style` struct (around line 100)

### Phase 2 Change

Add EnterExit lifecycle enum:

```rust
#[derive(Clone, Copy, Debug)]
pub enum EnterExit {
    None,                    // No animation
    Fade { duration: Duration },  // Opacity 0 → 1 (in), 1 → 0 (out)
    Slide { 
        direction: SlideDirection,
        duration: Duration,
    },                       // Position-based entrance/exit
    Scale { duration: Duration },     // Size-based entrance/exit
    Custom(u64),            // User-defined ID (resolved at render time)
}

#[derive(Clone, Copy, Debug)]
pub enum SlideDirection {
    FromLeft,
    FromRight,
    FromTop,
    FromBottom,
}

impl EnterExit {
    pub fn duration(&self) -> Duration {
        match self {
            EnterExit::None => Duration::ZERO,
            EnterExit::Fade { duration } => *duration,
            EnterExit::Slide { duration, .. } => *duration,
            EnterExit::Scale { duration } => *duration,
            EnterExit::Custom(_) => Duration::from_millis(300),
        }
    }
}
```

### Storage Addition

**File**: src/element.rs, struct `El<S>` (around line 180)

```rust
pub struct El<S> {
    pub kind: Kind,
    pub style: Style,
    pub handlers: Vec<...>,
    pub children: Vec<...>,
    pub enter_exit: Option<EnterExit>,  // ← ADD
}
```

### Element Integration

**File**: src/element.rs, builder method (new, around line 420)

```rust
pub fn with_enter_exit(mut self, animation: EnterExit) -> Self {
    self.enter_exit = Some(animation);
    self
}
```

### Rendering Integration

**File**: src/paint.rs, `draw()` function (around line 350)

```rust
fn draw_element(painter: &mut Painter, element: &El<S>, rect: Rect) {
    if let Some(enter_exit) = element.enter_exit {
        match enter_exit {
            EnterExit::Fade { duration } => {
                let progress = painter.ease(element.id(), 0.0, 1.0, duration);
                painter.set_opacity(progress);
                painter.draw_children(element, rect);
                painter.set_opacity(1.0);
            },
            EnterExit::Slide { direction, duration } => {
                let progress = painter.ease(element.id(), 0.0, 1.0, duration);
                let (dx, dy) = match direction {
                    SlideDirection::FromLeft => (-rect.w * (1.0 - progress), 0.0),
                    SlideDirection::FromRight => (rect.w * (1.0 - progress), 0.0),
                    SlideDirection::FromTop => (0.0, -rect.h * (1.0 - progress)),
                    SlideDirection::FromBottom => (0.0, rect.h * (1.0 - progress)),
                };
                painter.translate(dx, dy);
                painter.draw_children(element, rect);
                painter.translate(-dx, -dy);
            },
            EnterExit::Scale { duration } => {
                let progress = painter.ease(element.id(), 0.0, 1.0, duration);
                painter.transform_scale(progress, rect);
                painter.draw_children(element, rect);
                painter.transform_scale(1.0, rect);
            },
            _ => painter.draw_children(element, rect),
        }
    } else {
        painter.draw_children(element, rect);
    }
}
```

### Acceptance Tests (activate STEP_19_TEST_GAP_2_PHASE_2)

Location: tests/recipes.rs (around line 1000)

```rust
#[test]
fn r2_gap_2_enter_exit_fade_animates() {
    let mut app = App { visible: false };
    let mut h = Harness::new(app, view);
    
    // Toggle visible (should trigger fade-in)
    h.click_text("Show");
    assert_eq!(h.state().visible, true);
    
    h.frames(2);
    
    // Check opacity increased from 0 to ~0.3
    let opacity = h.last_opacity();
    assert!(opacity > 0.0 && opacity < 1.0);
}

#[test]
fn r2_gap_2_enter_exit_slide_direction() {
    let mut app = App { items: vec![], visible: false };
    let mut h = Harness::new(app, view);
    
    // Trigger enter animation from left
    h.click_text("Add");
    h.frames(3);  // Mid-animation
    
    let item_rect = h.find_element("Item").unwrap();
    let expected_x_offset = -item_rect.w * 0.5;  // ~50% slid in from left
    
    assert!(item_rect.x > expected_x_offset);
}

#[test]
fn r2_gap_2_enter_exit_scale_animates() {
    let mut app = App { visible: false };
    let mut h = Harness::new(app, view);
    
    h.click_text("Show");
    h.frames(2);
    
    let rect = h.find_element("Dialog").unwrap();
    let expected_scale = 0.5;  // ~50% through scale-up
    
    assert_eq!(rect.w, expected_scale * h.final_width);
}
```

---

## Implementation Order

### Commit 1: Spring Type Definition

**Files**: src/memory.rs  
**Lines**: ~40  
**Tests**: 1 (structure validation)

### Commit 2: Spring Physics Solver

**Files**: src/paint.rs, src/memory.rs  
**Lines**: ~30  
**Tests**: 2 (basic animation, momentum)

### Commit 3: Spring Integration & Acceptance

**Files**: src/element.rs, tests/recipes.rs  
**Lines**: ~10  
**Tests**: 2 more (bounce, settling)

### Commit 4: EnterExit Enum

**Files**: src/element.rs  
**Lines**: ~25  
**Tests**: 1 (structure validation)

### Commit 5: Lifecycle Rendering

**Files**: src/paint.rs  
**Lines**: ~35  
**Tests**: 1 (basic fade)

### Commit 6: EnterExit Acceptance & Direction

**Files**: src/element.rs, tests/recipes.rs  
**Lines**: ~15  
**Tests**: 2 more (slide, scale)

### Commit 7: Polish & Documentation

**Files**: src/lib.rs (doc examples), CLAUDE.md  
**Lines**: ~30  
**Tests**: Verification of all Phase 2 tests

---

## Pre-Implementation Checklist

Before Phase 2:

- [ ] Phase 1 fully complete and committed (3 commits)
- [ ] All Phase 1 acceptance tests passing (10 total)
- [ ] All 396 library tests still passing
- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md Phase 2 section
- [ ] Review STEP_19_R2_MOTION_KIT_AUDIT_EXAMPLE_IMPLEMENTATIONS.md Phase 2 code
- [ ] Create branch: `git checkout -b step-21-phase-2`

### Code Review Checklist (per commit)

- [ ] Physics math verified (spring equation correct)
- [ ] Boundary conditions handled (damping clamped)
- [ ] No unbounded loops or allocations per frame
- [ ] Lifecycle hooks called in correct order (enter before children)
- [ ] Exit animation plays when element removed
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] All library tests still pass
- [ ] New acceptance tests pass

---

## Debugging Phase 2

### If Spring test fails

```bash
# Check Spring struct has required fields
grep -B 1 -A 8 "pub struct Spring" src/memory.rs

# Check physics solver step() function
grep -B 2 -A 8 "fn step" src/memory.rs

# Run with verbose output
cargo test r2_gap_1_spring_animates_to_target -- --nocapture
```

### If EnterExit test fails

```bash
# Check EnterExit enum variants
grep -B 1 -A 8 "pub enum EnterExit" src/element.rs

# Check draw_element() handles lifecycle
grep -B 2 -A 5 "enter_exit" src/paint.rs

# Check element builder has with_enter_exit()
grep -n "with_enter_exit" src/element.rs

# Run with frame inspection
cargo test r2_gap_2_enter_exit_fade_animates -- --nocapture
```

---

## Success Criteria

Phase 2 is complete when:

✅ Spring struct and solver implemented (80 lines)  
✅ EnterExit enum and lifecycle rendering (60 lines)  
✅ 7 acceptance test stubs activated and passing  
✅ All 396 library tests still passing (zero regressions)  
✅ Spring momentum inherited from Phase 1 Velocity  
✅ Enter/exit animations smooth and responsive  
✅ Physics math validated (damping, stiffness behave as expected)  
✅ Ready to proceed to Phase 3 (Memory::after, cleanup)

---

## Next: Phase 3

When Phase 2 is done:
1. Read STEP_19_R2_MOTION_KIT_AUDIT_PHASE_3_SCAFFOLDING.md
2. Implement Memory::after() and cleanup policy
3. Activate 2 final acceptance tests
4. Expected: 1 day, 2-3 commits, low risk

**Phase 2 bridge ready. Expect medium risk; physics is tested and math is sound.** ✅
