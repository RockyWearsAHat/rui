# STEP 19: R2 Motion Kit Audit — Example Implementations

**Document**: Copy-paste ready code snippets for implementing each R2 gap  
**Audience**: Implementers ready to start coding Phase 1/2/3  
**Last Updated**: 2026-09-02

---

## Phase 1: Example Implementations (STEP 20)

### Gap 5: Metrics.motion accessibility check

**What it does**: Collapses animations when user prefers reduced motion

**Current code** (src/theme.rs, Metrics struct):
```rust
pub struct Metrics {
    pub spacing: fn(Level) -> f32,
    pub corner: fn(Level) -> f32,
    pub height: fn(Level) -> f32,
}
```

**Phase 1 code** (add one line):
```rust
pub struct Metrics {
    pub spacing: fn(Level) -> f32,
    pub corner: fn(Level) -> f32,
    pub height: fn(Level) -> f32,
    pub motion: f32,  // NEW: 1.0 = full animation, 0.0 = instant (accessibility)
}
```

**Usage in animation** (src/memory.rs, ease function):
```rust
// CURRENT
pub fn ease(&mut self, key: u64, value: f32, target: f32, duration: Duration) {
    let duration_ms = duration.as_millis() as u32;
    self.eased.insert(key, EasedValue {
        from: value,
        to: target,
        duration_ms,
        elapsed_ms: 0,
    });
}

// PHASE 1
pub fn ease(&mut self, key: u64, value: f32, target: f32, duration: Duration, metrics: &Metrics) {
    let effective_duration = if metrics.motion == 0.0 {
        Duration::ZERO  // Skip animation if motion disabled
    } else {
        duration
    };
    
    let duration_ms = effective_duration.as_millis() as u32;
    if duration_ms == 0 {
        // Animation duration collapsed — jump to target immediately
        // No entry in eased HashMap, caller sees target value immediately
        return;
    }
    
    self.eased.insert(key, EasedValue {
        from: value,
        to: target,
        duration_ms,
        elapsed_ms: 0,
    });
}
```

**Test** (tests/recipes.rs, Phase 1 acceptance stub):
```rust
#[test]
fn acceptance_accessibility_motion_preferences() {
    // Create app with motion disabled
    let mut theme = Theme::default().with_motion(0.0);
    let mut h = Harness::new(App { value: 0.0 }, view).theme(theme);
    
    // Request 1-second animation
    h.state_mut().animate_smoothly(1000); // 1000ms duration
    
    // Verify animation is collapsed (no frames needed)
    h.frames(0);  // Zero frames — no animation
    assert_eq!(h.state().value, expected_target);  // Jumped to target immediately
    
    // Now enable motion
    h.set_theme(Theme::default().with_motion(1.0));
    h.state_mut().animate_smoothly(1000);
    h.frames(0);
    assert!(h.state().value < expected_target);  // Animation in progress
}
```

---

### Gap 6: Velocity inheritance for momentum

**What it does**: Captures drag velocity and passes it to spring animations

**Current code** (src/memory.rs, no Velocity type):
```rust
pub struct EasedValue {
    from: f32,
    to: f32,
    duration_ms: u32,
    elapsed_ms: u32,
}
```

**Phase 1 code** (add Velocity type):
```rust
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    pub pixels_per_second: f32,
}

impl Velocity {
    pub fn from_drag(pixels_moved: f32, time_elapsed: Duration) -> Self {
        let seconds = time_elapsed.as_secs_f32();
        Velocity {
            pixels_per_second: if seconds > 0.0 {
                pixels_moved / seconds
            } else {
                0.0
            }
        }
    }
    
    pub fn zero() -> Self {
        Velocity { pixels_per_second: 0.0 }
    }
}
```

**Usage during drag** (src/paint.rs, handler after drag):
```rust
// CURRENT: Drag updates value, no velocity tracking
on_drag(|state: &mut App, drag: Drag| {
    state.position += drag.delta.x;
})

// PHASE 1: Capture velocity
on_drag(|state: &mut App, drag: Drag| {
    state.position += drag.delta.x;
    state.drag_velocity = Velocity::from_drag(
        drag.delta.x,
        state.memory.elapsed_since_frame_start(),
    );
})
```

