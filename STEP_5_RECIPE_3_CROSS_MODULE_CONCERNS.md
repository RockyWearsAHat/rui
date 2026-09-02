# Recipe 3: Checkbox Control — Cross-Module Concerns

## Module Interaction Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                      View Function (app.rs)                       │
│                      fn view(app: &App) -> El<App>               │
│                      view returns checkbox(...) call              │
└──────────┬───────────────────────────────────────────────────────┘
           │
           ├──────────────────────────────────────────────────┐
           │                                                  │
      ┌────▼──────────────────────────────────────────┐   ┌──▼──────────────────────┐
      │         widgets.rs: checkbox()               │   │   paint.rs: Painter    │
      │         Constructor takes state param        │   │   Renders draw closure  │
      │         Returns El<S> with on_click          │   │   Paints filled/empty   │
      │         Handler is Fn(&mut S)                │   │   Applies theme colors  │
      └────┬─────────────────────────────────────────┘   └──────────────────────┘
           │                                                       ▲
           │                                                       │
      ┌────▼──────────────────────────────────────────────────────┘
      │
      │  draw() closure captures:
      │  - checked: bool (state parameter)
      │  - move |painter| { ... }
      │  - Conditional styling based on checked
      │
      ├──────────────────────────────────────────────────┐
      │                                                  │
  ┌───▼────────────────────┐                   ┌────────▼──────────────┐
  │   element.rs: El       │                   │  theme.rs: Theme      │
  │   Contains handlers    │                   │  Tone roles resolve   │
  │   .on_click(handler)   │                   │  Accent, Sunken, etc. │
  │   .key() for identity  │                   │  Provides colors      │
  └───┬────────────────────┘                   └──────────────────────┘
      │
      ├──────────────────────────────────────────────────┐
      │                                                  │
  ┌───▼──────────────────────┐                   ┌──────▼──────────────────┐
  │  memory.rs: Memory       │                   │  input.rs: Event/Input  │
  │  Focus state             │                   │  Pointer click → handler │
  │  Interaction lifetime    │                   │  Modifiers              │
  │  Path-based identity     │                   │  Handler invocation     │
  └───┬──────────────────────┘                   └─────────────────────────┘
      │
  ┌───▼────────────────────────────────────────────────────────┐
  │          accessibility.rs: Accessibility Tree             │
  │          Checkbox is leaf node (takes_focus=true)          │
  │          Tab order determined by path traversal            │
  └────────────────────────────────────────────────────────────┘
```

---

## 1. Identity & Persistence (element.rs ↔ memory.rs)

**Problem**: How does checkbox state survive frame rebuilds?

**Solution**: Identity is path-based; Memory persists interaction state.

### Interaction Flow

1. **Frame 1**: `view()` returns checkbox at path `/col[0]/row[0]/draw`
2. **User clicks**: Handler is invoked, state changes
3. **Frame 2**: `view()` returns checkbox at SAME path `/col[0]/row[0]/draw`
4. **Memory**: Recognizes same path, state persists
5. **Render**: Checkbox draws with new state (filled instead of empty)

### How It Works

- **element.rs**: `El` tree is immutable and recreated every frame. Each element has a path (breadcrumb of position in tree)
- **memory.rs**: Stores interaction state keyed by element path
  - Focus: which path has keyboard focus
  - Scroll: scroll position for scrollable elements
  - Hover: which path is being hovered
  - Animation: easing/phase values for animated elements
  - IME: composition state for text input

### Override with `.key()`

If checkbox list reorders:
```rust
// WITHOUT key: state stays at [0], follows position
for item in items { checkbox(&item.name, state, handler) }

// WITH key: state follows item
for item in items {
    checkbox(&item.name, state, handler).key(item.id)
}
```

### Common Pitfalls

**Pitfall 1**: State lost after reorder
- **Cause**: Path-based identity, element moved to different position
- **Fix**: Use `.key(item.id)` to override path-based identity
- **Lesson**: Never remove `.key()` from reordered or conditional elements

**Pitfall 2**: Multiple checkboxes share state
- **Cause**: Identical path (e.g., same if-branch renders same checkbox)
- **Fix**: Separate branches in conditional, use `.key()` for conditional rendering
- **Lesson**: Path must be unique per logical element

### Verification

```rust
// Test: State persists across frames
let mut h = Harness::new(App { checked: false }, view);
assert_eq!(h.state().checked, false);
h.click_at_point(Point::new(20.0, 20.0));  // Click checkbox
assert_eq!(h.state().checked, true);
h.frames(10);  // Rebuild 10 times
assert_eq!(h.state().checked, true, "State should persist");
```

---

## 2. State Flow (widgets.rs ↔ paint.rs)

**Problem**: How does state parameter reach the rendering code?

**Solution**: State flows as upvalue through closure into draw.

### Data Flow

```
checkbox(label, checked: bool, handler)
  ↓
  .draw(Size, move |painter, rect| {
    // ↑ 'checked' is captured here as upvalue
    let fill = if checked { Tone::Accent } else { Tone::Sunken };
    painter.fill(rect, Radius::Units(4.0), fill);
    if checked {
      painter.fill(tick(rect), Radius::Units(1.0), Tone::OnAccent);
    }
  })
  ↓
  paint.rs: Painter draws to canvas
