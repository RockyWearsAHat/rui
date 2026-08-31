# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rui** is a declarative interface library for Rust with **zero dependencies**. It unifies structure (layout), style (appearance), and behavior (interaction) into a single Rust expression, rendered by its own TrueType parser, glyph rasteriser, and platform-specific window backends (macOS, Windows, X11).

**Core design principles:**
- **View is a pure function of state.** The `view` function rebuilds the entire UI description from application data each frame—no retained widget tree.
- **Handlers are functions of state, not closures.** `on_click(|app: &mut App| …)` receives mutable state as an argument, eliminating `Rc`, `RefCell`, and interior mutability.
- **Roles, not values.** Colors are named by semantic role (`Tone::Surface`, `Tone::Muted`), so the same description works in light and dark modes.
- **Foundations, not a catalogue.** The library provides primitives (`draw`, `on_drag`, `on_key`, `layer`) and recipes (e.g., `checkbox`, `segmented`, `meter`) for building custom controls. Recipes are blueprints, not constraints—copy and modify them freely.

## Setup & Requirements

- **Rust 1.85+** (Edition 2021), verified by `tests/setup.rs`. Use `rustup update` if needed.
- **No external dependencies**—the full renderer, font handling, and window management are in this crate.
- **Platforms:** macOS (Cocoa), Windows (WinAPI), X11/Wayland (via X11 server).
- **Pre-commit hook:** Runs `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` (`.git/hooks/pre-commit`). Prevents commits with formatting issues or lint warnings. Executable after first git setup.

## Common Commands

```bash
# Build
cargo build                                      # Debug build
cargo build --release                           # Optimized build

# Run examples
cargo run -p rui --example counter               # Interactive counter app
cargo run -p rui --example controls              # Control showcase with checkbox, slider, etc.
cargo run -p rui --example gallery -- .          # Render every UI element to PNG (no window)
cargo run -p rui --example segmented             # Segmented control exemplar (copy & modify template)
cargo run -p rui --example meter                 # Meter widget exemplar (passive display-only)
cargo run -p rui --example parity -- target/parity  # Build native reference frame for WASM parity test

# Test
cargo test                                       # Run all tests
cargo test --test setup                          # Verify Rust version and pre-commit hook
cargo test --lib                                 # Unit tests only
cargo test --test interaction -- --nocapture     # Run one test file with output
cargo test --test integration                    # Run integration tests
cargo test --test recipes -- widget_name         # Run a specific widget test (e.g., segmented_control)

# Format & Lint
cargo fmt                                        # Auto-format all code
cargo fmt --check                                # Check formatting without changing files
cargo clippy                                     # Run linter

# Documentation
cargo doc --no-deps --open                       # Generate and open docs
```

## Examples Directory

All examples can be run with `cargo run -p rui --example <name>`. Each example demonstrates a different aspect of the library:

| Example | Purpose |
|---------|---------|
| `counter` | The simplest app: increment/decrement with persistent state. Entry point for learning rui. |
| `controls` | Showcase of built-in widgets: button, checkbox, slider, segmented control, etc. |
| `gallery` | Renders every UI element to PNG files (no window). Used to verify visual appearance without launching an app. |
| `checkbox` | Exemplar: a minimal, self-contained binary toggle (26 lines). Copy and modify to build custom check/radio/toggle controls. |
| `segmented` | Exemplar: a minimal, self-contained choice selector (33 lines). Copy and modify to build new interactive controls. |
| `meter` | Exemplar: a passive progress bar showing how to build read-only widgets. |
| `parity` | Builds a native reference frame for pixel-perfect WASM backend comparison. |
| `icon` | Generates macOS `.iconset` and `.icns` app icons by drawing them at all required sizes. |
| `segmented_modified` | Verification that the documented "Copy and Modify" path from CLAUDE.md actually works. |

**Learning Path:** Start with `counter` (state and event loop), then `checkbox` (binary toggle), then `segmented` (multiple choice), then `meter` (passive display). Explore other examples as needed.

## Test Suite

All tests can be run with `cargo test`. The project includes 12 test files covering different aspects:

| Test File | Purpose |
|-----------|---------|
| `setup.rs` | Verifies Rust version (1.85+) and pre-commit hook configuration. Run with `cargo test --test setup`. |
| `layout.rs` | Unit tests for the flexbox-like layout engine; tests spacing, alignment, sizing. Run with `cargo test --test layout`. |
| `rendering.rs` | Tests the rendering pipeline: colors, shapes, text layout, transformations. Run with `cargo test --test rendering`. |
| `recipes.rs` | Widget examples and integration tests: checkbox, switch, slider, radio, tooltip, segmented control, meter. Run specific: `cargo test --test recipes -- slider` (tests all slider-related tests). |
| `interaction.rs` | Event handling tests: clicks, drags, keyboard input, focus management. Run with `cargo test --test interaction -- --nocapture` to see output. |
| `integration.rs` | End-to-end integration tests combining layout, rendering, and interaction. Run with `cargo test --test integration`. |
| `external_driving.rs` | Frame-stepping tests without an event loop; verifies the frame driver abstraction. Used to test WASM compatibility. Run with `cargo test --test external_driving`. |
| `recipe_1_verification.rs` | Verification gates for Recipe 1 (WASM Backend implementation). Confirms memory persistence and parity. Run with `cargo test --test recipe_1_verification`. |
| `wasm_integration.rs` | Browser integration tests for WASM backend; run in Firefox with `wasm-pack test --headless --firefox`. |
| `wasm_events.rs` | WASM-specific event handling tests (DOM events, mousemove, wheel, keyboard). Run with `wasm-pack test --headless --firefox`. |
| `wasm_fonts.rs` | WASM font loading and text rendering tests. Run with `wasm-pack test --headless --firefox`. |
| `wasm_parity.rs` | Pixel-perfect parity tests: compares WASM rendering to native reference frames (light and dark modes). Run with `cargo test --test wasm_parity`. |

**Test Strategy:**

- **Unit tests** (`cargo test --lib`): Fast, isolated tests of individual modules (layout, color, text, geom).
- **Integration tests** (`tests/*.rs`): Full-stack tests using the `Harness` testing framework. Tests typically create a `Harness` with an app state and view function, then simulate user interactions (clicks, drags, keystrokes) and assert state changes.
- **Platform tests** (`wasm_*.rs`): Browser-specific tests for WASM backend. Require Firefox and `wasm-pack`.
- **Verification gates** (`recipe_1_verification.rs`): Ensure major features (WASM integration, platform backends) remain correct after changes.

