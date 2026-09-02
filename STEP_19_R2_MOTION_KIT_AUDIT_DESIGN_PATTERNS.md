# STEP 19: R2 Motion Kit Design Patterns & Real-World Scenarios

**Purpose**: Document common animation scenarios and their implementation with current state and R2.

---

## Pattern Library: Current State vs R2

### Pattern 1: Dismiss on Click (Currently: Defer + Transition)

**Scenario**: Show a toast notification that auto-dismisses after 3 seconds.

#### Current Implementation (STEP 19)
```rust
fn toast_view(app: &App) -> El<App> {
    if app.show_toast {
        col((
            text("Saved!"),
            button("Dismiss", |a| a.show_toast = false),
        ))
        .on_click(|app| {
            // Use defer to auto-dismiss
            app.memory.defer(toast_id, 3.0);
        })
    } else {
        empty()
    }
}

// In update handler (called after frame):
// if memory.deferred_expired(toast_id) {
//     app.show_toast = false;
// }
```

**Limitations**:
- Defer only triggers once
- Must manually check expiration in update handler
- No cascading dismissals (multiple toasts)

#### R2 Implementation (STEP 20)
```rust
fn toast_view(app: &App) -> El<App> {
    if app.show_toast {
        col((
            text("Saved!"),
            button("Dismiss", |a| a.show_toast = false),
        ))
        .on_click(|_| {})  // Click still works
        .after(3.0, |app| app.show_toast = false)  // NEW: auto-dismiss
    } else {
        empty()
    }
}
```

**Improvements**:
- ✅ `Memory::after()` executes callback after delay
- ✅ No manual timer checking in update
- ✅ Callback is closure, can close over app state
- ✅ Supports cascading: `after(delay1, ...).after(delay2, ...)`

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_memory_after_dismisses_toast() {
    let mut h = Harness::new(App { show_toast: true }, view);
    h.frames(1);  // Toast appears
    assert!(h.state().show_toast);
    
    h.frames(180);  // 3 seconds at 60fps (180 frames)
    // After waiting 3s, toast should auto-dismiss
    assert!(!h.state().show_toast);
}
```

---

### Pattern 2: Draggable Slider (Currently: Transitions + Manual Handler)

**Scenario**: Drag to adjust a volume slider; animate back to nearest step when released.

#### Current Implementation (STEP 19)
```rust
struct App {
    volume: f32,       // 0.0..1.0
}

fn slider_view(app: &App) -> El<App> {
    let volume = app.volume;
    draw(
        Size::new(200.0, 20.0),
        move |painter, rect| {
            painter.fill(rect, Radius::Pill, Tone::Sunken);
            let filled = rect.split_left(rect.w * volume);
            painter.fill(filled, Radius::Pill, Tone::Accent);
        }
    )
    .on_drag(|app, drag| {
        // Direct update to volume
        app.volume = drag.fraction().x;
    })
}
```

**Limitations**:
- No animation on release (jumps to handler result)
- No inertia or momentum
- Doesn't animate to nearest step

#### R2 Implementation (STEP 21)
```rust
struct App {
    volume: f32,
    volume_animating: bool,
}

fn slider_view(app: &App) -> El<App> {
    let volume = app.volume;
    draw(
        Size::new(200.0, 20.0),
        move |painter, rect| {
            // Draw current animated value
            painter.fill(rect, Radius::Pill, Tone::Sunken);
            let filled = rect.split_left(rect.w * volume);
            painter.fill(filled, Radius::Pill, Tone::Accent);
        }
    )
    .on_drag(|app, drag| {
        // While dragging, update directly
        app.volume = drag.fraction().x.clamp(0.0, 1.0);
        app.volume_animating = false;
    })
    .on_drag_end(|app, drag| {
        // On release, animate to nearest 0.1 step
        let target = (drag.fraction().x * 10.0).round() / 10.0;
        app.volume_animating = true;
        
        // NEW: Spring with velocity inheritance
        app.memory.spring(
            volume_id,
            target,
            &SpringConfig { bounce: 0.4, damping: 0.9 },
            drag.velocity  // Smooth continuation
        );
    })
}
```

**Improvements**:
- ✅ Spring animation on release
- ✅ Velocity inheritance from drag
- ✅ Bounce/damping for organic feel
- ✅ Animates to nearest step

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_spring_animates_slider_to_nearest_step() {
    let mut h = Harness::new(App { volume: 0.0 }, view);
    
    // Simulate drag to 0.34
    h.drag_text("Slider", Point::new(68.0, 0.0));
    assert_eq!(h.state().volume, 0.34);
    
    // Release drag - spring animates to 0.30 (nearest 0.1)
    h.release_drag();
    h.frames(30);  // 0.5s animation
    
    // Should snap close to 0.30
    assert!((h.state().volume - 0.30).abs() < 0.01);
}
```

