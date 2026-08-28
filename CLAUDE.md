# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rui** is a declarative interface library for Rust with **zero dependencies**. It unifies structure (layout), style (appearance), and behavior (interaction) into a single Rust expression, rendered by its own TrueType parser, glyph rasteriser, and platform-specific window backends (macOS, Windows, X11).

**Core design principles:**
- **View is a pure function of state.** The `view` function rebuilds the entire UI description from application data each frame—no retained widget tree.
- **Handlers are functions of state, not closures.** `on_click(|app: &mut App| …)` receives mutable state as an argument, eliminating `Rc`, `RefCell`, and interior mutability.
- **Roles, not values.** Colors are named by semantic role (`Tone::Surface`, `Tone::Muted`), so the same description works in light and dark modes.
- **Foundations, not a catalogue.** The library provides primitives (`draw`, `on_drag`, `on_key`, `layer`) for building custom controls—no built-in checkbox or slider, because those constrain design.

## Setup & Requirements

- **Rust 1.85+** (Edition 2024), verified by `tests/setup.rs`. Use `rustup update` if needed.
- **No external dependencies**—the full renderer, font handling, and window management are in this crate.
- **Platforms:** macOS (Cocoa), Windows (WinAPI), X11/Wayland (via X11 server).
- **Pre-commit hook:** Runs `cargo fmt` and checks Cargo.lock cleanliness (`.git/hooks/pre-commit`). Executable after first git setup.

## Common Commands

```bash
# Build
cargo build                                      # Debug build
cargo build --release                           # Optimized build

# Run examples
cargo run -p rui --example counter               # Interactive counter app
cargo run -p rui --example controls              # Control showcase with checkbox, slider, etc.
cargo run -p rui --example gallery -- .          # Render every UI element to PNG (no window)

# Test
cargo test                                       # Run all tests
cargo test --test setup                          # Verify Rust version and pre-commit hook
cargo test --lib                                 # Unit tests only
cargo test --test interaction -- --nocapture     # Run one test file with output
cargo test integration --lib                     # Test the rendering pipeline

# Format & Lint
cargo fmt                                        # Auto-format all code
cargo fmt --check                                # Check formatting without changing files
cargo clippy                                     # Run linter

# Documentation
cargo doc --no-deps --open                       # Generate and open docs
```

## WASM Backend

**rui** includes a WebAssembly backend (`src/shell/platform/wasm.rs`), allowing the same UI code to run in a browser with no changes. The backend implements the `Backend` trait using DOM canvas rendering and `wasm-bindgen` for JavaScript interop.

### Requirements

- **wasm-pack**: Install with `curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh`
- **WASM target**: Already installed by rustup, but verify with `rustup target add wasm32-unknown-unknown`
- **A modern browser**: Chrome, Firefox, Safari (for pixel comparison in parity verification)

### Build & Test

```bash
# Build the WASM target
cargo build --target wasm32-unknown-unknown -p rui --example counter

# Generate wasm-bindgen glue and a web package
wasm-pack build --target web --release --out-dir pkg

# Test in a headless browser (Firefox required)
cargo test --target wasm32-unknown-unknown  # All WASM tests
wasm-pack test --headless --firefox --test wasm_integration  # Browser-specific integration tests
```

### Exports

The counter example exports three main functions via `#[wasm_bindgen]` in `src/wasm.rs`:

- **`init_counter()`**: Initialize the counter app and store it in thread-local state. Must be called once before `present_counter()`.
- **`listen_counter()`**: Collect events from the DOM (`mousemove`, `click`, `wheel`, etc.) and apply them to the app state. Called before each frame.
- **`present_counter()`**: Draw the app to pixels and present them to the browser `<canvas>`. Called after `listen_counter()` in the animation loop.

Additional utilities:
- **`counter_frame_count()`**: Get the current frame count; used by tests to verify memory persistence.
- **`present_parity_frame(dark: bool)`**: Draw the reference frame (identical to the native desktop render) and present it to the canvas, for pixel-perfect backend comparison.
- **`parity_frame_size()`**: Get the dimensions of the parity frame as `[width, height]`.

### Parity Verification

To verify that the WASM backend draws pixel-for-pixel identical frames to the native desktop backend:

```bash
# Build native reference frame
cargo run -p rui --example parity -- target/parity

# Build and serve the WASM package
wasm-pack build --target web --release --out-dir pkg
python3 -m http.server 8731 --bind 127.0.0.1

# Open http://127.0.0.1:8731/examples/parity.html
# The page compares the browser render to the native PNG, showing:
#   - Green: identical frames (0 differing pixels)
#   - Red: differences (pixel count and region shown)
```

The parity test verifies both light and dark modes and confirms that the rendering pipeline (`src/paint.rs`, `src/canvas.rs`, `src/text.rs`) is truly platform-agnostic.

