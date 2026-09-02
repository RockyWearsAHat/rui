# STEP 19: R2 Motion Kit Integration & Migration Checklist

**Purpose**: Provide step-by-step checklists for implementing R2 features and migrating existing animations.

---

## Pre-Implementation Checklist (Before STEP 20)

### Code Review & Understanding
- [ ] Read `STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md` (2 min overview)
- [ ] Read `STEP_19_R2_MOTION_KIT_AUDIT.md` (10 min high-level)
- [ ] Scan `STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md` for overall structure
- [ ] Review 4 working primitives in code:
  - [ ] `Painter::ease()` at src/paint.rs:147–153
  - [ ] `Memory::ease()` at src/memory.rs:259–285
  - [ ] `Painter::phase()` at src/paint.rs:155–178
  - [ ] `Memory::phase()` at src/memory.rs:287–310
- [ ] Understand 5 Memory fields:
  - [ ] `Memory.eased` (HashMap<Id, Eased>)
  - [ ] `Memory.cycles` (HashMap<Id, Cycle>)
  - [ ] `Memory.deferred` (HashMap<Id, f32>)
  - [ ] `Memory.transitions` (HashMap<Id, (f32, f32)>)
  - [ ] `Memory.accumulated_time` (f32)

### Environment Setup
- [ ] Verify `cargo test --lib` passes (all 396 tests)
- [ ] Verify `cargo test --test r2_motion_kit_audit` shows 27 passing, 12 ignored
- [ ] Set up performance baseline:
  ```bash
  cargo run --release --example gallery > baseline_perf.txt
  ```
- [ ] Clone working branch or create feature branch:
  ```bash
  git checkout -b r2-implementation
  ```

### Acceptance Criteria Review
- [ ] Read Phase 1 acceptance tests (3 stubs):
  - [ ] `r2_acceptance_metrics_motion_zero_disables_animation`
  - [ ] `r2_acceptance_velocity_inheritance_smooth_retargeting`
  - [ ] `r2_acceptance_2_live_loop_budget_enforced`
- [ ] Read Phase 2 acceptance tests (2 stubs):
  - [ ] `r2_acceptance_spring_integration_basic`
  - [ ] `r2_acceptance_enter_exit_transitions`
- [ ] Read Phase 3 acceptance tests (2 stubs):
  - [ ] `r2_acceptance_memory_after_callback_execution`
  - [ ] `r2_acceptance_animation_cleanup_on_element_removal`

---

## STEP 20: Phase 1 Implementation Checklist

### Gap 5: Metrics.motion Check (Very Low Complexity)

**Understanding Phase**
- [ ] Find where animations are initialized:
  - [ ] `Painter::ease()` call site (src/paint.rs:147)
  - [ ] `Memory::ease()` call site (src/memory.rs:259)
  - [ ] `Painter::phase()` call site (src/paint.rs:155)
- [ ] Understand Metrics structure:
  - [ ] Where Metrics comes from (Theme)
  - [ ] Where motion field is defined
  - [ ] What values are valid (0.0 = disabled, 1.0 = normal)

**Implementation Phase**
- [ ] Modify `Painter::ease()`:
  ```rust
  pub fn ease(&mut self, key: impl Into<Id>, to: f32, secs: f32) {
      if self.theme.metrics.motion == 0.0 {
          // Animation disabled, jump to target immediately
          self.memory.eased.remove(&key.into());
          return;
      }
      // Existing ease logic
  }
  ```
- [ ] Modify `Painter::phase()` (same check)
- [ ] Add test for motion=0 behavior:
  ```bash
  cargo test r2_acceptance_metrics_motion_zero_disables_animation
  ```
- [ ] Verify test passes (currently ignored, uncomment after implementation)

**Verification Phase**
- [ ] Run: `cargo test --test r2_motion_kit_audit` (27 baseline + 1 Phase 1)
- [ ] Run: `cargo test --lib` (should show 396 passing, no regressions)
- [ ] Benchmark: `cargo run --release --example gallery`
  - Expected: No performance change (metrics check is ~1ns)

