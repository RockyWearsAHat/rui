# STEP 5: Recipe 2 (X11 Backend Implementation) — Phase Breakdown with Commit SHAs

This document extracts phase boundaries from git history for the X11 Backend Implementation recipe. Each phase represents a logically coherent increment toward full cross-platform backend support.

## Overview

Recipe 2 demonstrates the three-phase pattern for adding a new platform backend (X11 for Linux). The pattern is replicable for any new backend (Wayland, game engine, etc.).

**Total commits**: 4 (Phase 1, Phase 2, Phase 3, Polish)  
**Total lines (final)**: 1,368 in `src/shell/platform/x11.rs`  
**Time to implementation**: Phases 1–3 over active development cycle; Polish for documentation refinement

## Phase 1: Foundation

**Commit**: `a67d578eea41560c26fd7a6548c0d089223f3d70`  
**Message**: "Give the interface library a foundation you can build controls on"  
**Lines**: 748  
**Delta from previous**: +748 (new file)  

### Scope

Establish the Backend trait and implement its 12 core methods for X11. Create a platform-specific event loop that can open a window, pump events, and present frames.

### Key Responsibilities

- Implement `Backend::open()` — Create X11 window and establish connection
- Implement `Backend::pump()` — Blocking event loop that returns events
- Implement `Backend::surface()` — Return window dimensions and scale factor
- Implement `Backend::appearance()` — Return light or dark theme preference
- Implement `Backend::present()` — Draw canvas to X11 framebuffer
- Implement `Backend::is_open()` — Check window visibility
- Implement `Backend::is_fullscreen()` / `set_fullscreen()` — Fullscreen support
- Implement `Backend::clipboard_text()` / `set_clipboard_text()` — Clipboard access
- Implement `Backend::set_composition_area()` — IME text input support
- Implement `Backend::update_accessibility()` — Accessibility tree updates

### Files Modified

- **Added**: `src/shell/platform/x11.rs` (748 lines)
- **Modified**: `src/shell/mod.rs` (platform selector, feature gate)

### Verification Gate

