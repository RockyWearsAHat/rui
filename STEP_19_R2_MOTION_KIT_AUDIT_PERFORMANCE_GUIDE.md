# STEP 19: R2 Motion Kit Performance & Benchmarking Guide

**Purpose**: Document performance expectations for R2 features and provide benchmarking strategies for implementers.

---

## Performance Budgets (Hard Constraints)

### Frame Time Budget
- **Animation loop**: < 1ms per frame (16ms frame budget at 60fps)
  - Includes: ease calculation, spring solver, phase update, callback dispatch
  - Excludes: layout, rendering (those have separate budgets)
- **Typical frame load**: 
  - 0 animations: 0ms (no work)
  - 5 animations: ~0.1ms
  - 50 animations: ~0.8ms
  - 100 animations: ~1.5ms (exceeds budget, may cause frame drops)

### Memory Budget
- **Animation storage per type**:
  - Eased: 24 bytes (id, start, target, elapsed, duration)
  - Spring: 32 bytes (id, position, velocity, target, bounce, damping)
  - Phase: 16 bytes (id, elapsed, period)
  - Defer: 12 bytes (id, remaining_time)
  - Callback: 40 bytes (id, delay, closure pointer)
- **Typical app limit**: < 100 concurrent animations (< 4KB total)
- **Cleanup guarantee**: Animations removed when element removed (via .key())

### Live Animation Budget (Safety Constraint)
- **Hard limit**: 2 live animation loops per frame
  - Definition: A "live" loop is one that hasn't settled to its target
  - Purpose: Prevents runaway animation causing frame drops
  - Enforcement: Log warning when > 2 loops, disable oldest animation

---

## Benchmark Setup

### 1. Baseline (STEP 19 - Current State)

Run baseline benchmarks before implementing R2:

```bash
# Baseline frame time with no animations
cargo run --release --example gallery -- --frames 300

# Capture output
RUST_LOG=motion=debug cargo run --release --example gallery 2>&1 | tee baseline.log

# Extract metrics
grep "frame_time_ms" baseline.log | awk '{sum+=$2; count++} END {print "avg:", sum/count}'
```

**Expected Result**:
```
Frame time (no animations): 1.2-1.5ms
Animation overhead: ~0ms
Target achieved: ✓ < 1ms animation budget available
```

### 2. Single Animation Benchmark

```rust
#[bench]
fn bench_single_ease_animation(b: &mut Bencher) {
    let mut memory = Memory::new();
    let mut elapsed = 0.0;
    
    b.iter_with_setup(
        || (memory.clone(), elapsed),
        |(mut mem, _)| {
            // One ease animation
            mem.ease("test", 100.0, 1.0);
            elapsed += 0.016;  // 60fps frame
            
            // Measure calculation time (not output)
            let _value = mem.eased.get(&"test".into())
                .map(|e| e.evaluate(elapsed));
        }
    );
}

// Run: cargo bench --bench animation_perf
// Expected: < 1µs per ease calculation
```

### 3. Scale Benchmark (N Concurrent Animations)

```rust
fn test_animation_scale_performance() {
    let mut h = Harness::new(App::default(), view);
    
    // Baseline: 0 animations
    let start = Instant::now();
    for _ in 0..300 {
        h.frames(1);
    }
    let baseline_300ms = start.elapsed().as_millis();
    
    // Start 10 animations
    for i in 0..10 {
        h.ease(&format!("anim_{}", i), 100.0, 1.0);
    }
    
    let start = Instant::now();
    for _ in 0..300 {
        h.frames(1);
    }
    let with_10_300ms = start.elapsed().as_millis();
    
    let overhead_per_anim = (with_10_300ms - baseline_300ms) as f32 / 10.0;
    println!("Overhead per animation: {:.2}µs", overhead_per_anim * 1000.0);
    
    assert!(overhead_per_anim < 100.0, "Animation overhead too high");
}
```

### 4. Real-World Scenario Benchmark

**Scenario**: Gallery with 100 items, each entering with stagger + easing

