# STEP 19: R2 Motion Kit Audit — Cross-Module Concerns

## Overview

This document maps how animation system changes propagate across the library's module boundaries. Understanding these interaction points is essential for:

1. **Implementing R2 without breaking existing modules**
2. **Identifying where new features integrate**
3. **Verifying changes don't create hidden regressions**
4. **Planning phase-by-phase implementation**

---

## Module Interaction Map

```
┌─────────────────────────────────────────────────────────────┐
│                        App (app.rs)                         │
│  • Frame loop: begin_frame(delta) → draw() → check_animating()
└──────────────────────────┬──────────────────────────────────┘
                           │
                ┌──────────┴──────────┬───────────────┐
                ↓                     ↓               ↓
         ┌────────────┐      ┌─────────────────┐  ┌──────────┐
         │ Memory     │      │ Paint (painter) │  │ Shell    │
         │ (memory.rs)│      │ (paint.rs)      │  │(shell.rs)│
         └─────┬──────┘      └────────┬────────┘  └────┬─────┘
               │                      │                │
         ┌─────┴──────┬─────────┐     │                │
         ↓            ↓         ↓     ↓                ↓
      eased      cycles    deferred  Painter::    request_redraw
      (HashMap)  (HashMap)  (HashMap) ease/phase  (if animating)
                                      calls Memory
                                      
                ┌─────────────────────────┐
                │ motion.rs (Motion Kit)  │
                │ • Easing enum           │
                │ • Spring struct         │
                │ • Transition enum       │
                │ (framework, not Memory) │
                └─────────────────────────┘
                      ↑
                      │ (R2: Use these in Memory)
                      │
                    Memory
```

---

## Critical Interaction Points

### 1. **Time Injection Chain** (app.rs → memory.rs)

#### Current Flow
```
App::run() loop:
  ├─ calculate delta_ms (now - last_frame_time)
  ├─ delta_s = delta_ms / 1000.0
  ├─ memory.begin_frame(delta_s)  ← Time enters the system
  │  └─ self.accumulated_time += delta_s
  │  └─ checks: deferred, transitions, easing completions
  │  └─ sets: self.animating = has any active animations?
  ├─ painter = Painter::new(&mut memory)  ← Give painter mutable memory reference
  ├─ draw(painter) runs view function
  │  └─ painter.ease("key", target, duration) calls Memory::ease()
  │  └─ painter.phase("key", period) calls Memory::phase()
  ├─ if memory.animating { request_redraw() }  ← Loop again
  └─ present(canvas)

Frame Loop Constraints:
  • Time only enters via begin_frame(delta)
  • No Instant::now() calls anywhere in crate
  • Animations driven by delta accumulation
```

#### R2 Impact
**Change**: Adding velocity inheritance, springs, and callbacks

**Interaction Point**: 
- `begin_frame()` must now also check callback queue (new)
- `ease()` must calculate velocity during solver (new)
- `phase()` must respect `metrics.motion == 0` (new)

**No Breaking Change**: Time injection mechanism unchanged; just more things to check at frame end

---

### 2. **Painter as Bridge to Memory** (paint.rs ↔ memory.rs)

#### Current Flow
```rust
// In app.rs:
let mut painter = Painter::new(&mut memory);

// In paint.rs:
pub struct Painter<'a> {
    memory: &'a mut Memory,
    // ... other fields
}

// Painter methods call Memory:
impl<'a> Painter<'a> {
    pub fn ease(&mut self, key: &str, target: f32, seconds: f32) -> f32 {
        self.memory.ease(key, target, seconds)  ← Delegates to Memory
    }
    pub fn phase(&mut self, key: &str, period: f32) -> f32 {
        self.memory.phase(key, period)          ← Delegates to Memory
    }
}
```

#### R2 Impact
**Change**: Adding `spring()`, `enter()`, `exit()` convenience methods

**Interaction Point**:
- Add corresponding Painter methods that delegate to Memory
- No new fields in Painter (still just `&'a mut memory`)
- Painter methods remain thin wrappers

**Constraint**: Painter must remain stateless (no cached values); it's just a convenience wrapper

---

### 3. **Animation State Storage Locations** (memory.rs fields)