**Commit**
```bash
git commit -m "STEP 20 Phase 1a: Add Metrics.motion check to animation functions

- Painter::ease(), phase() skip animation if Metrics.motion == 0.0
- Accessibility: Users with motion sickness get snappy UI
- Performance: Check is O(1), negligible overhead
- Test: r2_acceptance_metrics_motion_zero_disables_animation passes
- No regression: All 396 library tests passing"
```

### Gap 6: Velocity Inheritance (Low Complexity)

**Understanding Phase**
- [ ] Find velocity source:
  - [ ] `Pointing` struct definition (src/input.rs)
  - [ ] Where velocity is calculated from drag distance
  - [ ] How velocity flows to handlers
- [ ] Understand use case:
  - [ ] Dragging slider and releasing should continue with momentum
  - [ ] Spring should respect velocity for smooth transition

**Implementation Phase**
- [ ] Add velocity field to `Eased` struct:
  ```rust
  pub struct Eased {
      start: f32,
      target: f32,
      elapsed: f32,
      duration: f32,
      velocity: f32,  // ← NEW
  }
  ```
- [ ] Modify `Painter::ease()` to accept optional velocity:
  ```rust
  pub fn ease_with_velocity(&mut self, key: impl Into<Id>, to: f32, secs: f32, velocity: f32) {
      // Velocity affects initial slope but doesn't change target
      // Implementation: modify evaluate() to apply velocity
  }
  ```
- [ ] Add test:
  ```rust
  #[test]
  fn r2_acceptance_velocity_inheritance_smooth_retargeting() {
      let mut h = Harness::new(App::default(), view);
      
      // Drag slider fast
      h.drag_text("Slider", Point::new(100.0, 0.0), velocity: 50.0);
      
      // Release - should continue with velocity
      h.release_drag();
      h.frames(10);
      
      // Slider position should have moved further due to velocity
      assert!(h.state().slider_value > 0.5);
  }
  ```

**Verification Phase**
- [ ] Run: `cargo test r2_acceptance_velocity_inheritance_smooth_retargeting`
- [ ] Manual test: Drag slider quickly, should feel continuous
- [ ] Benchmark: Should add < 1µs per ease calculation

**Commit**
```bash
git commit -m "STEP 20 Phase 1b: Add velocity inheritance to easing

- Eased struct now tracks velocity for smooth retargeting
- Painter::ease_with_velocity() accepts initial velocity
- Dragging and releasing now feels continuous (no jump)
- Test: r2_acceptance_velocity_inheritance_smooth_retargeting passes
- Performance: +1µs per ease (negligible, < 0.1% budget)"
```

### Gap 4: 2-Live-Loop Budget (Very Low Complexity)

**Understanding Phase**
- [ ] Definition of "live loop":
  - [ ] Animation that hasn't settled to its target
  - [ ] Spring oscillating or ease still progressing
- [ ] Find where animations are tracked:
  - [ ] `Memory.animating` field (current flag)
  - [ ] Where animating is set/cleared
- [ ] Purpose of budget:
  - [ ] Prevent runaway animations causing frame drops
  - [ ] Safety constraint (hard limit 2 per frame)

**Implementation Phase**
- [ ] Add live loop counter to `Memory`:
  ```rust
  pub struct Memory {
      // ... existing fields ...
      live_animation_count: usize,
  }
  ```
- [ ] In `Memory::begin_frame()`:
  ```rust
  pub fn begin_frame(&mut self, elapsed: f32, redraw: &mut bool) {
      self.live_animation_count = 0;
      
      for (_, eased) in self.eased.iter() {
          if !eased.is_settled() {
              self.live_animation_count += 1;
          }
      }
      
      if self.live_animation_count > 2 {
          // Log warning and disable oldest animation
          warn!("Live animation budget exceeded: {} > 2", 
              self.live_animation_count);
          // Disable oldest animation to enforce budget
      }
  }
  ```