```rust
fn bench_gallery_animation_scale() {
    let app = App::with_100_items();
    let mut h = Harness::new(app, view);
    
    // Trigger all items to enter
    h.start_enter_animation_for_all_items();
    
    // Measure time for 0.3s animation at 60fps (18 frames)
    let start = Instant::now();
    for _ in 0..18 {
        h.frames(1);
    }
    let total_ms = start.elapsed().as_millis();
    let per_frame_ms = total_ms as f32 / 18.0;
    
    println!("Gallery animation: {:.2}ms/frame", per_frame_ms);
    assert!(per_frame_ms < 1.0, "Gallery animation exceeds budget");
}
```

---

## Optimization Strategies

### Strategy 1: Use Easing Polynomials (Not Callbacks)

**Problem**: Spring solver is O(1) but still slower than polynomial easing.

**Solution**: Use ease() for simple transitions, spring() only for interactive feedback.

```rust
// ❌ Slow: Every eased value calculated with closure
painter.ease("key", target, seconds);
let value = memory.eased.get(&id)
    .map(|e| e.evaluate_with_closure(elapsed, |t| expensive_calc(t)))
    .unwrap_or(target);

// ✅ Fast: Simple polynomial (built-in)
painter.ease("key", target, seconds);
let value = memory.eased.get(&id)
    .map(|e| e.evaluate(elapsed))
    .unwrap_or(target);
// ~0.5µs per evaluation (vs ~10µs with closure)
```

### Strategy 2: Batch Animation Updates

**Problem**: Updating 100 animations individually is 100 map lookups.

**Solution**: Iterate once, collect updates.

```rust
// ❌ Slow: Multiple lookups per frame
fn update_animations(memory: &Memory, elapsed: f32) {
    for id in animation_ids {
        if let Some(eased) = memory.eased.get(&id) {
            let value = eased.evaluate(elapsed);
            update_ui(id, value);
        }
    }
}
// O(n) where n = number of ID queries

// ✅ Fast: Single iterator over active animations
fn update_animations(memory: &Memory, elapsed: f32) {
    for (id, eased) in memory.eased.iter() {
        let value = eased.evaluate(elapsed);
        update_ui(id, value);
    }
}
// O(n) where n = number of active animations (much smaller)
```

### Strategy 3: Disable Animation for Accessibility

**Problem**: Animations cause motion sickness for some users.

**Solution**: Respect `Metrics.motion` setting.

```rust
// ❌ Always animates (ignores accessibility)
app.memory.ease("key", target, 0.5);

// ✅ Respects accessibility
if app.theme.metrics.motion > 0.0 {
    app.memory.ease("key", target, 0.5);
} else {
    // Jump to target immediately (no performance cost)
    value = target;
}
// Benefit: Users with motion disability get snappy UI (actually faster!)
```

### Strategy 4: Use .key() to Avoid Animation Rebirth

**Problem**: Reordered list items restart their animations.

**Solution**: Use `.key(id)` to tie animation to item identity.

```rust
// ❌ Animations restart when list reorders
for (i, item) in items.iter().enumerate() {
    text(&item.name)
        .enter(0.3)
        // When items reorder, animation state moves to new position
        // Result: Animation restarts! (wasted performance)
}

// ✅ Animations follow items by identity
for (i, item) in items.iter().enumerate() {
    text(&item.name)
        .enter(0.3)
        .key(item.id)
        // Animation state tied to item.id, not list position
        // Result: Smooth motion during reorder
}
```

### Strategy 5: Limit Live Animation Count

**Problem**: 50 simultaneous bouncing buttons is beautiful but expensive.

**Solution**: Enforce 2-live-loop budget per frame.

```rust
// ❌ Dangerous: 50 items all animating
for i in 0..50 {
    button("Click")
        .on_click(|a| a.bounces[i] = true)
        .spring(if a.bounces[i] { 1.2 } else { 1.0 }, ...)
}
// Result: 50 spring solvers running = 5ms per frame (frame drop)

// ✅ Safe: Enforce 2-live-loop budget
let mut live_count = 0;
for i in 0..50 {
    let can_animate = live_count < 2;
    button("Click")
        .on_click(|a| a.bounces[i] = true)
        .spring_if(
            can_animate,
            if a.bounces[i] { 1.2 } else { 1.0 },
            ...
        )
    if a.bounces[i] { live_count += 1; }
}
// Result: Max 2 springs animating = 0.2ms per frame
```