---

### Pattern 3: Loading State with Spinner (Currently: Phase)

**Scenario**: Show a rotating spinner while loading data. Hide after load completes.

#### Current Implementation (STEP 19)
```rust
struct App {
    loading: bool,
}

fn spinner_view(app: &App) -> El<App> {
    if app.loading {
        // Phase creates a looping 0→1→0 cycle
        draw(Size::new(24.0, 24.0), move |painter, rect| {
            let rotation = painter.phase("spinner", 1.0);  // 1-second cycle
            painter.fill_rotated(rect, rotation * 360.0, Radius::Round, Tone::Accent);
        })
    } else {
        empty()
    }
}
```

**Limitations**:
- No enter/exit animation (appears/disappears instantly)
- No way to fade spinner out when complete
- Spinning speed is hardcoded (1.0s)

#### R2 Implementation (STEP 21)
```rust
struct App {
    loading: bool,
    spinner_visible: bool,
}

fn spinner_view(app: &App) -> El<App> {
    if app.loading {
        // Spinner enters with fade-in
        draw(Size::new(24.0, 24.0), move |painter, rect| {
            let rotation = painter.phase("spinner", 1.0);
            let alpha = painter.ease("spinner_fade", 1.0, 0.3);  // Fade in over 0.3s
            
            painter.fill_rotated(
                rect,
                rotation * 360.0,
                Radius::Round,
                Tone::Accent.with_alpha(alpha)
            );
        })
        .enter(0.3)  // NEW: Fade in over 300ms
    } else if app.spinner_visible {
        // Spinner exits with fade-out
        draw(Size::new(24.0, 24.0), move |painter, rect| {
            let rotation = painter.phase("spinner", 1.0);
            let alpha = painter.ease("spinner_fade", 0.0, 0.3);  // Fade out
            
            painter.fill_rotated(
                rect,
                rotation * 360.0,
                Radius::Round,
                Tone::Accent.with_alpha(alpha)
            );
        })
        .exit(0.3)  // NEW: Fade out over 300ms
        .after(0.3, |app| app.spinner_visible = false)  // Hide after exit
    } else {
        empty()
    }
}

// In update handler:
// if data_loaded {
//     app.loading = false;
//     app.spinner_visible = true;  // Keep spinner while exiting
// }
```

**Improvements**:
- ✅ `.enter(duration)` for fade-in entrance
- ✅ `.exit(duration)` for fade-out exit
- ✅ Choreography: load complete → spinner fades out → removed
- ✅ Smooth visual transition

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_enter_exit_spinner_choreography() {
    let mut h = Harness::new(App { loading: true, spinner_visible: false }, view);
    
    h.frames(1);
    // Spinner appears with fade-in
    assert!(h.rendered_text("Loading...").is_some());
    
    h.frames(18);  // 0.3s at 60fps
    // Fade-in complete
    assert_eq!(h.painted_alpha("spinner"), 1.0);
    
    // Simulate load complete
    h.receive_event(LoadComplete);
    h.frames(1);
    assert!(h.state().loading == false);
    
    h.frames(18);  // 0.3s fade-out
    // Spinner now invisible
    assert!(h.rendered_text("Loading...").is_none());
}
```

---

### Pattern 4: Bounce Animation (Currently: Ease, Desired: Spring)

**Scenario**: Click a button and watch it bounce back with elastic motion.

#### Current Implementation (STEP 19)
```rust
struct App {
    scale: f32,
}

fn button_view(app: &App) -> El<App> {
    let scale = app.scale;
    draw(Size::new(100.0, 40.0), move |painter, rect| {
        let scaled_rect = rect.scaled_from_center(scale);
        painter.fill(scaled_rect, Radius::Round, Tone::Accent);
        painter.text("Click me", scaled_rect);
    })
    .on_click(|app| {
        // Ease can only animate to a fixed value (no bounce)
        app.scale = 0.95;
        app.memory.ease("button_scale", 1.0, 0.2);  // Back to 1.0 over 200ms
    })
}
```

**Limitations**:
- No bounce/overshoot
- Only linear easing
- Doesn't feel "alive"

#### R2 Implementation (STEP 21)
```rust
struct App {
    scale: f32,
}

