# Scout Findings - September 1, 2026

**Status:** Worklist (index.dx block bulleted-list-7) identified as empty after previous scout run.

**Analysis:** All tests pass (260+ unit tests), all examples build successfully. Documentation is mostly complete but has identified gaps and ROADMAP items ready for work.

## Recommended Worklist Items (5 total)

The following items should be added to the worklist for autonomous dispatch:

### 1. Fix documentation accuracy: clarify X11-only support on Linux

**Scope:** Documentation correction  
**Priority:** High (clarity)

CLAUDE.md and README currently state "X11/Wayland (via X11 server)" but only X11 backend is implemented. Update to clarify that Wayland is not yet supported (planned for v0.2.0).

**Verification:**
- grep for "Wayland" in CLAUDE.md and README returns only v0.2.0 ROADMAP references and template examples
- Examples on X11-only systems run without error
- No claims of current Wayland support remain in user-facing documentation

### 2. Document 3 undocumented examples (calculator, theme_switcher, todo_app)

**Scope:** Documentation and audit  
**Priority:** High (completeness)

Currently CLAUDE.md lists only 8 examples but 11 exist in examples/. Add calculator (numeric input), theme_switcher (appearance toggle), and todo_app (list rendering, stateful updates) to the examples table with descriptions and learning path placement.

**Verification:**
- CLAUDE.md examples table lists all 11 .rs files with accurate descriptions
- `cargo build --examples` exits 0
- Each example's docstring matches its CLAUDE.md description
- Learning path updated to incorporate new examples appropriately

### 3. Implement Wayland backend (src/shell/platform/wayland.rs)

**Scope:** Major feature (ROADMAP v0.2.0)  
**Priority:** Medium (ROADMAP alignment)

Add native Wayland backend following Backend trait pattern established by X11 (Recipe 2). Handle pointer events, keyboard input, system appearance detection via wayland-client bindings. Auto-detect and use Wayland when available, fallback to X11.

**Verification:**
- `cargo build --target x86_64-unknown-linux-gnu` succeeds
- Backend-selection logic compiles and chooses correct platform
- `cargo test --lib` passes (platform module doesn't affect core logic)
- Basic event handling works via protocol mocks in unit tests

### 4. Implement accessibility (a11y) framework foundation

**Scope:** API framework (ROADMAP v0.2.0)  
**Priority:** Medium (prerequisite for backends)

Add optional semantic annotations to El<S>: `accessible_name`, `accessible_role`, `accessible_description` fields. No backend integration yet; framework only. Verify existing code patterns (segmented, checkbox, meter) still build and render unchanged.

**Verification:**
- El<S> API compiles with new optional fields
- `cargo test --lib` passes 100% (all 260+ tests)
- Existing examples/counter runs unchanged
- New fields properly documented in rustdoc

### 5. Create v0.3.0 form widget template documentation

**Scope:** Documentation template (ROADMAP v0.3.0 preparation)  
**Priority:** Low (preparatory)

Document patterns for text_input, select, combobox widgets (ROADMAP v0.3.0) using primitives and memory module for caret/selection state. Provide skeleton implementations showing state shape, view function with text layout, keyboard handler (arrow keys, backspace), and Harness test patterns.

**Verification:**
- CLAUDE.md contains text_input skeleton with complete handler signature
- Test skeleton using Harness showing keyboard event simulation
- Reference to memory module for caret persistence
- No actual widget implementation (template only)

## Summary

- All 5 items use `[scout] [ask: charter-rui]` ask identifier
- Each item has specific, measurable verification gates
- Items are ordered by priority: documentation accuracy → completeness → ROADMAP alignment → preparation
- No items require exploratory work; all have clear scope and success criteria