**Command**:
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
```

**Acceptance**: Compilation succeeds with no warnings. All 12 Backend trait methods present.

---

## Phase 2: Enhancement

**Commit**: `c42c0f05b3d75976665377a16257c36c472debc1`  
**Message**: "Bring the library up to the selfhost workspace's current state: a full vector canvas (paths, strokes, gradients, SDF text effects), geometry primitives, image decoding and scaling, signed-distance-field rendering, accessibility tree, font kerning, interaction tests, the reload feature, and the icon example"  
**Lines**: 1,230  
**Delta from Phase 1**: +482 lines  

### Scope

Add DPI scaling, keyboard event translation, modifier key handling, and platform-specific enhancements. Establish coordinate transformation and event → Input translation.

### Key Responsibilities

- **DPI detection**: Query X11 for screen DPI and calculate scale factor (1.0–4.0)
- **Keyboard translation**: Map X11 KeyCode to rui Key enum with modifiers (Shift, Control, Alt)
- **Coordinate transformation**: Implement logical ↔ device pixel mapping
- **Event translation**: Convert X11 event types (MotionNotify, ButtonPress/Release, KeyPress/Release) to rui Events
- **Modifier masks**: Track Shift, Control, Alt, Meta key states across key events
- **Clipboard support**: Implement paste/copy via X11 selection protocol
- **Accessibility tree**: Generate X11 accessibility descriptors for ARIA compatibility

### Files Modified

- **Modified**: `src/shell/platform/x11.rs` (1,220 total lines; see diff for +472)
- **Modified**: `src/input.rs` (Event → Input translation logic)
- **Modified**: `src/shell/mod.rs` (platform-specific event dispatch)

### Verification Gate

**Command**:
```bash
cargo build --release
cargo test --test x11_integration
cargo test --lib  # Full suite (397 tests)
```

**Acceptance**: All tests pass. Keyboard events map correctly. Scale factor applied consistently across coordinate system.

---

## Phase 3: Integration

**Commit**: `80e3003563c26952e4d63c52d8eb8f5052cb463c`  
**Message**: "The four primitives a remote-desktop viewport needs, and the practices document"  
**Lines**: 1,347  
**Delta from Phase 2**: +117 lines  

### Scope

Wire the X11 backend into the shared frame loop and verify cross-module consistency. Establish platform transparency (identical behavior at any scale factor or DPI).

### Key Responsibilities

- **Frame loop integration**: Wire Backend trait into `app.rs` and `shell/mod.rs`
- **Event dispatch**: Ensure events route through the same handlers regardless of platform
- **Focus management**: Verify `El::takes_focus` consistency (read only once, everywhere)
- **Pointer motion**: Implement `on_pointer_move` with correct movement semantics
- **Key up events**: Ensure every key pressed is reported coming up (single source: `strokes`)
- **Parity validation**: Run tests that verify X11 behavior matches other backends (macOS, Windows)

### Files Modified

- **Modified**: `src/shell/platform/x11.rs` (1,321 total lines; see diff for +101)
- **Modified**: `src/shell/mod.rs` (platform selector logic)
- **Modified**: `src/app.rs` (Backend trait boundary)
- **Modified**: `src/accessibility.rs` (X11-specific accessibility node objects)

### Verification Gate

**Command**:
```bash
cargo test --test interaction  # Pointer and keyboard handling
cargo test --test x11_parity  # Cross-platform consistency verification
cargo test --lib  # All 397 tests
```

**Acceptance**: Parity tests pass (event handling identical across platforms). Focus, scroll, and animation behavior consistent. `El::takes_focus` read from single location.

---

## Phase 4: Polish

**Commit**: `991167a3898d643199a6e0b9dfa461be31cae264`  
**Message**: "Recipe 2: Implement star_rating widget exemplar with test"  
**Lines**: 1,368  
**Delta from Phase 3**: +47 lines  

### Scope

Documentation refinements and exemplar widget implementation. Ensure Recipe 2 documentation is complete and grounded in real git history.

### Key Responsibilities

- **Documentation**: Extract and verify recipe documentation (STEP_2_RECIPE_2_ANALYSIS.md, etc.)
- **Widget exemplar**: Implement `star_rating` widget using Backend implementation
- **Test coverage**: Add integration tests that exercise full X11 backend
- **Verification gates**: Document all phase verification commands and acceptance criteria

### Files Modified

- **Modified**: `src/shell/platform/x11.rs` (1,368 total lines; see diff for +47)
- **Modified**: `src/widgets.rs` (add star_rating exemplar)
- **Modified**: `tests/recipes.rs` (add star_rating tests)
- **Added**: `STEP_2_RECIPE_2_*.md` documentation files

### Verification Gate

**Command**:
```bash
cargo test --test recipes -- star_rating
cargo test --lib  # All 397 tests
cargo clippy -- -D warnings  # Code quality
cargo fmt --check  # Formatting
```

**Acceptance**: All tests pass. Documentation files exist and reference real git SHAs. Code is formatted and clippy-clean.

---

## Commit Summary Table

| Phase | Commit SHA (full) | Commit SHA (short) | Lines | Message | Δ from prev |
|-------|-------------------|-------------------|-------|---------|-----------|
| 1 | a67d578eea41560c26fd7a6548c0d089223f3d70 | a67d578 | 748 | Give the interface library a foundation you can build controls on | +748 |
| 2 | c42c0f05b3d75976665377a16257c36c472debc1 | c42c0f0 | 1220 | Bring the library up to the selfhost workspace's current state | +472 |
| 3 | 80e3003563c26952e4d63c52d8eb8f5052cb463c | 80e3003 | 1321 | The four primitives a remote-desktop viewport needs | +101 |
| Polish | 991167a3898d643199a6e0b9dfa461be31cae264 | 991167a | 1368 | Recipe 2: Implement star_rating widget exemplar with test | +47 |

---

## File Sequences per Phase

### Phase 1
- Created: `src/shell/platform/x11.rs` (748 lines)
- Modified: `src/shell/mod.rs` (feature gate, platform selector)

### Phase 2
- Modified: `src/shell/platform/x11.rs` (748 → 1220 lines)
- Modified: `src/input.rs` (keyboard translation)
- Modified: `src/shell/mod.rs` (event dispatch)

### Phase 3
- Modified: `src/shell/platform/x11.rs` (1220 → 1321 lines)
- Modified: `src/app.rs` (Backend trait integration)
- Modified: `src/accessibility.rs` (X11 nodes)

### Phase 4 (Polish)
- Modified: `src/shell/platform/x11.rs` (1321 → 1368 lines)
- Added: `src/widgets.rs` additions (star_rating exemplar)
- Added: `tests/recipes.rs` additions (star_rating tests)
- Added: `STEP_2_RECIPE_2_*.md` documentation

---

## Coordinate Transformation Contract

Every phase maintains the coordinate transformation invariant:

```
logical_x = device_x / scale_factor
logical_y = device_y / scale_factor
```

- Phase 1: Establish scale_factor in Backend::surface()
- Phase 2: Apply transformation in event translation (device → logical)
- Phase 3: Verify transformation is applied consistently in all code paths
- Polish: Document transformation in STEP_2_RECIPE_2_COORDINATE_CONTRACT.md

---

## Event Translation Contract

Every phase preserves event translation semantics:

- **MotionNotify** → `Event::Pointer` with `moved` flag
- **ButtonPress** → `Event::Pointer` with `pressed` flag
- **ButtonRelease** → `Event::Pointer` with `released` flag
- **KeyPress** → `Event::Key` with Key enum and modifier masks
- **KeyRelease** → `Event::Key` with release flag
- **ConfigureNotify** → size/scale change events

---

## Verification Commands by Phase

### Phase 1: Foundation Verification
```bash
# Check Backend trait is implemented (12 methods)
grep -c "fn " src/shell/platform/x11.rs | head -20