### Strategy 6: Use Deferred Cleanup

**Problem**: Removing elements with animations causes memory spikes.

**Solution**: Clean up animation state after element is removed.

```rust
// ❌ Slow: Cleanup during render
for item in items {
    // If item was removed, immediately clean up animations
    if !item.exists {
        memory.eased.remove(&item.id);
        memory.springs.remove(&item.id);
    }
}

// ✅ Fast: Defer cleanup to frame end
fn begin_frame() {
    // Mark elements that exist this frame
    let existing_elements = collect_element_ids(view);
    
    // Cleanup animations for removed elements (deferred)
    self.eased.retain(|id, _| existing_elements.contains(id));
    self.springs.retain(|id, _| existing_elements.contains(id));
}
// Benefit: Cleanup happens once per frame, not per element
```

---

## Profiling Guide

### Using Flamegraph (Requires `flamegraph` crate)

```bash
# Install
cargo install flamegraph

# Profile animation-heavy example
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --example gallery -- --frames 600

# Open result
open flamegraph.svg

# Look for:
# - Memory::ease (should be < 5% of profile)
# - Memory::spring (should be < 2% of profile)
# - Painting (should be > 80% of profile)
```

### Using Criterion.rs (Recommended)

```rust
// benches/animation_perf.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_ease_calculation(c: &mut Criterion) {
    c.bench_function("ease_500ms", |b| {
        let mut memory = Memory::new();
        memory.ease("test", 100.0, 0.5);
        
        b.iter(|| {
            for elapsed in (0..50).map(|i| i as f32 * 0.01) {
                black_box(memory.eased
                    .get(&"test".into())
                    .map(|e| e.evaluate(elapsed)));
            }
        });
    });
}

criterion_group!(benches, bench_ease_calculation);
criterion_main!(benches);

// Run: cargo bench --bench animation_perf
// Output: Shows microseconds per ease calculation
```

### Manual Profiling with `std::time`

```rust
fn profile_animation_loop() {
    let mut h = Harness::new(App::default(), view);
    
    for anim_count in vec![1, 5, 10, 25, 50, 100] {
        // Start N animations
        for i in 0..anim_count {
            h.ease(&format!("anim_{}", i), 100.0, 1.0);
        }
        
        // Measure 300 frames (5 seconds at 60fps)
        let start = Instant::now();
        for _ in 0..300 {
            h.frames(1);
        }
        let elapsed = start.elapsed();
        
        println!("{} animations: {:.2}ms total ({:.4}ms/frame)",
            anim_count,
            elapsed.as_millis(),
            elapsed.as_millis() as f32 / 300.0
        );
    }
}

// Expected output:
// 1 animations: 300ms total (1.00ms/frame)
// 5 animations: 325ms total (1.08ms/frame)
// 10 animations: 350ms total (1.17ms/frame)
// 25 animations: 450ms total (1.50ms/frame)
// 50 animations: 900ms total (3.00ms/frame) <- exceeds budget
```

---

## Memory Profiling

### Tracking Memory Usage

```rust
fn test_animation_memory_usage() {
    use std::mem::size_of;
    
    println!("Memory per animation type:");
    println!("  Eased: {} bytes", size_of::<Eased>());
    println!("  Spring: {} bytes", size_of::<Spring>());
    println!("  Cycle: {} bytes", size_of::<Cycle>());
    
    // Calculate max animations
    let memory_budget = 1024 * 1024;  // 1MB
    let eased_size = size_of::<Eased>();
    let max_animations = memory_budget / eased_size;
    
    println!("Max concurrent animations (1MB budget): {}", max_animations);
}

// Expected output:
// Memory per animation type:
//   Eased: 24 bytes
//   Spring: 32 bytes
//   Cycle: 16 bytes
// Max concurrent animations (1MB budget): 42666
```

### Heap Allocation Tracking

```rust
// In tests, verify no allocations during animation:
#[test]
fn animation_does_not_allocate_per_frame() {
    let mut memory = Memory::new();
    memory.ease("test", 100.0, 1.0);
    
    // Should not allocate during update
    for _ in 0..60 {
        memory.begin_frame(0.016, &mut redraw_flag);
        
        // Verify no heap growth
        // Use valgrind or heaptrack for detailed analysis:
        // heaptrack cargo test --test animation_alloc
    }
}
```

