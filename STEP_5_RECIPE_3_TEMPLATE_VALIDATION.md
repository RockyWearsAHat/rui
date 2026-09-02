# Recipe 3: Widget Exemplar — Template Validation

**Purpose**: Verify that the checkbox widget exemplar pattern (state → view → handlers) is replicable across different interactive controls and is not specific to checkboxes.

**Status**: Validation complete — pattern holds for all widget types.

---

## Validation Scope

The Recipe 3 template claims that any interactive control can be built using this pattern:

```
State:   struct App { field: T }
View:    fn view(app: &App) -> El<App> { widget(..., app.field, handler) }
Handler: |app: &mut App| { app.field = new_value }
```

This document validates the claim by checking that pattern holds for:
1. **Segmented control** (choice selector, multiple options)
2. **Meter widget** (passive/read-only display)
3. **Button** (stateless action)
4. **Custom slider** (continuous value, drag interaction)

---

## Validation Method

For each widget exemplar, verify:
- ✓ State is a simple field (no closures, no RefCell)
- ✓ View function rebuilds every frame
- ✓ Handler is `Fn(&mut S)` receiving mutable state
- ✓ No special framework magic required (no lifecycle, no retained tree)
- ✓ Implementation is ≤50 lines
- ✓ Pattern is identical to checkbox

---

## Segmented Control Validation

**Location**: src/widgets.rs lines 367–398

**State**:
```rust
struct App {
    selected: usize,  // which choice is selected
}
```

**View**:
```rust
fn segmented<S>(choices: &[&str], selected: usize, on_change: impl Fn(&mut S, usize) + 'static) -> El<S> {
    // Rebuild every frame based on state
    // Buttons created for each choice; state determines which is highlighted
}
```

**Handler**:
```rust
|app: &mut App, index: usize| { app.selected = index; }  // Receives &mut S
```

**Pattern conformance**: ✓ Identical to checkbox
- State: simple usize
- View: rebuilds every frame
- Handler: receives &mut S directly
- Tested in: `tests/recipes.rs` segmented control tests

---

## Meter Widget Validation

**Location**: src/widgets.rs lines 284–309

**State**:
```rust
struct App {
    progress: f32,  // 0.0 to 1.0
}
```

**View**:
```rust
fn meter<S>(value: f32, tone: Tone) -> El<S> {
    // Display-only widget; reads state, no handler needed
    draw(Size::new(160.0, 18.0), move |painter, rect| {
        // Conditional styling based on value parameter
        let filled = rect.w * value;
        painter.fill(rect, ..., Tone::Sunken);
        painter.fill(filled_rect, ..., tone);
    })
}
```

**Handler**: None (read-only widget)

**Pattern conformance**: ✓ Valid reduction (no handler case)
- State: simple f32
- View: rebuilds every frame, conditions on state
- Handler: optional for passive displays
- Tested in: `tests/recipes.rs` meter tests

---

## Button Validation

**Location**: src/widgets.rs lines 155–180

**State**:
```rust
struct App {
    count: usize,  // state affected by button
}
```

**View**:
```rust
fn button<S>(label: &str, on_click: impl Fn(&mut S) + 'static) -> El<S> {
    // Stateless control (state is external)
    draw(...).on_click(move |app: &mut S| on_click(app))
}
```

**Handler**:
```rust
|app: &mut App| app.count += 1  // Receives &mut S directly
```

**Pattern conformance**: ✓ Valid reduction (no internal state case)
- State: external (in App)
- View: rebuilds every frame
- Handler: receives &mut S directly
- Tested in: `tests/recipes.rs` button tests

---

## Custom Slider Validation

**Location**: examples/gallery.rs (volume slider example)

**State**:
```rust
struct App {
    volume: f32,  // 0.0 to 1.0
}
```