- [ ] Add test:
  ```rust
  #[test]
  fn r2_acceptance_2_live_loop_budget_enforced() {
      let mut h = Harness::new(App::default(), view);
      
      // Start 3 animations
      h.ease("anim1", 100.0, 1.0);
      h.ease("anim2", 100.0, 1.0);
      h.ease("anim3", 100.0, 1.0);
      
      h.frames(1);
      
      // Should log warning about exceeding budget
      // Oldest animation should be disabled
      assert_eq!(h.live_animation_count(), 2);
  }
  ```

**Verification Phase**
- [ ] Run: `cargo test r2_acceptance_2_live_loop_budget_enforced`
- [ ] Check log output for budget warning
- [ ] Verify no frame drops with 50 concurrent animations (budget enforces limit)

**Commit**
```bash
git commit -m "STEP 20 Phase 1c: Enforce 2-live-loop budget for animation safety

- Memory tracks live_animation_count
- Warning logged when > 2 animations live simultaneously
- Oldest animation disabled to enforce budget
- Prevents runaway animation from causing frame drops
- Test: r2_acceptance_2_live_loop_budget_enforced passes
- Performance: +1 integer comparison per frame (negligible)"
```

### Phase 1 Sign-Off
- [ ] All 3 gaps implemented
- [ ] All 3 acceptance tests pass (uncommented)
- [ ] All 396 library tests still passing
- [ ] Performance baseline unchanged (< 0.05ms overhead)
- [ ] Code review passed (checked for correctness)
- [ ] Commit message references all 3 gaps
- [ ] Branch ready to merge to main

**Phase 1 Commit Message Template**:
```
STEP 20: Phase 1 - Animation foundation (metrics + velocity + budget)

Implements 3 foundation gaps:
- Gap 5: Metrics.motion check (animation disabled for accessibility)
- Gap 6: Velocity inheritance (smooth momentum after drag)
- Gap 4: 2-live-loop budget (safety constraint, max 2 live per frame)

Tests:
- 3 Phase 1 acceptance tests now passing
- All 396 library tests passing, no regressions
- Performance overhead: +0.05ms per frame (< 1% budget)

Ready for Phase 2 (springs + enter/exit).
```

---

## STEP 21: Phase 2 Implementation Checklist

### Gap 1: Springs with Bounce (Moderate Complexity)

**Understanding Phase**
- [ ] Read spring physics theory:
  - [ ] Hooke's law: F = -kx (restoring force)
  - [ ] Damping: -c*v (velocity-dependent friction)
  - [ ] Numerical solver: Euler integration
- [ ] Review similar implementation (if exists in codebase)
- [ ] Understand parametrization:
  - [ ] `bounce`: overshoot amount (1.0 = no bounce, 1.2 = 20% overshoot)
  - [ ] `damping`: how quickly spring settles (0.7-0.9 typical)

**Implementation Phase**
- [ ] Add Spring struct to Memory:
  ```rust
  pub struct Spring {
      position: f32,
      velocity: f32,
      target: f32,
      bounce: f32,
      damping: f32,
  }
  
  impl Spring {
      pub fn step(&mut self, dt: f32) {
          let displacement = self.target - self.position;
          let force = displacement * self.bounce;
          self.velocity += force * dt;
          self.velocity *= 1.0 - self.damping * dt;
          self.position += self.velocity * dt;
      }
      
      pub fn is_settled(&self) -> bool {
          (self.target - self.position).abs() < 0.001
      }
  }
  ```
- [ ] Add springs HashMap to Memory:
  ```rust
  pub struct Memory {
      // ... existing fields ...
      springs: HashMap<Id, Spring>,
  }
  ```
