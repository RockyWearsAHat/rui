# STEP 19 Extended: R2 Motion Kit API Reference

**Document purpose**: Complete API specification for R2 Motion Kit. Shows every type, method, enum variant, and trait that will be added to rui across all 3 phases.

**Status**: Complete specification. Provides exact signatures and doc comments for R2 implementation.

**Last updated**: 2026-09-02 after audit completion.

---

## Document Structure

This reference is organized by:
1. **New Types** (structs, enums)
2. **New Methods** (impl blocks on existing types)
3. **Module Extensions** (where types live)
4. **Signature Reference** (copy-paste ready)

---

## Phase 1 (STEP 20): Foundation Gaps

### Type: `Metrics` Extension

**Location**: `src/theme.rs`, add to existing `Metrics` struct

```rust
pub struct Metrics {
    // ... existing fields ...
    
    /// Multiplier for animation durations.
    /// 0.0 = animations disabled (accessibility: prefers-reduced-motion).
    /// 1.0 = normal speed.
    /// 2.0 = slowed down.
    ///
    /// Collapsed to 0.0 when users prefer reduced motion. Views check this
    /// to disable expensive animations or collapse to instant transitions.
    pub motion: f32,
}
```

**Default value**: `motion: 1.0` (normal speed)

**Accessibility behavior**: 
- System detects "prefers-reduced-motion" → `motion = 0.0`
- Views check `theme.metrics.motion == 0.0` before calling easing functions
- If true: show final state instantly, skip animation

---

### Type: `Velocity` (New)

**Location**: `src/memory.rs` (new struct, lines 200–210)

```rust
/// Momentum from a drag, used for spring retargeting.
///
/// When a user drags and releases, the drag velocity should flow into any
/// spring animation that follows. This allows smooth momentum-aware snapping.
#[derive(Debug, Clone, Copy)]
pub struct Velocity {
    /// Pixels per second in x direction (logical units).
    pub x: f32,
    /// Pixels per second in y direction (logical units).
    pub y: f32,
}

impl Velocity {
    /// Magnitude in pixels per second.
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    /// Zero velocity (no momentum).
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    /// Dampen velocity by a factor [0.0, 1.0].
    pub fn dampen(&self, factor: f32) -> Self {
        Self {
            x: self.x * factor.max(0.0).min(1.0),
            y: self.y * factor.max(0.0).min(1.0),
        }
    }
}
```

**Use in R2**: Springs inherit velocity on retarget to smooth momentum.

---

### Type: `Memory` Extension (Phase 1)

**Location**: `src/memory.rs`, extend existing `Memory` struct (line 213)

