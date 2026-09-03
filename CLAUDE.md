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

## Quick Reference

Use this guide to navigate CLAUDE.md and the project:

| Question | Answer |
|----------|--------|
| **How do I build a UI?** | Read "View-State-Handler Pattern" (view = `Fn(&S) -> El<S>`, state is your data, handlers are `Fn(&mut S)`) |
| **How do I add a new widget?** | Copy "Checkbox Exemplar" or "Widget Exemplars" section; use primitives from `widgets.rs` |
| **How do I implement a platform backend?** | Follow "Recipe 2: X11 Backend Implementation" three-phase pattern (Foundation, Enhancement, Integration) |
| **How do I test my code?** | Use `Harness` (read "Testing with Harness") for headless pipeline tests with exact pixel assertions |
| **What are the design principles?** | See "Stellar UI Practices" for spacing, color, contrast, interaction, and motion rules |
| **What invariants must I preserve?** | Read "Key Invariants" section — 18 load-bearing constraints for future editors |
| **How do I debug a regression?** | See "Debugging a Regression" under Contributor Workflow |
| **What's been implemented and what's next?** | Check "Library Roadmap" for landed features and priority items |
| **Where's the authoritative source for everything?** | `rui.dx` is the working document with full module map, invariants, and implementation status |

## Module Structure

The rui library is organized into 19 core modules (see `rui.dx` for the complete technical map):

**Core Element System:**
- **lib.rs** — Crate documentation, design rationale, flat module re-exports, text! and code! macros
- **element.rs** — `El<S>`: Node kind, Style, children, handlers; every builder setter (w/h/grow/gap/pad/flow/scroll/wrap/on_click/on_key/...)
- **style.rs** — `Style`, `Tone` (color roles, never raw RGB), `Length` (Auto/Fixed/Fill/Fraction), alignment enums

**Layout and Measurement:**
- **layout.rs** — Two-pass layout (measure → place), stack/flow, distribute (multi-pass grow shares honoring max), shrink (content-sized give first, growers to minimums second)
- **geom.rs** — Point/Size/Rect/Insets in logical units; only canvas.rs multiplies by display scale to reach device pixels

**Rendering and Graphics:**
- **color.rs** — `Color` (8-bit sRGB), blend_over (straight alpha, not gamma-corrected to match OS compositors), contrast ratio validation
- **canvas.rs** — CPU rasterizer: every shape from signed distance fields, 0xAARRGGBB words, vertical-only gradients, Bgra blit primitive
- **sdf.rs** — Composable signed-distance-field shape algebra: Shape as distance function, Fill/Stroke/Glow/Bevel all derived from one field
- **image.rs** — From-scratch PNG writer (stored deflate blocks, no compressor) so a frame can be saved for inspection
- **paint.rs** — `Painter`: one tree walk draws AND resolves input; handlers deferred to frame end; ease/phase animation values

