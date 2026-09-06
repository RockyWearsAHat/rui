# Scout Report: rui Project Next Worklist Items

**Date:** 2026-08-31  
**Status:** Worklist (index.dx block bulleted-list-7) was empty

## Summary

After reviewing project documents, git history, and source code structure, identified 4 critical worklist items to append:

1. **Update CLAUDE.md documentation** — Examples and test suite lists are out of sync (8→11 examples, 12→13 tests)
2. **Create v0.3.0 widget template guide** — Blocks implementation of planned v0.3.0 widgets
3. **Implement Wayland backend** — v0.2.0 roadmap item (Q4 2026)
4. **Design a11y framework foundation** — v0.2.0 roadmap item (Q4 2026)

All items follow charter-rui ask, include verification gates, and are unattended-workable.

## Worklist Items (in dx checklist format)

```
- [ ] [scout] [ask: charter-rui] Update CLAUDE.md project documentation — examples and test suite lists are out of sync. CLAUDE.md lists 8 examples (counter, controls, gallery, segmented, meter, parity, icon, segmented_modified) but 11 exist; lists 12 test files but 13 exist (backend_consistency.rs). Verify: (1) Run `cargo build --example calculator`, `cargo build --example theme_switcher`, `cargo build --example todo_app` — all succeed, (2) Update Examples Directory section to list all 11, (3) Verify all 13 test files and update Test Suite section to include backend_consistency.rs.

- [ ] [scout] [ask: charter-rui] Create v0.3.0 widget implementation template guide — ROADMAP.md v0.3.0 lists planned widgets (text_input, select, combobox, form controls, data display, navigation) but no implementation guide exists. Recipe 3 shows star_rating example but lacks comprehensive patterns. Create new "v0.3.0 Widget Template Patterns" section in CLAUDE.md with step-by-step examples for text_input and select widgets, showing state shape, view function, handler structure, test pattern, and verification gates. Verify: (1) Guide follows same state→view→handlers pattern as Recipe 3, (2) Code examples compile (`cargo test --test recipes` passes with new examples), (3) Each widget example includes concrete test case using Harness.

- [ ] [scout] [ask: charter-rui] Implement Wayland backend for Linux platforms — ROADMAP.md v0.2.0 lists Wayland support as required for Linux. X11 backend (Recipe 2) proves Backend trait pattern works for platform abstraction. Implement native Wayland backend following established template: create `src/shell/platform/wayland.rs` implementing `Backend` trait with all six methods (open, pump, surface, appearance, present, is_open), wire platform selector into `src/shell/platform/mod.rs`, add `#[cfg(target_os = "linux")]` guards. Verify: (1) `cargo build --target x86_64-unknown-linux-gnu` compiles successfully, (2) `cargo test --lib` passes (no platform-specific logic in core), (3) Example `cargo run -p rui --example counter` launches and responds to input in Wayland session, (4) Platform auto-detection selects Wayland or X11 appropriately.

- [ ] [scout] [ask: charter-rui] Design accessibility (a11y) framework foundation — ROADMAP.md v0.2.0 lists a11y as required feature (screen readers, keyboard-only navigation, ARIA-like annotations). Before implementation, design framework: extend `src/element.rs` El<T> type with semantic annotations (role, label, description), plan integration hooks for platform backends (VoiceOver on macOS, Narrator on Windows, Orca on Linux), design keyboard navigation state machine. Verify: (1) Design document in CLAUDE.md "Accessibility Framework Design" section with API sketch, (2) Identify files to modify (element.rs, shell/mod.rs, shell/platform/*.rs), (3) Write verification gates for screen reader support on each platform, (4) Confirm approach follows "no dependencies" constraint.
```

## Findings Details

### Defect 1: Examples Documentation Drift
- Listed in CLAUDE.md: 8 examples
- Actually in examples/ directory: 11 examples
- Missing from documentation: calculator, theme_switcher, todo_app
- Impact: Misleads new learners; documentation is first thing users read

### Defect 2: Test Suite Documentation Drift
- Listed in CLAUDE.md Test Suite section: 12 test files
- Actually in tests/ directory: 13 test files
- Missing from documentation: backend_consistency.rs
- Impact: Reference documentation is unreliable

### Gate 3: v0.3.0 Widget Implementation Guide Missing
- ROADMAP.md v0.3.0 lists planned widgets: text_input, select, combobox, table, list, tree, menu_bar, dialog, alert, file_picker
- Recipe 3 exists but shows only star_rating example
- No comprehensive patterns for common v0.3.0 widgets documented
- Impact: Cannot guide future widget implementations; v0.3.0 roadmap blocked on documentation

### Gate 4: Wayland Backend (v0.2.0)
- ROADMAP.md v0.2.0 lists Wayland as required for Q4 2026
- X11 backend (Recipe 2, 6 commits, ~750 lines) proves Backend trait pattern works for platform abstraction
- Template established and documented in CLAUDE.md "Template for the Next Backend"
- Effort: ~200 LOC platform module (per ROADMAP.md estimate)

### Gate 5: Accessibility Framework (v0.2.0)
- ROADMAP.md v0.2.0 lists a11y as required for Q4 2026
- Requires design phase before implementation
- Design needed: semantic annotations, platform backend hooks, keyboard navigation
- Effort: ~500 LOC core + platform-specific hooks (per ROADMAP.md estimate)

## Previous Scout Run Status

Commit ef3544a (2026-08-31 10:08:18) identified 4 items:
1. V0.3.0 widget implementation template guide (ROADMAP)
2. Audit all 8 examples in CLAUDE.md
3. Wayland backend (ROADMAP v0.2.0)
4. Accessibility framework (ROADMAP v0.2.0)

That scout run's items were never appended to worklist. These 4 items replace and refine that earlier scan with specific verification gates and unattended-workable details.

---

**Scout Analysis Date:** 2026-08-31  
**Co-Authored-By:** Claude Haiku 4.5 <noreply@anthropic.com>