fn button_view(app: &App) -> El<App> {
    let scale = app.scale;
    draw(Size::new(100.0, 40.0), move |painter, rect| {
        let scaled_rect = rect.scaled_from_center(scale);
        painter.fill(scaled_rect, Radius::Round, Tone::Accent);
        painter.text("Click me", scaled_rect);
    })
    .on_click(|app| {
        // NEW: Spring with bounce
        app.memory.spring(
            "button_scale".into(),
            1.0,
            &SpringConfig {
                bounce: 1.2,   // Overshoot 20%
                damping: 0.8,  // Natural damping
            },
            0.0  // No initial velocity
        );
    })
}
```

**Improvements**:
- ✅ Spring creates natural elastic motion
- ✅ `bounce` parameter controls overshoot (1.0 = no bounce, 1.2 = 20% overshoot)
- ✅ `damping` settles the spring smoothly
- ✅ Feels interactive and alive

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_spring_bounce_button_press() {
    let mut h = Harness::new(App { scale: 1.0 }, view);
    
    h.click_text("Click me");
    
    // Spring oscillates: 1.0 → 1.2 (overshoot) → 1.0 (settle)
    h.frames(3);   // ~50ms, approaching peak
    let max_scale = h.painted_scale("button");
    assert!(max_scale > 1.15);  // Peak overshoot
    
    h.frames(20);  // ~330ms total, spring settles
    assert!((h.state().scale - 1.0).abs() < 0.01);  // Back to rest
}
```

---

### Pattern 5: Cascading List Animation (Currently: Manual, Desired: Enter with Stagger)

**Scenario**: Show a list of items with each one fading in with a slight delay.

#### Current Implementation (STEP 19)
```rust
struct App {
    items: Vec<String>,
    visible_count: usize,
}

fn list_view(app: &App) -> El<App> {
    let visible_count = app.visible_count;
    col(
        app.items
            .iter()
            .enumerate()
            .take(visible_count)
            .map(|(i, item)| {
                text(item)
                    .on_first_render(|app| {
                        // Use defer to stagger each item's entry
                        let delay = i as f32 * 0.1;  // 100ms stagger
                        app.memory.defer(item_id(i), delay);
                    })
            })
            .collect::<Vec<_>>()
    )
}

// Must manually track which items have completed their defer
// and trigger visibility change in update handler
```

**Limitations**:
- Manual stagger logic (error-prone)
- No built-in choreography
- Requires manual tracking of defer state
- Can't easily change stagger delay

#### R2 Implementation (STEP 21)
```rust
struct App {
    items: Vec<String>,
}

fn list_view(app: &App) -> El<App> {
    col(
        app.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                text(item)
                    .enter(0.3)  // Fade in over 300ms
                    .stagger(i as f32 * 0.1)  // NEW: 100ms delay per item
                    .key(i)  // Keep identity during reordering
            })
            .collect::<Vec<_>>()
    )
}
```

**Improvements**:
- ✅ `.stagger()` automatically delays entry
- ✅ Built-in choreography for lists
- ✅ Easy to adjust stagger delay
- ✅ Works with reordered lists (via `.key()`)

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_enter_stagger_list_choreography() {
    let mut h = Harness::new(
        App { items: vec!["A".to_string(), "B".to_string(), "C".to_string()] },
        view
    );
    
    // Item A visible immediately
    h.frames(1);
    assert!(h.rendered_text("A").is_some());
    
    // Item B enters after 100ms (6 frames at 60fps)
    h.frames(5);
    assert!(h.rendered_text("B").is_none());  // Still invisible
    
    h.frames(1);  // At 100ms
    assert!(h.rendered_text("B").is_some());  // Now visible
    
    // Item C enters after 200ms (12 frames)
    h.frames(5);
    assert!(h.rendered_text("C").is_none());
    
    h.frames(1);  // At 200ms
    assert!(h.rendered_text("C").is_some());
}
```

---

### Pattern 6: Disabled State with Visual Feedback (Currently: Static, Desired: Transition)

**Scenario**: Button transitions to disabled state with dim animation.

#### Current Implementation (STEP 19)
```rust
struct App {
    can_submit: bool,
}

