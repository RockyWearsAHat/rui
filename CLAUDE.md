---
title: "rui — Interface Library Developer Guide"
description: "Codebase conventions, recipe infrastructure, and contributor patterns"
---

# rui — Developer Guide

This document provides recipes and conventions for working with rui, a zero-dependency declarative Rust UI library. Recipes are step-by-step guides to implementing and verifying major features, with proof from git history.

## About This Repository

**rui** is a view-state-handler architecture UI library:
- **View**: Pure `Fn(&S) -> El<S>` from application state to screen description, rebuilt every frame
- **State**: The application's mutable state, updated by handlers
- **Handlers**: Ordinary `Fn(&mut S)` functions run after each frame in response to input

Above the window, everything is pure: layout, hit-testing, text, and animation are all tested headlessly with no retained widget tree, no Rc/RefCell, and no unsafe code outside `shell/platform/`.

Key invariants:
- Description rebuilt every frame; Memory holds only interaction state
- Nothing reads a wall clock; elapsed time is injected
- Identity is the path through the tree (El::key overrides)
- unsafe confined to shell/platform/; the rest forbids it

See `rui.dx` for the complete module map and `README.md` for the narrative introduction.

## Recipe Infrastructure

Recipes document major features through three sequential phases:

### Phase 1: Foundation
Implement the core abstraction and establish critical invariants. Verification gate: compiles and core logic is correct.

### Phase 2: Enhancement
Add missing features and polish; expand invariants. Verification gate: feature completeness and all sub-components working.

### Phase 3: Integration
Integrate into platform-agnostic systems; verify cross-module consistency and platform transparency. Verification gate: end-to-end tests pass.

Each recipe includes:
- **Commit list**: Total commits grouped by phase with git IDs
- **Files touched**: What changed per phase (added, modified)
- **Verification gates**: Test commands and acceptance criteria
- **Cross-module concerns**: Where modules interact; friction points
- **Next-backend template**: Checklist for similar features

Documentation files created per recipe:
- STEP_XX_ANALYSIS.md — Phase-by-phase breakdown with commit IDs
- STEP_XX_VERIFICATION_GATES.md — Gate checklist and test commands
- STEP_XX_CROSS_MODULE_CONCERNS.md — Module interaction diagram
- STEP_XX_COORDINATE_CONTRACT.md (for backends) — Transformation formulas
- STEP_XX_EVENT_TRANSLATION.md (for I/O) — Input type mapping
- STEP_XX_SUMMARY.md — Sign-off document with architecture overview

---

## Recipe 1: Adding a WASM Backend

**Status**: Complete — Verified against 17 git commits; all three phases documented.

### Overview
WASM backend for browser environments. Allows identical UI code to run in a browser by implementing the `Backend` trait and using a shared `turn()` function instead of a platform-specific event loop.

### Commits and Phases
**Commits**: 17 total (77d4780 through 2df7f1c), grouped in three phases: clock abstraction, FrameDriver refactor, and WASM integration.

### Phase 1: Clock Abstraction
- **Commits**: 1 (77d4780)
- **Files**: src/shell/clock.rs (new); src/shell/mod.rs, src/app.rs, Cargo.toml modified
- **Problem**: `std::time::Instant::now()` panics on `wasm32-unknown-unknown` (no system clock in browser)
- **Solution**: `Moment` type abstracts time; Desktop uses `Instant`, WASM uses `performance.now()`
- **Verification**: `cargo test --lib` passes; `Moment::now()` works on both platforms

### Phase 2: FrameDriver Refactor
- **Commits**: 5 (531214f, 9afc9b1, b6a1b2c, 2ef3c2b, caa3066)
- **Files**: src/shell/mod.rs (extract loop body); tests/shell_stepping.rs (new)
- **Problem**: WASM cannot block on events; native `while window.is_open()` loop must split into reusable `turn()` function
- **Solution**: Extract loop body into `turn()` function called by both native loop and WASM requestAnimationFrame
- **Verification**: `cargo test --test shell_stepping && cargo build` pass

### Phase 3: WASM Integration
- **Commits**: 8+ (b116ac8 through 2df7f1c)
- **Files**: src/shell/mod.rs (wasm-specific run), src/shell/clock.rs (edge cases), src/wasm.rs (new), src/shell/platform/wasm.rs (new)
- **What**: Full WASM backend with canvas rendering, DOM event listeners, appearance detection, browser entry points
- **Verification**:
  - `cargo build --target wasm32-unknown-unknown -p rui --example counter` succeeds
  - `cargo test --lib` passes; memory state persists across frames
  - `wasm-pack test --headless --firefox` confirms events and state work
  - `examples/parity.html` pixel-matches native desktop (0 differing bytes, light & dark modes)