**Test** (tests/recipes.rs, Phase 1 acceptance stub):
```rust
#[test]
fn acceptance_velocity_inheritance_from_drag() {
    let mut h = Harness::new(App::new(), view);
    
    // Simulate fast drag (high velocity)
    h.drag(Point::new(100.0, 50.0), Point::new(200.0, 50.0), Duration::from_millis(100));
    
    // Verify velocity was captured
    assert_eq!(h.state().drag_velocity.pixels_per_second, 1000.0);  // 100px in 100ms
    
    // Slow drag (low velocity)
    h.drag(Point::new(0.0, 0.0), Point::new(50.0, 0.0), Duration::from_millis(500));
    assert_eq!(h.state().drag_velocity.pixels_per_second, 100.0);  // 50px in 500ms
}
```

---

### Gap 4: 2-live-loop animation budget enforcement

**What it does**: Prevents more than 2 animations from running in parallel (performance budget)

**Current code** (src/app.rs, draw function):
```rust
fn draw(memory: &mut Memory, /* ... */) {
    // No budget check
    for (key, anim) in &memory.eased {
        // Animate...
    }
}
```

**Phase 1 code** (add budget assertion):
```rust
const ANIMATION_BUDGET: usize = 2;

fn draw(memory: &mut Memory, /* ... */) {
    let active_animations = memory.eased.len() + memory.cycles.len() + memory.transitions.len();
    
    // Debug assertion to catch budget violations during development
    debug_assert!(
        active_animations <= ANIMATION_BUDGET,
        "Animation budget exceeded: {} animations running (max: {})",
        active_animations,
        ANIMATION_BUDGET
    );
    
    // In release builds, gracefully degrade:
    // Priority order: ease > phase > transitions
    if active_animations > ANIMATION_BUDGET {
        let to_drop = active_animations - ANIMATION_BUDGET;
        // Drop lowest-priority animations
        if memory.transitions.len() > 0 {
            let keys: Vec<_> = memory.transitions.keys().take(to_drop).copied().collect();
            for key in keys {
                memory.transitions.remove(&key);
            }
        }
    }
}
```

**Test** (tests/recipes.rs, Phase 1 acceptance stub):
```rust
#[test]
fn acceptance_animation_budget_enforcement() {
    let mut h = Harness::new(App::new(), view);
    
    // Start first animation
    h.state_mut().animate_value_1(500); // Gap 1 ease
    h.frames(1);
    assert_eq!(h.state().active_animations(), 1);
    
    // Start second animation
    h.state_mut().animate_value_2(500); // Gap 2 phase
    h.frames(1);
    assert_eq!(h.state().active_animations(), 2);
    
    // Try to start third animation (should fail or drop lowest priority)
    h.state_mut().animate_value_3(500); // Gap 4 transition
    h.frames(1);
    
    // Verify budget was enforced
    assert!(h.state().active_animations() <= 2);
    
    // In debug: Would panic with "Animation budget exceeded"
    // In release: Lowest-priority animation was dropped
}
```

**Safety invariant** (to add to CLAUDE.md):
> The 2-live-loop budget is a design constraint, not a hard limit. When exceeded, lowest-priority animations (transitions > phase > ease) are dropped to prevent frame jank. This budget is validated with `debug_assert!` in dev builds and enforces gracefully in release.

---

## Phase 2: Example Implementations (STEP 21)

### Gap 1: Springs with bounce control

**What it does**: Physics-based animation with configurable damping and bounce

**Current code**: No Spring type exists

**Phase 2 code** (add Spring struct to src/memory.rs):
```rust
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub stiffness: f32,    // 0.0–1.0, controls oscillation frequency
    pub damping: f32,      // 0.0–1.0, controls how quickly it settles
    pub mass: f32,         // 1.0 default, affects oscillation period
}

impl Spring {
    /// Ease out (no bounce): stiffness=0.4, damping=0.95
    pub fn easeout() -> Self {
        Spring { stiffness: 0.4, damping: 0.95, mass: 1.0 }
    }
    
    /// Bounce: stiffness=0.6, damping=0.7 (oscillates ~3 times before settling)
    pub fn bounce() -> Self {
        Spring { stiffness: 0.6, damping: 0.7, mass: 1.0 }
    }
    
    /// Smooth: stiffness=0.3, damping=0.85 (minimal bounce, settles quickly)
    pub fn smooth() -> Self {
        Spring { stiffness: 0.3, damping: 0.85, mass: 1.0 }
    }
}
```

**Storage in Memory** (src/memory.rs):
```rust
pub struct Memory {
    pub springs: HashMap<u64, SpringState>,  // NEW: Spring animation tracking
    // ... existing fields ...
}

#[derive(Debug, Clone)]
pub struct SpringState {
    pub from: f32,
    pub to: f32,
    pub velocity: f32,
    pub spring: Spring,
    pub elapsed_ms: u32,
}
```