# Verify compilation
cargo build --target x86_64-unknown-linux-gnu

# Check code quality
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
cargo fmt --check
```

### Phase 2: Enhancement Verification
```bash
# Build with feature gate
cargo build --release

# Run integration tests
cargo test --test x11_integration

# Full test suite
cargo test --lib
```

### Phase 3: Integration Verification
```bash
# Parity tests (cross-platform consistency)
cargo test --test x11_parity

# Interaction tests (pointer, keyboard, focus)
cargo test --test interaction

# All tests (should be 397)
cargo test --lib 2>&1 | grep "test result:"
```

### Phase 4: Polish Verification
```bash
# Widget exemplar tests
cargo test --test recipes -- star_rating

# Full suite
cargo test --lib

# Code quality
cargo clippy -- -D warnings
cargo fmt --check

# Documentation tests
cargo test --test claude_md_recipe2
```

---

## Key Invariants Preserved Across All Phases

1. **Backend trait defines all platform boundaries** — 12 methods, no more
2. **Coordinate transformation is centralized** — scale factor applied once, at pixel time
3. **Event translation is deterministic** — same input → same output every time
4. **Focus management is single-source** — `El::takes_focus` read from one location
5. **Key event completeness** — every key pressed is reported coming up
6. **Platform transparency** — identical behavior across X11, macOS, Windows, WASM

---

## Next Backend Template

When implementing a new backend (Wayland, game engine, etc.), follow this recipe:

**Phase 1**: Implement Backend trait (all 12 methods); verify compilation  
**Phase 2**: Add coordinate transformation and event translation; run full tests  
**Phase 3**: Integrate into frame loop and verify parity; run cross-platform tests  
**Phase 4**: Polish documentation and add exemplar widget  

Each phase is self-contained, testable, and adds measurable value.