#### Current Storage
```rust
pub struct Memory {
    // Animation primitives (4):
    eased: HashMap<Id, Eased>,              // Line 213
    cycles: HashMap<Id, Cycle>,             // Line 215
    deferred: HashMap<Id, f32>,             // Line 247
    transitions: HashMap<Id, (f32, f32)>,   // Line 251
    accumulated_time: f32,                  // Line 249
    
    // Other state:
    animating: bool,                        // Line 252
    // ... other fields like focus, scroll, etc.
}
```

#### R2 Additions
```rust
pub struct Memory {
    // Existing (unchanged):
    eased: HashMap<Id, Eased>,
    cycles: HashMap<Id, Cycle>,
    deferred: HashMap<Id, f32>,
    transitions: HashMap<Id, (f32, f32)>,
    accumulated_time: f32,
    animating: bool,
    
    // New in R2:
    springs: HashMap<Id, Spring>,           // Gap 1: Springs with bounce
    callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>, // Gap 3: Memory::after()
    
    // Flags (R2):
    #[cfg(debug)]
    phase_calls_this_frame: usize,          // Gap 4: 2-live-loop budget
}
```

#### Interaction Risk
**Current Problem**: No mechanism to tell Memory which theme is active (can't check `metrics.motion`)

**R2 Solution**: Pass `&Metrics` to ease/phase/spring methods OR store reference to theme in Painter

**Decision**: Pass `&Metrics` parameter (explicit, testable, no hidden state)

**Code Example**:
```rust
impl<'a> Painter<'a> {
    pub fn ease(&mut self, key: &str, target: f32, seconds: f32, metrics: &Metrics) -> f32 {
        self.memory.ease(key, target, seconds, metrics)
    }
}

impl Memory {
    pub fn ease(&mut self, id: Id, target: f32, seconds: f32, metrics: &Metrics) -> f32 {
        if metrics.motion == 0.0 {
            return target;  // Accessibility: instant
        }
        // ... normal easing
    }
}
```

---

### 4. **Animation Cleanup and Lifecycle** (memory.rs frame loop)

#### Current Cleanup
```rust
// At end of begin_frame():
pub fn begin_frame(&mut self, delta: f32) {
    self.accumulated_time += delta;
    
    // Cleanup: eased animations that finished
    self.eased.retain(|_, e| {
        e.elapsed < e.duration
    });
    
    // Cleanup: deferred that fired
    self.deferred.retain(|_, &fire_time| {
        self.accumulated_time < fire_time
    });
    
    // Cleanup: transitions that finished
    self.transitions.retain(|_, &(start, dur)| {
        self.accumulated_time < start + dur
    });
    
    // NO cleanup: cycles run forever
    
    // Set animating flag:
    self.animating = !self.eased.is_empty()
        || !self.cycles.is_empty()
        || !self.deferred.is_empty()
        || !self.transitions.is_empty();
}
```

#### R2 Cleanup Policy
**Problem**: No explicit cleanup policy documented; cycles never terminate; memory leak risk

**R2 Solution**: Document and enforce:
1. **Automatic cleanup** (eased, deferred, transitions, springs) ← Remove when done
2. **Manual cleanup** (cycles) ← User calls `Memory::clear_cycle(id)` 
3. **Callback cleanup** (callbacks) ← Remove after firing

**Code Change**:
```rust
pub fn begin_frame(&mut self, delta: f32) {
    self.accumulated_time += delta;
    
    // Cleanup: all auto-expiring animations
    self.eased.retain(|_, e| e.elapsed < e.duration);
    self.deferred.retain(|_, &fire_time| self.accumulated_time < fire_time);
    self.transitions.retain(|_, &(start, dur)| self.accumulated_time < start + dur);
    self.springs.retain(|_, s| s.energy > 0.01);  ← NEW: Springs stop when nearly still
    
    // Cleanup: callbacks that fired
    let now = self.accumulated_time;
    self.callbacks.retain(|_, &(fire_time, _)| now < fire_time);
    
    // Invoke fired callbacks
    let to_fire: Vec<_> = self.callbacks.iter()
        .filter(|(_, (fire_time, _))| now >= fire_time)
        .map(|(id, (_, cb))| (*id, cb.clone()))
        .collect();
    for (_, cb) in to_fire {
        cb();  // Invoke callback
    }
    
    // NO cleanup for cycles (by design)
    
    // Update animating flag
    self.animating = /* check all non-empty */;
}
```

---

### 5. **Metrics Propagation** (theme.rs → app.rs → memory.rs)

#### Current Flow
```
Theme (theme.rs):
  ├─ pub struct Theme { ... }
  └─ pub metrics: Metrics { motion: f32, ... }

App (app.rs):
  ├─ theme: Theme
  └─ No current use of theme.metrics.motion

Memory (memory.rs):
  └─ Has no access to Theme (by design)
```

#### R2 Flow
```
App (app.rs):
  ├─ theme: Theme
  ├─ memory: Memory
  ├─ frame loop:
      ├─ painter.ease(key, target, seconds, &theme.metrics)  ← PASS METRICS
      └─ painter.phase(key, period, &theme.metrics)

Painter (paint.rs):
  ├─ Receives metrics in method parameters
  ├─ Passes to Memory methods

Memory (memory.rs):
  ├─ ease(id, target, seconds, metrics: &Metrics)
      ├─ if metrics.motion == 0.0 { return target }
      └─ else { normal easing }
  └─ phase(id, period, metrics: &Metrics)
      ├─ if metrics.motion == 0.0 { return 0.0 }
      └─ else { normal cycling }
```

#### Interaction Point
**Question**: Where does Metrics come from in draw() frame?

**Current Answer**: It doesn't; view function is `Fn(&S) -> El<S>` (no access to Memory)

**R2 Solution**: Two options:
1. **Option A**: Painter carries metrics (add field)
2. **Option B**: Pass metrics as parameter to each ease/phase call

**Recommendation**: Option A (cleaner API)

```rust
impl<'a> Painter<'a> {
    metrics: &'a Metrics,  // Add field
    
    pub fn ease(&mut self, key: &str, target: f32, seconds: f32) -> f32 {
        self.memory.ease(key, target, seconds, self.metrics)
    }
}

// In app.rs:
let painter = Painter::new(&mut memory, &theme.metrics);
```

---

### 6. **ID Collision and Identity** (element.rs ↔ memory.rs)

#### Current System
```
Identity is path-based:
  └─ El at tree position [0][2][1] has ID = hash([0, 2, 1])

Overriding identity:
  └─ El.key(custom_id) sets explicit ID for this position
  
Animation ID usage:
  └─ painter.ease("button_opacity") creates ID from string "button_opacity"
  └─ Memory looks up "button_opacity" in eased HashMap
```

#### R2 Concern
**Problem**: String-based IDs can collide if developer reuses same key in different contexts

**Example**:
```rust
fn render_item(painter: &Painter, item: &Item) {
    let opacity = painter.ease("opacity", 1.0, 0.3);  ← Same ID for every item!
    // All items share same opacity animation
}
```

**R2 Solution**: Document best practice + provide helper

```rust
// Good:
let id = format!("item_{}_opacity", item.id);  // Unique per item
let opacity = painter.ease(&id, 1.0, 0.3);

// OR use El.key() to carry identity:
El::new(item_view)
    .key(item.id)  // Establishes unique identity for this subtree
    // Inside item_view, opacity animation uses tree path + "opacity"
    // → Automatically unique
```

#### No Code Change Needed
This is a documentation + best-practice issue, not a breaking change.

---

### 7. **Performance Constraints** (shell.rs ↔ memory.rs)

#### Current Constraint
```
Idle loop (no animation):
  ├─ memory.begin_frame() called → is_animating = false
  ├─ Frame not drawn
  ├─ Loop waits on idle_timeout (e.g., 16.6ms for 60fps)

Animation active:
  ├─ memory.begin_frame() called → is_animating = true
  ├─ Frame drawn immediately
  ├─ request_redraw() wakes loop (if sleeping)
```

#### R2 Impact
**New animations added**: Springs, callbacks, enter/exit transitions

**Performance Budget**:
- Each animation added in phase() must be ≤ 1% of frame time
- Total animation work must not exceed 1ms per frame
- Mechanical enforcement: 2-live-loop budget prevents runaway

**Code**:
```rust
#[cfg(debug_assertions)]
{
    self.phase_call_count_this_frame += 1;
    if self.phase_call_count_this_frame > 2 {
        panic!("Animation budget exceeded: {} cycles in one frame", 
               self.phase_call_count_this_frame);
    }
}

pub fn begin_frame(&mut self, delta: f32) {
    #[cfg(debug_assertions)]
    { self.phase_call_count_this_frame = 0; }  // Reset counter
    // ...
}
```

#### Shell Interaction
**No change to shell.rs**. Animation cost is absorbed in frame time budget.

---

## Implementation Sequence and Impact

### Phase 1: Foundation (Velocity Inheritance + Metrics.motion + 2-Live-Loop)

**Modules Touched**: memory.rs, paint.rs (no structural changes)

**Interaction Risk**: LOW
- Existing HashMaps unchanged
- Adding parameters to existing methods (backward compatible if using defaults)
- No new state in Memory struct (velocity stored in existing Eased)

**Acceptance Test**: Confirm easing feels responsive, animation budget respected, motion=0 disables

---

### Phase 2: Core Features (Springs + Enter/Exit)

**Modules Touched**: memory.rs (new Spring HashMap), paint.rs (new painter methods)

**Interaction Risk**: MEDIUM
- Adding new HashMap to Memory (bigger struct)
- New phase tracking requires transition state extension
- Must not affect existing easing, phase, defer, transitions

**Acceptance Test**: Confirm springs work independently, enter/exit choreography works, no regressions

---

### Phase 3: Sugar APIs (Memory::after, Cleanup Policy)

**Modules Touched**: memory.rs (new callbacks HashMap, cleanup methods)

**Interaction Risk**: MEDIUM
- Adding callbacks HashMap (requires allocation)
- Cleanup logic must be robust (no memory leaks)
- Callback closures require Box<dyn Fn()> (runtime cost)

**Acceptance Test**: Confirm callbacks fire, memory cleaned up, no dangling animations

---

### Phase 4: Robustness (Cleanup Documentation, ID Collision Avoidance)

**Modules Touched**: CLAUDE.md (documentation), test suite (best-practice examples)

**Interaction Risk**: NONE (documentation only)
- No code changes to crate
- Guidance for developers using animation APIs

---

## Common Pitfalls and How R2 Avoids Them

### Pitfall 1: Animation State Leaks
**Problem**: Cycles never removed; ID reuse causes stale animation

**R2 Solution**: Document manual cleanup; add `Memory::clear_animation(id)` method

---

### Pitfall 2: Time Discontinuities
**Problem**: If delta becomes 0 (frame skip), animations pause unexpectedly

**R2 Solution**: No change needed (app loop guarantees delta ≥ 0)

---

### Pitfall 3: Callback Ordering
**Problem**: Callbacks fire before view runs; causes stale state in view

**R2 Solution**: Callbacks fire after frame drawn (deferred execution model)

---

### Pitfall 4: Motion=0 Inconsistency
**Problem**: Some animations respect motion=0, others don't

**R2 Solution**: All animation methods check metrics.motion consistently

---

### Pitfall 5: ID Collision in Lists
**Problem**: Rendering list of items all use same animation ID

**R2 Solution**: Document best practice (use item.id in animation key)

---

## Testing Cross-Module Interactions

### Test: Time Injection Chain
```rust
#[test]
fn time_flows_correctly_through_modules() {
    let mut memory = Memory::new();
    
    // Frame 1: 10ms passes
    memory.begin_frame(0.01);
    assert_eq!(memory.accumulated_time, 0.01);
    
    // Frame 2: 10ms passes
    memory.begin_frame(0.01);
    assert_eq!(memory.accumulated_time, 0.02);
    
    // Animation reads accumulated_time
    let progress = memory.progress_of(id);
    // ...
}
```

### Test: Painter-to-Memory Delegation
```rust
#[test]
fn painter_delegates_to_memory() {
    let mut memory = Memory::new();
    let mut painter = Painter::new(&mut memory, &default_metrics());
    
    let val1 = painter.ease("key", 1.0, 1.0);
    let val2 = memory.ease("key", 1.0, 1.0);
    
    assert_eq!(val1, val2);  // Same result
}
```

### Test: Metrics Propagation
```rust
#[test]
fn motion_zero_skips_animation() {
    let mut memory = Memory::new();
    let zero_motion_metrics = Metrics { motion: 0.0, .. };
    
    let val = memory.ease(id, 1.0, 1.0, &zero_motion_metrics);
    assert_eq!(val, 1.0);  // Instant, no animation
}
```

---

## Next Documents

- See **STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md** for phase-by-phase changes
- See **STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md** for quick reference