**Animation update** (src/memory.rs, new method):
```rust
pub fn spring(&mut self, key: u64, from: f32, to: f32, spring: Spring) {
    self.springs.insert(key, SpringState {
        from,
        to,
        velocity: 0.0,
        spring,
        elapsed_ms: 0,
    });
}

fn step_springs(&mut self, dt: Duration) {
    let dt_seconds = dt.as_secs_f32();
    
    for (_, state) in &mut self.springs {
        let distance = state.to - state.from;
        
        // Hooke's law: F = -kx
        let acceleration = state.spring.stiffness * distance - state.spring.damping * state.velocity;
        
        // Update velocity and position
        state.velocity += acceleration * dt_seconds;
        state.from += state.velocity * dt_seconds;
        
        // Check if settled
        if state.from.abs_diff(state.to) < 0.01 && state.velocity.abs() < 0.01 {
            state.from = state.to;
            state.velocity = 0.0;
        }
    }
    
    self.springs.retain(|_, state| state.from != state.to);
}
```

**Test** (tests/recipes.rs, Phase 2 acceptance stub):
```rust
#[test]
fn acceptance_spring_animation_applies_damping() {
    let mut h = Harness::new(App::new(), view);
    
    // Spring animation with bounce
    h.state_mut().spring_to_position(100.0, Spring::bounce());
    
    // First frame: velocity increases, position changes
    h.frames(1);
    let pos_frame1 = h.state().position;
    assert!(pos_frame1 > 0.0 && pos_frame1 < 100.0);
    
    // Several frames: oscillates around target
    h.frames(5);
    let pos_frame6 = h.state().position;
    
    // Many frames: settles to target
    h.frames(100);
    assert_eq!(h.state().position, 100.0);
    
    // Verify overshoot (characteristic of bounce)
    assert!(pos_frame6 > 100.0 || pos_frame6 < 100.0);
}
```

---

### Gap 2: Enter/exit transitions

**What it does**: Automatic animations when elements appear/disappear

**Current code**: No EnterExit enum

**Phase 2 code** (add to src/widgets.rs):
```rust
#[derive(Debug, Clone, Copy)]
pub enum EnterExit {
    FadeInOut,           // Opacity 0→1 on enter, 1→0 on exit
    SlideUpDown,         // Slide from top, out to bottom
    SlideLeftRight,      // Slide from left, out to right
    ScaleInOut,          // Scale 0→1 on enter, 1→0 on exit
    Custom(f32, f32),    // Custom duration for enter and exit
}

impl EnterExit {
    pub fn enter_duration(&self) -> Duration {
        match self {
            EnterExit::FadeInOut => Duration::from_millis(300),
            EnterExit::SlideUpDown => Duration::from_millis(400),
            EnterExit::SlideLeftRight => Duration::from_millis(400),
            EnterExit::ScaleInOut => Duration::from_millis(250),
            EnterExit::Custom(enter_ms, _) => Duration::from_millis(*enter_ms as u64),
        }
    }
    
    pub fn exit_duration(&self) -> Duration {
        match self {
            EnterExit::FadeInOut => Duration::from_millis(200),
            EnterExit::SlideUpDown => Duration::from_millis(300),
            EnterExit::SlideLeftRight => Duration::from_millis(300),
            EnterExit::ScaleInOut => Duration::from_millis(200),
            EnterExit::Custom(_, exit_ms) => Duration::from_millis(*exit_ms as u64),
        }
    }
}
```

**Element extension** (src/element.rs):
```rust
pub struct El<S> {
    // ... existing fields ...
    enter_exit: Option<EnterExit>,  // NEW
    is_entering: bool,               // NEW
}

impl<S: 'static> El<S> {
    pub fn enter_exit(mut self, animation: EnterExit) -> Self {
        self.enter_exit = Some(animation);
        self
    }
}
```

**Paint logic** (src/paint.rs):
```rust
fn paint_element(el: &El, memory: &Memory, painter: &mut Painter) {
    if let Some(enter_exit) = el.enter_exit {
        // Determine if element is appearing or disappearing
        if memory.is_element_appearing(el.id) {
            // Apply enter animation
            let alpha = memory.get_enter_animation(el.id, enter_exit);
            painter.set_alpha(alpha);
        } else if memory.is_element_disappearing(el.id) {
            // Apply exit animation
            let alpha = memory.get_exit_animation(el.id, enter_exit);
            painter.set_alpha(alpha);
        }
    }
    
    // Paint normally
    painter.fill(rect, Tone::Primary);
}
```