- [ ] Add Painter::spring() method:
  ```rust
  pub fn spring(&mut self, key: impl Into<Id>, to: f32, 
                config: &SpringConfig, velocity: f32) {
      if self.theme.metrics.motion == 0.0 {
          return;  // Skip if motion disabled
      }
      // Create spring and store in Memory
      let spring = Spring {
          position: current_value,  // Get from render state
          velocity,
          target: to,
          bounce: config.bounce,
          damping: config.damping,
      };
      self.memory.springs.insert(key.into(), spring);
  }
  ```
- [ ] Update Memory::begin_frame() to step springs:
  ```rust
  for spring in self.springs.values_mut() {
      spring.step(elapsed);
      if spring.is_settled() {
          spring.position = spring.target;  // Snap to target
      }
  }
  ```
- [ ] Add test:
  ```rust
  #[test]
  fn r2_acceptance_spring_integration_basic() {
      let mut h = Harness::new(App { value: 0.0 }, view);
      
      h.spring("test", 100.0, &SpringConfig {
          bounce: 1.2,
          damping: 0.8,
      }, 0.0);
      
      h.frames(1);
      // Spring should start moving toward 100.0
      assert!(h.state().value > 0.0);
      
      h.frames(29);  // ~500ms at 60fps
      // Spring should overshoot 100.0 then settle back
      let peak = h.rendered_value("spring");
      assert!(peak > 100.0 && peak < 120.0);  // 20% overshoot
      
      h.frames(30);  // Another ~500ms
      // Spring should settle near 100.0
      let final_value = h.state().value;
      assert!((final_value - 100.0).abs() < 1.0);
  }
  ```

**Verification Phase**
- [ ] Run: `cargo test r2_acceptance_spring_integration_basic`
- [ ] Visual test: `cargo run --example gallery` and watch spring animations
- [ ] Benchmark: Measure spring solver overhead
  ```bash
  cargo bench --bench animation_perf -- spring
  # Expected: < 2µs per spring per frame
  ```
- [ ] Scale test: 50 springs should not exceed budget
  ```bash
  cargo test r2_acceptance_scale_50_springs
  # Expected: < 1ms per frame total
  ```

**Commit**
```bash
git commit -m "STEP 21 Phase 2a: Implement spring physics for elastic animations

- Spring struct with position, velocity, target, bounce, damping
- Numerical solver using Euler integration (O(1) per spring)
- Painter::spring() method for triggering spring animations
- Configurable bounce (1.0-1.5) and damping (0.7-0.9)
- Test: r2_acceptance_spring_integration_basic passes
- Performance: ~0.1ms per animation (within budget)"
```

### Gap 2: Enter/Exit Transitions (Low Complexity)

**Understanding Phase**
- [ ] Enter: Element appears with fade-in/slide animation
- [ ] Exit: Element disappears with fade-out/slide animation
- [ ] Both driven by springs (for elastic feel)
- [ ] Need to track enter/exit state per element

**Implementation Phase**
- [ ] Add EnterExit state to Memory:
  ```rust
  pub enum ElementState {
      Entering { progress: f32, duration: f32 },
      Active,
      Exiting { progress: f32, duration: f32 },
      Removed,
  }
  
  pub struct Memory {
      // ... existing fields ...
      element_states: HashMap<Id, ElementState>,
  }
  ```
- [ ] Add El methods:
  ```rust
  impl<S: 'static> El<S> {
      pub fn enter(mut self, duration: f32) -> Self {
          self.enter_duration = Some(duration);
          self
      }
      
      pub fn exit(mut self, duration: f32) -> Self {
          self.exit_duration = Some(duration);
          self
      }
  }
  ```
- [ ] In paint loop, apply enter/exit transforms:
  ```rust
  if let Some(state) = memory.element_states.get(&element_id) {
      match state {
          ElementState::Entering { progress, .. } => {
              let alpha = painter.ease("enter_fade", *progress, 0.0);
              painter.set_alpha(alpha);
          }
          ElementState::Exiting { progress, .. } => {
              let alpha = painter.ease("exit_fade", 1.0 - *progress, 0.0);
              painter.set_alpha(alpha);
          }
          _ => {}
      }
  }
  ```
