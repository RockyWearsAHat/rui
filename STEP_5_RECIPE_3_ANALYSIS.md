# Recipe 3: Checkbox Control — Phase Analysis

## Overview
Checkbox demonstrates the minimal interactive control: a single boolean that toggles on click. It proves that even the smallest custom widget follows the state-view-handler pattern without requiring any special framework support.

**Pattern**: State (bool) → View (render conditional appearance) → Handler (toggle on click)

**Key insight**: Zero framework support required. No special widget class, no retained tree, no lifecycle. Just state → view function → handlers.

**Files touched per phase**:
- Phase 1: src/widgets.rs, tests/recipes.rs
- Phase 2: examples/controls.rs (lines 57–85), src/widgets.rs, tests/recipes.rs
- Phase 3: src/widgets.rs (enhanced styling), examples/controls.rs, tests/recipes.rs
- Phase 4: tests/recipes.rs (integration tests), src/testing/mod.rs (Harness)

---

## Phase 1: State Definition

**Problem**: How do we know if a control needs any special framework support to work?

**Solution**: Define the simplest possible state (a single bool) and build from there.

**Scope**:
- Create state struct with one boolean field
- Write state-only test (no UI yet)
- Verify basic Rust compilation

**State shape**:
```rust
struct App {
    checked: bool,
}
```

**Test command**:
```bash
cargo test --test recipes -- a_checkbox_changes_state_on_click
```

**Expected outcome**:
- State compiles without error
- Test runs and passes
- No UI code needed yet

**Files modified**: 
- src/widgets.rs (constructor function)
- tests/recipes.rs (test case)

---

## Phase 2: Element Tree Construction

**Problem**: How do state changes flow through the view function into visual appearance?

**Solution**: Build checkbox from primitives; state parameter determines conditional styling.

**Scope**:
- Implement checkbox constructor that takes state parameter
- Use `draw()` closure to render box conditionally
- Wire `on_click` handler to toggle state
- Verify visual output with Harness

**Implementation** (examples/controls.rs lines 57–85):
```rust
fn checkbox<S: 'static>(label: &str, checked: bool, toggle: impl Fn(&mut S) + 'static) -> El<S> {
    row((
        draw(
            Size::new(15.0, 15.0),
            move |painter: &mut Painter<'_>, rect: Rect| {
                let fill = if checked { Tone::Accent } else { Tone::Sunken };
                painter.fill(rect, Radius::Units(4.0), fill);
                painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
                if checked {
                    painter.fill(tick(rect), Radius::Units(1.0), Tone::OnAccent);
                }
            },
        ),
        text(label),
    ))
    .gap(8.0)
    .on_click(move |state: &mut S| toggle(state))
}
```

**Key insight**: `checked` parameter flows as an upvalue into the `draw()` closure; conditional styling proves state determines appearance.

**Test command**:
```bash
cargo run -p rui --example controls
```

**Expected outcome**:
- Checkbox renders with box and label
- Box is filled when checked, empty when unchecked
- Clicking toggles the state
- Visual output is correct

**Files modified**:
- examples/controls.rs (checkbox implementation)
- src/widgets.rs (constructor)
- tests/recipes.rs (visual test cases)

---

## Phase 3: Enhancement (Styling & Visual Polish)

**Problem**: Does the checkbox look correct across light/dark modes and match the design system?

**Solution**: Add platform-appropriate styling (rounded corners, focus ring, disabled state, hover).

**Scope**:
- Add focus ring when keyboard-focused
- Add disabled state styling (0.38 content alpha)
- Add hover highlight
- Verify across light/dark modes
- Ensure contrast ratios (≥4.5 secondary, ≥7 text)

**Additions**:
- `.fill()` customization to allow theme colors
- Focus ring rendering
- Disabled state styling
- Hover effect
- Contrast ratio validation

**Test commands**:
```bash
cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover
cargo run -p rui --example controls  # Visual inspection
```

**Expected outcome**:
- Focus ring appears when checkbox is keyboard-focused
- Disabled checkboxes render at 0.38 alpha
- Hover state shows clear visual feedback
- All palettes meet contrast requirements
- Works identically in light and dark mode