**Test** (tests/recipes.rs, Phase 2 acceptance stub):
```rust
#[test]
fn acceptance_element_enter_exit_animations() {
    let mut h = Harness::new(App { show_modal: false }, view);
    
    // Element hidden initially
    assert!(!h.rendered_text("Modal").is_empty()); // Not visible yet
    
    // Show modal
    h.state_mut().show_modal = true;
    h.frames(0);  // Frame 0: entering
    let opacity_frame0 = h.opacity("Modal");
    assert!(opacity_frame0 > 0.0 && opacity_frame0 < 1.0);
    
    // Several frames: fading in
    h.frames(5);
    let opacity_frame5 = h.opacity("Modal");
    assert!(opacity_frame5 > opacity_frame0);
    
    // Complete animation: fully visible
    h.frames(20);
    assert_eq!(h.opacity("Modal"), 1.0);
    
    // Hide modal
    h.state_mut().show_modal = false;
    h.frames(0);
    let opacity_exiting = h.opacity("Modal");
    assert!(opacity_exiting < 1.0);
    
    // Complete exit
    h.frames(20);
    assert_eq!(h.opacity("Modal"), 0.0);
}
```

---

## Phase 3: Example Implementations (STEP 22)

### Gap 3: Easing enum support

**What it does**: Built-in easing curves (Linear, EaseIn, EaseOut, etc.) without closures

**Current code** (incomplete):
```rust
pub enum Easing {
    Custom(fn(f32) -> f32),  // Only option currently
}
```

**Phase 3 code** (expand enum):
```rust
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    Custom(fn(f32) -> f32),
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        // t: 0.0–1.0, progress through animation
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            },
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => 1.0 + (t - 1.0) * (t - 1.0) * (t - 1.0),
            Easing::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 + (t - 1.0) * (2.0 * (t - 2.0)) * (2.0 * (t - 2.0))
                }
            },
            Easing::Custom(f) => f(t),
        }
    }
}
```

**Usage in Memory** (src/memory.rs):
```rust
pub struct EasedValue {
    from: f32,
    to: f32,
    duration_ms: u32,
    elapsed_ms: u32,
    easing: Easing,  // NEW
}

impl EasedValue {
    pub fn current_value(&self) -> f32 {
        let t = self.elapsed_ms as f32 / self.duration_ms as f32;
        let eased_t = self.easing.apply(t.clamp(0.0, 1.0));
        self.from + (self.to - self.from) * eased_t
    }
}
```

**Test** (tests/recipes.rs, Phase 3 acceptance stub):
```rust
#[test]
fn acceptance_easing_enum_curves() {
    let mut h = Harness::new(App::new(), view);
    
    // Linear easing: proportional progress
    h.state_mut().ease_with(0.0, 100.0, 1000, Easing::Linear);
    h.frames(1);  // 8ms elapsed
    let linear_value = h.state().value;
    assert!(linear_value > 0.0 && linear_value < 100.0);
    
    // EaseOut: faster start, slower end
    h.state_mut().ease_with(0.0, 100.0, 1000, Easing::EaseOutQuad);
    h.frames(1);
    let easeout_value = h.state().value;
    // EaseOut should progress faster than linear in first frame
    assert!(easeout_value > linear_value);
    
    // EaseIn: slower start, faster end
    h.state_mut().ease_with(0.0, 100.0, 1000, Easing::EaseInQuad);
    h.frames(1);
    let easein_value = h.state().value;
    // EaseIn should progress slower than linear in first frame
    assert!(easein_value < linear_value);
}
```

---

### Gap 7: Cleanup policy & Memory::after() sugar

**What it does**: Automatic handler delay and cleanup of completed animations

**Phase 3 code** (add to Memory):
```rust
pub struct DeferredHandler {
    pub delay_ms: u32,
    pub handler: Box<dyn FnOnce(&mut S)>,
}

impl Memory {
    /// Schedule a handler to run after a delay
    pub fn after<S: 'static>(&mut self, delay: Duration, handler: impl FnOnce(&mut S) + 'static) {
        self.deferred.push(DeferredHandler {
            delay_ms: delay.as_millis() as u32,
            handler: Box::new(handler),
        });
    }
}
```

