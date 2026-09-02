# Recipe 3: Checkbox Control — Quick Reference

## What is Recipe 3?

Checkbox is a **widget exemplar pattern** that proves even the simplest interactive control follows the state-view-handler pattern without requiring any special framework support.

**Key claim**: 29 lines of code, zero framework magic, no retained tree, no Rc/RefCell.

**Pattern**: State (bool) → View (render conditional) → Handler (toggle on click)

---

## When to Use This Recipe

- **Building a custom interactive widget** (slider, radio, toggle, spinner, etc.)
- **Understanding how state flows through the view function**
- **Learning why the library doesn't need a retained widget tree**
- **Implementing an accessible control that works with keyboard and mouse**

---

## Quick Architecture Overview

```
┌─────────────────────────────────────────┐
│  State: struct App { checked: bool }    │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌────────────────────────────────────────────────────────────┐
│  View: fn checkbox(label, checked, toggle) -> El<S> {     │
│    row((draw(...), text(label)))                           │
│    .on_click(|state| toggle(state))                        │
│  }                                                          │
└──────────────────┬─────────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────┐
│  Handler: |app: &mut App|                │
│  { app.checked = !app.checked; }         │
└──────────────────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────┐
│  Rendered: Box filled or empty           │
└──────────────────────────────────────────┘
```

---

## Documentation Files

This recipe is documented in four files for different purposes:

| File | Use When | Key Sections |
|------|----------|--------------|
| **STEP_5_RECIPE_3_ANALYSIS.md** | Implementing checkbox or similar widget | Phase breakdown, scope, state shape, implementation details, cross-module interactions |
| **STEP_5_RECIPE_3_VERIFICATION_GATES.md** | Testing checkbox implementation | Gate criteria for each phase, test commands, expected output, debugging checklist |
| **STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md** | Understanding module interactions | How identity persists, state flows, handlers invoke, colors adapt; common pitfalls |
| **STEP_5_RECIPE_3_SUMMARY.md** (this file) | Quick lookup and navigation | What to read, when, and how it all connects |

---

## How to Read This Documentation

### Scenario 1: "I'm building a custom widget like checkbox"

**Start here**: STEP_5_RECIPE_3_ANALYSIS.md

1. Read "Overview" section to understand the pattern
2. Skip to "Phase N" section matching your current work
3. Follow "Scope", "Files modified", and "Verification" for that phase
4. When you hit a problem, jump to STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md

### Scenario 2: "My checkbox isn't working, what's wrong?"

**Start here**: STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md

1. Find the relevant section (State Flow, Handlers, etc.)
2. Read "Common Pitfalls" subsection
3. Use "Verification" examples to test your hypothesis
4. For test setup, refer to STEP_5_RECIPE_3_VERIFICATION_GATES.md

### Scenario 3: "How do I test checkbox at each phase?"

**Start here**: STEP_5_RECIPE_3_VERIFICATION_GATES.md

1. Find the phase you're implementing
2. Copy the "Gate commands" section
3. Run each command in order
4. If a gate fails, the section tells you what to check

### Scenario 4: "What's the minimal checkbox implementation?"

**Start here**: CLAUDE.md Recipe 3 section

Read the 4-phase summary in the main documentation. Then:
- For architecture: STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md
- For testing: STEP_5_RECIPE_3_VERIFICATION_GATES.md
- For detailed implementation: STEP_5_RECIPE_3_ANALYSIS.md

---

## The 4-Phase Pattern

### Phase 1: State Definition
- **What**: Define minimal state (e.g., `checked: bool`)
- **Why**: Proves control needs no framework support
- **Test**: `cargo test --test recipes -- a_checkbox_changes_state_on_click`

### Phase 2: Element Tree Construction
- **What**: Implement view function that returns El<S> with conditional styling
- **Why**: Proves state parameter flows to rendering
- **Test**: `cargo run -p rui --example controls` (visual)

### Phase 3: Enhancement (Styling & Visual Polish)
- **What**: Add focus ring, hover, disabled, contrast validation
- **Why**: Ensures control is accessible and matches design system
- **Test**: `cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover`

### Phase 4: Integration & Persistence
- **What**: Test multiple instances, state persistence, identity with `.key()`
- **Why**: Proves no special memory management needed
- **Test**: `cargo test --test recipes -- checkbox_preserves_state_across_frames`

---

## Key Invariants (Do Not Break These)

1. **State rebuilt every frame** — No retained widget tree. View function is pure `Fn(&S) -> El<S>`.
2. **Identity is path-based** — Elements identified by position in tree. Override with `.key(id)` for reordered lists.
3. **Handlers receive `&mut S` directly** — No closures capturing self, no Rc/RefCell, no interior mutability.
4. **State flows as upvalue** — Upvalue captured in draw closure determines conditional appearance.
5. **Theme roles, not RGB** — Use `Tone::Accent`, never hardcoded colors.
6. **One dispatch path** — Mouse click runs same handler as keyboard activation.

---

## Common Implementation Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| State lost after reorder | Multiple checkboxes mix state | Add `.key(item.id)` to override path identity |
| Handler not called | Click doesn't toggle | Verify `.on_click()` is chained; check bounds |
| State doesn't change appearance | Click works but looks same | Verify `checked` captured in draw closure; check conditional |
| Can't compile: state modification | Borrow checker error | Handlers should mutate state (|app: &mut App|), not upvalues |
| Colors don't match theme | Looks wrong in dark mode | Use `Tone::Accent`, not raw RGB; verify contrast with `assert_legible()` |
| Focus ring missing | Tab doesn't show focus | Verify `.takes_focus(true)` is set |