```

### Key Insight

State does NOT flow through a retained widget tree. Instead:
1. View function is called every frame
2. State parameter is passed to checkbox()
3. Upvalue captured in draw closure
4. Closure is stored in El as FnMut
5. When frame is drawn, closure is called with Painter
6. Closure uses upvalue (checked) to decide appearance

### Why This Matters

**Benefit 1**: State changes are immediate
- No need to mutate internal state
- No watchers or property bindings
- Direct: state → appearance

**Benefit 2**: Testable
- State is immutable input to view
- Same state → same output (deterministic)
- Snapshot testing works: `h.render().save_png("checkbox.png")`

**Benefit 3**: No memory leaks
- Closure is dropped when El is dropped
- No Rc/RefCell cycle
- Garbage collector not needed

### Common Mistakes

**Mistake 1**: Trying to mutate state in closure
```rust
// WRONG: can't mutate through upvalue in closure
.draw(..., move |painter, rect| {
    checked = false;  // Error: can't mutate captured upvalue
})

// RIGHT: mutation happens in handler, not view
.on_click(move |state: &mut S| state.checked = !state.checked)
```

**Mistake 2**: Using Arc/Rc to share state
```rust
// WRONG: unnecessary, breaks pattern
let checked = Arc::new(Mutex::new(false));
checkbox("", Arc::clone(&checked), handler)

// RIGHT: just pass bool by value
checkbox("", app.checked, handler)
```

### Verification

```rust
// Test: State changes appearance
let mut h = Harness::new(App { checked: false }, view);
assert_eq!(h.render().count_pixels(Tone::Accent), 0);  // Empty
h.state_mut().checked = true;
h.frames(1);  // Rebuild
assert_eq!(h.render().count_pixels(Tone::Accent), 50);  // Filled (estimate)
```

---

## 3. Handlers (input.rs ↔ paint.rs)

**Problem**: How do click events become handler calls?

**Solution**: Click events are translated to handler invocations after frame is drawn.

### Event → Handler Flow

```
Platform (macOS/Windows/X11)
  ↓ (mouse click at (x, y))
  ↓
paint.rs: Single tree walk
  - Draw everything to canvas
  - Check hit-test for each on_click area
  - Record handler in deferred queue
  ↓
input.rs: Handler invocation
  - Event converted to Input
  - Input queues handler call
  ↓
app.rs: Frame loop
  - Handler is called: handler(&mut state)
  - State changed by handler
  - Next frame rebuilds view with new state
  ↓
View function called again
  - view(&state) returns new El tree
  - Checkbox constructor called with new checked value
  - Cycle repeats
```

### Key Invariants

**Invariant 1**: Single dispatch path
- All handlers run the same regardless of input source
- Mouse click runs same handler as keyboard Enter
- Accessibility activation runs same handler
- → One action, not two parallel code paths

**Invariant 2**: Handlers receive `&mut S` directly
```rust
// Handlers are plain functions, not closures over self
|app: &mut S| app.checked = !app.checked

// This works because:
// - State is the only thing being modified
// - No self reference, no borrow conflicts
// - Multiple handlers can run in one frame
```

**Invariant 3**: Deferred execution
```rust
// Handlers run AFTER frame is drawn
// Paint happens, click is recorded, then handler runs
// This means:
// - Click is accurate to the rendered frame
// - No race conditions
// - Next frame sees the changed state
```

### How Checkbox Wires Handlers

```rust
// In checkbox constructor:
.on_click(move |state: &mut S| toggle(state))

// What happens:
// 1. Renderer walks tree and draws checkbox
// 2. If click is inside checkbox bounds, record handler
// 3. After frame, run all recorded handlers
// 4. toggle(state) is called, state.checked flips
// 5. Next frame view() called with new state
// 6. Checkbox drawn with new appearance
```

### Common Pitfalls

**Pitfall 1**: Handler not called
- **Cause**: on_click handler not attached, or bounds are wrong
- **Fix**: Verify .on_click() is chained, verify draw() bounds are correct
- **Test**: Add println! in handler and rebuild

**Pitfall 2**: State changes don't take effect until next frame
- **Cause**: View function isn't called immediately
- **Fix**: This is by design; handlers are deferred
- **Note**: Next frame is fast (~8ms), so delay is imperceptible

**Pitfall 3**: Multiple handlers conflict
- **Cause**: Multiple on_click handlers on same element
- **Fix**: Only one on_click per element; combine logic if needed
- **Note**: Multiple elements CAN have handlers; they're independent

### Verification

```rust
// Test: Handler is called on click
let mut h = Harness::new(App { checked: false }, view);
h.click_text("Checkbox");
assert_eq!(h.state().checked, true, "Handler should toggle state");
```

---

## 4. Appearance (theme.rs ↔ widgets.rs)

**Problem**: How do colors adapt to light/dark mode?

**Solution**: Tone roles (Accent, Sunken, Border) resolve against Theme.

### Color Resolution

```
draw() closure references: Tone::Accent
  ↓