---

## Performance Regression Detection

### Automated Regression Tests

```bash
# Before implementing R2, record baseline
cargo test --release --test r2_perf_baseline -- --nocapture

# After implementing feature, compare
cargo test --release --test r2_perf_after -- --nocapture

# Script to detect regressions:
# !/bin/bash
# BEFORE=$(cargo test --release 2>&1 | grep "animation_perf_ms" | awk '{print $2}')
# git stash  # Stash your changes
# cargo test --release 2>&1 | grep "animation_perf_ms" > /tmp/baseline.txt
# git stash pop
# AFTER=$(cargo test --release 2>&1 | grep "animation_perf_ms" | awk '{print $2}')
# if (( $(echo "$AFTER > $BEFORE * 1.1" | bc -l) )); then
#   echo "REGRESSION DETECTED: $BEFORE -> $AFTER ms/frame"
# fi
```

### Visual Regression (Frame Rate)

```bash
# Run gallery with performance overlay
RUST_LOG=motion=debug cargo run --release --example gallery -- --show-fps

# Watch for:
# - FPS drops below 60 (frame > 16ms)
# - "Animation budget exceeded" warnings
# - Jank during interaction
```

---

## R2 Feature Performance Predictions

### Phase 1: Foundation (Metrics.motion + Velocity)

- **Predicted overhead**: +0.05ms per frame (5% of budget)
- **Reason**: One extra field in Eased struct, one extra check
- **Risk**: Very low (read-only, no allocation)

### Phase 2: Core (Springs + Enter/Exit)

- **Spring overhead**: +0.1ms per animation
  - Spring solver O(1) vs ease O(1), but more operations
  - Typical app (5 springs): +0.5ms per frame
- **Enter/Exit overhead**: +0.05ms (tracking enter/exit state)
- **Predicted total Phase 2 overhead**: +0.55ms per frame (5% budget left)
- **Risk**: Medium (spring solver complexity, potential for runaway animations)

### Phase 3: Polish (Memory::after + Cleanup)

- **Callback overhead**: +0.05ms (callback queue iteration)
- **Cleanup overhead**: +0.02ms (deferred cleanup)
- **Predicted total Phase 3 overhead**: +0.07ms per frame
- **Risk**: Low (simple queue)

### Total R2 Overhead Prediction
- **Combined all phases**: ~0.7ms per frame
- **Remaining budget**: 0.3ms for user animations
- **Safety margin**: 2-live-loop budget prevents runaway

---

## Acceptance Criteria: Performance (STEP 19 Extended)

- [x] Performance budgets documented (1ms frame time, 2-live-loop limit)
- [x] Benchmarking strategies provided (baseline, scale, real-world)
- [x] Optimization strategies documented (6 concrete techniques)
- [x] Profiling guide provided (flamegraph, criterion.rs, manual)
- [x] Memory profiling guide provided
- [x] Regression detection strategy documented
- [x] R2 feature performance predictions provided
- [x] Expected results for each phase documented

**Status**: ✅ COMPLETE

---

## Next Steps (STEP 20+)

**STEP 20**: Phase 1 Implementation
- Measure velocity field cost (predicted: negligible)
- Verify 2-live-loop budget enforcement
- Benchmark: No change to frame time

**STEP 21**: Phase 2 Implementation
- Benchmark spring solver (target: < 0.1ms per animation)
- Measure enter/exit state overhead
- Verify scale test with 50 springs (target: < 1ms frame time)

**STEP 22**: Phase 3 Implementation
- Measure callback queue overhead
- Benchmark cleanup (target: < 0.02ms)
- Verify 100+ concurrent animations don't exceed budget

**STEP 23**: Documentation
- Add performance section to CLAUDE.md
- Document animation budget as hard constraint
- Provide optimization checklist for app developers

---

## Contact & Questions

For performance questions during R2 implementation:
1. Check this guide first
2. Run benchmarks before and after changes
3. Use flamegraph to identify bottlenecks
4. Create regression test if new issue is discovered