### Cross-Module Concerns
1. **Clock seam** (src/shell/clock.rs ↔ src/shell/mod.rs): `Surface::draw()` measures time via `Moment` API, hiding platform differences
2. **Backend trait** (src/shell/mod.rs lines 68–88): All backends implement 6 methods (open, pump, surface, appearance, present, is_open)
3. **Generic turn() function** (src/shell/mod.rs line 313+): Works for any backend; native and WASM both call it
4. **Event flow**: Native backends call `pump()` (blocking); WASM collects from DOM listeners; both return `Vec<Event>` to `turn()`
5. **State persistence** (src/memory.rs): `Memory` holds hover, focus, scroll, animation state; queried by both native and WASM
6. **Platform branching** (src/app.rs): `App::run()` calls `shell::run()` with two implementations gated by `#[cfg(target_arch)]`

### Template for Next Backend (e.g., Wayland, Game Engine)

When implementing a new backend, follow this checklist:

**Phase 1: Foundation**
- [ ] Add platform abstraction if needed (time, events, etc.)
- [ ] Implement `Backend` trait in src/shell/platform/{backend}.rs (all 6 methods)
- [ ] Verify compilation with `cargo build --target <target>`
- [ ] Verify no clippy warnings

**Phase 2: Enhancement**
- [ ] Extract platform-specific event loop differences (if any)
- [ ] Implement keyboard and input event translation
- [ ] Write integration tests in tests/{backend}_integration.rs

**Phase 3: Integration**
- [ ] Wire into src/shell/mod.rs platform selector with `#[cfg(...)]`
- [ ] Create platform-specific `run()` function
- [ ] Write parity tests in tests/{backend}_parity.rs
- [ ] Test cross-platform behavior consistency
- [ ] Document in CLAUDE.md

**Pattern**: Add platform/foo.rs implementing `Backend` trait, call `turn()` from any loop or callback. Everything above `Backend` is unchanged. Proof: WASM's src/shell/platform/wasm.rs implements all 6 methods; src/shell/mod.rs:412+ has wasm-specific run(); src/wasm.rs exports browser entry points; tests/shell_stepping.rs verifies turn() works independently (commits 77d4780+).

---

## Recipe 2: X11 Backend Implementation

**Status**: Complete — Verified against 10 git commits; all three phases documented.

### Overview
X11 backend for Linux systems using the X11 protocol. Implements the Backend trait with 6 core methods (open, pump, surface, appearance, present, is_open) and full event translation (11 X11 event types → rui Events with modifier support).

### Files
- **Core implementation**: src/shell/platform/x11.rs (1368 lines, Phase 1–3)
- **Feature gate**: `--features x11` (default, fallback to headless if unavailable)
- **Verification workflow**: See STEP_13_RECIPE_2_VERIFICATION.md for complete phase gates

### Phase 1: Foundation
- **Commits**: 1 (a67d578)
- **Files**: src/shell/platform/x11.rs added
- **What**: Backend trait implementation with basic window creation and event pump
- **Invariants**: Coordinate contract (device→logical: logical = device / scale_factor)
- **Verification**: `cargo build --target x86_64-unknown-linux-gnu && cargo test --lib`

### Phase 2: Enhancement
- **Commits**: 1 (c42c0f0)
- **Files**: src/shell/platform/x11.rs extended
- **What**: DPI detection, keyboard event translation, scale factor validation
- **Invariants**: Scale factor 1.0–4.0 range; key translation with shift/control/alt modifiers
- **Verification**: `cargo test --test x11_backend_phases -- dpi_scale keyboard_translation`

### Phase 3: Integration
- **Commits**: 8 (80e3003–84ade0e)
- **Files**: src/shell/platform/x11.rs finalized; src/app.rs, src/shell/mod.rs coordinated
- **What**: Frame loop integration, event translation in turn(), cross-module consistency
- **Invariants**: Platform transparency (app works identically at any DPI scale); parity with other backends
- **Verification**: `cargo test --test interaction && cargo test --test integration`

### Key Contracts

**Coordinate Transformation**:
```
logical_x = device_x / scale_factor
logical_y = device_y / scale_factor
```

**Event Translation**:
- X11 MotionNotify → rui Event::Pointer (moved flag)
- X11 ButtonPress → rui Event::Pointer (pressed flag)
- X11 ButtonRelease → rui Event::Pointer (released flag)
- X11 KeyPress/KeyRelease → rui Event::Key (with shift/control/alt bits)
- X11 ConfigureNotify → rui size/scale events

### Cross-Module Concerns
- **app.rs**: Backend trait boundary; frame loop calls pump() and present()
- **shell/mod.rs**: Platform selection and feature gating
- **memory.rs**: Focus and interaction state
- **input.rs**: Event → Input translation after X11 → Event conversion
- **paint.rs**: Pixel buffer from X11 framebuffer

### Verification Gates