theme.rs: Palette
  - Light mode: Accent = #007AFF (blue)
  - Dark mode: Accent = #0A84FF (lighter blue)
  ↓
color.rs: Color::blend_over()
  - Blend into canvas at current location
  ↓
Canvas: Pixel written with theme color
```

### Theme Structure

```rust
pub struct Theme {
    palette: Palette,
    metrics: Metrics,
    corner_style: CornerStyle,
    type_scale: TypeScale,
}

pub struct Palette {
    // Semantic roles
    primary: Color,
    secondary: Color,
    success: Color,
    warning: Color,
    danger: Color,
    accent: Color,
    sunken: Color,
    border: Color,
}
```

### How Checkbox Uses Tones

```rust
let fill = if checked { Tone::Accent } else { Tone::Sunken };
painter.fill(rect, Radius::Units(4.0), fill);
```

**Tone::Accent** (for checked):
- Light: Vivid blue
- Dark: Bright blue
- Semantics: Primary action, positive state

**Tone::Sunken** (for unchecked):
- Light: Light grey
- Dark: Dark grey
- Semantics: Recessive, empty state

**Tone::Border**:
- Light: Medium grey
- Dark: Lighter grey
- Semantics: Structure, not content

### Light vs Dark Mode

```rust
// In paint loop:
let theme = match appearance() {
    Appearance::Light => Theme::light(),
    Appearance::Dark => Theme::dark(),
};

// Then painter uses theme to resolve Tones
painter.fill(rect, Radius, theme.palette.accent);
```

### Custom Palettes

Apps can swap palettes:
```rust
let theme = Theme::default()
    .with_palette(my_custom_palette);

app.run(&my_theme)?;
```

### Contrast Validation

All palettes must meet WCAG AA:
```rust
theme.palette.assert_legible()  // Panics if contrast < 4.5:1
```

### Common Pitfalls

**Pitfall 1**: Using raw RGB instead of Tone
```rust
// WRONG: color is hardcoded
painter.fill(rect, Radius, Color::new(0, 122, 255, 255));

// RIGHT: tone resolves against theme
painter.fill(rect, Radius, Tone::Accent);
```

**Pitfall 2**: Contrast fails in dark mode
- **Cause**: Palette colors too close
- **Fix**: Adjust Palette brightness difference
- **Verify**: Run `assert_legible()` on all palettes

**Pitfall 3**: Focus ring invisible on accent
- **Cause**: Focus ring is same color as filled checkbox
- **Fix**: Use different Tone for focus (e.g., Tone::OnAccent)
- **Note**: Focus and selection must be visually distinct

### Verification

```rust
// Test: Colors match theme
let mut h = Harness::new(App { checked: false }, view);
let unchecked = h.render();  // Render unchecked
// Count pixels with Sunken color (should be ~225 for 15x15 box)

h.state_mut().checked = true;
h.frames(1);
let checked = h.render();  // Render checked
// Count pixels with Accent color (should be ~225 for filled box)

assert!(unchecked.pixel_count(Tone::Sunken) > checked.pixel_count(Tone::Sunken));
assert!(checked.pixel_count(Tone::Accent) > unchecked.pixel_count(Tone::Accent));
```

---

## Summary Table

| Concern | Modules | How | Verification |
|---------|---------|-----|--------------|
| **Identity & Persistence** | element.rs ↔ memory.rs | Path-based identity, state in Memory | State survives frame rebuild |
| **State Flow** | widgets.rs ↔ paint.rs | Upvalue capture in draw closure | Same state → same pixels |
| **Handlers** | input.rs ↔ paint.rs | Hit-test → deferred handler queue | Handler called on click |
| **Appearance** | theme.rs ↔ widgets.rs | Tone roles resolve against Palette | Colors match theme |

---

## Integration Checklist

When implementing checkbox in a new backend:

- [ ] Verify element.rs path identity is unique per checkbox
- [ ] Verify memory.rs focus state includes checkbox path
- [ ] Verify paint.rs draw() closure captures state parameter correctly
- [ ] Verify input.rs routes clicks to checkbox on_click handler
- [ ] Verify theme.rs palettes have sufficient contrast (4.5:1)
- [ ] Run `cargo test --lib` to verify no regressions
- [ ] Run `cargo test --test recipes -- checkbox` to verify behavior

---

End of STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md