**Example: Running a single test**

```bash
cargo test --lib geometry                        # Run geometry unit tests
cargo test --test recipes -- meter               # Run meter widget tests
cargo test --test wasm_parity                    # Run WASM parity verification
wasm-pack test --headless --firefox --test wasm_integration  # Run browser tests
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
| `text` | TrueType parser, glyph rasterising, text layout with kerning/ligatures; `FontId` indexes loaded fonts. |
| `color` | RGB(A) colors and sRGB gamma handling. |
| `demo` | The counter, in one place. `examples/counter.rs`, `src/wasm.rs`, and `examples/parity.rs` all drive this one description, which is what makes "every backend draws the identical frame" checkable rather than merely claimed. |
| `geom` | Primitives: `Rect`, `Point`, `Size`, `Insets`. |
| `image` | PNG encoder for rendering to files (used by `gallery` example). |
| `input` | Input events and per-frame view of them; translates raw `Event` stream to immediate-mode queries. |
| `theme` | Colors, spacing, and type sizes for the entire UI; `Appearance` (light/dark) and `Tone` (semantic roles). |
| `syntax` | Syntax highlighting tokenizer for code display; supports Rust, Python, JavaScript, Bash, Diff. |
| `shell` | Platform window management: macOS/Windows/X11/WASM backends implement `Backend` trait. Event loop lives here. |
| `memory` | Stateful interaction data (hover, focus, scroll position, caret, animations); keyed by element identity. |
| `app` | Application entry point; `run()` couples state, view function, and the event loop. |
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

Platform-specific code (macOS/Windows/X11/WASM) implements a `Backend` trait with six methods: `open()`, `pump()`, `surface()`, `appearance()`, `present()`, `is_open()`. All layout, painting, and state logic is platform-agnostic above that line.

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

### Segmented Control Exemplar

The `segmented` widget is a minimal, self-contained exemplar showing how to build an interactive choice selector. It is small enough to copy and modify immediately—the example is 59 lines total, with just 19 substantive lines of code (the pattern itself is trivial: state struct, view function, handler closure).

**Pattern at a Glance:**
```
State:   struct App { selected: usize }
View:    fn view(app: &App) -> El<App> { ... widget here ... }
Handler: |app: &mut App, index| { app.selected = index; }
```
State describes your data. View turns data into UI. Handler modifies state on user input. That's the entire pattern.

This exemplar teaches:
- How state shapes the view (`app.selected` determines which button is highlighted)
- How handlers update state (the handler function receives mutable state as an argument)
- How to build custom controls from primitives (`row`, `on_click`, `Painter`)

**Try it first:**
```bash
cargo run -p rui --example segmented
```
Click the buttons to change selection; state persists across frames.

**State:**
```rust
struct App {
    selected: usize,  // index of the choice
}
```
The state is just an index—no closures, no `Rc<RefCell<>>`. This simplicity is rui's design.

**View:**
```rust
fn view(app: &App) -> El<App> {
    let choices = ["Small", "Medium", "Large"];

    col((
        text("Pick a size:"),
        widgets::segmented(
            &choices,
            app.selected,
            |app: &mut App, index| {
                app.selected = index;
            },
        ),
    ))
}
```

The handler is a function that receives mutable state as an argument, not a closure capturing a reference. This means you can freely modify `app` without any interior mutability tricks.

**How to modify:**
- Change `["Small", "Medium", "Large"]` to any `&[&str]` slice
- Replace the label `text("Pick a size:")` with your own description
- To add more buttons: extend the choices array and extend the width
- To change colors: call `.fill()` on the widget to style the background

**Implementation details:**
The widget is built entirely from primitives; see `src/widgets.rs` line 333–365. It uses:
- `row()` to lay out buttons horizontally
- `on_click()` to handle clicks and call the handler
- `Painter` to draw the background highlight

**Verification:**
- Run the example: `cargo run -p rui --example segmented`
- Inspect the test: `tests/recipes.rs` line 410 shows `a_segmented_control_changes_selection_when_clicked`
- Copy the entire pattern to build new controls: state type → view function → handler closure

**Getting Started: Copy and Modify**

To build your own widget from this exemplar:

1. **Copy the example file:**
   ```bash
   cp examples/segmented.rs examples/my_control.rs
   ```

2. **Modify the state to fit your needs:**
   ```rust
   struct App {
       size: String,  // Change from usize to fit your domain
   }
   ```

3. **Update the view function to use your state:**
   ```rust
   fn view(app: &App) -> El<App> {
       col((
           text("My custom control:"),
           // Replace segmented with your widget logic
       ))
   }
   ```

4. **Run your modified example:**
   ```bash
   cargo run -p rui --example my_control
   ```

5. **Write a test to verify it works (copy from `tests/recipes.rs` line 410):**
   ```rust
   #[test]
   fn my_control_changes_state_when_clicked() {
       let mut harness = Harness::new(App { size: "small".into() }, view);
       harness.click_text("Large");
       assert_eq!(harness.state().size, "large");
   }
   ```

6. **Run your test:**
   ```bash
   cargo test my_control_changes_state_when_clicked
   ```

If your test passes, your custom widget works. If it fails, the failure message will guide you to the issue.

**Next: Building Custom Controls**

Once you understand this exemplar, here are common next steps:

1. **Add more state fields:**
   ```rust
   struct App {
       selected: usize,
       confirmed: bool,  // Add a confirmation step
   }
   ```

2. **Update the handler to do more:**
   ```rust
   widgets::segmented(&choices, app.selected, |app: &mut App, index| {
       app.selected = index;
       app.confirmed = false;  // Reset confirmation when selection changes
   }),
   ```

3. **Build from primitives instead (copy `src/widgets.rs` line 333–365):**
   ```rust
   row(choices.iter().enumerate().map(|(i, label)| {
       widgets::button(*label, move |app: &mut App| {
           app.selected = i;
       })
   }))
   ```

4. **Connect to the test:** Copy `tests/recipes.rs` line 410 as a template for verifying your custom control.

5. **Explore other controls:** Look at `checkbox`, `switch`, `slider`, `radio` in `tests/recipes.rs`. Each follows the same state-view-handler pattern.

### Checkbox Exemplar

The `checkbox` widget is a minimal exemplar showing how to build a **binary interactive control**. Unlike segmented (which manages a choice among many), checkbox toggles a single boolean value.

**Pattern at a Glance:**
```
State:   struct App { notify: bool }
View:    fn view(app: &App) -> El<App> { checkbox("Enable notifications", app.notify, |app: &mut App| app.notify = !app.notify) }
Handler: |app: &mut App| { app.notify = !app.notify }
```
State describes a boolean preference. View turns that boolean into a checkbox. Handler toggles it on click.

It teaches:
- How state shapes the view (checked vs. unchecked appearance)
- How handlers update state (the handler function receives mutable state as an argument)
- How to build toggle controls from primitives (`draw`, `on_click`, `Painter`)

**Try it first:**
```bash
cargo run -p rui --example checkbox
```
Click the checkbox to toggle between ON and OFF; state persists across frames.

**State:**
```rust
struct App {
    notify: bool,  // the checked/unchecked state
}
```
The state is just a boolean—no closures, no `Rc<RefCell<>>`. This simplicity is rui's design.

**View:**
```rust
fn view(app: &App) -> El<App> {
    col((
        text("Your preferences:"),
        widgets::checkbox("Enable notifications", app.notify, |app: &mut App| {
            app.notify = !app.notify;
        }),
        row((text("Notifications: "), text(if app.notify { "ON" } else { "OFF" }))),
    ))
}
```

The handler is a function that receives mutable state as an argument. It toggles the boolean. This means you can freely modify `app` without any interior mutability tricks.

**How to modify:**
- Change `"Enable notifications"` to your own label text
- Replace `app.notify` with any boolean field in your state
- To add more checkboxes: call `widgets::checkbox()` multiple times, once per field
- To change colors or size: call `.fill()` or `.w()` on the widget to style it

**Implementation details:**
The widget is built entirely from primitives; see `src/widgets.rs` line 259–283. It uses:
- `draw()` to render a square checkbox (filled if checked, empty if unchecked)
- `row()` to lay out the checkbox and label horizontally
- `text()` to render the label text
- `on_click()` to handle clicks and call the handler

**Verification:**
- Run the example: `cargo run -p rui --example checkbox`
- Inspect the test: `tests/recipes.rs` line 414 shows `a_checkbox_draws_differently_once_it_is_ticked`
- Copy the entire pattern to build new controls: state type → view function → handler closure

**Getting Started: Copy and Modify**

To build your own toggle control from this exemplar:

1. **Copy the example file:**
   ```bash
   cp examples/checkbox.rs examples/my_toggle.rs
   ```

2. **Modify the state to fit your needs:**
   ```rust
   struct App {
       darkMode: bool,  // Change the field name
   }
   ```

3. **Update the view function to use your state:**
   ```rust
   fn view(app: &App) -> El<App> {
       col((
           text("Settings:"),
           widgets::checkbox("Dark mode", app.darkMode, |app: &mut App| {
               app.darkMode = !app.darkMode;
           }),
       ))
   }
   ```

4. **Run your modified example:**
   ```bash
   cargo run -p rui --example my_toggle
   ```

5. **Write a test to verify it works (copy from `tests/recipes.rs` line 414):**
   ```rust
   #[test]
   fn my_toggle_changes_state_when_clicked() {
       let mut harness = Harness::new(App { darkMode: false }, view);
       harness.click_text("Dark mode");
       assert_eq!(harness.state().darkMode, true);
   }
   ```

6. **Run your test:**
   ```bash
   cargo test my_toggle_changes_state_when_clicked
   ```

If your test passes, your custom toggle works. If it fails, the failure message will guide you to the issue.

**Next: Building Multiple Checkboxes**

Once you understand this exemplar, here are common next steps:

1. **Add more boolean fields:**
   ```rust
   struct App {
       notify: bool,
       darkMode: bool,  // Add another toggle
   }
   ```

2. **Render multiple checkboxes:**
   ```rust
   col((
       widgets::checkbox("Notifications", app.notify, |app: &mut App| app.notify = !app.notify),
       widgets::checkbox("Dark mode", app.darkMode, |app: &mut App| app.darkMode = !app.darkMode),
   ))
   ```

3. **Use a helper function for DRY:**
   ```rust
   fn pref(label: &str, value: bool, setter: impl Fn(&mut App, bool)) -> El<App> {
       widgets::checkbox(label, value, move |app: &mut App| setter(app, !value))
   }
   ```

4. **Connect to the test:** Copy `tests/recipes.rs` line 414 as a template for verifying each checkbox.

5. **Explore radio buttons and tabs:** Look at `radio`, `switch` in `tests/recipes.rs`. They follow the same pattern but with different state shapes (indices instead of booleans).

### Meter Widget Exemplar

The `meter` widget is a minimal exemplar of a **passive/display-only** widget. Unlike segmented (which responds to clicks), meter simply displays a value as a progress bar.

**Pattern at a Glance:**
```
State:   struct App { progress: f32 }
View:    fn view(app: &App) -> El<App> { meter(app.progress, Tone::Accent) }
Handler: (none — no user interaction)
```
Passive widgets read state and display it. No handler needed. State determines appearance.

It teaches:
- How state flows into the view without user interaction
- How to build visual feedback from primitives (`draw`, `Painter`, `Rect`)
- The difference between interactive and read-only widgets

**Try it first:**
```bash
cargo run -p rui --example meter
```
Click to watch the meter display progress from 0% to 100%.

**State:**
```rust
struct App {
    progress: f32,  // a value from 0.0 to 1.0
}
```

**View:**
```rust
fn view(app: &App) -> El<App> {
    col((
        text("Upload progress:"),
        meter(app.progress, Tone::Accent),
    ))
}
```

The `meter()` widget takes only two arguments:
- `fraction: f32` — clamped to 0.0–1.0 for display
- `tone: Tone` — the color role (e.g., `Accent`, `Success`, `Warning`)

**How to modify:**
- Change `Tone::Accent` to `Tone::Ok`, `Tone::Warn`, `Tone::Bad`, etc. for different colors
- To customize the bar width/height, copy the implementation from `src/widgets.rs` line 259–280 and adjust `Size::new(80.0, 6.0)`
- To add animation, update `app.progress` over time in your event loop

**Implementation details:**
The meter is a draw primitive showing a filled rectangle inside a track; see `src/widgets.rs` line 259–280. It uses:
- `draw()` to paint the meter directly to the canvas
- `Painter` to fill the track (sunken background) and filled portion (accent color)
- Corner rounding for a polished appearance

**Verification:**
- Run the example: `cargo run -p rui --example meter`
- Inspect the test: `tests/recipes.rs` line 451 shows `a_meter_displays_progress_as_a_fraction`
- The meter renders at any fraction value; display behavior is deterministic

**Key difference from segmented:**
- **Segmented:** Interactive. State changes on click. Handler runs user code.
- **Meter:** Passive. State flows to view. No handler; just displays.

Copy the meter pattern when building read-only visualizations: progress bars, volume indicators, status lights, or any gauge that reads state without changing it.

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

**Commits:** 18 total from 77d4780 to 2df7f1c (1 base + 17 subsequent), grouped in three phases: clock abstraction, FrameDriver refactor, and WASM integration.

The WASM backend allows the same UI code to run in a browser with no changes to the view function. The implementation required three coordinated changes to the shell:

**Phase 1: Clock Abstraction (Commit 77d4780 — "Split the loop's driving from the loop's frame")**

Commits in this phase: `77d4780`

Files touched:
- `src/shell/clock.rs` (new): Platform-agnostic clock abstraction. Desktops use `std::time::Instant`; WASM uses `performance.now()` (since `Instant::now()` panics on `wasm32-unknown-unknown`). Both return a `Moment` type that understands its platform.
- `src/shell/mod.rs` (line 55): Import `clock::Moment` and replace `Instant` with `Moment` in the `Surface` struct (line 199). Update `begin_frame()` calls to use `Moment::since()` instead of `saturating_duration_since()` (line 238).
- `src/app.rs`: Add `'static` bound to `run()` and `run_with_fonts()` (required for WASM closures to capture state across frame boundaries).
- `Cargo.toml`: Add WASM dependencies (`Performance`, `console` to `web-sys` features).