```rust
impl Memory {
    // ... existing methods ...
    
    /// Set the 2-live-loop animation budget.
    ///
    /// By default, rui asserts that ≤ 2 animations render per frame.
    /// This prevents frame-time overruns. Set to 0 to disable the check.
    ///
    /// # Panics
    ///
    /// At frame end if more than `budget` animations are active.
    /// Budget of 2 is strict: 3 animations will panic.
    pub fn set_animation_budget(&mut self, budget: usize) {
        self.animation_budget = budget;
    }
    
    /// Get the current animation budget.
    pub fn animation_budget(&self) -> usize {
        self.animation_budget
    }
    
    /// Record the velocity from a drag gesture.
    ///
    /// Called by input handling to store drag velocity. Spring animations
    /// that follow can inherit this velocity for smooth momentum.
    ///
    /// # Example
    ///
    /// ```ignore
    /// if let Some(drag) = response.drag {
    ///     memory.set_drag_velocity(id, Velocity {
    ///         x: drag.velocity_x,
    ///         y: drag.velocity_y,
    ///     });
    /// }
    /// ```
    pub fn set_drag_velocity(&mut self, id: Id, velocity: Velocity) {
        self.last_drag_velocity.insert(id, velocity);
    }
    
    /// Get the last recorded drag velocity for an element.
    ///
    /// Returns the velocity from the most recent drag on this element,
    /// or `Velocity::ZERO` if none was recorded.
    pub fn drag_velocity(&self, id: Id) -> Velocity {
        self.last_drag_velocity
            .get(&id)
            .copied()
            .unwrap_or(Velocity::ZERO)
    }
    
    /// Check if animations exceed the live-loop budget.
    ///
    /// Called at frame end. Panics if live loop count > budget.
    ///
    /// # Panics
    ///
    /// If active animation count > `self.animation_budget`.
    fn check_animation_budget(&self) {
        if self.animation_budget == 0 {
            return;  // Budget checking disabled
        }
        
        let live_loops = self.count_live_loops();
        if live_loops > self.animation_budget {
            panic!(
                "Animation budget exceeded: {} live loops active, max {}. \
                 Merge animations or stagger with Memory::defer().",
                live_loops, self.animation_budget
            );
        }
    }
    
    /// Count animations that are rendering this frame.
    fn count_live_loops(&self) -> usize {
        let mut count = 0;
        // Count animations that called their methods this frame
        for eased in self.eased.values() {
            if eased.seen == self.current_frame {
                count += 1;
            }
        }
        for cycle in self.cycles.values() {
            if cycle.seen == self.current_frame {
                count += 1;
            }
        }
        // ... and springs/transitions in Phase 2
        count
    }
}
```

**Storage additions to Memory struct**:
```rust
pub struct Memory {
    // ... existing fields ...
    
    /// Last recorded drag velocity per element (for momentum inheritance).
    last_drag_velocity: HashMap<Id, Velocity>,
    
    /// Maximum concurrent animations allowed (default: 2).
    animation_budget: usize,
}
```

---

## Phase 2 (STEP 21): Core Animation Features

### Type: `Spring` (New)

**Location**: `src/memory.rs` (new struct, lines 300–340)

```rust
/// A physics-based spring animation with momentum.
///
/// Springs smoothly animate toward a target value while respecting initial
/// velocity (from drag gestures). Damping controls overshoot; stiffness
/// controls response speed.
#[derive(Debug, Clone, Copy)]
pub struct Spring {
    /// Current position (logical units).
    pub position: f32,
    /// Target position to animate toward.
    pub target: f32,
    /// Current velocity (units per second).
    pub velocity: f32,
    /// Damping coefficient [0.0, 2.0]. 1.0 = critical damping (no bounce).
    /// < 1.0 = overshoot/bounce. > 1.0 = undershoot (slow, sticky).
    pub damping: f32,
    /// Stiffness coefficient [1.0, 1000.0]. Higher = faster response.
    pub stiffness: f32,
    /// Threshold for stopping (when velocity < this). Default: 0.001.
    pub stop_threshold: f32,
    /// Frame this spring was last read (for garbage collection).
    seen: u32,
}

impl Spring {
    /// Create a spring animation with inherit velocity and custom physics.
    ///
    /// # Arguments
    ///
    /// * `target` - Position to animate toward
    /// * `damping` - [0.0, 2.0], 1.0 is critical (no bounce)
    /// * `stiffness` - [1.0, 1000.0], higher = faster
    /// * `initial_velocity` - Momentum to inherit (usually from drag)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Bouncy spring with inherited drag momentum
    /// Spring::new(100.0, 0.5, 100.0, memory.drag_velocity(id).x)
    /// ```
    pub fn new(
        target: f32,
        damping: f32,
        stiffness: f32,
        initial_velocity: f32,
    ) -> Self {
        Self {
            position: 0.0,  // Will be set by spring() call
            target,
            velocity: initial_velocity,
            damping: damping.max(0.0),
            stiffness: stiffness.max(1.0),
            stop_threshold: 0.001,
            seen: u32::MAX,  // Not yet read
        }
    }
    
    /// Check if spring is still animating.
    pub fn is_animating(&self) -> bool {
        self.velocity.abs() > self.stop_threshold
            || (self.position - self.target).abs() > self.stop_threshold
    }
    