fn form_view(app: &App) -> El<App> {
    button("Submit", |a| { /* ... */ })
        .disabled(!app.can_submit)
        // Appears disabled instantly - no animation
}
```

**Limitations**:
- No visual transition to disabled
- Disabled state is binary (on/off)
- No feedback that something changed

#### R2 Implementation (STEP 21)
```rust
struct App {
    can_submit: bool,
}

fn form_view(app: &App) -> El<App> {
    button("Submit", |a| { /* ... */ })
        .disabled(!app.can_submit)
        // NEW: Transition to disabled with 200ms fade
        .when_disabled(|painter| {
            // Animate alpha to 0.38 (disabled state convention)
            painter.ease("button_disabled", if app.can_submit { 1.0 } else { 0.38 }, 0.2)
        })
}
```

**Improvements**:
- ✅ `.when_disabled()` hook for state transitions
- ✅ Smooth fade instead of instant jump
- ✅ Follows design system (0.38 alpha for disabled)
- ✅ Visual feedback that state changed

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_transition_disabled_with_alpha() {
    let mut h = Harness::new(App { can_submit: true }, view);
    
    h.frames(1);
    let initial_alpha = h.painted_alpha("Submit");
    assert_eq!(initial_alpha, 1.0);
    
    // Disable the button
    h.receive_event(FormError);
    h.frames(1);
    assert!(!h.state().can_submit);
    
    h.frames(11);  // 200ms fade at 60fps
    let final_alpha = h.painted_alpha("Submit");
    assert!((final_alpha - 0.38).abs() < 0.05);
}
```

---

### Pattern 7: Modal Dialog with Overlay (Currently: Basic, Desired: Enhanced)

**Scenario**: Show a confirmation dialog with backdrop fade and enter/exit choreography.

#### Current Implementation (STEP 19)
```rust
struct App {
    show_dialog: bool,
}

fn dialog_view(app: &App) -> El<App> {
    col((
        if app.show_dialog {
            // Backdrop (no animation)
            rect().fill(Color::BLACK.with_alpha(0.5))
        } else {
            empty()
        },
        if app.show_dialog {
            // Dialog (no animation)
            col((
                text("Confirm?"),
                button("Yes", |a| a.show_dialog = false),
                button("No", |a| a.show_dialog = false),
            ))
            .pad(16.0)
        } else {
            empty()
        }
    ))
}
```

**Limitations**:
- No enter/exit animations
- Backdrop appears instantly
- Dialog feels abrupt

#### R2 Implementation (STEP 21)
```rust
struct App {
    show_dialog: bool,
}

fn dialog_view(app: &App) -> El<App> {
    col((
        if app.show_dialog {
            // Backdrop fades in
            rect()
                .fill(Color::BLACK.with_alpha(0.5))
                .enter(0.2)  // Fade in backdrop
        } else {
            empty()
        },
        if app.show_dialog {
            // Dialog slides up and fades in
            col((
                text("Confirm?"),
                button("Yes", |a| a.show_dialog = false),
                button("No", |a| a.show_dialog = false),
            ))
            .pad(16.0)
            .enter(0.3)    // Fade in + slide up
            .exit(0.2)     // Fade out + slide down
            .transform_enter(Transform::slide_up(20.0))  // Slide from 20px below
        } else {
            empty()
        }
    ))
}
```

**Improvements**:
- ✅ Backdrop `.enter()` for smooth fade
- ✅ Dialog `.enter()` + `.transform_enter()` for choreography
- ✅ `.exit()` for clean dismissal
- ✅ Professional feeling with motion

**Test Example** (R2 acceptance test stub):
```rust
#[test]
#[ignore]
fn r2_acceptance_modal_enter_exit_choreography() {
    let mut h = Harness::new(App { show_dialog: false }, view);
    
    // Show dialog
    h.click_text("Confirm");
    h.frames(1);
    assert!(h.state().show_dialog);
    
    // Backdrop should be fading in
    h.frames(6);  // ~100ms of 200ms fade
    let backdrop_alpha = h.painted_alpha("backdrop");
    assert!(backdrop_alpha > 0.2 && backdrop_alpha < 0.5);
    
    h.frames(6);  // Complete fade-in
    assert_eq!(h.painted_alpha("backdrop"), 0.5);
    
    // Dialog should be fading in + sliding up
    let dialog_y = h.painted_position("dialog").y;
    assert!(dialog_y < 100.0);  // Moved up
}
```

---

## Migration Guide: Current → R2

