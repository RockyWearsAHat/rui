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