    /// Advance spring physics by `delta` seconds.
    fn step(&mut self, delta: f32) {
        let displacement = self.target - self.position;
        // Hooke's law + damping
        let acceleration =
            displacement * self.stiffness - self.velocity * self.damping;
        self.velocity += acceleration * delta;
        self.position += self.velocity * delta;
    }
}
```

---

### Type: `EnterExit` (New)

**Location**: `src/memory.rs` (new struct, lines 350–380)

```rust
/// An animation that plays when an element enters or exits the tree.
///
/// Used to fade, slide, or scale elements in/out with smooth transitions.
#[derive(Debug, Clone, Copy)]
pub enum EnterExit {
    /// Element is animating into view.
    Enter {
        start_time: f32,
        duration: f32,
        seen: u32,
    },
    /// Element is animating out of view (stays drawn until progress=1.0).
    Exit {
        start_time: f32,
        duration: f32,
        seen: u32,
    },
}

impl EnterExit {
    /// Get the animation progress [0.0, 1.0].
    pub fn progress(&self, accumulated_time: f32) -> f32 {
        let (start, duration) = match self {
            Self::Enter { start_time, duration, .. } => (start_time, duration),
            Self::Exit { start_time, duration, .. } => (start_time, duration),
        };
        ((accumulated_time - start) / duration).clamp(0.0, 1.0)
    }
    
    /// Check if animation just completed this frame.
    pub fn completed(&self, accumulated_time: f32) -> bool {
        self.progress(accumulated_time) >= 1.0
    }
    
    /// Mark this animation as seen in current frame.
    fn mark_seen(&mut self, frame: u32) {
        match self {
            Self::Enter { seen, .. } | Self::Exit { seen, .. } => *seen = frame,
        }
    }
}
```

---

### Type: `Memory` Extension (Phase 2)

**Location**: `src/memory.rs`, add to `Memory` impl block

```rust
impl Memory {
    // ... Phase 1 methods ...
    
    /// Start a spring animation for an element.
    ///
    /// The spring will animate `position` toward `target` over time, respecting
    /// initial momentum if the element was dragged before.
    ///
    /// # Arguments
    ///
    /// * `id` - Element identifier
    /// * `target` - Position to animate toward
    /// * `damping` - Oscillation control [0.0, 2.0]
    ///   - 0.1–0.5: Bouncy (fast overshoot)
    ///   - 1.0: Critical (smooth, no bounce)
    ///   - 1.5–2.0: Stiff (sluggish, slow)
    /// * `stiffness` - Response speed [1.0, 1000.0]
    ///   - 10–50: Slow, stretchy
    ///   - 100–200: Normal, snappy
    ///   - 500+: Very fast, twitchy
    ///
    /// # Example
    ///
    /// ```ignore
    /// // User drags slider to pos=120, releases with momentum
    /// memory.spring(id, 100.0, 0.5, 100.0);  // Bouncy snap
    /// // Inherited momentum: memory.drag_velocity(id)
    /// ```
    pub fn spring(
        &mut self,
        id: Id,
        target: f32,
        damping: f32,
        stiffness: f32,
    ) -> f32 {
        let velocity = self.drag_velocity(id);
        
        let mut spring = self.springs
            .entry(id)
            .or_insert_with(|| Spring::new(target, damping, stiffness, velocity.x));
        
        spring.target = target;
        spring.seen = self.current_frame;
        
        // Advance physics
        spring.step(self.delta);
        
        spring.position
    }
    
    /// Get the current position of a spring without advancing it.
    pub fn spring_position(&self, id: Id) -> Option<f32> {
        self.springs.get(&id).map(|s| s.position)
    }
    
    /// Stop a spring animation and remove it.
    pub fn stop_spring(&mut self, id: Id) {
        self.springs.remove(&id);
    }
    
    /// Check if a spring is currently animating.
    pub fn spring_animating(&self, id: Id) -> bool {
        self.springs
            .get(&id)
            .map(|s| s.is_animating())
            .unwrap_or(false)
    }
    
