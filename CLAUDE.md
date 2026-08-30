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

- **Rust 1.85+** (Edition 2021), verified by `tests/setup.rs`. Use `rustup update` if needed.
- **No external dependencies**—the full renderer, font handling, and window management are in this crate.
- **Platforms:** macOS (Cocoa), Windows (WinAPI), X11/Wayland (via X11 server).
- **Pre-commit hook:** Runs `cargo fmt --check` and `cargo clippy` (`.git/hooks/pre-commit`). Executable after first git setup.

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
cargo test --test integration                    # Run integration tests

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

The `segmented` widget is a minimal, self-contained exemplar showing how to build an interactive choice selector. It is small enough (33 lines) to copy and modify immediately.

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
- Change `Tone::Accent` to `Tone::Success`, `Tone::Warning`, etc. for different colors
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

### Recipe 2: Add a New Widget

**Commits:** 1 per verification gate; typically 3–5 total.

A widget is a function that builds an `El<S>` from application state and wires handlers to mutate it. The simplest widgets (like `meter`) are passive display; more complex ones (like `segmented`) handle clicks or drags. Both follow the same pattern: state → view → handlers.

**Files Touched:**
- `src/widgets.rs`: Define the widget function
- `tests/recipes.rs`: Add a test using `Harness`
- `examples/` (optional): Add a standalone example

**Steps in Order:**

1. **Decide on state and appearance** — What does the widget display? What user interaction does it need? (e.g., "a choice between options" → state is `selected: usize`)

2. **Write the widget function in `src/widgets.rs`** — Build an `El<S>` from primitives. Use `draw()` for custom shapes, `row()`/`col()` for layout, `.on_click()` / `.on_drag()` for handlers. No closures; handlers receive `&mut S`.

3. **Add a test in `tests/recipes.rs`** — Use `Harness::new(state, view)` to drive the widget with no window. Call `harness.click_text()` or other interactions and assert state changed as expected.

4. **Verify `cargo test --test recipes -- widget_name` passes** — The test proves the widget responds to input and updates state correctly.

5. **Run an example to confirm visually** — If interactive, add a minimal example in `examples/` or use the counter example with your widget in the view.

**Verification Gates:**

- `cargo test --test recipes -- widget_name` passes
- `cargo build` succeeds with no warnings
- Widget renders at all state values (test 0%, 50%, 100% for progress; all options for choice)

**How the Pattern Scales:**

- **Simple, passive (like `meter`):** Just draw; no handlers. State flows in, pixels flow out.
- **Interactive (like `segmented`):** Add `.on_click()` handlers. Handler closures take `&mut S` and mutate state. Handlers run *after* the frame is drawn, so no re-entrancy.
- **Complex (like `tabs`):** Combine layout (`row` of buttons), styling (highlight current), and handlers (call user's choice callback). Still built from primitives; no special support needed.

**Cross-Module Coordination:**

If your widget uses `draw()`, it needs `Painter` (from `paint::Painter`) to fill shapes and stroke outlines. The painter is stateful: colors are looked up via `painter.color(tone)` to respect themes (light/dark). If your widget has text, inherit `text_size` and `color` from the parent—these flow automatically to children. If your widget has interactive state (hover, focus, caret), it lives in `memory::Memory`, keyed by element identity; preserve element identity across frame rebuilds with `.key()` for reordered lists.

#### End-to-End Example: Building a Custom Widget

Here's how to add a new widget following Recipe 2:

**1. Choose state and appearance:**
Suppose you want a `star_rating` widget: display 1–5 stars, clickable to set the rating.

```rust
struct App {
    rating: usize,  // 0..=5
}
```

**2. Write the function in `src/widgets.rs`:**
```rust
pub fn star_rating<S: 'static>(
    rating: usize,
    set_rating: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    row((1..=5).map(|i| {
        let filled = i <= rating;
        draw(Size::new(16.0, 16.0), move |painter: &mut Painter<'_>, rect: Rect| {
            let color = if filled { Tone::Accent } else { Tone::Muted };
            painter.fill(rect, Radius::None, painter.color(color));
        })
        .key(format!("star-{}", i))
        .on_click(move |state: &mut S| set_rating(state, i))
    }).collect::<Vec<_>>())
    .gap(4.0)
}
```

**3. Add a test in `tests/recipes.rs`:**
```rust
#[test]
fn a_star_rating_updates_when_clicked() {
    let mut harness = Harness::new(App { rating: 0 }, |app: &App| {
        col(star_rating(app.rating, |app: &mut App, r| app.rating = r))
    });
    harness.click(Point::new(48.0, 8.0)); // Click the 4th star
    assert_eq!(harness.state().rating, 4);
}
```

**4. Run `cargo test --test recipes -- star_rating`** — Test passes ✓

**5. Verify visually** (optional):
Add to your view function or the counter example:
```rust
col((
    text("Rate this:"),
    star_rating(app.rating, |app: &mut App, r| app.rating = r),
))
```

Done. The widget is ready to use anywhere state is a Rust struct with a `rating` field.

## Workflow Notes

- **Unsafe code:** Confined to `shell/platform/*.rs` (one file per OS). Everything above that—elements, layout, rendering, fonts—is safe Rust.
- **No dependencies:** When adding features, build within the crate. Resist reaching for external crates.
- **Identity & keys:** Elements get unique identity from their path in the tree. For lists that reorder, use `.key(&item.id)` to preserve state (hover, focus, scroll) across frame rebuilds.
- **Appearance (light/dark mode):** Read via `prefers-color-scheme` media query in the `Backend`. Test both modes in examples.
- **Text inherits; layout does not:** `text_size`, `color`, `face` flow to children; `pad`, `gap`, `fill` do not (a child padding itself is usually surprising).

## Git & CI

- **Hook:** Pre-commit runs `cargo fmt --check` and `cargo clippy`. Bypass with `git commit --no-verify` only for emergencies (then fix the hook cause).
- **Cache/state files ignored:** `.cache/`, `.doc/index.db`, `.engine/` (build artifacts) are in `.gitignore`; working directory must stay clean.
- **Commits:** Prefix with the platform or feature touched (e.g., "Add X11 input handling", "Refactor layout engine", "Improve text rendering").