- [ ] Add test:
  ```rust
  #[test]
  fn r2_acceptance_enter_exit_transitions() {
      let mut h = Harness::new(App { show_item: false }, view);
      
      // Show item with enter animation
      h.click_text("Show");
      h.frames(1);
      assert!(h.state().show_item);
      
      // Item should fade in over 300ms
      h.frames(9);  // ~150ms
      let alpha = h.painted_alpha("item");
      assert!(alpha > 0.3 && alpha < 0.7);  // Mid-fade
      
      h.frames(9);  // Complete fade-in
      assert!(h.painted_alpha("item") >= 0.95);
      
      // Hide item with exit animation
      h.click_text("Hide");
      h.frames(1);
      assert!(!h.state().show_item);
      
      h.frames(9);  // ~150ms of fade-out
      let alpha = h.painted_alpha("item");
      assert!(alpha > 0.3 && alpha < 0.7);  // Mid-fade
      
      h.frames(9);  // Complete fade-out
      assert!(h.painted_alpha("item") < 0.05);
  }
  ```

**Verification Phase**
- [ ] Run: `cargo test r2_acceptance_enter_exit_transitions`
- [ ] Visual test: Gallery items should fade in/out smoothly
- [ ] Verify stagger works with enter:
  ```rust
  for (i, item) in items.iter().enumerate() {
      text(&item.name)
          .enter(0.3)
          .stagger(i as f32 * 0.1)
  }
  ```

**Commit**
```bash
git commit -m "STEP 21 Phase 2b: Implement enter/exit transitions

- ElementState enum (Entering/Active/Exiting/Removed)
- El::enter(duration) and El::exit(duration) methods
- Alpha fading driven by easing/spring
- Optional transform_enter/transform_exit for slide/scale
- Stagger support for list choreography
- Test: r2_acceptance_enter_exit_transitions passes
- Performance: +0.05ms (tracking enter/exit state)"
```

### Phase 2 Sign-Off
- [ ] Both gaps implemented (springs + enter/exit)
- [ ] Both acceptance tests pass (uncommented)
- [ ] All 396 library tests still passing
- [ ] Performance validated (< 0.6ms total overhead for Phase 1+2)
- [ ] Gallery example shows smooth spring animations
- [ ] List items cascade smoothly with stagger
- [ ] Code review passed

---

## STEP 22: Phase 3 Implementation Checklist

### Gap 3: Memory::after() Sugar (Low Complexity)

**Understanding Phase**
- [ ] Currently: Defer waits, then you check in update handler
- [ ] Desired: After() waits, then runs callback automatically
- [ ] Callback is a closure that captures and mutates state

**Implementation Phase**
- [ ] Add Callback struct:
  ```rust
  pub struct Callback {
      delay_remaining: f32,
      handler: Box<dyn Fn(&mut S)>,
  }
  ```
- [ ] Add to Memory:
  ```rust
  pub struct Memory {
      // ... existing fields ...
      callbacks: HashMap<Id, Callback>,
  }
  ```
- [ ] Add Memory::after() method:
  ```rust
  pub fn after<S: 'static>(&mut self, delay: f32, handler: impl Fn(&mut S) + 'static) -> Id {
      let id = Id::new();
      self.callbacks.insert(id, Callback {
          delay_remaining: delay,
          handler: Box::new(handler),
      });
      id
  }
  ```
- [ ] In paint loop, invoke callbacks:
  ```rust
  let mut expired = Vec::new();
  for (id, callback) in self.callbacks.iter_mut() {
      callback.delay_remaining -= elapsed;
      if callback.delay_remaining <= 0.0 {
          expired.push(id.clone());
      }
  }
  for id in expired {
      if let Some(callback) = self.callbacks.remove(&id) {
          callback.handler(&mut state);  // Invoke callback
      }
  }
  ```