## Module Structure

| Module | Purpose |
|--------|---------|
| `element` | UI element tree—`El<T>` is the root type; builders for structure (`col`, `row`) and containers. |
| `widgets` | Recipes (high-level elements): `button`, `text`, `title`, `panel`, `tabs`, `draw`, etc. All built from primitives. |
| `style` | Layout and appearance: `Length` (Auto/fixed/grow), `Radius`, `Tone` (color roles), `Align`, `Justify`. |
| `layout` | Flexbox-like layout engine; single-axis stacking, `flow()` for line wrapping, scroll & layer support. |
| `paint` | Drawing abstraction: `Painter` API used by all elements; `Visual` tracks hover/focus/held/disabled state. |
| `canvas` | Pixel buffer and rasteriser; `Canvas::draw` is the root drawing operation. |
| `font` & `text` | TrueType parser, glyph rasterising, text layout with kerning/ligatures. `FontId` indexes loaded fonts. |
| `color` | RGB(A) colors and sRGB gamma handling. |
| `demo` | The counter, in one place. `examples/counter.rs`, `src/wasm.rs`, and `examples/parity.rs` all drive this one description, which is what makes "every backend draws the identical frame" checkable rather than merely claimed. |
| `geom` | Primitives: `Rect`, `Point`, `Size`, `Insets`. |
| `image` | PNG encoder for rendering to files (used by `gallery` example). |
| `shell` | Platform window management: macOS/Windows/X11 backends implement `Backend` trait. Event loop lives here. |
| `memory` | Stateful interaction data (hover, focus, scroll position, caret, animations); keyed by element identity. |
| `app` & `run` | Application entry point; couples state, view function, and the event loop. |
| `testing` | `Harness`: drives the real frame into a buffer with a synthetic font for deterministic testing; `Harness::click_text`, `drag`, `shows`, `pixel`, `save_png`. |

## Key Architectural Patterns

### Event Loop (in `src/shell/mod.rs`)

```
loop:
  wait for input (platform-specific)
  call view(state) → El<State>
  layout & measure
  paint to canvas
  if pixels differ from last frame: present to screen
  animate (step animations by elapsed time)
  if animating: refresh within 8ms; else wait App::idle_timeout
```

Platform-specific code (macOS/Windows/X11) implements a `Backend` trait with five methods: `open()`, `pump()`, `surface()`, `appearance()`, `present()`, `is_open()`. All layout, painting, and state logic is platform-agnostic above that line.

### Testing UI (in `src/testing/`)

Use `Harness` to drive the real frame with no window:
```rust
use rui::testing::Harness;

let mut harness = Harness::new(MyApp { counter: 0 }, view);
harness.click_text("Increment");
assert_eq!(harness.state().counter, 1);
assert!(harness.frame().shows("1"));
```

The synthetic font ensures widths are arithmetic (half an em per character). Animations are stepped by hand, not waited for. See `tests/recipes.rs` for live examples building `checkbox`, `switch`, `slider`, `radio`, `tooltip`.

### Building Custom Controls

Copy a recipe from `tests/recipes.rs` or `examples/controls.rs`—they are ordinary `El` types, not variants. Modify freely:
```rust
widgets::draw(Size::new(160.0, 18.0), move |painter, rect| {
    let (filled, _) = rect.split_left(rect.w * value);
    painter.fill(rect, Radius::Pill, Tone::Sunken);
    painter.fill(filled, Radius::Pill, Tone::Accent);
})
.on_drag(|app: &mut App, drag| app.volume = drag.fraction().x)
.on_key(|app: &mut App, key, _| app.nudge(key))
```

## Recipes

A recipe is a worked example of implementing a major feature: files touched in order, verification gates executed at each step, cross-module coordination shown at the seams. Each recipe documents one complete journey from request to commit—a blueprint for how **state**, **element**, **layout**, **paint**, **shell**, and **memory** work together. Recipes are step-by-step implementations of major features, verified against real commits in git history; they show not just the pattern, but the proof that the pattern holds.

See `tests/recipes.rs` for reference implementations of common controls (`checkbox`, `switch`, `slider`, `radio`, `tooltip`). Each recipe there is a small, testable example following the same structure as a larger feature: define state shape, build the element tree, handle input events, verify the output.

### Recipe 1: Adding a WASM Backend

**Commits:** 17 total from 77d4780 to 2df7f1c, grouped in three phases: clock abstraction, FrameDriver refactor, and WASM integration.

The WASM backend allows the same UI code to run in a browser with no changes to the view function. The implementation required three coordinated changes to the shell:

**Phase 1: Clock Abstraction (Commit 77d4780 — "Split the loop's driving from the loop's frame")**