**Cleanup policy** (src/app.rs, draw function):
```rust
fn step_deferred_handlers(memory: &mut Memory, dt: Duration) {
    for handler in &mut memory.deferred {
        handler.delay_ms = handler.delay_ms.saturating_sub(dt.as_millis() as u32);
    }
    
    // Execute handlers that finished their delay
    // This maintains FIFO order: first-deferred, first-executed
    let mut to_execute = Vec::new();
    memory.deferred.retain(|h| {
        if h.delay_ms == 0 {
            to_execute.push(h.take());  // Remove and execute later
            false
        } else {
            true
        }
    });
    
    for handler in to_execute {
        handler.call(state);
    }
    
    // Clean up completed animations (drop from HashMap)
    memory.eased.retain(|_, value| value.elapsed_ms < value.duration_ms);
    memory.springs.retain(|_, state| state.from != state.to);
    memory.transitions.retain(|_, trans| !trans.is_complete());
}
```

**Test** (tests/recipes.rs, Phase 3 acceptance stub):
```rust
#[test]
fn acceptance_memory_after_sugar() {
    let mut h = Harness::new(App { counter: 0 }, view);
    
    // Schedule handler to run after 500ms
    h.state_mut().memory.after(Duration::from_millis(500), |app| {
        app.counter = 42;
    });
    
    assert_eq!(h.state().counter, 0);  // Not executed yet
    
    // Step 300ms: handler still waiting
    h.frames(300 / 8);  // ~37 frames at 8ms each
    assert_eq!(h.state().counter, 0);
    
    // Step remaining 200ms: handler executes
    h.frames(200 / 8 + 1);
    assert_eq!(h.state().counter, 42);  // Handler ran
}
```

---

## Common Patterns and Error Recovery

### Pattern: Activating Phase N Acceptance Tests

```bash
# Before starting STEP 20 (Phase 1):
sed -i '/acceptance_phase_1/s/#\[ignore\]//' tests/recipes.rs

# Run to verify tests are now active
cargo test --test recipes -- acceptance_phase_1 2>&1
```

### Pattern: Debugging Animation State

```rust
// In Harness test:
h.state_mut().animate(/* ... */);
h.frames(1);

// Print current animation state
println!("Eased animations: {:?}", h.state().memory.eased);
println!("Springs: {:?}", h.state().memory.springs);
println!("Deferred handlers: {}", h.state().memory.deferred.len());
```

### Error: "Spring oscillates forever"

**Symptom**: Test expects settling at 10 frames but animation continues

**Fix**: Check damping value
```rust
// Verify damping > 0.5 for fast settling
assert!(state.spring.damping > 0.5, "Increase damping to settle faster");

// Expected: Spring::smooth() settles in ~20 frames
// If taking 100+ frames, damping is too low
```

### Error: "Enter animation never completes"

**Symptom**: Element stays partially transparent

**Fix**: Check element is not receiving conflicting opacity changes
```rust
// Verify no dual animation on same element
assert!(
    !memory.eased.contains_key(&element_id),
    "Element has conflicting animations"
);
```

---

## Implementation Checklist (Copy This)

### Phase 1 Preparation

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md (Phase 1 section)
- [ ] Review Gap 5, 6, 4 code examples above
- [ ] Copy acceptance test stubs from Phase 1 section
- [ ] Activate Phase 1 tests: `sed -i '/acceptance_phase_1/s/#\[ignore\]//' tests/recipes.rs`
- [ ] Verify activation: `cargo test --test recipes -- acceptance_phase_1 --list`

### Phase 2 Preparation

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md (Phase 2 section)
- [ ] Review Gap 1, 2 code examples above
- [ ] Copy acceptance test stubs from Phase 2 section
- [ ] Activate Phase 2 tests before starting STEP 21
- [ ] Verify no regressions in Phase 1 tests

### Phase 3 Preparation

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md (Phase 3 section)
- [ ] Review Gap 3, 7 code examples above
- [ ] Copy acceptance test stubs from Phase 3 section
- [ ] Activate Phase 3 tests before starting STEP 22
- [ ] Verify no regressions in Phase 1 & 2 tests

---

## Cross-References

**For running these examples**:
→ STEP_19_R2_MOTION_KIT_AUDIT_TEST_RUNBOOK.md (test execution commands)

**For acceptance test locations**:
→ tests/recipes.rs lines 1247–1331

**For API signatures**:
→ STEP_19_R2_MOTION_KIT_AUDIT_API_REFERENCE.md (exact type definitions)

**For performance constraints**:
→ STEP_19_R2_MOTION_KIT_AUDIT_PERFORMANCE_GUIDE.md (benchmarking strategy)

---