**Verification Phase**
- [ ] Test:
  ```rust
  #[test]
  fn r2_acceptance_memory_after_callback_execution() {
      let mut h = Harness::new(App { count: 0 }, view);
      
      h.after(1.0, |app| app.count += 1);
      h.frames(59);  // ~1 second at 60fps
      
      assert_eq!(h.state().count, 0);  // Not yet
      h.frames(1);   // At 1 second
      assert_eq!(h.state().count, 1);  // Callback executed
  }
  ```

### Gap 7: Cleanup Policy (Low Complexity)

**Understanding Phase**
- [ ] Problem: Animations linger after elements are removed
- [ ] Solution: Clean up when element key is removed from tree
- [ ] Deferred cleanup: Happens at frame end, not during render

**Implementation Phase**
- [ ] Track active element IDs during render:
  ```rust
  pub struct Memory {
      active_element_ids: HashSet<Id>,
      // ... existing fields ...
  }
  ```
- [ ] Collect IDs during render:
  ```rust
  fn collect_active_ids(element: &El<S>, ids: &mut HashSet<Id>) {
      ids.insert(element.id());
      for child in &element.children {
          collect_active_ids(child, ids);
      }
  }
  ```
- [ ] Clean up in begin_frame():
  ```rust
  pub fn begin_frame(&mut self, elapsed: f32, redraw: &mut bool) {
      // Collect active IDs from element tree
      let active_ids = self.collect_active_element_ids();
      
      // Remove animations for elements that disappeared
      self.eased.retain(|id, _| active_ids.contains(id));
      self.springs.retain(|id, _| active_ids.contains(id));
      self.callbacks.retain(|id, _| active_ids.contains(id));
  }
  ```

**Verification Phase**
- [ ] Test:
  ```rust
  #[test]
  fn r2_acceptance_animation_cleanup_on_element_removal() {
      let mut h = Harness::new(App { show_item: true }, view);
      
      h.ease("item_scale", 1.2, 0.5);
      h.frames(10);
      
      // Remove item from tree
      h.state_mut().show_item = false;
      h.frames(1);
      
      // Animation should be cleaned up
      assert!(!h.memory().has_animation("item_scale"));
  }
  ```

### Phase 3 Sign-Off
- [ ] Both gaps implemented (after + cleanup)
- [ ] All 12 acceptance tests passing
- [ ] All 396 library tests passing
- [ ] Zero memory leaks (animations cleaned up when removed)
- [ ] Performance validated (< 0.7ms total overhead for all 3 phases)

---

## Migration Checklist: Existing Code

### Step 1: Find All Manual Timer Checking
```bash
grep -r "deferred_expired\|defer" src/ examples/
```
- [ ] For each `defer` call:
  - [ ] Replace with `.after()` if deferred action needs to happen
  - [ ] Use `.after(delay, |app| { })` closure to run action

### Step 2: Find All Manual Animations
```bash
grep -r "ease\|phase" examples/
```
- [ ] For each animation, check:
  - [ ] Is it a simple value transition? Use `.ease()`
  - [ ] Is it a bouncy interaction? Use `.spring()`
  - [ ] Is it periodic? Use `.phase()`
- [ ] Add velocity to drag-release animations
- [ ] Add enter/exit to modal and list items

### Step 3: Audit Accessibility
```bash
grep -r "Metrics.motion" src/
```
- [ ] Every animation should check `theme.metrics.motion > 0.0`
- [ ] Add: `if theme.metrics.motion == 0.0 { jump_to_target() }`

### Step 4: List Reordering
```bash
grep -r "for.*enumerate" examples/
```
- [ ] Check if any list is reordered or filtered
- [ ] Add `.key(item.id)` to prevent animation state from moving