**Why this order:** The loop's driving logic (wait for events, draw, present) needs to work identically on desktop and browser. But a desktop waits on a blocking system call, while a browser calls back via `requestAnimationFrame`. To unify these, we first abstract time (since `Instant` cannot be used on WASM), then split the loop into a reusable `turn()` function that both drivers can call.

**Verification gate:** `cargo test --lib` passes; time is correctly measured on both platforms (desktop via system clock, browser via `performance.now()`).

**Phase 2: FrameDriver Refactor (Commits 531214f, 9afc9b1, b6a1b2c, 2ef3c2b, caa3066 — preparation and testing for frame-stepping abstraction)**

Commits in this phase: `531214f` (fix docs), `9afc9b1` (frame-stepping test), `b6a1b2c` (WASM documentation), `2ef3c2b` (Step 8: backend selector gate), `caa3066` (Step 3: refactor native run)

Files touched:
- `src/shell/mod.rs` (line 325): Extract the core loop body into a new `turn()` function that both native and WASM drivers call. Introduce `continues()` helper (line 359). The native driver still owns the `while` loop (line 383); the browser driver calls `turn()` from `requestAnimationFrame` (line 472).
- `tests/external_driving.rs`: Test that drives the app frame-stepping without owning an event loop, verifying that the abstraction is sound before WASM tries to use it. The test `state_mut_between_frames_drives_the_next_frame` confirms app state can be mutated between calls to `app.frame()`.