**Typography:**
- **text.rs** + **font/** — From-scratch TrueType engine (SFNT, glyf, analytic scanline raster, kern+GPOS); one advance path so text can never measure at one width and draw at another; wrap breaks at spaces and hyphens; grapheme.rs implements UAX #29

**Theming:**
- **theme.rs** — `Theme` = Palette + Metrics + CornerStyle + type scale; apps swap palette/corners via with_palette/with_corners
- **widgets.rs** — Constructor functions (button, col, row, field, tabs, segmented, meter, title/heading/text/caption/micro/code...); each sets a Role

**Interaction and Input:**
- **memory.rs** — Interaction state that outlives a frame: focus, scroll, easing, IME composition; time is injected, never read from clock
- **input.rs** — `Event` → `Input` translation; pressed vs held; pointer_moved (cleared each frame); IME composition; Key (meaning) vs KeyCode (position); shortcut(); Drag and Pointing share fraction_within
- **accessibility.rs** — The element tree IS the accessibility tree; audit() finds violations; `El::takes_focus` unifies focusable check

**Application Loop and Backends:**
- **app.rs** + **shell/** — App loop, `Backend` trait (12 methods: window/input/clipboard/accessibility/composition), platform backends (macOS/Windows/X11/WASM, confined to unsafe)
- **reload.rs** — (feature="reload" only, compiled out of release) Running window notices executable changed, saves state, restarts in place

**Testing:**
- **testing/** — `Harness`: real pipeline headless, no window; synthetic font (char = size/2) so layout tests assert exact numbers

See `rui.dx` for complete module responsibilities and 18 load-bearing invariants that must not be broken.

## Key Invariants

These invariants are load-bearing—future editors must not break them:

1. **Description rebuilt every frame** — No retained widget tree; Memory holds only interaction state (focus, scroll, easing, IME). Stale-screen bugs are unwritable by design.
2. **No wall-clock reads** — Nothing calls `std::time::Instant::now()`; elapsed time is injected via `Memory::begin_frame()`. Tests can step time exactly.
3. **Identity is path-based** — Elements are identified by their position in the tree. `El::key()` overrides to make state follow reordered rows.
4. **Single dispatch path** — An accessibility activation runs the same handler a mouse click would; one action, not two.
5. **Coordinate transformation** — Display scale is only multiplied in canvas.rs when reaching device pixels; all layout in logical units.
6. **Layout stability** — Spare room dealt to growers in proportion and re-dealt when one hits max; short room taken from content-sized children first, then growers down to minimums.
7. **Text measure-draw parity** — measure_stack re-measures at real width; wrapped text in a growing child measures at its actual width. One advance path ensures measure and draw use identical dimensions.
8. **Shape algebra from SDF** — Every shape is a signed distance field; Round and Cut corners cost the same; gradients are vertical-only (one color per row).
9. **Blending contracts** — Deliberately not gamma-corrected (matches OS compositors); buffer always opaque; alpha is replaced in blit_bgra, never honoured.
10. **No hostile fonts** — All font-file reads are bounds-checked Options; a malformed font fails to load, never panics.
11. **Identical frames never presented** — Loop runs 8ms only while animating, else idle_timeout; a Redraw handle may shorten wait but never idle timeout; a turn that nothing asked for draws nothing.
12. **Stroke completeness** — Every key pressed is reported coming up; strokes is the single source; keys()/released_keys() are filters, not independent. A press without release is unwritable.
13. **Key identity** — `Key` is the layout's answer (for widgets); `KeyCode` is platform position (for forwarding). A stroke with neither is dropped.
14. **Bitmap precision** — blit_bgra copies, never scales or resamples; alpha replaced, not honoured. Bgra::new is the one place frame sizes are checked against buffer.
15. **Focus consistency** — `El::takes_focus` is `focusable && !disabled`, read nowhere else. Focus walk, focus ring, and Harness probes all ask it, so audit and walk cannot disagree.
16. **Pointer motion semantics** — on_pointer_move reports movement, never presence; fires only when `Input::pointer_moved` is true for that frame. Resting hand and animating window run no handler.
17. **Graceful shutdown** — Application ends by loop noticing no visible window; `App::run` returns and application destructors run. macOS `terminate:` closes windows and cancels, never lets AppKit tear down under the stack.
18. **unsafe confinement** — unsafe code confined to shell/platform/; rest of crate forbids it. Platform boundaries are the only place.

## Key Architectural Patterns

### View-State-Handler Pattern

**View is a pure function of state:**
```rust
fn view(app: &App) -> El<App> {
    col((
        text("Click to increment:"),
        button("Increment", |app: &mut App| app.count += 1),
    ))
}
```

**State describes your data:**
```rust
struct App {
    count: usize,
}
```

**Handlers update state without closures:**
```rust
|app: &mut App| app.count += 1  // receives mutable state as argument
```

### Backend Trait Pattern

Platform-specific code implements a core trait with 12 methods (simplified view of essential methods shown below; see `src/shell/mod.rs` line 183 for complete definition):

```rust
pub trait Backend: Sized {
    fn open(options: &WindowOptions) -> Result<Self, Error>;
    fn pump(&mut self, timeout: Duration, events: &mut Vec<Event>, redraw: &mut dyn FnMut(&Self)) -> Result<(), Error>;
    fn surface(&self) -> (u32, u32, f32);  // width, height, scale_factor
    fn appearance(&self) -> Appearance;    // light or dark
    fn present(&self, canvas: &Canvas) -> Result<(), Error>;
    fn is_open(&self) -> bool;
    fn is_fullscreen(&self) -> bool;
    fn set_fullscreen(&self, filling: bool) -> Result<(), Error>;
    fn clipboard_text(&self) -> Result<Option<String>, Error>;
    fn set_clipboard_text(&self, text: &str) -> Result<(), Error>;
    fn set_composition_area(&self, area: Option<Rect>) -> Result<(), Error>;
    fn update_accessibility(&self, update: &AccessUpdate) -> Result<(), Error>;
}
```

All platform-agnostic code (layout, paint, handlers) sits above this line. Each backend (macOS, Windows, X11, WASM) implements these methods; everything else is shared. Core methods are open/pump/surface/appearance/present; additional methods support fullscreen, clipboard, composition input, and accessibility.

### Testing with Harness

Drive the real pipeline headless, no window:
```rust
let mut h = Harness::new(App { count: 0 }, view);
h.click_text("Increment");
assert_eq!(h.state().count, 1);
```

The synthetic test font makes every character exactly half the text size wide, so width assertions are exact and deterministic.

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

**Purpose**: Exemplar pattern for cross-platform backend implementation. The three-phase structure applies to any new backend (Wayland, DirectX, game engine, etc.).

### Overview
WASM backend for browser environments. Allows identical UI code to run in a browser by implementing the `Backend` trait and using a shared frame-loop function instead of a platform-specific event pump.

### Extracted Documentation

Complete documentation for Recipe 1 WASM backend pattern:

- **STEP_4_RECIPE_1_ANALYSIS.md** — Three-phase pattern breakdown with line counts (748/1220/1321), timeline, and detailed implementation checklist per phase
- **STEP_4_RECIPE_1_VERIFICATION_GATES.md** — Phase-by-phase acceptance criteria, test commands, and verification checklist; gates at each phase prevent regressions
- **STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md** — 7 friction points (time injection, Backend trait, generic draw, event flow, state persistence, platform branching, accessibility) with resolution patterns and module interaction map
- **STEP_4_RECIPE_1_COORDINATE_CONTRACT.md** — Browser coordinate transformation (client → canvas → logical), scale factor handling, implementation per phase, common pitfalls
- **STEP_4_RECIPE_1_EVENT_TRANSLATION.md** — 6 DOM event types (mouse, touch, keyboard, wheel, composition, focus/resize) with mapping to rui Events, implementation per phase, testing strategy
- **STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md** — Validation that template claims hold for Recipe 2 (X11); proves pattern is replicable
- **STEP_4_RECIPE_1_SUMMARY.md** — Quick reference for implementers: architecture overview, when to use which document, how to run verification tests

### How to Implement WASM Backend (Using Recipe 1 Documentation)

**For new implementers:**

1. **Start here**: STEP_4_RECIPE_1_SUMMARY.md — Understand the architecture and which documents to read in what order
2. **Create Phase 1 plan**: Read STEP_4_RECIPE_1_ANALYSIS.md (Phase 1 section) + STEP_4_RECIPE_1_VERIFICATION_GATES.md (Phase 1 gates)
3. **Understand coordinate contract**: Read STEP_4_RECIPE_1_COORDINATE_CONTRACT.md before writing transform code
4. **Map events**: Read STEP_4_RECIPE_1_EVENT_TRANSLATION.md to understand DOM → rui Event mapping
5. **Check cross-module interactions**: Read STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md to identify friction points and how other backends solved them
6. **Run verification**: Use gates from STEP_4_RECIPE_1_VERIFICATION_GATES.md at end of each phase

**Proof of pattern validity**: STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md verifies all template claims against Recipe 2 (X11 backend, fully implemented). Same pattern holds for any backend.

### The Three-Phase Pattern

**Phase 1: Foundation** — Implement the Backend trait
- **Goal**: Platform-specific window + event pump working
- **Scope**: Create src/shell/platform/{backend}.rs implementing all 12 Backend trait methods
- **Verification**: `cargo build --target <target>` succeeds
- **Pattern example**: For reference, see src/shell/platform/x11.rs Phase 1 (fundamental trait implementation)

**Phase 2: Enhancement** — Add platform-specific features
- **Goal**: Full feature parity with other backends
- **Scope**: DPI detection, event translation, keyboard support, clipboard
- **Verification**: Platform-specific test suite passes; `cargo test --lib` succeeds
- **Pattern example**: For reference, see src/shell/platform/x11.rs Phase 2 (platform-specific enhancements)

**Phase 3: Integration** — Wire into shared systems
- **Goal**: Seamless cross-platform operation
- **Scope**: Event translation, coordinate transformation, feature gates in src/shell/mod.rs
- **Verification**: Parity tests pass; visual output identical to other backends; `cargo build --all-features` succeeds
- **Example file**: src/shell/mod.rs (platform selector logic)

### Cross-Module Concerns
1. **Time injection** (src/shell/mod.rs): Platform loop injects elapsed time; `Memory::begin_frame()` receives it, never reads wall clock
2. **Backend trait** (src/shell/mod.rs line 183+): All backends implement the trait with 12 core methods (window, input, clipboard, accessibility)
3. **Generic draw() function** (src/shell/mod.rs line 305+): Works for any backend; native and WASM both call it
4. **Event flow**: Native backends call `pump()` (blocking); WASM collects from DOM listeners; both return `Vec<Event>` to `draw()`
5. **State persistence** (src/memory.rs): `Memory` holds hover, focus, scroll, animation state; queried by both native and WASM
6. **Platform branching** (src/app.rs): `App::run()` calls `shell::run()` with two implementations gated by `#[cfg(target_arch)]`

### Template for Next Backend (e.g., Wayland, Game Engine)

When implementing a new backend, follow this checklist:

**Phase 1: Foundation**
- [ ] Add platform abstraction if needed (time, events, etc.)
- [ ] Implement `Backend` trait in src/shell/platform/{backend}.rs (all 12 methods)
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

**Pattern**: Add platform/foo.rs implementing `Backend` trait (12 methods), call a shared event loop from platform-specific entry point. Everything above the trait boundary is platform-agnostic. Proof: See src/shell/platform/ for macOS/Windows/X11 implementations; each implements all 12 trait methods identically; src/shell/mod.rs handles platform selection via `#[cfg(...)]`; src/app.rs calls Backend generically.

---

## Recipe 2: X11 Backend Implementation

**Purpose**: Detailed exemplar of the three-phase pattern applied to a real platform backend. Reference implementation for Linux X11 systems.

### Overview
X11 backend for Linux systems using the X11 protocol. Implements the Backend trait (12 methods) and full event translation (pointer, keyboard, window lifecycle, DPI changes).

### Example Implementation
- **File**: src/shell/platform/x11.rs (1368 lines, all three phases)
- **Feature gate**: `--features x11` (included by default, optional for embedded builds)
- **Coordinate contract**: device pixels → logical (logical = device / scale_factor)
- **Event translation**: X11 event types → rui Event with modifier masks

### Extracted Documentation

Complete documentation for Recipe 2 X11 backend pattern:

- **STEP_5_RECIPE_2_ANALYSIS.md** — Three-phase pattern breakdown with timeline and implementation
- **STEP_5_RECIPE_2_VERIFICATION_GATES.md** — Acceptance criteria, test commands, verification checklist
- **STEP_5_RECIPE_2_CROSS_MODULE_CONCERNS.md** — Module interactions (Backend, events, coordinates, state)
- **STEP_5_RECIPE_2_COORDINATE_CONTRACT.md** — Device-to-logical transformation, scale factors, pitfalls
- **STEP_5_RECIPE_2_EVENT_TRANSLATION.md** — X11 event types mapped to rui Events (Motion, Button, Key)
- **STEP_5_RECIPE_2_TEMPLATE_VALIDATION.md** — Proves pattern replicability for Wayland, game engines
- **STEP_5_RECIPE_2_SUMMARY.md** — Quick reference: architecture overview and verification test navigation

### Commit list

**Phase 1: Foundation**
- Commit: `a67d578eea41560c26fd7a6548c0d089223f3d70`
- Message: "Give the interface library a foundation you can build controls on"
- Lines: 748 initial implementation

**Phase 2: Enhancement**
- Commit: `c42c0f05b3d75976665377a16257c36c472debc1`
- Message: "Bring the library up to the selfhost workspace's current state: a full vector canvas (paths, strokes, gradients, SDF text effects), geometry primitives, image decoding and scaling, signed-distance-field rendering, accessibility tree, font kerning, interaction tests, the reload feature, and the icon example"
- Lines: 1220 (full Backend trait + event translation + DPI/keyboard/clipboard)

**Phase 3: Integration**
- Commit: `80e3003563c26952e4d63c52d8eb8f5052cb463c`
- Message: "The four primitives a remote-desktop viewport needs, and the practices document"
- Lines: 1321 (Canvas::blit_bgra, key_up, on_raw_key, on_pointer_move, takes_focus consistency)

**Polish**
- Commit: `991167a3898d643199a6e0b9dfa461be31cae264`
- Message: "Recipe 2: Implement star_rating widget exemplar with test"
- Lines: 1368 (documentation refinements + star_rating widget exemplar)

**Line counts verified** by tests/claude_md_recipe2.rs via `git show --stat` for each commit SHA. Regression test ensures documentation remains grounded in real git history.

### Phase 1: Foundation
- **Scope**: Basic window creation, event pump loop, display connection
- **Key methods**: `Backend::open()`, `Backend::pump()`, `Backend::surface()`, `Backend::present()`
- **Coordinate handling**: Establish scale factor and display metrics
- **Files modified**: src/shell/platform/x11.rs (window setup), src/shell/mod.rs (platform selector)
- **Verification**: 
  ```bash
  cargo build --target x86_64-unknown-linux-gnu
  cargo test --lib  # Core library tests unchanged
  ```

### Phase 2: Enhancement
- **Scope**: DPI scaling, keyboard event translation, modifier key handling, accessibility setup
- **Event types**: MotionNotify → pointer moved, ButtonPress/Release → pointer pressed/released, KeyPress/Release → key events
- **Keyboard translation**: Map X11 KeyCode → rui Key with shift/control/alt modifiers
- **Scale factor**: Detect DPI and validate range (1.0–4.0)
- **Files modified**: src/shell/platform/x11.rs (event handling), src/input.rs (Event → Input translation)
- **Verification**:
  ```bash
  cargo test --test x11_integration
  cargo test --lib  # Verify keyboard and scale_factor logic
  ```

### Phase 3: Integration
- **Scope**: Frame loop wiring, cross-module coordination, parity validation
- **Files modified**: src/shell/mod.rs (platform selector, event dispatch), src/app.rs (backend trait boundary), src/accessibility.rs (X11 node objects)
- **Invariants**: 
  - Platform transparency (identical behavior at any DPI)
  - Parity with other backends (macOS/Windows)
  - Single dispatch path for all input sources
- **Verification**:
  ```bash
  cargo test --test interaction  # Pointer and keyboard handling
  cargo test --test x11_parity  # Cross-platform consistency
  ```

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
grep -n "fn open\|fn pump\|fn surface\|fn appearance\|fn present\|fn is_open\|fn is_fullscreen\|fn set_fullscreen\|fn clipboard_text\|fn set_clipboard_text\|fn set_composition_area\|fn update_accessibility" src/shell/platform/x11.rs
cargo fmt --check
cargo test --lib  # Verify no test regressions
```

**Phase 2: Integration Verification**
```bash
cargo build --release
cargo test --test x11_integration
cargo test --lib  # Full suite (379 tests)
```

**Phase 3: Parity Verification**
```bash
cargo test --test x11_parity
```

---

## Recipe 3: Checkbox Control

**Status**: Complete — Verified as replicable pattern for custom widget implementation.

### Overview
Checkbox demonstrates the minimal interactive control: a single boolean that toggles on click. It proves that even the smallest custom widget follows the state-view-handler pattern without requiring any special framework support.

### Extracted Documentation

Complete documentation for Recipe 3 checkbox widget pattern:

- **STEP_5_RECIPE_3_ANALYSIS.md** — Four-phase breakdown with implementation details, scope, state shape, and cross-module interactions for each phase
- **STEP_5_RECIPE_3_VERIFICATION_GATES.md** — Phase-by-phase acceptance criteria, test commands, and debugging checklist; gates at each phase prevent regressions
- **STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md** — Four key interactions (identity & persistence, state flow, handlers, theme colors) with module diagrams, common pitfalls, and verification examples
- **STEP_5_RECIPE_3_TEMPLATE_VALIDATION.md** — Validation that pattern holds for segmented, button, meter, slider; proves checkbox exemplar is replicable for any custom widget
- **STEP_5_RECIPE_3_SUMMARY.md** — Quick reference for implementers: when to use which document, testing checklist, common mistakes, and debugging commands

### How to Implement Custom Widgets (Using Recipe 3 Documentation)

**For new implementers:**

1. **Start here**: STEP_5_RECIPE_3_SUMMARY.md — Understand the architecture and which documents to read in what order
2. **Create Phase 1 plan**: Read STEP_5_RECIPE_3_ANALYSIS.md (Phase 1 section) + STEP_5_RECIPE_3_VERIFICATION_GATES.md (Phase 1 gates)
3. **Understand state flow**: Read STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md section "State Flow" before writing draw() closures
4. **Check module interactions**: Read STEP_5_RECIPE_3_CROSS_MODULE_CONCERNS.md to identify friction points and how other widgets solved them
5. **Run verification**: Use gates from STEP_5_RECIPE_3_VERIFICATION_GATES.md at end of each phase

**Proof of pattern validity**: Checkbox is 29 lines of code with zero framework magic. Same pattern applies to button, slider, radio, toggle, custom charts—any interactive element.

### Phase 1: State Definition
- **Problem**: How do we know if a control needs any special framework support to work?
- **Solution**: Define the simplest possible state (a single bool) and build from there
- **State shape**:
```rust
struct App {
    checked: bool,
}
```
- **Files**: src/widgets.rs, tests/recipes.rs
- **Verification**: `cargo test --test recipes -- a_checkbox_changes_state_on_click` passes

### Phase 2: Element Tree Construction
- **Problem**: How do state changes flow through the view function into visual appearance?
- **Solution**: Build checkbox from primitives; state parameter determines conditional styling
- **Implementation** (examples/controls.rs lines 57–85):
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
- **Key insight**: `checked` parameter flows as an upvalue into the `draw()` closure; conditional styling proves state determines appearance
- **Verification**: `cargo run -p rui --example controls` shows working checkbox

### Phase 3: Enhancement (Styling & Visual Polish)
- **Problem**: Does the checkbox look correct across light/dark modes and match the design system?
- **Solution**: Add platform-appropriate styling (rounded corners, focus ring, disabled state, hover)
- **Files**: src/widgets.rs (enhanced styling), examples/controls.rs (showcase), tests/recipes.rs (visual tests)
- **Additions**:
  - `.fill()` customization to allow theme colors
  - Focus ring when keyboard-focused
  - Disabled state styling (0.38 content alpha)
  - Hover highlight
- **Verification**: 
  ```bash
  cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover
  cargo run -p rui --example controls  # Visual inspection
  ```

### Phase 4: Integration & Persistence
- **Problem**: Can multiple checkbox instances coexist with independent state?
- **Solution**: Verify state persists across frames and multiple instances manage their own identity
- **Files**: tests/recipes.rs (integration tests), src/testing/mod.rs (Harness implementation)
- **Tests**:
  ```bash
  cargo test --test recipes -- checkbox_preserves_state_across_frames
  cargo test --test recipes -- checkbox_works_with_multiple_instances
  cargo test --lib memory
  ```
- **Key invariant**: Identity is path-based; reordered checkboxes preserve state via `.key()`
- **Verification**: All integration tests pass; memory module handles checkbox focus/state correctly

### Cross-Module Concerns
1. **Identity & Persistence** (element.rs ↔ memory.rs): Element path determines identity; focus and interaction state live in Memory
2. **State Flow** (widgets.rs ↔ paint.rs): State parameter passed to checkbox flows as upvalue into draw closure
3. **Handlers** (input.rs ↔ paint.rs): Click events call handlers after frame drawn; handlers receive `&mut S` directly
4. **Appearance** (theme.rs ↔ widgets.rs): Tone roles (Accent, Sunken, Border) resolve against Theme for light/dark mode

### Template for Building Custom Widgets

When implementing a new interactive control, follow this pattern:

**Phase 1: State**
- [ ] Define minimal state shape (struct with one or two fields)
- [ ] Write state-only test (no UI yet)

**Phase 2: View**
- [ ] Build element tree from primitives (draw, row, col, on_click, on_drag)
- [ ] Pass state as parameter, not closure
- [ ] Use conditionals in primitives (draw closures, text, styling) to shape appearance
- [ ] Write rendering test with Harness

**Phase 3: Polish**
- [ ] Add focus ring, hover effects, disabled state
- [ ] Test across light/dark modes
- [ ] Ensure contrast ≥ 4.5 secondary, ≥ 7 text

**Phase 4: Integration**
- [ ] Test multiple instances with independent state
- [ ] Verify state persists across 10+ frames
- [ ] Use `.key()` for reordered lists

**Pattern proof**: Checkbox is 29 lines of code (examples/controls.rs lines 57–85). Zero framework support required. No special widget class, no retained tree, no lifecycle. Just state → view function → handlers. This pattern works for button, segmented, slider, radio, custom charts—any interactive element.

### For Next Backends (Template Checklist)

When implementing a new platform backend, follow the Recipe 2 three-phase pattern:

**Phase 1: Foundation**
- [ ] Implement Backend trait (12 methods across window, input, clipboard, accessibility)
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

## Widget Exemplars

### Segmented Control Exemplar

The `segmented` widget demonstrates building an interactive choice selector from primitives. It is self-contained (32 lines, src/widgets.rs lines 367–398) and shows the state-view-handler pattern clearly.

**Pattern:**
```
State:   struct App { selected: usize }
View:    fn view(app: &App) -> El<App> { segmented(&choices, app.selected, handler) }
Handler: |app: &mut App, index| { app.selected = index; }
```

**Key insight:** The handler receives `&mut S`, not a closure capturing a reference. This eliminates `Rc<RefCell<>>` and interior mutability.

**Try it:**
```bash
cargo run -p rui --example gallery -- .
```

Then find the segmented control in the gallery. Click buttons to change selection; state persists across frames.

**Modification checklist:**
- [ ] Change `["Small", "Medium", "Large"]` to your own choices
- [ ] Replace the label with your description
- [ ] Extend or shrink the choices array
- [ ] Change colors with `.fill()` method
- [ ] Copy from `src/widgets.rs` line 367–398 to customize appearance

**Verification:**
- `cargo run -p rui --example gallery -- .` shows segmented control in gallery
- `cargo test --test recipes` runs and includes segmented examples
- Pattern can be copied directly to build other choice controls

### Checkbox Exemplar

The `checkbox` widget demonstrates a binary toggle control. Unlike segmented (one choice among many), checkbox toggles a single boolean value.

**Pattern:**
```
State:   struct App { notify: bool }
View:    fn view(app: &App) -> El<App> { checkbox("Enable notifications", app.notify, handler) }
Handler: |app: &mut App| { app.notify = !app.notify }
```

**Key insight:** Toggle controls flip a boolean. The handler is simple: receive `&mut S`, invert the field, done.

**Try it:**
```bash
cargo run -p rui --example controls
```

Click the checkbox to toggle ON/OFF; state persists.

**Modification checklist:**
- [ ] Change `"notifications"` to your own label
- [ ] Replace `app.notify` with any boolean field in your state
- [ ] Add more checkboxes by calling `checkbox()` multiple times
- [ ] Change colors or size in the draw closure
- [ ] Copy from `examples/controls.rs` line 57–85 to customize rendering

**Verification:**
- `cargo run -p rui --example controls` shows working checkbox
- `cargo test --test recipes` includes checkbox tests
- Pattern works for any binary preference or flag

### Meter Widget Exemplar

The `meter` widget demonstrates a passive/read-only control. Unlike segmented or checkbox (which respond to input), meter simply displays a value as a progress bar. No handler needed.

**Pattern:**
```
State:   struct App { progress: f32 }
View:    fn view(app: &App) -> El<App> { meter(app.progress, Tone::Accent) }
Handler: (none — display-only)
```

**Key insight:** Passive widgets read state and display it. No user interaction, no handlers.

**Try it:**
```bash
cargo run -p rui --example gallery -- .
```

Then find the meter control in the gallery. Watch the meter animate from 0% to 100%.

**Modification checklist:**
- [ ] Change `Tone::Accent` to `Tone::Success`, `Tone::Warning`, etc.
- [ ] Customize bar width/height by copying `src/widgets.rs` line 284–309
- [ ] Animate `app.progress` over time in your event loop
- [ ] Copy the pattern for other display-only visualizations (volume, status lights, gauges)

**Verification:**
- `cargo run -p rui --example gallery -- .` shows meter control in gallery
- `cargo test --test recipes` runs and includes meter-related tests
- Pattern works for any read-only visualization

### Building Custom Controls

Copy an exemplar and modify freely. All widgets are built from primitives:

```rust
widgets::draw(Size::new(160.0, 18.0), move |painter, rect| {
    let (filled, _) = rect.split_left(rect.w * value);
    painter.fill(rect, Radius::Pill, Tone::Sunken);
    painter.fill(filled, Radius::Pill, Tone::Accent);
})
.on_drag(|app: &mut App, drag| app.volume = drag.fraction().x)
.on_key(|app: &mut App, key, _| app.nudge(key))
```

**Next steps:**
1. Copy `examples/gallery.rs` to `examples/my_control.rs`
2. Modify state struct to fit your domain
3. Update view function to use your state
4. Run `cargo run -p rui --example my_control`
5. Copy test from `tests/recipes.rs` and verify behavior
6. Run `cargo test my_control_changes_state_when_clicked`

Pattern: State → view function → handler closure. That is the entire pattern. Build custom controls from primitives in `widgets.rs` (row, col, draw, button, field, etc.); they compose to form any interface.

---

## Build and Test

```bash
# Full test suite
cargo test -p rui                       # 379 tests, headless

# Run the gallery (visual verification)
cargo run -p rui --example gallery -- .

# Benchmark frame cost
cargo run -p rui --release --example cost

# Platform-specific tests
cargo test --test x11_integration
cargo test --test x11_parity
cargo test --test interaction
```

## Stellar UI Practices

Distilled from SwiftUI/HIG, Material, GPUI, Linear, Zed, and rui's peers (egui, iced, Xilem, Slint). These are behaviors the library enforces, not suggestions:

**Spacing, Scale, and Typography:**
- All spacing from one 4-based named scale; raw f32 gaps are a defect. Proximity groups before boxes or lines.
- One type ramp with few sizes; roles, not numbers. Hierarchy from weight and ink (muted vs primary), not new sizes.
- Mono only for machine output (paths, logs, fingerprints, addresses); placeholders are not machine output.
- Tabular digits for anything that updates in place — jittering readouts read as broken instrumentation.

**Color and Contrast:**
- Every color reaches the screen through a role; status ink and tint travel inseparably.
- Amber/red only with cause; motion only with cause. Budget ≤2 live animation loops, asserted mechanically.
- One accent; chrome stays neutral. Never alias status onto brand channel.
- Contrast is CI, not review: text/bg ≥ 7, secondary ≥ 4.5, UI boundaries and focus ≥ 3. Asserted over every palette the theme accepts via `Color::contrast_ratio()`.

**Elevation and Visual Depth:**
- Dark elevation is lightness, not shadow; shadow only under genuinely floating things, fainter in dark.
- Hairlines are 1 physical pixel on grid, rationed: separate first by spacing and value. Border only where surfaces actually meet.

**Interaction and Motion:**
- Instant acknowledge, then animate: pressed lands the same frame as mouse-down; only the consequence animates; release eases ~100ms.
- Motion bands: micro 50–200ms, transitions 120–300ms, nothing past 600ms; exit ≈ 2/3 of enter; decelerate in, accelerate out; linear only for continuous progress.
- Springs (bounce 0) for anything interruptible; a redirected animation that jumps is a bug.
- Hover is one value step up; pressed is sunken; selection is persistent and distinct from hover.
- Disabled = 0.38 content alpha, never a new grey.
- Focus ring is keyboard-only, offset, ≥3:1, drawn by library — never the same mark as selection. Focus and selection are different facts.

**Keyboard and Accessibility:**
- Everything mouse-doable is keyboard-doable, in declaration order, proven by the Harness. `assert_tab_order()` verifies Tab reaches exactly what it should.
- Focus walk, focus ring, and accessibility audit all ask `El::takes_focus` (`focusable && !disabled`), so they cannot disagree.

**Loading and Empty States:**
- Nothing under 300ms shows a loading state; keep stale data visible and marked stale.
- Never draw a fetch-in-progress as an empty state — the UI must not claim what it hasn't learned.
- Empty states are furnished, not blank: one muted line, one action, Idle tone.

**Interaction Safety:**
- Destructive actions arm before they fire; the failure hue appears only at the confirmation step.
- Instruments hold their geometry: rows reserve their slots, readouts change value not position, the primary control never moves between states.

**Layout and Performance:**
- True two-pass layout, never last-frame size caching — frame 1 renders at final size.
- Idle windows draw zero frames; ease/phase/after are the only wakeup sources.
- Deterministic clock, golden pixels: all motion reads injected time; snapshot widget × state × theme and diff exact pixels.

**Visual Refinement:**
- Optical over geometric: center text on cap-height, align icons to optical center.

## Library Roadmap

The library is under active development. Key items landed and in progress:

### Landed (2026-09-02)

**R2 Motion Kit — Complete physics-based animation system:**
- `Spring` — Physics solver with stiffness (0.1–2.0) and damping (0.0–1.0); velocity inheritance for momentum transfer
- `EnterExit` — Lifecycle animations (Fade, Slide with 4 directions, Scale) for element appearance/disappearance
- `Easing` — 6 standard curves (Linear, EaseIn, EaseOut, EaseInOut, EaseInCirc, EaseOutCirc)
- `Metrics.motion` — Accessibility flag for prefers-reduced-motion; collapses all animations when disabled
- `Memory::after_animation()` — Deferred action callbacks for post-animation cleanup
- Animation budget assertion — Enforces 2-live-loop maximum per frame for predictable performance

**Previous landed (2026-08-10):**
- `Canvas::blit_bgra` + `Bgra` — 1:1 device pixel blitting with clip, negative origin, stride padding, crops, Retina support
- `El::on_key_up` + `Input::released_keys` — Every key pressed is always reported coming up; strokes is the single source
- `El::on_raw_key` + `KeyCode`/`KeyStroke` — Platform key positions flow through all backends unchanged
- `App::redraw()` → `Redraw` — Thread-safe notification for frames arriving on other threads
- `Color::contrast_ratio()` + `Palette::assert_legible()` — WCAG compliance for all palettes (text ≥7, secondary ≥4.5)
- `Memory::FocusSource` — Focus ring renders only on keyboard focus; fields keep source always
- `El::on_pointer_move` + `Pointing { at, rect }` — Pointer movement tracking (fires on motion, not presence)
- `El::takes_focus` consistency — One place reads `focusable && !disabled`; focus walk, ring, and audit all agree
- Caret blinking in text fields (1.1s phase, solid while typing)
- Graceful application shutdown (macOS `terminate:` closes windows and lets destructors run)

### In Roadmap (priority order)

- **R1** Theme roles end-to-end: TextRole/Space/Height enums; delete duplicate size constants (COMPLETE)
- **R3** Pressed style struct and disabled = 0.38 alpha convention (COMPLETE)
- **R4** Pixel-grid crispness: hairline snap, glyph raster cache, gamma-boost LUT (COMPLETE)
- **R5** Elevation ramp: surface (0%), raised (3%), floating (6%), modal (9%) lightness boosts (COMPLETE)
- **R6** Overlay semantics: Dropdown (z=1), Popover (z=2), Modal (z=3) with placement (COMPLETE)
- **R7** Two-layer shadows: primary (soft) + secondary (sharp) for visual depth (COMPLETE)
- **R9** Scrollbar as interactive control (COMPLETE)
- **R10** Loading/empty state recipes: furnished UI with icon, message, action (COMPLETE)
- **R12** Golden-image regression net: pixel-perfect visual testing (COMPLETE)
- **R13** Palette::derive: dynamic theme generation from accent color (COMPLETE)

For complete roadmap details, see `rui.dx` under "Library roadmap toward those practices".

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

### Motion and Animation

**Spring physics**: Springs are interruptible; a spring retargeted mid-animation picks up momentum from the previous velocity (`Velocity` type) and continues smoothly without jumping.

**Easing curves**: Choose curves per interaction: EaseIn for entering, EaseOut for exiting, EaseInOut for transitions that need both. Linear only for continuous progress (sliders, meters).

**Enter/exit animations**: Use `EnterExit::Fade` for opacity (works everywhere), `Slide` for spatial entrance (paired direction + exit), `Scale` for growth (content appearing/disappearing). Each type has duration in Metrics.

**Motion accessibility**: Always check `Metrics::is_motion_enabled()` (respects prefers-reduced-motion). When disabled, animations complete instantly and `on_enter` / `on_exit` handlers still fire; apps must handle both instant and animated states.

**Deferred actions**: Use `Memory::after_animation(delay, action)` for post-animation cleanup: dismissing modals, refocusing, state changes that must happen after visual transition completes, never during.

**Animation budget**: Maximum 2 animations per frame are allowed (enforced by `assert_animation_budget()`). Exceeding this indicates a UI problem: simultaneous transitions on too many elements. Design for sequential transitions instead (one starts after another).

---

## Common Patterns and Edge Cases

### Key Path Identity

Elements are identified by their position in the tree. When lists reorder, state follows the old position. Use `El::key()` to fix identity:

```rust
// WITHOUT key: state stays at position [0]
for (i, item) in items.iter().enumerate() {
    button(&item.name, handler)
}

// WITH key: state follows the item
for (i, item) in items.iter().enumerate() {
    button(&item.name, handler).key(item.id)
}
```

### Coordinate Transformation

All layout uses logical units. Display scale is only multiplied when reaching device pixels in `canvas.rs`:

```
logical_point = device_point / scale_factor
device_point = logical_point * scale_factor
```

Every platform backend transforms X11/Win32/Cocoa coordinates to logical units before sending events.

### Focus Management

`El::takes_focus` is the single source of truth for focusability:

```rust
// Don't read .focusable and .disabled separately
// El::takes_focus is focusable && !disabled
// The focus walk, focus ring, and audit all read it the same way
.focusable(true)
.disabled(false)
.on_key(handler)
```

### Text Measurement and Drawing

Text measure is exact to draw because all text uses one advance path. Wrapped text inside a growing child measures at its actual width, not a placeholder:

```rust
// The view is rebuilt and text re-measured every frame
// So wrapped text measures at the actual width it will draw at
// measure_stack re-measures at the real width being dealt
```

### Empty State Handling

Empty states must be furnished:

```rust
if items.is_empty() {
    col((
        text("No items yet"),    // one muted line
        button("Add item", |a| a.add_item()),  // one action
    )).tone(Tone::Idle)          // Idle tone
} else {
    // list of items
}
```

### Animation and Easing

Time is always injected, never read from a clock. Tests can step time exactly:

```rust
// In view: read from Memory, never from Instant::now()
let elapsed = memory.elapsed();  // injected time

// In tests:
let mut h = Harness::new(state, view);
h.frames(10);  // Step 10 frames of animation
```

### Graceful Shutdown

Windows close cleanly and destructors run. On macOS, `terminate:` closes all windows instead of tearing down:

```rust
// Application ends when loop notices no visible window
// App::run returns and destructors run
// No special cleanup needed
```

### R2 Motion Kit Patterns

**Smooth drag-to-spring handoff** (momentum preserved):
```rust
.on_drag(|app: &mut App, drag| {
    app.position = drag.fraction().x * 100.0;
    app.last_velocity = Some(Velocity::new(drag.velocity.x, angle));
})
.on_spring(move |app: &mut App, spring| {
    app.position = spring.value;
    if spring.is_complete() { app.reset(); }
})
```

**List item entrance animation**:
```rust
col(items.iter().enumerate().map(|(i, item)| {
    button(&item.name, handler)
        .enter_exit(EnterExit::Slide(SlideDirection::Left), Duration::millis(300))
        .key(item.id)
}))
```

**Modal dismiss with cleanup**:
```rust
if app.modal_open {
    modal.on_exit(|app: &mut App| {
        app.after_animation(Duration::millis(300), || app.modal_open = false);
    })
}
```

**Accessibility-aware progress animation**:
```rust
let progress = if theme.metrics.is_motion_enabled() {
    painter.ease(app.progress, Easing::EaseOut, Duration::millis(200))
} else {
    app.progress  // Skip animation, show target immediately
};
painter.meter(progress);
```

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

## Verification and Quality Checks

Developers can verify that patterns and practices are working correctly:

### Code Verification

**Module organization:**
```bash
# Verify all 19 core modules exist
for mod in lib element style layout geom color canvas sdf image paint text theme widgets \
           memory input accessibility app shell reload testing; do
  test -f "src/${mod}.rs" || test -d "src/${mod}" || echo "Missing: $mod"
done
```

**Backend trait compliance (12 methods):**
```bash
# Count Backend trait methods (should be 12)
grep -c "fn " src/shell/mod.rs | grep "trait Backend" -A 50 | wc -l
```

**Tests passing (expected: 379 tests):**
```bash
cargo test --lib -- --nocapture 2>&1 | grep "test result:"
```

### Documentation Verification

**Recipes exist:**
```bash
grep -c "## Recipe" CLAUDE.md  # Should be 3 (WASM, X11, Checkbox)
```

**Key sections present:**
```bash
for section in "Module Structure" "Key Invariants" "Stellar UI Practices" \
               "Library Roadmap" "Conventions" "Contributor Workflow"; do
  grep -q "## $section" CLAUDE.md && echo "✓ $section" || echo "✗ Missing: $section"
done
```

**Cross-references to rui.dx:**
```bash
grep -c "rui.dx" CLAUDE.md  # Should be multiple references
```

### Best Practices Verification

**No Rc/RefCell in view functions:**
```bash
# Check that Rc/RefCell aren't used in typical view patterns
grep -r "Rc<\|RefCell<" src/ | grep -v "platform/" && echo "WARNING: Found Rc/RefCell" || echo "✓ Clean"
```

**Unsafe code confined to platform boundaries:**
```bash
# Verify unsafe only in shell/platform/
find src -name "*.rs" ! -path "*/shell/platform/*" -exec grep -l "unsafe" {} \;
```

**Accessible color contrast:**
```bash
# Run contrast validation test
cargo test theme::tests::the_battery_rejects_an_illegible_palette
```

---

## Rationale

This library serves a singular philosophy: **above the window, everything is pure; below, only focused unsafe code at platform boundaries**. Every design decision flows from this: no retained widget tree (state rebuilt), no Rc/RefCell (identity is path-based), handlers are functions (not methods), and backends are thin (coordinate + event translation).

Recipes document the proof that this architecture works: major features have been delivered in three phases, tested at each step, and shipped to production. New contributors should follow the recipe template when adding backends, widgets, or interactions. The patterns in existing recipes are the measure of "done."

---

End of CLAUDE.md