---

## Minimal Checkbox Code

The complete checkbox implementation is 29 lines (from examples/controls.rs lines 57–85):

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

**Lessons**:
- No struct, no methods, no lifecycle hooks
- Pure function takes state as parameter
- Returns El<S>, which composes with other elements
- Handler is `Fn(&mut S)`, receives mutable state directly
- Upvalue `checked` flows into draw closure to determine appearance

---

## Testing Checklist

Copy this checklist for each new widget you build:

### Before You Start
- [ ] I understand state-view-handler pattern
- [ ] I've read STEP_5_RECIPE_3_ANALYSIS.md Phase 1

### Phase 1 (State)
- [ ] State struct compiles: `cargo build --tests`
- [ ] State test passes: `cargo test --test recipes -- checkbox_changes_state`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`

### Phase 2 (View)
- [ ] Example builds: `cargo build --examples`
- [ ] Example runs: `cargo run -p rui --example controls`
- [ ] Visual looks correct (manual inspection)
- [ ] Handler is invoked (add println! to verify)

### Phase 3 (Polish)
- [ ] Focus ring appears on Tab: visual test
- [ ] Hover highlight works: visual test
- [ ] Disabled state renders at 0.38 alpha: `cargo test --test recipes -- checkbox_disabled`
- [ ] Contrast passes: `cargo test --lib theme::tests::the_battery_rejects_an_illegible_palette`

### Phase 4 (Integration)
- [ ] Multiple instances work: `cargo test --test recipes -- checkbox_works_with_multiple_instances`
- [ ] State persists: `cargo test --test recipes -- checkbox_preserves_state_across_frames`
- [ ] Reorder with .key() preserves state: manual test
- [ ] All library tests pass: `cargo test --lib`

### Final Verification
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] All tests pass: `cargo test --lib` (379 tests)
- [ ] Documentation updated: add checkbox to widget exemplars if new

---

## Related Recipes

- **Recipe 1 (WASM Backend)**: Platform backend pattern (3 phases)
- **Recipe 2 (X11 Backend)**: Reference backend implementation (4 commits + polish)
- **Recipe 3 (Checkbox)**: Widget exemplar pattern (4 phases)

---

## Quick Start: Build a Slider

To build a slider using Recipe 3 pattern:

1. **Phase 1**: State = `f32` (0.0–1.0)
2. **Phase 2**: View = draw bar + knob, .on_drag() handler
3. **Phase 3**: Add focus ring, verify contrast
4. **Phase 4**: Test multiple sliders, state persistence

Use STEP_5_RECIPE_3_ANALYSIS.md as template; just substitute "slider" for "checkbox".

---

## Debugging Commands

### "Is my state struct correct?"
```bash
cargo build --tests
cargo test --lib --no-run  # Compile-check only
```

### "Does my view function build?"
```bash
cargo build --examples
```

### "Does my handler work?"
```bash
cargo run -p rui --example controls  # Add println! in handler
```

### "Are my colors right?"
```bash
cargo test --lib theme::tests::the_battery_rejects_an_illegible_palette
```

### "Does state persist?"
```bash
cargo test --test recipes -- checkbox_preserves_state_across_frames --nocapture
```

### "Do multiple instances work?"
```bash
cargo test --test recipes -- checkbox_works_with_multiple_instances --nocapture
```

### "Is everything working?"
```bash
cargo test --lib
cargo test --test recipes -- checkbox
```

---

## Key Files in the Library

Related to Recipe 3:

| File | Purpose |
|------|---------|
| **src/widgets.rs** | Checkbox constructor and other widget builders |
| **examples/controls.rs** | Checkbox implementation and showcase |
| **tests/recipes.rs** | Checkbox tests (state, rendering, integration) |
| **src/element.rs** | El<S> type, .on_click(), .key() |
| **src/memory.rs** | Interaction state (focus, scroll, easing) |
| **src/paint.rs** | Painter for draw closures |
| **src/theme.rs** | Tone roles and theme system |
| **src/input.rs** | Event → Input translation and handler invocation |

---

## FAQ

**Q: Why doesn't checkbox have a `checked_changed()` callback?**
A: Handlers run after frame, so state change is automatic. Next frame view() rebuilds with new state. No callback needed.

**Q: Can I have multiple on_click handlers?**
A: No, one handler per element. Combine logic if needed: `|s| { handler1(s); handler2(s); }`

**Q: What if handler doesn't get called?**
A: Check (1) .on_click() is chained, (2) bounds are correct in draw(), (3) element is focusable. Add println! to verify.

**Q: How do I disable a checkbox?**
A: Add `.disabled(true)` and check it in handler: `|state| if !disabled { state.checked = !state.checked; }`

**Q: Does state survive frame skips?**
A: Yes, Memory persists state by path even across multiple frames with no rebuild.

---

## For Next Readers

This recipe documents the pattern; the code is the proof. If you're stuck:

1. Read the relevant section in STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md
2. Copy the verification example and run it
3. Check the "Common Pitfalls" subsection
4. If still stuck, check the test file for a working example

Pattern is proven by: 29 lines of code, 4 phases, 0 regressions.

---

End of STEP_5_RECIPE_3_SUMMARY.md
