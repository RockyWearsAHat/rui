# Worklist Items for rui Project — Scout 2026-08-30

**Target:** `index.dx` block `bulleted-list-7`  
**Status:** Ready to append as checklist items  
**Open Ask:** `charter-rui` (core library features)

---

## Item 1: Document Recipe 3 as Specific Checkbox Widget Recipe

```
[scout] [ask: charter-rui]
- [ ] Document Recipe 3: Checkbox Widget Implementation — Follow Recipe 1 (WASM Backend) and Recipe 2 (X11 Backend) structure in CLAUDE.md to document the checkbox widget currently defined in tests/recipes.rs line 34. Create 2–3 phases showing: (Phase 1) widget state shape and primitive draw approach using `draw()` and `Painter`, (Phase 2) event handler integration with `.on_click()`, (Phase 3) testing strategy with `Harness` and visual verification. Reference actual code lines in tests/recipes.rs. Verification: CLAUDE.md section "### Recipe 3: Checkbox Widget" contains explicit commit references (if checkbox moves to src/widgets.rs), file paths with line numbers (tests/recipes.rs:34 minimum), 3 phases with verification gates, and cross-module coordination explanation of how painter colors and state flow work. `cargo test --test recipes -- checkbox` passes on all platforms.
```

---

## Item 2: Verify Parity Verification CI Gate Is Active and Running

```
[scout] [ask: charter-rui]
- [ ] Verify parity verification CI gate integration — Confirm that commit 23fe3ce "test: Add automated parity verification CI gate" and commit 19c285f "Merge parity verification CI gate implementation" properly integrate `cargo test --test wasm_parity` into the GitHub Actions CI pipeline. Verify that the gate runs on every PR and blocks merges if it fails. Verification: `.github/workflows/ci.yml` contains a workflow step that explicitly runs `cargo test --test wasm_parity`. Manual confirmation: Run `cargo test --test wasm_parity` locally and confirm it passes on main branch. No test exclusions or skipped markers in the gate.
```

---

## Item 3: Create Widget Building Guide for v0.3.0 Future Implementation

```
[scout] [ask: charter-rui]
- [ ] Create template guide for v0.3.0 form controls — Document how to implement the form controls listed in ROADMAP.md v0.3.0 ("text_input(), select(), combobox()") using the recipe pattern and primitives from the library. This serves as a blueprint for future work without requiring full implementation. Verification: New section added to CLAUDE.md after Recipe 3 showing: (1) a text_input widget template with state struct, view function, and on_key handler skeleton (pseudocode acceptable), (2) explanation of how focus and caret state live in memory module, (3) example test using Harness. Code need not compile; it serves as implementation guide. Estimated work: 200-300 words.
```

---

## Item 4: Audit Examples Documentation and Availability

```
[scout] [ask: charter-rui]
- [ ] Verify all documented examples build and are properly catalogued — CLAUDE.md "Examples Directory" table (lines ~46–66) lists 8 examples. Verify each exists, documents its purpose, and is callable via `cargo run -p rui --example <name>`. Verification: (1) All 8 examples exist in `examples/` directory: counter.rs, segmented.rs, meter.rs, gallery.rs, controls.rs, parity.rs, icon.rs, segmented_modified.rs. (2) `cargo build --examples` succeeds (does not require running them, only compiling). (3) No orphaned examples exist in `examples/` that are not mentioned in CLAUDE.md. (4) Each example's purpose in the table accurately describes what it demonstrates.
```

---

## Discovery Notes

### Why These Items?

1. **Recipe 3 Documentation:** Recipe 1 and 2 are fully documented with commits, phases, and verification gates. Recipe 3 currently provides only generic guidance. Documenting a specific widget (checkbox) as a recipe completes the pattern and proves recipes work for non-backend features.

2. **Parity CI Gate Verification:** Item 18 was marked complete ("Automated parity verification as CI gate") but the worklist shows no follow-up verification that the gate is actually integrated into the CI pipeline and running on every build.

3. **Future Widget Guide:** ROADMAP.md lists v0.3.0 features (text_input, select, etc.) but provides no implementation blueprint. A template guide helps future contributors understand the pattern.

4. **Example Audit:** The index.dx document and CLAUDE.md reference examples as a learning path, but there's no verification that all documented examples are available and functional.

### Deferred Items (Not Included)

- **Wayland Backend (ROADMAP v0.2.0):** Listed as future work; not a current blocker.
- **Accessibility (ROADMAP v0.2.0):** Listed as future work; not a current blocker.
- **Mobile Backends (ROADMAP v0.2.0):** Listed as exploration; not a current blocker.
- **CSS-Like Styling (ROADMAP v0.3.0):** Future scope; awaiting design input per ROADMAP.

### Test Suite Status

- Unit tests: 260 passing ✓
- Recipe tests: 14 passing ✓
- No failing tests or regression detected
- Parity test exists but CI integration needs verification (Item 2)

---