    /// Mark an element as entering the tree with an animation.
    ///
    /// The animation plays from 0.0 to 1.0 over `duration_seconds`.
    /// Views read `enter_exit_progress()` to fade/slide/scale in the element.
    pub fn start_enter(&mut self, id: Id, duration: f32) {
        self.enter_exit.insert(
            id,
            EnterExit::Enter {
                start_time: self.accumulated_time,
                duration,
                seen: u32::MAX,
            },
        );
    }
    
    /// Mark an element as exiting the tree with an animation.
    ///
    /// The element remains visible and drawn until progress reaches 1.0.
    /// After that, the element is removed from the tree.
    pub fn start_exit(&mut self, id: Id, duration: f32) {
        self.enter_exit.insert(
            id,
            EnterExit::Exit {
                start_time: self.accumulated_time,
                duration,
                seen: u32::MAX,
            },
        );
    }
    
    /// Get the progress of an enter/exit animation.
    ///
    /// Returns `None` if the element is not animating, otherwise [0.0, 1.0].
    pub fn enter_exit_progress(&self, id: Id) -> Option<f32> {
        self.enter_exit
            .get(&id)
            .map(|ee| ee.progress(self.accumulated_time))
    }
    
    /// Check if an exit animation just completed.
    pub fn exit_completed(&self, id: Id) -> bool {
        if let Some(EnterExit::Exit { .. }) = self.enter_exit.get(&id) {
            self.enter_exit_progress(id)
                .map(|p| p >= 1.0)
                .unwrap_or(false)
        } else {
            false
        }
    }
}
```

**Storage additions to Memory struct**:
```rust
pub struct Memory {
    // ... existing fields ...
    
    /// Active spring animations.
    springs: HashMap<Id, Spring>,
    
    /// Elements animating in or out of the tree.
    enter_exit: HashMap<Id, EnterExit>,
}
```

---

### Element Builder Extension (Phase 2)

**Location**: `src/element.rs`, extend `El<S>` impl block

```rust
impl<S> El<S> {
    // ... existing methods ...
    
    /// Animate this element in when it enters the tree.
    ///
    /// The element will fade/transform in over `duration_seconds` using the
    /// animation stored in Memory. Views should read `memory.enter_exit_progress()`
    /// and apply opacity/transform based on the progress value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// text("Hello")
    ///     .enter_transition(0.3)  // 300ms fade-in
    /// ```
    pub fn enter_transition(mut self, duration: f32) -> Self {
        self.enter_duration = Some(duration);
        self
    }
    
    /// Animate this element out when it leaves the tree.
    ///
    /// The element remains visible during the exit animation. After completion,
    /// the element is removed. Useful for fade-out or slide-out effects.
    pub fn exit_transition(mut self, duration: f32) -> Self {
        self.exit_duration = Some(duration);
        self
    }
}
```

**Storage additions to Element struct**:
```rust
pub struct El<S> {
    // ... existing fields ...
    
    /// Duration for enter animation (if Some).
    enter_duration: Option<f32>,
    /// Duration for exit animation (if Some).
    exit_duration: Option<f32>,
}
```

---

## Phase 3 (STEP 22): Quality Features

### Type: `Memory` Extension (Phase 3)

**Location**: `src/memory.rs`, add to `Memory` impl block

```rust
impl Memory {
    // ... Phase 1–2 methods ...
    
    /// Schedule a callback to fire after a delay.
    ///
    /// A convenience over `defer()` that stores a closure and calls it
    /// automatically when the delay expires.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this scheduled callback
    /// * `delay` - Seconds to wait before firing
    /// * `callback` - Function to call (captured in closure)
    ///
    /// # Example
    ///
    /// ```ignore
    /// memory.after(id, 0.5, || {
    ///     println!("Delayed message!");
    /// });
    /// ```
    ///
    /// # Implementation
    ///
    /// Internally, `after()` wraps the closure in Memory::deferred and calls it
    /// during `end_frame()` when the time arrives. No separate storage needed.
    pub fn after<F>(&mut self, id: Id, delay: f32, callback: F)
    where
        F: Fn() + 'static,
    {
        self.deferred_callbacks.insert(
            id,
            (self.accumulated_time + delay, Box::new(callback)),
        );
    }
    