**Why this order:** WASM cannot block (there is no thread to yield), so the loop cannot be a `while` at the top level. Extracting `turn()` makes the frame logic platform-agnostic; both drivers become thin wrappers that provide events and decide when to call `turn()` again. The test suite (frame-stepping tests and later WASM-specific tests) verifies the frame-stepping logic is correct before integration.

**Verification gate:** `cargo test --test external_driving -- state_mut_between_frames_drives_the_next_frame` passes (frame stepping works). `cargo build` for native still works (no regression). `cargo test --lib` confirms compiled tests pass.

**Phase 3: WASM Integration (Commits b116ac8, 32bf53d, d820ff6, e41376e, 929899a, 830033c, 2365866, 3062aba, 2b02fd0, 401a8a7, ce4acad, 2df7f1c)**

Commits in this phase: `b116ac8` (Step 5: verify memory persistence), `32bf53d` (Step 5: fix wasm config), `d820ff6` (scout: add Recipes to worklist), `e41376e` (worklist: close item 3), `929899a` (Step 5: verify memory), `830033c` (record backend parity check), `2365866` (check browser round trip), `3062aba` (prove native/wasm parity), `2b02fd0` (Step 4: error recovery), `401a8a7` (Step 5: expose FrameDriver), `ce4acad` (Step 6: integrate WASM events), `2df7f1c` (Step 7: parity test)

Files touched:
- `src/shell/mod.rs` (line 415): Add `#[cfg(target_arch = "wasm32")] pub(crate) fn run()` that creates a `Page` struct holding all loop state, registers a `requestAnimationFrame` callback, and returns immediately. The callback holds `Rc<RefCell<>>` of the page state and calls `turn()` on each repaint (line 472). Add `present()` and `listen()` functions for browser integration (lines 283, 295).
- `src/shell/clock.rs`: Update to handle WASM timing edge cases; add fallback to zero duration if performance API is unavailable.
- `src/wasm.rs` (new): WASM-specific bindings for the browser. Exports `init_counter()`, `listen_counter()`, `present_counter()` that call into the generic `turn()` loop. The page's event listener (DOM click, mousemove, wheel) collects events into the `Input` queue; each frame, `turn()` consumes them.
- `src/shell/platform/wasm.rs` (new): Implement the `Backend` trait for the browser (canvas rendering, event listening, `appearance()` from `prefers-color-scheme`).

**Why this order:** The native `run()` was already done and working. WASM adds a sibling `run()` (guarded by `#[cfg(target_arch = "wasm32")]`) that uses the same `turn()` but with no `while` — it relies on the browser to call the callback repeatedly. The `Backend` trait is unchanged; only the platform-specific initialization differs. Integration testing (memory persistence, parity checks) runs throughout to catch issues early.