### Step 5: Performance Validation
- [ ] Measure before: `cargo run --release --example gallery`
- [ ] Migrate all animations
- [ ] Measure after: Should be < 1.5ms frame time
- [ ] If exceeded, reduce live animation count (2-live-loop budget)

---

## Testing Checklist (Per Phase)

### Phase 1 Testing
```bash
# Compile check
cargo check

# Run Phase 1 acceptance tests
cargo test r2_acceptance_metrics_motion_zero_disables_animation
cargo test r2_acceptance_velocity_inheritance_smooth_retargeting
cargo test r2_acceptance_2_live_loop_budget_enforced

# Run all baseline tests (ensure no regression)
cargo test --test r2_motion_kit_audit

# Run full test suite
cargo test --lib

# Performance baseline
cargo run --release --example gallery 2>&1 | grep "frame_time"
```

### Phase 2 Testing
```bash
# Run Phase 2 acceptance tests
cargo test r2_acceptance_spring_integration_basic
cargo test r2_acceptance_enter_exit_transitions

# Scale testing
cargo test r2_acceptance_scale_50_springs
cargo test r2_acceptance_scale_100_staggered_items

# Performance check (should not exceed 1ms)
time cargo run --release --example gallery > /dev/null
```

### Phase 3 Testing
```bash
# Run Phase 3 acceptance tests
cargo test r2_acceptance_memory_after_callback_execution
cargo test r2_acceptance_animation_cleanup_on_element_removal

# Memory profiling
valgrind cargo test --lib 2>&1 | grep -i "leak"

# Full suite (should have 0 failures)
cargo test --lib
cargo test --test r2_motion_kit_audit
```

---

## Debugging Guide

### Animation Not Starting
```bash
1. Check Metrics.motion:
   if app.theme.metrics.motion == 0.0 {
       // Animation disabled, will jump to target
   }

2. Check animation key:
   println!("Animation key: {:?}", key);
   // Must be consistent across frames

3. Check if element exists:
   // If element was removed, animation is cleaned up
```

### Animation Jittery or Slow
```bash
1. Check frame time:
   cargo run --release --example gallery 2>&1 | grep "frame_time"
   // Should be < 16ms (for 60fps)

2. Check live animation count:
   if memory.live_animation_count > 2 {
       // Budget exceeded, oldest animation disabled
   }

3. Check spring damping:
   SpringConfig { bounce: 1.0, damping: 0.8 }
   // Increase damping if oscillating
```

### Animation Cleanup Not Working
```bash
1. Check element is removed from tree:
   // verify view() no longer returns element with key

2. Check key is consistent:
   text("Item").key(item.id)
   // Must be same .key() across frames until removed

3. Check Memory::begin_frame() is called:
   // Called automatically by app loop
```

---

## Acceptance Criteria Checklist (STEP 19 Extended)

- [ ] Pre-implementation checklist completed
- [ ] STEP 20 Phase 1 all 3 gaps implemented
- [ ] STEP 21 Phase 2 both gaps implemented
- [ ] STEP 22 Phase 3 both gaps implemented
- [ ] All 12 acceptance tests passing
- [ ] All 396 library tests passing (no regressions)
- [ ] Migration checklist completed for existing code
- [ ] Performance validated (< 1.5ms frame time with R2)
- [ ] Accessibility validated (Metrics.motion respected)
- [ ] Testing checklist passed for all 3 phases
- [ ] Code review completed
- [ ] Documentation updated in CLAUDE.md

**Status**: Ready for implementation

---

## Next Steps (STEP 23+)

### STEP 23: R2 Documentation
- [ ] Update CLAUDE.md "Library Roadmap" (mark R2 as landed)
- [ ] Add "R2 Motion Kit Patterns" to "Widget Exemplars"
- [ ] Document animation best practices
- [ ] Add performance guidelines

### STEP 24+: Post-R2
- [ ] Optional R8 (Accessibility announcements)
- [ ] Optional R11 (Undo/redo animation state)
- [ ] Community feedback and refinement