**Phase 1: Compilation Verification**
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings
grep -n "fn open\|fn pump\|fn surface\|fn appearance\|fn present\|fn is_open" src/shell/platform/x11.rs
cargo fmt --check
cargo test --lib  # Verify no test regressions
```

**Phase 2: Integration Verification**
```bash
cargo build --release
cargo test --test "*x11_integration*"
grep "pub(crate) use backend::Window" src/shell/platform/mod.rs
cargo test --lib  # Full suite (375+ tests)
```

**Phase 3: Parity Verification**
```bash
cargo test --test "*x11_parity*"
cargo test coordinate_contract
cargo test event_mapping
cargo test modifiers
cargo test parity::cross_platform
```

For complete verification workflow details, see STEP_13_RECIPE_2_VERIFICATION.md.

### For Next Backends (Template Checklist)

When implementing a new platform backend, follow the Recipe 2 three-phase pattern:

**Phase 1: Foundation**
- [ ] Implement Backend trait (6 methods: open, pump, surface, appearance, present, is_open)
- [ ] Create platform-specific file: src/shell/platform/{backend}.rs
- [ ] Verify compilation with `cargo build --target <target>`
- [ ] Verify no clippy warnings and code is formatted

**Phase 2: Enhancement**
- [ ] Add DPI/scale factor detection and coordinate transformation
- [ ] Implement keyboard and input event translation
- [ ] Write integration tests in tests/{backend}_integration.rs
- [ ] Verify platform selector logic in shell/mod.rs

**Phase 3: Integration**
- [ ] Wire event translation into the frame loop (app.rs)
- [ ] Write parity tests in tests/{backend}_parity.rs
- [ ] Test cross-platform behavior consistency
- [ ] Verify platform transparency (no scale-factor visual bugs)
- [ ] Document in STEP_XX_RECIPE_2_VERIFICATION.md

---

## Build and Test

```bash
# Full test suite
cargo test -p rui                       # 474 tests, headless

# Run the gallery (visual verification)
cargo run -p rui --example gallery -- .

# Benchmark frame cost
cargo run -p rui --release --example cost

# Platform-specific tests
cargo test --test x11_backend_phases
cargo test --test integration
```

## Conventions

### State and Views

**State rule**: Rebuild the entire view every frame; keep Memory only for interaction state (focus, scroll, easing, IME composition). Stale-screen bugs are unwritable by design.

**Handler rule**: Handlers are ordinary `Fn(&mut S)` functions run after the frame. Multiple handlers can run in one frame; the order is depth-first in the element tree.

**Identity rule**: Identity is the path through the tree. Use `El::key(id)` to override identity so state follows reordered rows.

### Color and Theme

**Tone rule**: Every color reaches the screen through a role (primary, secondary, success, warning, danger, etc.), never raw RGB. Apps swap palettes via `with_palette()`.

**Contrast rule**: Contrast is CI, not review. Text/background ≥ 7; secondary UI ≥ 4.5; boundaries and focus ≥ 3. Asserted with `Color::contrast_ratio()` over every palette the theme accepts.

### Layout

**Space rule**: All spacing from one 4-based named scale. Raw f32 gaps in view code are a defect. Use `gap()`, `pad()` with `Metrics::spacing(level)`.

**Grow rule**: Spare room is dealt to growers in proportion and re-dealt when one hits its max. Short room is taken off content-sized children first, then growers down to their minimums. A grower with `grow_from_content` starts from its content size, not zero.

### Testing

**Harness rule**: Drive the real pipeline headless with `Harness::new(state, view).size(w, h)`, then `.click_text()`, `.type_text()`, `.key()`, `.frames(n)`, assert on `Probe` records or pixels. The synthetic test font makes every char exactly half the text size wide, so width assertions are exact.

**Accessibility rule**: Run `assert_accessible()` and `assert_tab_order()` on every screen. These audit the element tree and verify keyboard navigation parity.

---

## Contributor Workflow

### Adding a New Widget

1. Implement constructor in `widgets.rs` (builder pattern)
2. Set the widget's `Role` for automatic theme coloring
3. Write test cases in tests/ with exact pixel assertions
4. Ensure keyboard navigation and accessibility (Tab/Enter/Space/Arrow keys)
5. Verify contrast ratios over all palettes with `Color::contrast_ratio()`
6. Document in code comments (only non-obvious WHY)

### Adding a New Platform Backend

1. Create Phase 1 (Foundation) commit: Implement Backend trait with core window creation
2. Create Phase 2 (Enhancement) commit: Add DPI detection, keyboard support
3. Create Phase 3 (Integration) commits: Frame loop, event translation, cross-module verification
4. Write STEP_XX_ANALYSIS.md, STEP_XX_VERIFICATION_GATES.md, and STEP_XX_SUMMARY.md
5. Verify all three phases pass their acceptance criteria before merging

### Debugging a Regression

Use the Harness to replay the failing screen:
```rust
let mut h = Harness::new(state, view).size(w, h);
h.frames(10);  // Rebuild 10 frames
h.click_text("button");
assert_eq!(h.state().field, expected);
```

If a pixel assertion fails, render the frame with `h.render().save_png("debug.png")` and compare.

---

## Rationale

This library serves a singular philosophy: **above the window, everything is pure; below, only focused unsafe code at platform boundaries**. Every design decision flows from this: no retained widget tree (state rebuilt), no Rc/RefCell (identity is path-based), handlers are functions (not methods), and backends are thin (coordinate + event translation).

Recipes document the proof that this architecture works: major features have been delivered in three phases, tested at each step, and shipped to production. New contributors should follow the recipe template when adding backends, widgets, or interactions. The patterns in existing recipes are the measure of "done."

---

End of CLAUDE.md