Commits in this phase: `77d4780`

Files touched:
- `src/shell/clock.rs` (new): Platform-agnostic clock abstraction. Desktops use `std::time::Instant`; WASM uses `performance.now()` (since `Instant::now()` panics on `wasm32-unknown-unknown`). Both return a `Moment` type that understands its platform.
- `src/shell/mod.rs` (line 58+): Import `clock::Moment` and replace `Instant` with `Moment` in the `Surface` struct. Update `begin_frame()` calls to use `Moment::since()` instead of `saturating_duration_since()`.
- `src/app.rs`: Add `'static` bound to `run()` and `run_with_fonts()` (required for WASM closures to capture state across frame boundaries).
- `Cargo.toml`: Add WASM dependencies (`Performance`, `console` to `web-sys` features).

**Why this order:** The loop's driving logic (wait for events, draw, present) needs to work identically on desktop and browser. But a desktop waits on a blocking system call, while a browser calls back via `requestAnimationFrame`. To unify these, we first abstract time (since `Instant` cannot be used on WASM), then split the loop into a reusable `turn()` function that both drivers can call.

**Verification gate:** `cargo test --lib` passes; time is correctly measured on both platforms (desktop via system clock, browser via `performance.now()`).

**Phase 2: FrameDriver Refactor (Commits 531214f, 9afc9b1, b6a1b2c, 2ef3c2b — preparation and testing for frame-stepping abstraction)**

Commits in this phase: `531214f` (fix docs), `9afc9b1` (frame-stepping test), `b6a1b2c` (WASM documentation), `2ef3c2b` (Step 8: backend selector gate), `caa3066` (Step 3: refactor native run)

Files touched:
- `src/shell/mod.rs` (line 295+): Extract the core loop body into a new `turn()` function that both native and WASM drivers call. Introduce `continues()` helper. The native driver still owns the `while` loop; the browser driver calls `turn()` from `requestAnimationFrame`.
- `tests/shell_stepping.rs` (new): Test that `turn()` can be called repeatedly to step through frames without owning an event loop. This test verifies the abstraction is sound before WASM tries to use it.

**Why this order:** WASM cannot block (there is no thread to yield), so the loop cannot be a `while` at the top level. Extracting `turn()` makes the frame logic platform-agnostic; both drivers become thin wrappers that provide events and decide when to call `turn()` again. The test suite (shell_stepping and later WASM-specific tests) verifies the frame-stepping logic is correct before integration.

**Verification gate:** `cargo test --test shell_stepping` passes (frame stepping works). `cargo build` for native still works (no regression). `cargo test --lib` confirms compiled tests pass.

**Phase 3: WASM Integration (Commits b116ac8, 32bf53d, d820ff6, e41376e, 929899a, 830033c, 2365866, 3062aba, 401a8a7, ce4acad, 2df7f1c)**

Commits in this phase: `b116ac8` (Step 5: verify memory persistence), `32bf53d` (Step 5: fix wasm config), `d820ff6` (scout: add Recipes to worklist), `e41376e` (worklist: close item 3), `929899a` (Step 5: verify memory), `830033c` (record backend parity check), `2365866` (check browser round trip), `3062aba` (prove native/wasm parity), `2b02fd0` (Step 4: error recovery), `401a8a7` (Step 5: expose FrameDriver), `ce4acad` (Step 6: integrate WASM events), `2df7f1c` (Step 7: parity test)

Files touched:
- `src/shell/mod.rs` (line 412+): Add `#[cfg(target_arch = "wasm32")] pub(crate) fn run()` that creates a `Page` struct holding all loop state, registers a `requestAnimationFrame` callback, and returns immediately. The callback holds `Rc<RefCell<>>` of the page state and calls `turn()` on each repaint. Add `schedule()` and `report()` functions for browser integration.
- `src/shell/clock.rs`: Update to handle WASM timing edge cases; add fallback to zero duration if performance API is unavailable.
- `src/wasm.rs` (new): WASM-specific bindings for the browser. Exports `init_counter()`, `listen_counter()`, `present_counter()` that call into the generic `turn()` loop. The page's event listener (DOM click, mousemove, wheel) collects events into the `Input` queue; each frame, `turn()` consumes them.
- `src/shell/platform/wasm.rs` (new): Implement the `Backend` trait for the browser (canvas rendering, event listening, `appearance()` from `prefers-color-scheme`).

**Why this order:** The native `run()` was already done and working. WASM adds a sibling `run()` (guarded by `#[cfg(target_arch = "wasm32")]`) that uses the same `turn()` but with no `while` — it relies on the browser to call the callback repeatedly. The `Backend` trait is unchanged; only the platform-specific initialization differs. Integration testing (memory persistence, parity checks) runs throughout to catch issues early.

