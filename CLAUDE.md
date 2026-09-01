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
- **app.rs** + **shell/** — App loop, `Backend` trait (open/pump/surface/appearance/present/is_open — 6 methods), platform backends (macOS/Windows/X11/WASM, confined to unsafe)
- **reload.rs** — (feature="reload" only, compiled out of release) Running window notices executable changed, saves state, restarts in place

**Testing:**
- **testing/** — `Harness`: real pipeline headless, no window; synthetic font (char = size/2) so layout tests assert exact numbers

See `rui.dx` for complete module responsibilities and 20+ invariants that must not be broken.

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

Platform-specific code implements a 6-method trait:
```rust
pub trait Backend {
    fn open(&mut self) -> Result<()>;
    fn pump(&mut self) -> Vec<Event>;
    fn surface(&mut self) -> &mut [u32];
    fn appearance(&self) -> Appearance;
    fn present(&mut self);
    fn is_open(&self) -> bool;
}
```

All platform-agnostic code (layout, paint, handlers) sits above this line. Each backend (macOS, Windows, X11, WASM) implements these six methods; everything else is shared.

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

---

## Recipe 3: Checkbox Control

**Status**: Complete — Verified as replicable pattern for custom widget implementation.

### Overview
Checkbox demonstrates the minimal interactive control: a single boolean that toggles on click. It proves that even the smallest custom widget follows the state-view-handler pattern without requiring any special framework support.

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
- **Implementation** (src/widgets.rs lines 259–283):
```rust
pub fn checkbox<S: 'static>(
    label: &str,
    checked: bool,
    toggle: impl Fn(&mut S) + 'static,
) -> El<S> {
    row((
        draw(Size::new(15.0, 15.0), move |painter, rect| {
            let tone = if checked { Tone::Accent } else { Tone::Sunken };
            painter.fill(rect, Radius::Units(4.0), tone);
            painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
        }),
        text(label),
    ))
    .gap(8.0)
    .on_click(move |state: &mut S| toggle(state))
}
```
- **Key insight**: `checked` parameter flows as an upvalue into the `draw()` closure; conditional styling proves state determines appearance
- **Verification**: `cargo test --test recipes -- a_checkbox_draws_differently_once_it_is_ticked` passes

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
- **Files**: tests/recipes.rs (integration tests), src/testing/harness.rs (if needed)
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

**Pattern proof**: Checkbox is 25 lines of code. Zero framework support required. No special widget class, no retained tree, no lifecycle. Just state → view function → handlers. This pattern works for button, segmented, slider, radio, custom charts—any interactive element.

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

## Widget Exemplars

### Segmented Control Exemplar

The `segmented` widget demonstrates building an interactive choice selector from primitives. It is self-contained (59 lines total) and shows the state-view-handler pattern clearly.

**Pattern:**
```
State:   struct App { selected: usize }
View:    fn view(app: &App) -> El<App> { segmented(&choices, app.selected, handler) }
Handler: |app: &mut App, index| { app.selected = index; }
```

**Key insight:** The handler receives `&mut S`, not a closure capturing a reference. This eliminates `Rc<RefCell<>>` and interior mutability.

**Try it:**
```bash
cargo run -p rui --example segmented
```

Click buttons to change selection; state persists across frames.

**Modification checklist:**
- [ ] Change `["Small", "Medium", "Large"]` to your own choices
- [ ] Replace the label with your description
- [ ] Extend or shrink the choices array
- [ ] Change colors with `.fill()` method
- [ ] Copy from `src/widgets.rs` line 333–365 to customize appearance

**Verification:**
- `cargo run -p rui --example segmented` works
- `cargo test --test recipes -- segmented` passes
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
cargo run -p rui --example checkbox
```

Click the checkbox to toggle ON/OFF; state persists.

**Modification checklist:**
- [ ] Change `"Enable notifications"` to your own label
- [ ] Replace `app.notify` with any boolean field in your state
- [ ] Add more checkboxes by calling `checkbox()` multiple times
- [ ] Change colors or size with `.fill()` or `.w()` methods
- [ ] Copy from `src/widgets.rs` line 259–283 to customize rendering

**Verification:**
- `cargo run -p rui --example checkbox` works
- `cargo test --test recipes -- checkbox` passes
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
cargo run -p rui --example meter
```

Watch the meter animate from 0% to 100%.

**Modification checklist:**
- [ ] Change `Tone::Accent` to `Tone::Success`, `Tone::Warning`, etc.
- [ ] Customize bar width/height by copying `src/widgets.rs` line 259–280
- [ ] Animate `app.progress` over time in your event loop
- [ ] Copy the pattern for other display-only visualizations (volume, status lights, gauges)

**Verification:**
- `cargo run -p rui --example meter` works
- `cargo test --test recipes -- meter` passes
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
1. Copy `examples/segmented.rs` to `examples/my_control.rs`
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
cargo test -p rui                       # 474 tests, headless

# Run the gallery (visual verification)
cargo run -p rui --example gallery -- .

# Benchmark frame cost
cargo run -p rui --release --example cost

# Platform-specific tests
cargo test --test x11_backend_phases
cargo test --test integration
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
