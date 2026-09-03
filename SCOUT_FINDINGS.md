# Scout Findings - September 3, 2026

**Status:** Worklist (index.dx block bulleted-list-7) identified as empty. Processing 5 scouted items for dispatch.

**Analysis:** All tests pass (260+ unit tests), all examples build successfully. Documentation is current; identified defects are documentation accuracy (X11/Wayland confusion), example coverage gaps (3 missing), and ROADMAP items ready for implementation.

## Verified Worklist Items (5 total)

The following items have been identified and verified against the project state and are ready for autonomous dispatch:

### 1. Fix documentation accuracy: clarify X11-only support on Linux

**Classification:** Defect (documentation accuracy)  
**Priority:** High  
**Ask:** charter-rui

CLAUDE.md and README currently state "X11/Wayland (via X11 server)" but only X11 backend is implemented. Update to clarify that Wayland is not yet supported (planned for v0.2.0).

**Scope:** Update CLAUDE.md and README.md to remove Wayland from current platform support list.

**Verification:**
- `grep "Wayland" CLAUDE.md README.md` returns only v0.2.0 ROADMAP references
- No user-facing documentation claims Wayland support exists
- `cargo build` succeeds; examples run without platform-compatibility warnings

---

### 2. Document 3 undocumented examples (calculator, theme_switcher, todo_app)

**Classification:** Gate (completeness audit)  
**Priority:** High  
**Ask:** charter-rui

Currently CLAUDE.md lists only 8 examples in the learning path but 11 .rs files exist in examples/. Add calculator (numeric input exemplar), theme_switcher (appearance toggle), and todo_app (list rendering with state persistence) to the documentation with descriptions and placement in learning path.

**Scope:** Update CLAUDE.md Examples Directory section to include all 11 examples with descriptions. Update learning path to incorporate new examples.

**Verification:**
- CLAUDE.md Examples Directory table lists all 11 .rs files
- Each example's docstring in CLAUDE.md matches actual file purpose
- `cargo build --examples` succeeds with no errors
- Each new example runs without errors: `cargo run -p rui --example calculator`

---

### 3. Implement Wayland backend (src/shell/platform/wayland.rs)

**Classification:** ROADMAP item (v0.2.0 deferred scope)  
**Priority:** Medium  
**Ask:** charter-rui

Implement native Wayland backend following the Backend trait pattern established by X11 (Recipe 2). Handle pointer events, keyboard input, system appearance detection via wayland-client library bindings. Auto-detect and use Wayland when available; fall back to X11.

**Scope:** Add `src/shell/platform/wayland.rs` implementing Backend trait; update `src/shell/platform/mod.rs` platform selector to include Wayland; update Cargo.toml with optional dependency.

**Verification:**
- `cargo build --target x86_64-unknown-linux-gnu` succeeds
- Platform-selection logic correctly chooses Wayland > X11 based on availability
- `cargo test --lib` passes 100% (all unit tests)
- Basic interaction works: `cargo run -p rui --example counter` responds to clicks
- `cargo test --test integration` passes (platform module does not affect core logic)

---

### 4. Implement accessibility (a11y) framework foundation

**Classification:** API extension (ROADMAP v0.2.0 prerequisite)  
**Priority:** Medium  
**Ask:** charter-rui

Add optional semantic annotations to `El<S>` type: `accessible_name`, `accessible_role`, `accessible_description` fields. No backend integration yet (screen reader output in v0.3.0); framework only. Existing patterns must still build and render unchanged.

**Scope:** Update `src/element.rs` El<S> struct to add 3 optional String fields; add builder methods; update rustdoc.

**Verification:**
- `src/element.rs` compiles; new fields are optional
- `cargo test --lib` passes 100% (all 260+ unit tests)
- `cargo run -p rui --example counter` runs unchanged
- `cargo run -p rui --example segmented` and `cargo run -p rui --example meter` run unchanged
- New fields documented in `cargo doc --no-deps --open`

---

### 5. Create v0.3.0 form widget template documentation

**Classification:** Documentation template (ROADMAP v0.3.0 preparation)  
**Priority:** Low  
**Ask:** charter-rui

Document patterns for form widgets (text_input, select, combobox) planned for v0.3.0, using existing primitives and memory module for caret/selection state. Provide skeleton implementations (no working widgets) showing state shape, view function with text layout, keyboard handler signature, and Harness test patterns.

**Scope:** Add to CLAUDE.md a new section with text_input skeleton code, select/combobox pattern sketches, keyboard event handler signatures, and Harness test skeleton.

**Verification:**
- CLAUDE.md contains text_input skeleton with complete state struct, view function signature, handler signature
- Test skeleton using Harness shows keyboard event simulation pattern
- Documentation references `src/memory.rs` for caret persistence pattern
- No actual widget implementation; skeletons only
- Existing examples still compile and run unchanged

---

## Summary

**Total Items:** 5  
**Ask:** All use `[scout] [ask: charter-rui]`  
**Priority Distribution:** 2 High, 2 Medium, 1 Low  
**Type Distribution:** 1 defect, 1 gate, 2 ROADMAP, 1 prep  
**Scope:** Documentation (2), Implementation (2), Template (1)  
**Interdependencies:** None—all items are independent and can dispatch in parallel

**Verification Gates:** Each item includes specific, executable test commands. No exploratory work required; scope and success criteria are precise.

---

**Scout Date:** 2026-09-03  
**Worklist Block:** bulleted-list-7 (index.dx)