**Verification gates:**
- Compiled verification: `cargo build --target wasm32-unknown-unknown -p rui --example counter` succeeds.
- Unit tests: `cargo test --lib` passes; memory and animation state persist across frames.
- Browser testing: `wasm-pack test --headless --firefox` confirms the app initializes and responds to DOM events.
- Parity verification: `cargo test --test wasm_parity` generates reference frames for light and dark modes. Browser comparison happens at `examples/parity.html` (open after running `wasm-pack build`).

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
cargo test --test external_driving -- state_mut_between_frames_drives_the_next_frame -- --nocapture
```
Confirms the frame-driving abstraction works without owning an event loop. The test verifies that app state can be mutated externally between frames and persists across calls to `app.frame()`. This is the crucial gate: if frame-stepping works, both native (which loops) and WASM (which doesn't) can call the same function.

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

#### Cross-Module Concerns

**Why `shell::clock::Moment` was needed (Instant::now() panics on WASM)**

On desktop, `std::time::Instant::now()` is trivial—it calls the system's clock. On WASM (`wasm32-unknown-unknown`), there is no system clock; `Instant::now()` is compiled from an unsupported platform and panics immediately. The solution is a platform-agnostic `shell::clock::Moment` type (in `src/shell/clock.rs`) that:
- Returns `Instant` on native (via `std::time::Instant`)
- Returns milliseconds since page load on WASM (via `web_sys::Performance::now()`)
- Exposes a unified API: `Moment::now()` and `Moment::since()` that work on both.

Anything that measures elapsed time must use `shell::clock::Moment`, not `Instant` directly. This allows the same frame-stepping logic to work on both platforms.

**How `shell::clock` flows through the frame loop**

`src/shell/mod.rs` imports `shell::clock::Moment` (line 55) and uses it in two places:
1. `Surface::drawn_at` (line 199): Stores the time the previous frame was drawn as a `Moment`.
2. `Surface::draw()` (line 237): Calls `Moment::now()` to get the current time, then `self.memory.begin_frame(now.since(self.drawn_at))` (line 238) to measure elapsed time for animations.

Both desktop `run()` and WASM `run()` call the same `Surface::draw()`, so both measure time correctly without additional logic.

**Why the generic `turn()` loop works for both backends**

The key abstraction is the `Backend` trait (line 152 in `src/shell/mod.rs`):
```rust
trait Backend: Sized {
    fn open(options: &WindowOptions) -> Result<Self, Error>;
    fn pump(&mut self, timeout: Duration, events: &mut Vec<Event>, redraw: &mut dyn FnMut(&Self)) -> Result<(), Error>;
    fn surface(&self) -> (u32, u32, f32); // width, height, scale
    fn appearance(&self) -> Appearance;
    fn present(&self, canvas: &Canvas) -> Result<(), Error>;
    fn is_open(&self) -> bool;
}
```

The `turn()` function (line 325) accepts a `Backend` and calls only these six methods. It does not know or care whether the backend is native, WASM, or something else:
- Native `run()` (line 369): Loops calling `turn()` with a native `Backend`.
- WASM `run()` (line 415): Registers a callback that calls `turn()` with a WASM `Backend`.

Both drivers call identical frame logic; the only difference is how they schedule the next frame.

**Coordination points: How modules communicate**

1. **`shell/clock.rs` ↔ `shell/mod.rs`:** `Surface::draw()` measures time via `shell::clock::Moment::now()` and `since()`. This abstraction hides platform differences.

2. **`Backend` trait ↔ `turn()` function:** `turn()` is the central coordination point. It accepts any `Backend` and calls the six-method interface. Both native and WASM platforms implement `Backend`; the rendering pipeline above it is unified.

3. **Event flow:** Native `Backend` collects events from `window.pump()` (blocking wait on platform); WASM `Backend` collects from DOM listeners. Both feed `Vec<Event>` into `surface.draw()`, which passes them to the view function. The `Input` module (in `src/input`) enqueues events; the same state machine processes them on both platforms.

4. **State persistence:** `Memory` (in `src/memory`) holds hover, focus, scroll, animation, and caret state. This is queried by the frame-stepping logic in both desktop and browser loops. A change to how state is animated or persisted only needs to be coded once.

5. **Platform initialization:** `App::run()` (in `src/app.rs`) is generic over the state type `S`. It calls `shell::run()`, which has two implementations:
   - `#[cfg(not(target_arch = "wasm32"))]`: Native loop with blocking event wait
   - `#[cfg(target_arch = "wasm32")]`: Callback-based loop with `requestAnimationFrame`

The view function and all frame logic are shared between both.

#### Template for the Next Backend

Adding a new backend (e.g., native Wayland, or a game engine) follows the same pattern. Here is the replicable checklist:

1. **Add platform abstraction if needed** (`src/shell/clock.rs` was added for WASM because `Instant::now()` panics there; a new platform may need similar handling)
   - If the platform has unusual time handling, add a platform-specific branch to `shell::clock`.
   - Test with: `cargo test --lib` (confirm time is measured correctly on the new platform).

2. **Implement the `Backend` trait** (add `src/shell/platform/wayland.rs` or similar)
   - Implement all six methods: `open()`, `pump()`, `surface()`, `appearance()`, `present()`, `is_open()`.
   - The implementation is entirely platform-specific; nothing above `Backend` changes.
   - Compile check: `cargo build --features "wayland"` or similar platform gate.

3. **Wire the new backend into `shell/mod.rs`** (the platform selector, line 152)
   - Add a new `#[cfg(target_os = "...")]` branch to the `Backend` trait or platform module.
   - Ensure `src/shell/mod.rs` either imports the new platform module or gates it with a feature flag.

4. **Conditional `run()` implementation in `shell/mod.rs`** (around line 369 for native, line 415 for WASM pattern)
   - Add `#[cfg(target_os = "...")]` to a new `pub(crate) fn run<S: 'static>(...)` that uses the new backend.
   - The function body is always: create a `Surface`, loop calling `turn()` with the backend, check `continues()`, repeat.
   - If the platform cannot block (like WASM), use the callback pattern instead (reference WASM implementation at line 415).

5. **Add platform detection and initialization** (in `src/app.rs`)
   - Verify the platform is available at compile time (use `#[cfg(...)]`).
   - Add a gate to `run()` and `run_with_fonts()` if needed (e.g., require `'static` for some platforms).

6. **Verify with a quick integration test**
   - Build the example: `cargo build --target wasm32-unknown-unknown -p rui --example counter` (or native equivalent).
   - Run the example and confirm basic interaction works (click, keyboard, mousemove).

7. **Add parity test** (in `examples/parity.rs` or `tests/integration.rs`)
   - Render the reference frame to pixels using the new backend.
   - Compare against a known-good frame (rendered by the native backend) to confirm pixel-perfect identical rendering.
   - Test both light and dark modes.