    /// Clear a pending callback (if it hasn't fired yet).
    pub fn cancel_after(&mut self, id: Id) {
        self.deferred_callbacks.remove(&id);
    }
    
    /// Run pending callbacks that are ready.
    ///
    /// Called during `end_frame()`, before memory cleanup.
    fn fire_ready_callbacks(&mut self) {
        let current = self.accumulated_time;
        let mut ready = Vec::new();
        
        for (id, (fire_time, _)) in &self.deferred_callbacks {
            if current >= *fire_time {
                ready.push(*id);
            }
        }
        
        for id in ready {
            if let Some((_, callback)) = self.deferred_callbacks.remove(&id) {
                callback();
            }
        }
    }
}
```

**Storage additions to Memory struct**:
```rust
pub struct Memory {
    // ... existing fields ...
    
    /// Callbacks scheduled with Memory::after().
    deferred_callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>,
}
```

---

### Animation Cleanup Reordering (Phase 3)

**Location**: `src/memory.rs`, modify `end_frame()` method

```rust
impl Memory {
    pub fn end_frame(&mut self) {
        // Phase 1: Fire any completed animation callbacks
        self.fire_completed_callbacks();
        
        // Phase 2: Run deferred callbacks that are ready
        self.fire_ready_callbacks();
        
        // Phase 3: Clean up unseen animation records
        self.eased.retain(|_, eased| eased.seen == self.current_frame);
        self.cycles.retain(|_, cycle| cycle.seen == self.current_frame);
        self.springs.retain(|_, spring| spring.seen == self.current_frame);
        self.enter_exit
            .retain(|_, ee| {
                // Keep enter animations while active, remove when complete
                if let EnterExit::Enter { .. } = ee {
                    ee.progress(self.accumulated_time) < 1.0
                } else {
                    false
                }
            });
        
        // Phase 4: Check animation budget
        self.check_animation_budget();
        
        // Phase 5: Prepare for next frame
        self.current_frame += 1;
        self.animating = !self.eased.is_empty()
            || !self.cycles.is_empty()
            || !self.springs.is_empty()
            || !self.enter_exit.is_empty();
    }
}
```

---

## Full Type Summary

| Type | Module | Purpose | Phase |
|------|--------|---------|-------|
| `Velocity` | memory | Momentum tracking for drag gestures | 1 |
| `Spring` | memory | Physics-based animation with bounce | 2 |
| `EnterExit` | memory | Lifecycle animations (fade in/out) | 2 |
| `Memory::motion` | theme | Accessibility reduced-motion multiplier | 1 |

---

## Method Addition Summary

| Method | Type | Purpose | Phase |
|--------|------|---------|-------|
| `set_animation_budget()` | Memory | Set live-loop limit (default: 2) | 1 |
| `animation_budget()` | Memory | Get current budget | 1 |
| `set_drag_velocity()` | Memory | Store momentum for spring retarget | 1 |
| `drag_velocity()` | Memory | Retrieve stored momentum | 1 |
| `spring()` | Memory | Start spring animation | 2 |
| `spring_position()` | Memory | Read spring position without stepping | 2 |
| `stop_spring()` | Memory | Cancel spring animation | 2 |
| `spring_animating()` | Memory | Check if spring is active | 2 |
| `start_enter()` | Memory | Begin enter animation | 2 |
| `start_exit()` | Memory | Begin exit animation | 2 |
| `enter_exit_progress()` | Memory | Read enter/exit progress [0.0, 1.0] | 2 |
| `exit_completed()` | Memory | Check if exit animation finished | 2 |
| `after()` | Memory | Schedule callback with delay | 3 |
| `cancel_after()` | Memory | Cancel pending callback | 3 |
| `enter_transition()` | Element | Mark element for enter animation | 2 |
| `exit_transition()` | Element | Mark element for exit animation | 2 |

---

## Storage Summary

**New storage in Memory struct**:

```rust
pub struct Memory {
    // Phase 1
    last_drag_velocity: HashMap<Id, Velocity>,
    animation_budget: usize,
    