### Step 1: Update Memory Calls
```rust
// OLD (STEP 19):
app.memory.ease("key", target, seconds);
app.memory.phase("key", period);
app.memory.defer(id, seconds);

// NEW (R2):
app.memory.ease_with("key", target, seconds, Easing::EaseInOutCubic);
app.memory.spring("key", target, &SpringConfig::default(), velocity);
app.memory.after(seconds, closure);
```

### Step 2: Add Enter/Exit Decorators
```rust
// OLD: Elements appear/disappear instantly
col(items).on_click(|a| a.show = false)

// NEW: Smooth transitions
col(items)
    .enter(0.3)
    .exit(0.2)
    .on_click(|a| a.show = false)
```

### Step 3: Replace Manual Defer with After
```rust
// OLD: Must check expiration in update handler
app.memory.defer(id, 3.0);
// In update: if memory.deferred_expired(id) { ... }

// NEW: Callback runs automatically
.after(3.0, |app| app.show_toast = false)
```

### Step 4: Migrate Motion Budget
```rust
// OLD: No budget check
let animating = app.memory.animating;

// NEW: Always check Metrics.motion
if app.theme.metrics.motion > 0.0 {
    // Animation enabled
    app.memory.spring(...);
}
```

---

## Common Mistakes & Fixes

### Mistake 1: Forgetting `.enter()` on List Items
```rust
// ❌ Items appear instantly in batches
col(items.iter().map(|i| text(i)))

// ✅ Items fade in with stagger
col(items.iter().enumerate().map(|(i, item)| {
    text(item)
        .enter(0.3)
        .stagger(i as f32 * 0.1)
        .key(i)
}))
```

### Mistake 2: Using Spring with Zero Damping
```rust
// ❌ Spring bounces forever
app.memory.spring("key", target, &SpringConfig {
    bounce: 1.0,
    damping: 0.0,  // Spring never settles!
}, 0.0);

// ✅ Set realistic damping (0.7-0.9)
app.memory.spring("key", target, &SpringConfig {
    bounce: 1.0,
    damping: 0.8,  // Settles in ~500ms
}, 0.0);
```

### Mistake 3: Chaining After Without Closure
```rust
// ❌ State can't change
.after(1.0, |_| {})

// ✅ Closure captures and mutates state
.after(1.0, |app| app.show_next = true)
```

### Mistake 4: Forgetting .key() on Reordered Lists
```rust
// ❌ Animation state moves with index when items reorder
for (i, item) in items.iter().enumerate() {
    text(&item.name).enter(0.3)
}

// ✅ Animation state follows item by key
for (i, item) in items.iter().enumerate() {
    text(&item.name)
        .enter(0.3)
        .key(item.id)  // State follows item
}
```

### Mistake 5: Not Checking Metrics.motion
```rust
// ❌ Animations run even if user disabled motion
app.memory.ease("key", target, 0.5);

// ✅ Respect accessibility setting
if app.theme.metrics.motion > 0.0 {
    app.memory.ease("key", target, 0.5);
} else {
    // Jump to target immediately
    app.some_value = target;
}
```

---

## Performance Considerations

### Animation Frame Budget
- Target < 1ms for animation calculations per frame
- Use spring solver (O(1) per animation, not O(n²))
- Avoid allocating in animation closures

### Memory Budget
- 2-live-loop limit prevents runaway cycles
- Callback queue bounded (FIFO cleanup)
- Keyed animations cleaned up when element removed

### Rendering Budget
- Animation updates don't trigger layout recalculation
- Only alpha/transform/scale update (no reflow)
- Batch animations together (< 10 per frame typical)

---

## Acceptance Criteria for Design Patterns (STEP 19 Extended)

- [x] 7 real-world animation patterns documented
- [x] Current implementation shown for each pattern
- [x] R2 implementation shown for each pattern
- [x] Test examples provided (as acceptance stubs)
- [x] Common mistakes identified with fixes
- [x] Migration guide provided
- [x] Performance considerations documented

**Status**: ✅ COMPLETE

---

## Next Steps

**STEP 20**: Implement Phase 1 gaps (Metrics.motion, velocity, 2-live-loop)
- Use these patterns as test inspiration
- Verify pattern 1 (dismiss) works with Memory::after()

**STEP 21**: Implement Phase 2 features (springs, enter/exit)
- All 7 patterns become testable
- Each pattern serves as acceptance test

**STEP 22**: Implement Phase 3 (cleanup policy)
- Verify all patterns work at scale (100+ animations)