**View**:
```rust
fn slider<S>(value: f32, on_drag: impl Fn(&mut S, Drag) + 'static) -> El<S> {
    draw(Size::new(160.0, 18.0), move |painter, rect| {
        // Visual appearance depends on value parameter
        let thumb_x = rect.w * value;
        painter.fill(rect, ..., Tone::Sunken);
        painter.fill(thumb_rect, ..., Tone::Accent);
    })
    .on_drag(move |app: &mut S, drag| on_drag(app, drag))
}
```

**Handler**:
```rust
|app: &mut App, drag: Drag| {
    app.volume = (drag.at.x / drag.rect.w).clamp(0.0, 1.0);
}
```

**Pattern conformance**: ✓ Identical to checkbox
- State: simple f32
- View: rebuilds every frame
- Handler: receives &mut S directly
- Tested in: `examples/gallery.rs` volume slider tests

---

## Cross-Widget Invariants

All validated widgets share these invariants:

1. **State is simple** — No Rc/RefCell, no closures, just plain fields (bool, usize, f32, String)
2. **View rebuilds every frame** — No cached layout, no retained widget tree
3. **Handler is `Fn(&mut S)`** — Receives mutable state directly, not a closure capturing borrowed state
4. **No lifecycle** — No init/setup/teardown; state is recreated every frame by view function
5. **Identity is path-based** — Elements identified by tree position; `.key()` overrides for reordering
6. **Visual appearance from state** — Conditional styling (draw closures, text, colors) based on state parameter

---

## Test Verification

**Run all widget exemplar tests**:
```bash
cargo test --test recipes -- --nocapture
```

**Expected output**: 14 recipe tests, all passing
- checkbox_changes_state_on_click ✓
- checkbox_preserves_state_across_frames ✓
- checkbox_works_with_multiple_instances ✓
- button_calls_handler_on_click ✓
- button_state_flows_through_view ✓
- meter_displays_progress_value ✓
- segmented_changes_selection_on_click ✓
- segmented_preserves_state_across_frames ✓
- slider_changes_value_on_drag ✓
- (5 additional focus, theme, and integration tests)

**Run visual verification**:
```bash
cargo run -p rui --example controls
cargo run -p rui --example gallery -- .
```

Expected: All widget exemplars render correctly and respond to input.

---

## Conclusion

**Pattern validity confirmed**: The checkbox widget exemplar (state → view → handlers) is replicable across all interactive control types:
- ✓ Checkbox (binary toggle)
- ✓ Segmented (choice selector)
- ✓ Button (stateless action)
- ✓ Meter (passive display)
- ✓ Slider (continuous value)

**No special framework magic needed** — Only the state-view-handler pattern, primitives (draw, row, col, on_click, on_drag), and conditional styling.

**Proof**: Implementation is ≤50 lines per widget. Pattern has delivered 5+ production widget types without Rc/RefCell, without lifecycle, without retained widget tree.

---

## For Next Widget Builders

When implementing a new custom control:

1. **Define state**: Plain field in your App struct (bool, usize, f32, enum)
2. **Build view**: Use primitives (draw, row, col, text) to build element tree; pass state as parameter
3. **Add handler**: `Fn(&mut S)` that updates the state field
4. **Test with Harness**: Verify state changes, visual appearance, and persistence
5. **Add keyboard support**: Use `on_key()` for accessibility
6. **Verify focus**: Test Tab order and focus ring visibility
7. **Check contrast**: Validate text/background ≥ 7, secondary UI ≥ 4.5

**No checklist beyond this** — if all seven items pass, the widget is production-ready.

---

## References

- **Pattern exemplar**: src/widgets.rs (checkbox, button, segmented, meter)
- **Test suite**: tests/recipes.rs (14 tests covering all exemplars)
- **Examples**: examples/controls.rs, examples/gallery.rs
- **Framework boundary**: src/element.rs (El type, on_click/on_drag/on_key builders)

See STEP_5_RECIPE_3_SUMMARY.md for navigation guide.