8. **Document the backend** (in `CLAUDE.md`, update the module index table to list the new platform file)
   - Add a row to the module structure table mentioning the new backend follows the `Backend` trait pattern.
   - Link to the recipe that shows how backends are added.

**Spot-check against WASM:**

- WASM's `src/shell/platform/wasm.rs`: Implements `Backend` trait entirely in platform-specific code. All six trait methods are implemented.
- WASM's `shell/mod.rs` conditional `run()` (line 415): Creates a `Page` struct, registers a callback, and returns. Calls `turn()` from the callback (line 472).
- WASM's `src/wasm.rs`: Exports WASM-specific functions (`init_counter`, `listen_counter`, `present_counter`) that drive the page loop. These are not part of the generic pattern; they are how the browser calls into Rust.

The pattern holds: add a platform module implementing `Backend`, wire it into the selector, add a `run()` that calls `turn()`, verify with parity tests. Everything above `Backend` is unchanged.

### Recipe 2: X11 Backend Implementation

**Commits:** 10 total, grouped in three phases: Foundation (1 commit), Enhancement (1 commit), Platform Integration (8 commits).

The X11 backend brings native Linux support to **rui**, allowing the same UI code to run in an X11 session with no changes. X11 is a legacy but still-ubiquitous display server on Linux; supporting it alongside Wayland requires careful platform abstraction and coordinate contract documentation.

**Phase 1: Foundation (Commit a67d578)**

Files touched:
- `src/shell/platform/x11.rs` (new): Implement the `Backend` trait for X11. FFI bindings via `xlib` or hand-rolled X11 protocol to `XOpenDisplay`, `XCreateWindow`, `XSelectInput`, `XNextEvent`. Translate X11 events to rui's unified `Event` type. `present()` renders the `Canvas` to `XImage` and calls `XPutImage` to blit pixels to the window.
- `src/shell/mod.rs`: Conditionally select X11 backend with `#[cfg(target_os = "linux")]`.

**Why this order:** The `Backend` trait is the platform abstraction boundary. X11 must implement all six methods (`open`, `pump`, `surface`, `appearance`, `present`, `is_open`) identically to native backends. By starting with the minimal trait implementation, we prove X11 can participate in the unified frame loop without special casing.

**Verification gate:** `cargo build --target x86_64-unknown-linux-gnu` succeeds. The Backend trait is correctly implemented at compile time; no X11-specific branch logic leaks above `src/shell/platform/x11.rs`.

**Phase 2: Enhancement (Commit c42c0f0)**

Files touched:
- `src/shell/platform/x11.rs`: Extend with full feature parity. Canvas rendering (vector lines, shapes), font loading from system directories, text layout with kerning, DPI scaling via `XDisplayWidth` and `XDisplayWidthMM` (physical width). Appearance detection: read `COLORFTERM` environment variable or `_NET_WM_APPEARANCE` window manager property to detect light/dark mode. Input event translation: map X11 KeySym to rui's `Key` enum, translate ButtonPress/ButtonRelease to Click, MotionNotify to Drag.
- `src/text.rs`, `src/paint.rs`, `src/canvas.rs` (if needed): Ensure font loading and rendering work on X11 (use system font paths like `/usr/share/fonts` if not embedded).
- `tests/x11_integration.rs` (new): Integration tests verifying the app responds to X11 events (click, drag, keyboard input) and renders correctly.

**Why this order:** Foundation proves X11 can hook into the loop. Enhancement ensures the app looks and behaves identically to native backends. Parity testing (visual comparison, event handling verification) gates this phase.

**Verification gates:**
- `cargo test --test x11_integration` passes; app responds to simulated X11 events.
- Visual inspection: `cargo run -p rui --example counter` on an X11 system renders identically to macOS/Windows.
- Appearance detection: `COLORFTERM=truecolor cargo run --example counter` and toggle theme; the UI switches modes correctly.
- DPI scaling: Connect two monitors with different DPI and verify scaling adapts (element sizes remain proportional).

**Phase 3: Platform Integration & Refinement (Commits 80e3003 through 84ade0e — 8 commits)**

Commits in this phase: Coordination, documentation, parity verification.

Files touched:
- `src/shell/mod.rs`: Add `EventLoopDriver` trait (or feature gate) to handle platform-specific event loop requirements. WASM cannot block; native loops can. X11 uses XNextEvent (blocking) like macOS/Windows but with different timeout handling (X11 select() vs. platform-native waits). Unify timeout semantics across backends.
- `src/shell/platform/x11.rs`: Document coordinate contract: window coordinates vs. screen coordinates, DPI scaling factor application, event coordinate translation. Verify X11 event coordinates are correctly mapped to logical pixels after DPI scaling.
- `tests/x11_parity.rs`: Pixel-perfect comparison tests. Render the same scene on X11 and native backend; compare PNG output to verify zero differing pixels (light and dark modes).
- `CLAUDE.md` (update): Document X11 setup in Troubleshooting section; explain `DISPLAY` variable, X11 development header requirements, how to verify X11 backend builds.
- `src/app.rs`: If needed, add platform-specific initialization (e.g., X11-specific font paths, locale setup).

**Why this order:** Once X11 renders correctly and responds to events, the focus shifts to platform consistency. The EventLoopDriver abstraction ensures timeout semantics are unified (so a 60fps refresh works on all platforms). Parity tests catch rendering differences; coordinate contract documentation prevents future bugs from DPI scaling or event translation mistakes. Finally, updating user-facing docs ensures downstream developers know how to set up and troubleshoot X11.

**Verification gates:**
- Compiled verification: `cargo build --target x86_64-unknown-linux-gnu` succeeds with no warnings.
- Timeout semantics: App maintains 60fps on X11 at same CPU cost as other backends (profile with `perf` to verify select() timeout is tuned correctly).
- Parity testing: `cargo test --test x11_parity` generates reference frames on X11 and compares to native renders. Green page (0 differing pixels) in both light and dark modes.
- Coordinate contract: Document pixel-perfect coordinate translation. Test that a click at screen position (100, 200) with 2x DPI triggers a Click event at logical (50, 100).
- Documentation completeness: Verify `CLAUDE.md` Troubleshooting section covers common X11 issues (no display, missing headers, permission errors).

#### Cross-Module Concerns

**Why coordinate contract matters (DPI scaling, event translation)**

X11 reports screen coordinates (physical pixels), but rui works in logical pixels (DPI-independent). A 1920×1080 monitor at 2x DPI is 960×540 logical pixels. When the user clicks at physical (200, 200), the X11 backend must translate to logical (100, 100) before passing the `Click` event to the frame.

