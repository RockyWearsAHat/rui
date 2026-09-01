# Scout Report: Next 4 Worklist Items for rui

**Scout Date:** 2026-08-31  
**Status:** Ready for dx_append to index.dx block bulleted-list-7

## Analysis Summary

The rui project has successfully completed v0.1.0 with:
- ✓ All 4 platform backends (macOS, Windows, X11, WASM)
- ✓ Core element system, layout engine, text rendering, event handling
- ✓ 347+ tests passing
- ✓ Comprehensive documentation (CLAUDE.md with 3 recipes)

The ROADMAP identifies clear next phases:
- **v0.2.0 (Q4 2026):** Platform completeness (Wayland), Accessibility (a11y)
- **v0.3.0 (Q1 2027):** Built-in widgets (text_input, select, combobox, etc.), CSS styling

Currently, the project has 11 examples (not just 8), and all are compiling. Future widget work requires a clear pattern/template to ensure consistency.

## Next 4 Worklist Items

These items should be appended to index.dx block bulleted-list-7 as checklist items:

### Item 1: Implement Wayland Backend
**Scope:** v0.2.0 platform completeness. Add native Wayland support for Linux alongside X11.  
**Verification:**
- `src/shell/platform/wayland.rs` exists and implements Backend trait
- `cargo build` succeeds on Linux with Wayland libraries installed
- `cargo test --lib` passes
- `cargo run -p rui --example counter` works on Wayland session
- X11 backend still works (no regression)

**Why:** X11 is aging; Wayland is the future for Linux. Supporting both ensures compatibility across distributions. Follows established Backend trait pattern from X11 recipe.

### Item 2: Establish Accessibility (a11y) Framework Foundation
**Scope:** v0.2.0 production readiness. Implement framework to support screen readers, keyboard navigation, semantic annotations.  
**Verification:**
- `src/element.rs` extended with `a11y()` builder method for semantic annotations
- `src/a11y.rs` (new) defines Role enum (Button, TextField, Label, etc.) and semantic annotation data structures
- `src/shell/platform/*.rs` each implement platform-specific a11y hooks (e.g., macOS VoiceOver, Windows Narrator)
- `cargo test --lib` passes
- Example (e.g., counter) can be tested with a screen reader
- CLAUDE.md documents the a11y pattern in a new Recipe 4 section

**Why:** Production UI libraries require accessibility support. This is table-stakes for adoption. v0.2.0 milestone depends on this.

### Item 3: Document v0.3.0 Widget Implementation Template
**Scope:** Establish repeatable pattern for building complex widgets (text_input, select, combobox, table, etc.).  
**Verification:**
- CLAUDE.md "Recipe 4: Building v0.3.0 Widgets" section documents the pattern:
  - State shape best practices (when to extend vs. compose)
  - Widget function signature (input → El<S>, no closures)
  - Keyboard event handling (Tab, Enter, Escape, arrows)
  - Focus management via element.rs `.key()` builder
  - Validation pattern (state → error → view)
- At least one v0.3.0 widget implemented as exemplar (e.g., text_input)
- Test in `tests/recipes.rs` for the exemplar widget passes
- `cargo test --test recipes -- text_input` passes

**Why:** v0.3.0 scope includes 10+ new widgets. Clear pattern ensures consistency and makes them easier to build and review. Exemplar prevents design drift.

### Item 4: Audit and Sync All Examples with CLAUDE.md
**Scope:** Documentation hardening. Verify all 11 examples exist, compile, and are documented in CLAUDE.md.  
**Verification:**
- CLAUDE.md "Examples Directory" section lists all 11 examples (currently lists only 8)
- Each example has one-line purpose and one-line command
- All 11 examples compile: `cargo build --examples`
- All 11 examples run (at least smoke test): `cargo run -p rui --example <name>` completes without panic
- Learning path (counter → segmented → meter) is still the primary entry point
- New examples (calculator, theme_switcher, todo_app) are categorized by skill level
- CLAUDE.md "Examples Directory" table is kept in sync with examples/ directory

**Why:** Documentation drift (CLAUDE.md mentions 8 examples, but 11 exist) confuses new users. Audit ensures one source of truth and catch any examples that bitrot.

## Items Ordered by Priority

1. **Item 2: Accessibility Framework** — Highest priority. Blocks v0.2.0 release and required for production adoption.
2. **Item 1: Wayland Backend** — v0.2.0 scope. Platform completeness for Linux.
3. **Item 3: v0.3.0 Widget Template** — Foundational for v0.3.0 scope. Enables subsequent widget implementations to proceed in parallel.
4. **Item 4: Example Audit** — Hardening task. Lower priority than features, but should run before next minor version.

## How to Apply

Each item should be appended to index.dx block bulleted-list-7 as:
```
- [ ] <Item Title> — <one-line summary>
```

Example format:
```
- [ ] Implement Wayland Backend — Add native Wayland support for Linux alongside X11. Verification: src/shell/platform/wayland.rs implements Backend trait, cargo build succeeds on Wayland, cargo test --lib passes, X11 still works.
- [ ] Establish Accessibility Framework Foundation — Implement a11y framework with screen reader support and keyboard navigation. Verification: src/element.rs extended with a11y() builder, src/a11y.rs defines semantic roles, platform hooks in shell/platform/*.rs, cargo test --lib passes, screen reader can navigate example.
- [ ] Document v0.3.0 Widget Implementation Template — Establish pattern for complex widgets. Verification: CLAUDE.md documents pattern, exemplar widget (e.g., text_input) implemented, cargo test --test recipes -- text_input passes.
- [ ] Audit and Sync All Examples with CLAUDE.md — Verify all 11 examples exist, compile, and are documented. Verification: CLAUDE.md lists all 11, all compile, all run without panic, learning path maintained.
```

## Charter Alignment

All items directly support charter-rui: "A declarative interface library for Rust with zero dependencies."

- Wayland: Extends platform support (zero-dependency)
- A11y: Enables production use (still zero-dependency)
- v0.3.0 widgets: Complete widget ecosystem (all built from primitives, zero external crates)
- Example audit: Ensures documentation accuracy and learning experience

## Scout Confidence

**HIGH** — Project is stable v0.1.0, ROADMAP is clear, and next phases are well-defined in planning documents. All items are straightforward extensions of established patterns (Backend trait for Wayland, element.rs for a11y, widget recipes for v0.3.0, docs sync for examples).