**Files modified**:
- src/widgets.rs (enhanced styling)
- examples/controls.rs (showcase)
- tests/recipes.rs (visual and accessibility tests)

---

## Phase 4: Integration & Persistence

**Problem**: Can multiple checkbox instances coexist with independent state?

**Solution**: Verify state persists across frames and multiple instances manage their own identity.

**Scope**:
- Test multiple checkbox instances
- Verify state persists across 10+ frames
- Test reordering with `.key()` override
- Verify memory module handles checkbox focus/state
- Integration with accessibility tree

**Tests**:
```bash
cargo test --test recipes -- checkbox_preserves_state_across_frames
cargo test --test recipes -- checkbox_works_with_multiple_instances
cargo test --lib memory
```

**Expected outcome**:
- Multiple checkboxes maintain independent state
- State survives frame rebuilds (10+ frames)
- Reordering with `.key()` preserves state correctly
- Focus state is maintained in Memory
- Accessibility tree includes all checkboxes
- Tab order is correct

**Files modified**:
- tests/recipes.rs (integration tests)
- src/testing/mod.rs (Harness for testing)
- Possibly src/memory.rs (interaction state handling)

**Key invariant**: Identity is path-based; reordered checkboxes preserve state via `.key()`.

---

## Summary of Pattern

| Phase | Goal | Scope | Verification |
|-------|------|-------|--------------|
| 1 | State definition | Create struct, write test | `cargo test --test recipes -- a_checkbox_changes_state_on_click` |
| 2 | Element tree | Implement view, handlers | `cargo run -p rui --example controls` |
| 3 | Visual polish | Add focus ring, hover, disabled | `cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover` |
| 4 | Integration | Multiple instances, persistence | `cargo test --test recipes -- checkbox_preserves_state_across_frames` |

**Total code**: 29 lines (examples/controls.rs lines 57–85) for complete checkbox implementation.

**Pattern proof**: This pattern works for button, segmented, slider, radio, custom charts—any interactive element. Zero special framework support needed.

---

## Key Cross-Module Interactions

1. **Identity & Persistence** (element.rs ↔ memory.rs): Element path determines identity; focus and interaction state live in Memory
2. **State Flow** (widgets.rs ↔ paint.rs): State parameter passed to checkbox flows as upvalue into draw closure
3. **Handlers** (input.rs ↔ paint.rs): Click events call handlers after frame drawn; handlers receive `&mut S` directly
4. **Appearance** (theme.rs ↔ widgets.rs): Tone roles (Accent, Sunken, Border) resolve against Theme for light/dark mode

---

## Common Modifications

When building similar widgets, follow this checklist:

- [ ] Phase 1: Define state struct with minimal fields
- [ ] Phase 1: Write state-only test before UI
- [ ] Phase 2: Build from primitives (draw, row, col, on_click)
- [ ] Phase 2: Pass state as parameter, not closure
- [ ] Phase 2: Use conditionals in draw closures to shape appearance
- [ ] Phase 3: Add focus ring, hover, disabled styling
- [ ] Phase 3: Verify contrast across all palettes
- [ ] Phase 4: Test multiple instances
- [ ] Phase 4: Verify state persistence across frames
- [ ] Phase 4: Use `.key()` for reordered lists

---

## Debugging Checklist

If checkbox behavior is wrong:

1. **State not changing on click?** → Check `on_click` handler is wired correctly; verify handler signature is `|state: &mut S|`
2. **Appearance doesn't update?** → Verify `checked` parameter flows into `draw()` closure; check conditional in `draw()` is correct
3. **Focus ring missing?** → Check `.takes_focus(true)` is set; verify `focuses` is true in theme when rendering
4. **Multiple instances share state?** → Check element path (identity); if reordered, add `.key(unique_id)`
5. **Disabled state not working?** → Check handler respects `.disabled(true)`; verify draw closure checks disabled flag

---

End of STEP_5_RECIPE_3_ANALYSIS.md