Mistakes here are subtle: the app renders at the correct size but clicks register in the wrong place. The coordinate contract (documented in `src/shell/platform/x11.rs` and verified in `tests/x11_parity.rs`) prevents this:
- **Physical → Logical:** Divide by DPI scale factor.
- **Logical → Physical:** Multiply by DPI scale factor.
- **Test:** Verify click-to-element correspondence: click the "Increment" button and confirm the state changes, for several DPI scales.

**Why appearance detection requires fallback**

Not all X11 sessions have `_NET_WM_APPEARANCE` (set by modern window managers). The fallback chain:
1. Query `_NET_WM_APPEARANCE` (modern: GNOME, KDE Plasma).
2. Check `COLORFTERM` environment variable (user may set to `truecolor` for dark shells).
3. Default to light (most conservative; many systems have light terminals by default).

The appearance is read once at `open()` time and re-queried on window manager signals (if needed). Test with `COLORFTERM=truecolor` to verify fallback works.

**How X11 ties into the frame loop**

`src/shell/mod.rs` line 369 (native `run()`) calls `pump()` with a timeout (typically `Duration::from_millis(8)` for 60fps). On X11:
- `pump()` calls `XNextEvent()` with a timeout (or select() on the X11 connection FD).
- If no event arrives, the frame is still redrawn (animations continue).
- If events arrive, they're translated and added to the `events` vector.

Both native and X11 loops use the same timeout-driven polling; only the platform-specific event collection differs.

#### Template for Adding X11 or Similar Backends

If you're adding Wayland, macOS, or another platform:

1. **Implement the `Backend` trait entirely in `src/shell/platform/new_platform.rs`** — Keep platform code isolated.
   - All six methods must be present: `open`, `pump`, `surface`, `appearance`, `present`, `is_open`.
   - Use platform-native FFI (wayland-client crate for Wayland, native Win32 for Windows, etc.).
   - Translate platform events to rui's unified `Event` type.

2. **Add a conditional in `src/shell/mod.rs`** — Wire the backend into the platform selector.
   - Use `#[cfg(target_os = "...")]` or feature gates to select the backend at compile time.

3. **Verify coordinate contract** — Document and test DPI scaling, event coordinate translation.
   - Create `tests/new_backend_parity.rs` with pixel-perfect comparison.
   - Compare renders on the new platform to a known-good backend.

4. **Appearance detection** — Read light/dark preference from the platform.
   - Implement `appearance()` to query the platform's theme setting.
   - Provide fallback if the setting is unavailable.

5. **Update CLAUDE.md** — Document setup, troubleshooting, and any platform-specific requirements.
   - Add a section in Troubleshooting for the new platform.
   - Explain environment variables, dependencies, or permissions required.

6. **Test thoroughly** — Run the full suite on the new platform.
   - `cargo build --target new_platform` succeeds.
   - `cargo test --target new_platform` passes all tests.
   - Parity test: `cargo test --test new_backend_parity` shows 0 differing pixels.
   - Visual inspection: Run `examples/counter` and interact with it; verify behavior is identical to other platforms.

**Spot-check against X11:**

- X11's `src/shell/platform/x11.rs`: Implements `Backend` trait entirely in platform-specific code.
- X11's conditional in `src/shell/mod.rs`: Selects X11 backend with `#[cfg(target_os = "linux")]`.
- X11's parity test: `tests/x11_parity.rs` generates reference frames and compares to native (macOS/Windows).
- X11's appearance detection: `COLORFTERM` env var + fallback to light.
- X11's coordinate contract: Documented in platform code; tests verify click→element correspondence at various DPI scales.

The pattern holds: platform isolation (one file per OS), trait implementation (six methods), coordinate contract (DPI scaling + event translation), parity testing (pixel-perfect comparison), and documentation (setup + troubleshooting). Everything above `Backend` is unchanged.

### Recipe 3: Checkbox Control

**Commits:** 1 total, focused on state definition and pattern foundation.

The checkbox is the simplest interactive control: a boolean that toggles on click. Unlike segmented or complex widgets, checkbox has no enumerated state—just checked or unchecked. This simplicity makes it an ideal recipe for learning the state-view-handler pattern. It is too small to justify being a full recipe (just 20 lines of code), but too foundational not to document as a worked example.

**Phase 1: State Definition (Commit a_checkbox_changes_state_on_click)**

Files touched:
- `src/widgets.rs`: Add `checkbox(label: &str, checked: bool, on_click: impl Fn(&mut S)) -> El<S>` function that builds a checkbox element from primitives (`draw`, `on_click`). No new widget type; reuse existing drawing and event handling infrastructure.
- `tests/recipes.rs`: Add test `a_checkbox_changes_state_on_click` verifying state toggles on click.

**State definition:**
```rust
struct App {
    checked: bool,
}
```

The state is a single boolean. No enum, no tagged variant, no Option. Checkbox is binary: ticked or not. This is why checkbox is the minimal interactive control—it requires only one boolean field.

**Why this order:** State definition is the foundation of the state-view-handler pattern. By starting with the simplest possible state (a single bool), we prove that even the smallest interactive control follows the same pattern. The checkbox demonstrates that state complexity is orthogonal to control complexity; a toggle is just as valid a state shape as a segmented choice or a slider position.

**Verification gate:** `cargo test --test recipes -- a_checkbox_changes_state_on_click` passes. The test verifies that clicking a checkbox with `checked: false` produces a new frame with `checked: true`, and vice versa. The test uses `Harness` to drive the frame without an event loop; no window is needed.

**Handler and integration:**
```rust
fn view(app: &App) -> El<App> {
    col((
        text("Enable notifications:"),
        widgets::checkbox("Notifications on", app.checked, |app: &mut App| {
            app.checked = !app.checked;
        }),
    ))
}
```

The handler is a closure that receives mutable state as an argument and modifies it (toggling the boolean). This is identical to the segmented and meter patterns; only the state shape changes. The checkbox is reusable: call it with any boolean field in any app state, and it works.

**Cross-module notes:**

The checkbox implementation touches only two modules:
1. **`src/widgets.rs`:** Builds the checkbox element from `draw()` and `on_click()` primitives. No new draw code; reuses existing shapes and colors. Line ~~350–370~~ (estimate; keep actual implementation tight).
2. **`tests/recipes.rs`:** Adds one test verifying the toggle behavior. Uses `Harness` to click the checkbox and assert state changed.