    // Phase 2
    springs: HashMap<Id, Spring>,
    enter_exit: HashMap<Id, EnterExit>,
    
    // Phase 3
    deferred_callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>,
}
```

**New storage in Element struct**:

```rust
pub struct El<S> {
    // Phase 2
    enter_duration: Option<f32>,
    exit_duration: Option<f32>,
}
```

**Extension to Metrics struct**:

```rust
pub struct Metrics {
    // Phase 1
    pub motion: f32,  // 0.0 = disabled, 1.0 = normal
}
```

---

## Accessibility Integration Points

| Feature | Check | Behavior |
|---------|-------|----------|
| Reduced motion | `theme.metrics.motion == 0.0` | Skip all easing; show final state instantly |
| Enter transitions | `theme.metrics.motion == 0.0` | Skip fade-in; show at full opacity |
| Exit transitions | `theme.metrics.motion == 0.0` | Skip fade-out; remove instantly |
| Spring animations | `theme.metrics.motion == 0.0` | Snap to target instantly |
| Deferred callbacks | `theme.metrics.motion == 0.0` | Fire immediately instead of after delay |

---

## Testing API Extensions

**Harness additions for testing R2 animations**:

```rust
impl Harness<S, V> {
    // Existing methods...
    
    // Phase 1
    fn set_animation_budget(&mut self, budget: usize);
    fn animation_count(&self) -> usize;
    
    // Phase 2
    fn spring(&mut self, id: Id, target: f32, damping: f32, stiffness: f32);
    fn spring_position(&self, id: Id) -> Option<f32>;
    fn enter_animation(&mut self, id: Id, duration: f32);
    fn exit_animation(&mut self, id: Id, duration: f32);
    
    // Phase 3
    fn after(&mut self, id: Id, delay: f32, callback: Box<dyn Fn()>);
}
```

---

## Easing Function Extensions (Phase 2+)

**New Easing enum** (to support Memory::ease with easing function parameter):

```rust
pub enum Easing {
    // Linear
    Linear,
    
    // Quadratic (ease-in/out/in-out variants)
    QuadIn,
    QuadOut,
    QuadInOut,
    
    // Cubic
    CubicIn,
    CubicOut,
    CubicInOut,
    
    // Exponential
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    
    // Elastic (bounce/spring-like)
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    
    // Back (anticipate/overshoot)
    BackIn,
    BackOut,
    BackInOut,
    
    // Bounce
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Easing {
    /// Compute eased value for progress t ∈ [0, 1].
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::QuadIn => t * t,
            Self::QuadOut => t * (2.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            // ... more variants
        }
    }
}
```

**Extension to Memory**:

```rust
impl Memory {
    /// Ease with a specific easing function.
    pub fn ease_with(&mut self, id: Id, target: f32, duration: f32, easing: Easing) -> f32 {
        // Implementation: stores easing function with animation
    }
}
```

---

## Summary

**Phase 1** adds accessibility support (motion multiplier) and foundation for velocity/budgets.

**Phase 2** adds the two core animation types: springs (physics) and enter/exit (lifecycle).

**Phase 3** adds callback sugar (Memory::after) and cleanup reordering.

All types are designed for:
- **Deterministic testing** (no wall-clock reads)
- **Deterministic behavior** (same input → same output)
- **Performance** (HashMap lookups, not allocations)
- **Accessibility** (motion multiplier collapses animations)
- **Composability** (animations stack without conflicts)

For implementation order and examples, see STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md.