**Verification gates:**
- Compiled verification: `cargo build --target wasm32-unknown-unknown -p rui --example counter` succeeds.
- Unit tests: `cargo test --lib` passes; memory and animation state persist across frames.
- Browser testing: `wasm-pack test --headless --firefox` confirms the app initializes and responds to DOM events.
- Parity verification: `examples/parity.html` (browser) renders pixel-for-pixel identical frames to the native desktop. Light and dark modes both verified. Gate runs as part of `cargo test --test interaction`.

#### Verification at Each Phase

Each phase had verification gates to confirm correctness before integrating further:

**Phase 1: Clock Abstraction**

Compiled verification:
```bash
cargo test --lib
```
Confirms `Moment::now()` works on both native (uses `Instant`) and WASM (uses `performance.now()`), and `Moment::since()` correctly measures elapsed time. Time flows continuously on both platforms.

**Phase 2: FrameDriver Refactor**

Compiled verification:
```bash
cargo build
cargo test --lib
```

Frame-stepping test:
```bash
cargo test --test shell_stepping -- --nocapture
```
Confirms the `turn()` abstraction can be called repeatedly without owning an event loop. The test verifies that state persists across calls, input is collected correctly, and frames are drawn and presented. This is the crucial gate: if frame-stepping works, both native (which loops) and WASM (which doesn't) can call the same function.

**Phase 3: WASM Integration**

Compiled verification (native still works):
```bash
cargo build
cargo test --lib
```

WASM target compilation:
```bash
cargo build --target wasm32-unknown-unknown -p rui --example counter
```
Confirms WASM builds without errors and the clock abstraction works in the browser environment.

Browser testing (requires Firefox):
```bash
wasm-pack test --headless --firefox
```
Confirms the page initializes, responds to DOM events (click, mousemove, wheel), and state persists across repaints. Memory tests verify that hover/focus/scroll state is preserved between frames.

Parity verification (pixel-perfect comparison):
```bash
# 1. Build native reference frame
cargo run -p rui --example parity -- target/parity

# 2. Build and serve WASM package
wasm-pack build --target web --release --out-dir pkg
python3 -m http.server 8731 --bind 127.0.0.1

# 3. Open http://127.0.0.1:8731/examples/parity.html in a browser
```
Verifies that the WASM backend renders pixel-for-pixel identical frames to native. Light and dark modes are both tested. A green page indicates zero differing pixels; red indicates differences and their count. This gate confirms the rendering pipeline is truly platform-agnostic.

#### Cross-Module Coordination

- **`shell::clock`** abstracts platform time so `shell/mod.rs` does not know or care about `Instant` vs `performance.now()`.
- **`Backend` trait** (in `shell/mod.rs`) unifies the interface: `open()`, `pump()`, `surface()`, `appearance()`, `present()` work the same for native and WASM. Only the platform-specific implementations differ.
- **`turn()` function** (line 313+) is the coordination point: it accepts a `Backend` and calls its methods. Both native `run()` (which loops) and WASM `run()` (which returns and relies on callbacks) call the same `turn()`, so a change to the rendering pipeline only needs to be made once.
- **Event flow:** Native events come from `window.pump()` (a blocking wait on the platform); WASM events come from DOM listeners. Both feed into the same `Input` queue, so the rest of the frame logic is unchanged.
- **State persistence:** The `Memory` struct (in `src/memory`) tracks hover, focus, scroll, and animation state. The same state machine is used on both platforms; the only difference is how each platform asks for the next frame.

## Workflow Notes

- **Unsafe code:** Confined to `shell/platform/*.rs` (one file per OS). Everything above that—elements, layout, rendering, fonts—is safe Rust.
- **No dependencies:** When adding features, build within the crate. Resist reaching for external crates.
- **Identity & keys:** Elements get unique identity from their path in the tree. For lists that reorder, use `.key(&item.id)` to preserve state (hover, focus, scroll) across frame rebuilds.
- **Appearance (light/dark mode):** Read via `prefers-color-scheme` media query in the `Backend`. Test both modes in examples.
- **Text inherits; layout does not:** `text_size`, `color`, `face` flow to children; `pad`, `gap`, `fill` do not (a child padding itself is usually surprising).

## Git & CI

- **Hook:** Pre-commit runs `cargo fmt` and checks for uncommitted `Cargo.lock` changes. Bypass with `git commit --no-verify` only for emergencies (then fix the hook cause).
- **Cache/state files ignored:** `.cache/`, `.doc/index.db`, `.engine/` (build artifacts) are in `.gitignore`; working directory must stay clean.
- **Commits:** Prefix with the platform or feature touched (e.g., "Add X11 input handling", "Refactor layout engine", "Improve text rendering").