Both changes are shallow: `widgets.rs` adds a new function that composes existing primitives, and `tests/recipes.rs` adds one test. Nothing in the rendering, layout, or event handling pipeline changes.

## Workflow Notes

- **Unsafe code:** Confined to `shell/platform/*.rs` (one file per OS). Everything above that—elements, layout, rendering, fonts—is safe Rust.
- **No dependencies:** When adding features, build within the crate. Resist reaching for external crates.
- **Identity & keys:** Elements get unique identity from their path in the tree. For lists that reorder, use `.key(&item.id)` to preserve state (hover, focus, scroll) across frame rebuilds.
- **Appearance (light/dark mode):** Read via `prefers-color-scheme` media query in the `Backend`. Test both modes in examples.
- **Text inherits; layout does not:** `text_size`, `color`, `face` flow to children; `pad`, `gap`, `fill` do not (a child padding itself is usually surprising).

## Git & CI

- **Hook:** Pre-commit runs `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. Bypass with `git commit --no-verify` only for emergencies (then fix the hook cause).
- **Cache/state files ignored:** `.cache/`, `.doc/index.db`, `.engine/` (build artifacts) are in `.gitignore`; working directory must stay clean.
- **Commits:** Prefix with the platform or feature touched (e.g., "Add X11 input handling", "Refactor layout engine", "Improve text rendering").

## Troubleshooting

### Build & Compilation

**Problem:** `error: could not compile rui`

- **Check Rust version:** Run `rustc --version`. Minimum is 1.85. Update with `rustup update`.
- **Check dependencies:** Run `cargo tree` to inspect dependency graph. rui has zero dependencies; unexpected crates indicate a configuration issue.
- **Clean build artifacts:** Run `cargo clean` and retry. This resolves stale cache issues.

**Problem:** `error: failed to resolve: use of undeclared crate`

- **Verify your current directory:** Run `pwd` to confirm you're in `/Users/alexwaldmann/Desktop/rui`. The project root contains `Cargo.toml`.
- **Verify Cargo.toml exists:** Run `ls -la Cargo.toml`. If missing, you're not in the project root.

### Tests

**Problem:** `cargo test --lib` fails with "test failures"

- **Read the failure message:** Each test includes an `assert!()` or assertion that tells you exactly what is failing.
- **Run a single test:** Use `cargo test --lib test_name` to isolate and debug one test in detail.
- **Example:** `cargo test --lib geometry` runs only geometry-related unit tests.

**Problem:** `cargo test --test setup` fails

- **Ensure clean git state:** Run `git status`. If it shows unstaged changes, stash them with `git stash`.
- **Run hook manually:** Run `bash .git/hooks/pre-commit` to see the exact error. The hook checks formatting and lints.
- **Fix formatting:** If the hook complains about formatting, run `cargo fmt` to auto-fix.
- **Run clippy to fix lints:** Run `cargo clippy --fix --allow-staged` to auto-fix linter warnings.

### Examples

**Problem:** Example fails to build or run

- **Verify the example exists:** Run `ls examples/` to list all examples. Spelling must match exactly.
- **Run with output:** Use `cargo run -p rui --example counter 2>&1` to see stderr if anything goes wrong.
- **Check platform requirements:** macOS examples need macOS, X11 examples need an X11 server, WASM examples need a browser and wasm-pack.

### Platform-Specific Setup

**macOS**

- No additional setup required beyond Rust and Xcode Command Line Tools.
- Verify Cocoa backend with: `cargo build` (native build defaults to macOS on Cocoa systems).

**Windows**

- No additional setup required beyond Rust (WinAPI is part of Windows SDK, linked by MSVC toolchain).
- Verify WinAPI backend with: `cargo build` (native build defaults to Windows on Windows systems).

**Linux (X11)**

- **Requires X11 server:** Verify with `echo $DISPLAY`. If empty, X11 is not running. Start it or ensure you're in an X11 session (not Wayland, yet).
- **X11 development headers:** Some distributions require X11 development headers. Install with `sudo apt-get install libx11-dev` (Ubuntu/Debian) or `sudo yum install libX11-devel` (RHEL/Fedora).
- **Verify X11 backend builds:** `cargo build --target x86_64-unknown-linux-gnu` should succeed. If compilation fails with "X11 library not found", install development headers above.
- **Cannot open display:** If you get `Error: cannot open display: (nil)`, ensure you're in an X11 session or set `DISPLAY=:0` before running examples.

**Problem:** "cannot open display" or "X connection broken" errors on Linux

- **Check X11 is running:** Run `echo $DISPLAY`. Should output `:0`, `:1`, etc. If empty, X11 is not available.
- **Verify XServer installation:** On headless systems, install Xvfb (X Virtual Framebuffer) with `sudo apt-get install xvfb` (Ubuntu/Debian).
- **Run in Xvfb:** `xvfb-run -a cargo run -p rui --example counter` to run without a physical display.

### WASM Backend

**Problem:** `wasm-pack build` fails with "wasm target not found"

- **Install WASM target:** Run `rustup target add wasm32-unknown-unknown`.
- **Verify wasm-pack is installed:** Run `wasm-pack --version`. Install if missing: `curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh`.

**Problem:** WASM browser example shows blank canvas or no interaction

- **Check browser console:** Open the browser's developer tools (F12) and look for JavaScript errors. They appear in the **Console** tab.
- **Verify serving locally:** Examples served from `file://` URLs won't work. Use `python3 -m http.server 8000` to serve locally.
- **Test in Firefox:** Firefox is the primary test browser for parity verification. Chrome and Safari may have subtle rendering differences.

### Performance & Debugging

**Problem:** Application is slow or rendering is stuttering

- **Use `--release` build:** Debug builds are much slower. Run `cargo build --release` and `cargo run -p rui --example counter --release` for optimized performance.
- **Check for infinite loops:** If the app hangs, it may be stuck in the view function or an event handler. Verify handlers eventually return.
- **Profile with Xcode Instruments:** On macOS, use `cargo build --release && open -a Instruments ./target/release/rui` to profile the app.

### Getting Help

- **Check git history:** Run `git log --oneline` to see recent commits. The commit messages document what changed and why.
- **Search for similar issues:** Run `grep -r "error message" src/` to find where an error is thrown.
- **Read test examples:** Look at `tests/recipes.rs` for working examples of each widget and pattern.
