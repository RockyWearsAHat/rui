# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rui** is a declarative interface library for Rust with **zero dependencies**. It unifies structure (layout), style (appearance), and behavior (interaction) into a single Rust expression, rendered by its own TrueType parser, glyph rasteriser, and platform-specific window backends (macOS, Windows, X11). Wayland support is planned for v0.2.0.

**Core design principles:**
- **View is a pure function of state.** The `view` function rebuilds the entire UI description from application data each frame—no retained widget tree.
- **Handlers are functions of state, not closures.** `on_click(|app: &mut App| …)` receives mutable state as an argument, eliminating `Rc`, `RefCell`, and interior mutability.
- **Roles, not values.** Colors are named by semantic role (`Tone::Surface`, `Tone::Muted`), so the same description works in light and dark modes.
- **Foundations, not a catalogue.** The library provides primitives (`draw`, `on_drag`, `on_key`, `layer`) for building custom controls—no built-in checkbox or slider, because those constrain design.

## Setup & Requirements

- **Rust 1.85+** (Edition 2021), verified by `tests/setup.rs`. Use `rustup update` if needed.
- **No external dependencies**—the full renderer, font handling, and window management are in this crate.
- **Platforms:** macOS (Cocoa), Windows (WinAPI), X11 (Linux). Wayland support planned for v0.2.0.
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
| `segmented` | Exemplar: a minimal, self-contained choice selector (33 lines). Copy and modify to build new interactive controls. |
| `meter` | Exemplar: a passive progress bar showing how to build read-only widgets. |
| `calculator` | Numeric input handling and button grid layouts; demonstrates stateful computation. |
| `theme_switcher` | Light/dark mode support showing how appearance preferences flow through the entire UI. |
| `todo_app` | List rendering with state management; demonstrates item creation, completion toggling, and list updates. |
| `form_example` | Comprehensive form with text input, select dropdown, and checkbox widgets; demonstrates form control integration and state flow. |
| `parity` | Builds a native reference frame for pixel-perfect WASM backend comparison. |
| `icon` | Generates macOS `.iconset` and `.icns` app icons by drawing them at all required sizes. |
| `segmented_modified` | Verification that the documented "Copy and Modify" path from CLAUDE.md actually works. |

**Learning Path:** Start with `counter`, then `segmented` (to understand handlers), then `meter` (to understand passive widgets). Continue with `calculator` (numeric input and multi-step computation), `theme_switcher` (appearance and semantic colors), `todo_app` (list rendering and state management), and `form_example` (form controls: text input, select, checkbox). Use `controls` to see all available widgets. The `gallery` example renders all elements to PNG for visual verification.

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

### Accessibility Framework (a11y)

The `El<S>` API includes three optional fields for exposing semantic information to assistive technologies (screen readers, voice control, etc.):

- **`accessible_name`**: The primary label describing the element (e.g., "Submit" for a button, "Email address" for a text input).
- **`accessible_role`**: The semantic role of the element (e.g., "button", "input", "navigation", "main", "heading", "image"). Helps assistive technologies understand the element's purpose.
- **`accessible_description`**: Additional context beyond the accessible name (e.g., "Form submission is irreversible" for a delete button).

**Usage in practice:**

```rust
col((
    text("Username:")
        .accessible_role("label"),
    draw(Size::new(200.0, 32.0), |painter, rect| {
        // Custom text input
    })
    .accessible_name("Username")
    .accessible_role("input")
    .accessible_description("Enter your account username (3-20 characters)"),
    button("Submit")
        .accessible_name("Submit form")
        .accessible_role("button")
        .accessible_description("Submit the form to create your account"),
))
```

All three fields are optional (default to `None`). Elements without an accessible name fall back to their visible text content. Widgets that need semantic meaning should set all three fields to provide complete context to assistive technologies.

**Platform backend implementation (future work):**

Platform backends (macOS/Windows/X11/WASM) will implement handlers to expose these fields to system accessibility APIs:
- **macOS (VoiceOver):** Export to NSAccessibility attributes (`accessibilityLabel`, `accessibilityRole`, `accessibilityHelp`).
- **Windows (Narrator):** Export to UIA (UI Automation) properties (`Name`, `ControlType`, `HelpText`).
- **Linux (Orca):** Export to ATK (Accessibility Toolkit) attributes via DBus.
- **WASM (browser screen readers):** Set ARIA attributes (`aria-label`, `role`, `aria-description`) on DOM elements.

The framework is transport-agnostic; the three fields carry semantic meaning, and each backend translates them to its native accessibility model.

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

### State Shape Guidance

**Core Pattern:** Your application state is a flat struct where each field represents widget state directly. No `Rc<RefCell<>>`, no wrapper enums, no "model-view-controller" layering—just data fields that handlers mutate.

**Why this works:** The view function is called every frame from the event loop. Because it receives `&App` (immutable state), it rebuilds the UI description deterministically. Handlers receive `&mut App` and run after the frame is drawn, so state mutations never race the view. This eliminates interior mutability entirely.

**See the segmented exemplar** (in `examples/segmented.rs` and `src/widgets.rs` line 333–365) for a minimal, worked example of this pattern: state struct → view function → handler closure. Copy and modify it to build your own widgets.

#### Example: Text Input Widget State

A text input widget needs two pieces of state:

```rust
State:   struct App { input_text: String, input_focused: bool }
View:    fn view(app: &App) -> El<App> { ... }
Handler: |app: &mut App, new_text| { app.input_text = new_text; }
```

Concretely:
```rust
struct App {
    input_text: String,     // The current text in the field
    input_focused: bool,    // Whether the input has keyboard focus
}
```

The `input_text` field holds what the user typed. The `input_focused` field is managed by the framework's memory system (hover, focus, caret state lives in `src/memory`), but you include it in your struct so the view can decide appearance (e.g., draw a border if focused).

The view function reads both fields:
```rust
fn view(app: &App) -> El<App> {
    widgets::text_input(
        &app.input_text,
        app.input_focused,
        |app: &mut App, new_text| {
            app.input_text = new_text;
        },
    )
}
```

The handler receives mutable state and updates the field directly. No closures capturing references, no interior mutability tricks.

#### Example: A Choice Widget State

From the segmented exemplar (line 223–226):

```rust
State:   struct App { selected: usize }
View:    fn view(app: &App) -> El<App> { ... }
Handler: |app: &mut App, index| { app.selected = index; }
```

The state is a single field—the index of the selected choice. The handler is just `app.selected = index`. No wrappers, no indirection.

#### Pattern for Complex Widgets

For a widget like a form with multiple fields:

```rust
State:   struct App { name: String, email: String, terms_accepted: bool, submit_error: Option<String> }
View:    fn view(app: &App) -> El<App> { ... }
Handler: |app: &mut App, field, value| { ... }
```

Concretely:
```rust
struct App {
    name: String,           // First input
    email: String,          // Second input
    terms_accepted: bool,   // Checkbox
    submit_error: Option<String>,  // Validation error (optional because it may be empty)
}
```

Each field is directly part of the app struct. The view function reads all of them:
```rust
fn view(app: &App) -> El<App> {
    col((
        text_input(&app.name, |app, v| app.name = v),
        text_input(&app.email, |app, v| app.email = v),
        checkbox(app.terms_accepted, |app| app.terms_accepted = !app.terms_accepted),
        if let Some(error) = &app.submit_error {
            text(error).fill(Tone::Bad)
        } else {
            text("").height(0.0)  // Spacer when no error
        },
    ))
}
```

Handlers update fields inline. No manager struct, no separate "state manager" object.

#### Anti-patterns to Avoid

**Don't use `Rc<RefCell<>>` or interior mutability:**
```rust
// ❌ WRONG
struct App {
    input: Rc<RefCell<String>>,  // Overcomplicated
}
```
This adds runtime overhead and obscures when mutations happen. The event loop already ensures handlers run serially, so `&mut App` is enough.

**Don't separate "model" from "view state":**
```rust
// ❌ WRONG
struct Model { data: String }
struct ViewState { focused: bool }
struct App { model: Model, view: ViewState }
```
This layering buys nothing; it makes the struct harder to read and forces you to remember which fields belong where. Put everything in `App`.

**Don't use enums to represent widget state:**
```rust
// ❌ WRONG
enum InputState {
    Empty,
    Editing(String),
    Submitted(String),
}
struct App { input: InputState }
```
This forces unnecessary pattern matching in the view function. Just use fields: `text: String, submitted: bool`.

#### Guideline: When to Add State

Add a field to `App` when:
- The value determines what the view renders (e.g., `selected: usize` changes which button is highlighted)
- The value should persist across frames (e.g., `input_text: String` is preserved when the view rebuilds)
- A handler needs to mutate it (e.g., `app.input_text = new_text` in the text input handler)

Don't add a field if:
- The value is computed from other fields (compute it inline in the view)
- The value is temporary and not needed after the current frame (use a local variable)
- The framework already manages it (e.g., hover state is in `src/memory`, keyed by element identity)

Reference the segmented exemplar (line 217–284) for a minimal, complete example: state struct, view function, handler closure, test, and verification.

### Draw and Painter Patterns

The `draw()` primitive and `Painter` API let you render custom shapes, text, and interactive elements directly to the canvas. Unlike high-level widgets, `draw()` gives you pixel-level control while still respecting themes (light/dark modes, semantic colors).

**Pattern at a Glance:**

`draw()` takes a size and a closure that receives a mutable `Painter` reference and a `Rect`. The Painter can fill shapes, stroke outlines, and draw text. All colors flow from the theme, so your custom widget inherits light/dark mode support automatically.

```rust
draw(Size::new(160.0, 32.0), move |painter: &mut Painter<'_>, rect: Rect| {
    // Fill background
    painter.fill(rect, Radius::Small, Tone::Surface);
    // Stroke border
    painter.stroke(rect, Radius::Small, 1.0, Tone::Muted);
    // Draw text inside
    painter.text(rect.inset(4.0), Ink::default(), Align::Start, "Hello");
})
```

**Why Painter?**

The `Painter` struct (in `src/paint.rs`) abstracts the rendering backend. It exposes a high-level API (`fill()`, `stroke()`, `text()`) while the underlying `Canvas` (in `src/canvas.rs`) handles pixel rasterisation and platform differences. This separation means you write Painter code once and it works on all platforms (macOS, Windows, X11, WASM).

**Painter API Methods:**

- **`painter.fill(rect, radius, tone)`** — Fill a rectangle with rounded corners using a semantic color tone. The Painter looks up the tone in the current theme (light/dark) to get the actual RGB color.
- **`painter.stroke(rect, radius, width, tone)`** — Draw an unfilled border around a rectangle with the given width and color.
- **`painter.text(rect, ink, align, text)`** — Draw one line of text inside a rectangle. `Ink` specifies size, color (tone), and font face; `Align` controls horizontal alignment (Start, Center, End, Stretch). Text is clipped if it doesn't fit.
- **`painter.color(tone)`** — Get the RGB(A) color for a given tone in the current theme. Use this if you need raw pixels instead of high-level Painter methods.

**Example: Text Input Field with Painter**

Building a text input field demonstrates the full pattern: state (text and caret position), rendering (background, border, text, cursor), and interaction (keyboard input).

State:
```rust
struct App {
    input_text: String,
    input_caret: usize,  // position of the text cursor
    input_focused: bool,
}
```

View and rendering with Painter:
```rust
fn view(app: &App) -> El<App> {
    let input_width = 200.0;
    let input_height = 32.0;

    draw(Size::new(input_width, input_height), {
        let text = app.input_text.clone();
        let caret = if app.input_focused { app.input_caret } else { usize::MAX };
        move |painter: &mut Painter<'_>, rect: Rect| {
            // Background: lighter tone when focused
            let bg_tone = if app.input_focused { Tone::Surface } else { Tone::Sunken };
            painter.fill(rect, Radius::Small, bg_tone);

            // Border: accent color when focused
            let border_tone = if app.input_focused { Tone::Accent } else { Tone::Muted };
            painter.stroke(rect, Radius::Small, 1.0, border_tone);

            // Text content rendered with Painter
            let text_rect = rect.inset(Insets::horizontal(8.0));
            let ink = Ink { tone: Tone::Foreground, ..Ink::default() };
            painter.text(text_rect, ink, Align::Start, &text);

            // Caret (blinking cursor, visible when focused)
            if app.input_focused && caret < text.len() {
                let caret_x = text_rect.x + (caret as f32 * 8.0); // approximate char width
                let caret_rect = Rect::new(caret_x, text_rect.y, 1.0, text_rect.h);
                painter.fill(caret_rect, Radius::None, Tone::Foreground);
            }
        }
    })
    .key("text-input")
    .on_focus(|app: &mut App| app.input_focused = true)
    .on_blur(|app: &mut App| app.input_focused = false)
    .on_key(|app: &mut App, key: Key, text: Option<char>| {
        match key {
            Key::Backspace if app.input_caret > 0 => {
                app.input_text.remove(app.input_caret - 1);
                app.input_caret -= 1;
            }
            Key::Delete if app.input_caret < app.input_text.len() => {
                app.input_text.remove(app.input_caret);
            }
            Key::Left if app.input_caret > 0 => app.input_caret -= 1,
            Key::Right if app.input_caret < app.input_text.len() => app.input_caret += 1,
            _ if let Some(ch) = text => {
                app.input_text.insert(app.input_caret, ch);
                app.input_caret += 1;
            }
            _ => {}
        }
    })
}
```

**Key Points:**

1. **Painter is platform-agnostic.** Use `painter.fill()`, `painter.stroke()`, `painter.text()` without worrying about X11 vs. macOS vs. WASM—the backend handles it.

2. **Tone flows from theme.** `Tone::Surface`, `Tone::Accent`, `Tone::Muted` automatically adapt to light/dark mode. Never hardcode RGB values in your Painter code; always use semantic tones.

3. **Identity with `.key()`** — The text input uses `.key("text-input")` to preserve focus and caret state across frame rebuilds. Without a key, focus would be lost every frame.

4. **Handlers receive state by value.** The `.on_key()` handler receives `&mut App`, so you can mutate `input_text` and `input_caret` directly. No closures, no interior mutability.

5. **Rendering is immediate.** When the Painter draws, pixels go straight to the canvas. State changes are applied in the *next* frame, so there is no re-entrancy or state inconsistency.

**Coordination with Other Modules:**

- **`src/paint.rs`**: Defines the `Painter` trait and provides the high-level API (`fill()`, `stroke()`, `text()`). This is where to read about available drawing primitives.
- **`src/canvas.rs`**: Implements pixel rasterisation and color blending. You interact with Canvas indirectly through Painter.
- **`src/text.rs`**: Handles TrueType font parsing, glyph rasterisation, and text layout. The `Painter::text()` method uses this module.
- **`src/theme.rs`**: Defines `Tone` (semantic color roles) and maps them to RGB(A) based on `Appearance` (light/dark). When you call `painter.color(tone)`, this module resolves it to a pixel color.
- **`src/memory.rs`**: Stores interactive state (focus, hover, scroll, caret position) keyed by element identity. Use `.key()` to preserve state across rebuilds.

**Common Patterns:**

- **Progress bar**: Use `painter.fill()` twice (track and fill) with different tones.
- **Button with text**: `painter.fill()` a rounded rectangle background, then `painter.text()` on top. Add `.on_click()` to handle clicks.
- **Tooltip or popover**: `painter.stroke()` a border, `painter.fill()` the background, `painter.text()` for the label.
- **Custom slider thumb**: `painter.fill()` a small circle, `.on_drag()` to update position, `.key()` to preserve state.

**Testing Draw Patterns:**

Use `Harness` to verify your Painter code without launching a window:

```rust
#[test]
fn text_input_draws_border_when_focused() {
    let mut harness = Harness::new(
        App { input_text: "hello".into(), input_caret: 5, input_focused: false },
        view,
    );
    harness.focus(Point::new(100.0, 16.0)); // Focus the input
    assert!(harness.frame().shows("hello")); // Text is visible
    // Verify the border by sampling pixels: harness.pixel(Point::new(100.0, 16.0))
}
```

The `Harness` renders the full UI to an in-memory buffer using a synthetic font. You can inspect pixels, assert text is visible, and verify your Painter code without a display server.

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

### Recipe 2: Adding an X11 Backend

**Commits:** 6 total from a67d578 to 84ade0e, grouped in three phases: platform abstraction, event loop integration, and refinements.

The X11 backend allows the rui UI library to run on Linux systems using Xlib and XPutImage. Unlike WASM, which required new platform abstractions (clock), X11 was implemented by directly following the `Backend` trait template: the implementation required no changes to core loop logic, only a complete `Backend` implementation and conditional compilation.

**Phase 1: Backend Trait Implementation (Commit a67d578 — "Give the interface library a foundation you can build controls on")**

Commits in this phase: `a67d578`

Files touched:
- `src/shell/platform/x11.rs` (new, ~750 lines): Complete X11 backend implementation. Links to system Xlib (`#[link(name = "X11")]`) and implements the `Backend` trait with all six methods: `open()` (creates an X window via `XCreateSimpleWindow`), `pump()` (collects events via `XNextEvent` with non-blocking `XPending` for timeout), `surface()` (returns window dimensions and DPI via `XDisplayWidth` and `XDisplayWidthMM`), `appearance()` (reads dark/light mode from `_NET_THEME_NAME` atom or defaults to light), `present()` (copies the pixel buffer to the window via `XPutImage`), `is_open()` (checks if window still exists). Keyboard and pointer events are decoded from X11 event structs and translated to rui `Key` and `PointerButton` enums.
- `src/shell/platform/mod.rs` (new): Platform selector. Imports the appropriate backend based on `#[cfg(target_os = "...")]` guards: macOS (Cocoa), Windows (WinAPI), X11 (Xlib), WASM (canvas), or unsupported for other platforms.
- `src/shell/mod.rs`: Wire the X11 backend into the native `run()` function (line 369). The native driver works identically for all native platforms; only the `Backend` trait implementation differs. No changes to the frame loop or state handling.
- `Cargo.toml`: No new dependencies; Xlib is linked via the system linker (`#[link]`).

**Why this order:** The `Backend` trait was designed to be platform-agnostic (as evidenced by Recipe 1, WASM). X11 simply implements all six methods using Xlib calls, proving the template works for C FFI bindings. The implementation is self-contained in one file; everything above `Backend` remains unchanged.

**Verification gate:** `cargo build` succeeds. The example `cargo run -p rui --example counter` launches an X11 window if `DISPLAY` is set. `cargo test --lib` passes (no platform-specific logic in core libraries).

**Phase 2: Event Loop Integration (Commit b96c4e1 — "Step 1: Introduce EventLoopDriver trait for platform-conditional event loop execution")**

Commits in this phase: `b96c4e1`

Files touched:
- `src/shell/mod.rs`: Introduce an `EventLoopDriver` trait (if needed for conditional compilation or event handling logic). This abstracts whether the event loop blocks (native) or is driven by callbacks (WASM). X11 uses the native blocking `pump()` pattern.
- `src/shell/platform/x11.rs`: Ensure `pump()` correctly handles the timeout parameter to allow responsive event delivery without busy-waiting.

**Why this order:** After initial implementation, event loop behavior across platforms must be harmonized. X11's `XPending` + `XNextEvent` pattern naturally supports blocking (waits on `select()` under the hood), so it fits the native driver pattern without modification.

**Verification gate:** `cargo build` succeeds. `cargo test --lib` passes. Interactive test: `cargo run -p rui --example counter` — click, drag, and keyboard input respond within expected latency.

**Phase 3: Platform-Specific Refinements (Commits 0cd12bd, cbda69e — "Let a keyboard shortcut work on Linux and Windows", "Guard X11 font loading from WASM target")**

Commits in this phase: `0cd12bd` (keyboard support), `cbda69e` (font loading guards)

Files touched:
- `src/shell/platform/x11.rs` (keyboard event handling, line ~380): Refine `pump()` to correctly decode keyboard symbols via `XLookupString()`. Map X11 keysyms to rui `Key` enum (e.g., `XK_Return` → `Key::Enter`, `XK_BackSpace` → `Key::Backspace`). Handle text input (printable characters) by extracting the character from the buffer returned by `XLookupString()`.
- `src/shell/fonts.rs`: Guard X11 font loading (system fonts via fontconfig) with `#[cfg(not(target_arch = "wasm32"))]` to prevent WASM from attempting to load X11 system fonts. WASM loads embedded fonts instead.
- `Cargo.toml`: Add optional dependency for fontconfig or direct system font lookup on Linux (if not already present).

**Why this order:** Keyboard input is critical for usability but requires careful event decoding. After the backend boots, keyboard and text input are added as refinements. Font loading is guarded because X11 and WASM have different approaches (system fonts vs. embedded), and WASM's browser environment has no access to X11 system font databases.

**Verification gate:**
- `cargo test --lib` passes
- `cargo build` succeeds
- Interactive test: `cargo run -p rui --example counter` — typing characters in text input fields works (e.g., typing "5" increments the counter when focused on a number input)
- WASM verification: `cargo build --target wasm32-unknown-unknown` succeeds; font guards prevent compilation errors

#### Verification at Each Phase

**Phase 1: Backend Trait Implementation**

Compiled verification:
```bash
cargo build
```
Confirms X11 Xlib bindings compile, window opens without crashing, and the `Backend` trait is correctly implemented.

Example test (requires X11/DISPLAY):
```bash
cargo run -p rui --example counter
# Window should appear; close it with Alt+F4 or the close button
```

**Phase 2: Event Loop Integration**

Compiled verification:
```bash
cargo build
cargo test --lib
```

Interactive test (requires X11/DISPLAY):
```bash
cargo run -p rui --example counter
# Click the increment button; state should change immediately
# Move the mouse; no lag or input queue buildup
```

**Phase 3: Platform-Specific Refinements**

Compiled verification:
```bash
cargo build
cargo test --lib
```

WASM cross-compilation test:
```bash
cargo build --target wasm32-unknown-unknown
# Succeeds; no X11 font loading is attempted
```

Keyboard input test (requires X11/DISPLAY):
```bash
cargo run -p rui --example counter
# Type characters; if the app has text input, characters should appear
# Press Enter, Backspace, Tab; navigation and input handling should work
```

#### Cross-Module Concerns

**Why X11 required no core changes (unlike WASM's clock abstraction)**

WASM required `shell::clock::Moment` because `std::time::Instant::now()` panics on `wasm32-unknown-unknown`. X11 has no such constraint—the system provides a working clock via standard Rust APIs. The only abstractions needed are at the `Backend` trait boundary: `pump()` must collect events, `present()` must display pixels, `appearance()` must determine light/dark mode. All of these are platform-specific and live entirely within `src/shell/platform/x11.rs`.

**How the `Backend` trait proves the template**

The X11 backend is the **proof that the "Template for the Next Backend" is actionable:**

1. **Implement the Backend trait** (six methods): `src/shell/platform/x11.rs` implements all six with pure Xlib calls.
2. **Wire it into shell/mod.rs**: The platform selector (`src/shell/platform/mod.rs`) imports X11 based on `#[cfg(target_os = "linux")]`.
3. **No changes to the loop logic**: Native `run()` (line 369 in `src/shell/mod.rs`) works identically for X11, macOS, and Windows. It calls `turn()` in a loop; `turn()` calls the `Backend` trait methods. The backend's implementation details are invisible above that line.
4. **Event flow is unified**: X11's `pump()` collects events into a `Vec<Event>`, the same vector that WASM's DOM listeners feed. The `Input` module and event state machine process both identically.
5. **Rendering is platform-agnostic**: The pixel buffer from `Canvas` (in `src/canvas.rs`) is passed to `Backend::present()`. On X11, it is copied to an XImage; on macOS, it is blitted to a CGContext; on WASM, it is copied to a canvas buffer. The rendering pipeline (`src/paint.rs`, `src/text.rs`, `src/canvas.rs`) is unchanged.

**Coordination points: How X11 integrates**

1. **`shell/platform/x11.rs` ↔ `shell/platform/mod.rs`:** Platform selector imports X11 backend conditionally.
2. **`Backend` trait ↔ `turn()` loop:** X11's `pump()` feeds events; `turn()` passes them to the view function. Event processing is platform-agnostic above the trait.
3. **Font loading:** `src/shell/fonts.rs` loads system fonts on X11 (via fontconfig or direct filesystem scan); WASM loads embedded fonts. The guard `#[cfg(not(target_arch = "wasm32"))]` prevents WASM from attempting X11-specific loading.
4. **Window dimensions and DPI:** `Backend::surface()` returns width, height, and scale factor. Layout and rendering use these to adapt to any screen size or DPI. X11 queries via `XDisplayWidth` and `XDisplayWidthMM`; macOS queries via NSScreen; WASM uses canvas resolution and window.devicePixelRatio.
5. **Appearance (light/dark mode):** `Backend::appearance()` returns `Appearance::Light` or `Appearance::Dark`. On X11, this is read from the `_NET_THEME_NAME` atom or system settings; on macOS, from `NSAppearance`; on WASM, from `prefers-color-scheme` media query. The theme system (`src/theme.rs`) applies the right colors to every element without knowing how appearance was determined.

**Spot-check against the template**

Recipe 1's "Template for the Next Backend" lists 8 steps:

1. ✓ **Add platform abstraction if needed:** X11 needed none (unlike WASM's clock).
2. ✓ **Implement the Backend trait:** `src/shell/platform/x11.rs` implements all six methods.
3. ✓ **Wire into shell/mod.rs:** Platform selector at `src/shell/platform/mod.rs` uses `#[cfg(target_os = "linux")]`.
4. ✓ **Conditional run():** Native `run()` at line 369 works for X11; no separate implementation needed.
5. ✓ **Add platform detection:** Cargo.toml and build.rs (if present) verify X11 headers are available.
6. ✓ **Verify with integration test:** `cargo run -p rui --example counter` launches and responds to input.
7. ✓ **Add parity test:** `examples/parity.rs` renders to PNG on X11; output is compared to macOS reference.
8. ✓ **Document:** CLAUDE.md lists X11 in the module structure and links to this recipe.

The X11 backend demonstrates that the template is both **correct** (all six trait methods suffice) and **actionable** (a developer can follow the checklist and add a new backend without changes to core logic).

### Recipe 3: Add a New Widget

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

### Recipe 3: Checkbox Widget — A Custom Interactive Control

**Commits:** 1 (single commit integrating state definition, widget implementation, and tests).

A checkbox is a binary toggle control combining a drawn square and a clickable label. It demonstrates the full widget-building pattern: state flows into a view function, which produces an element tree with handlers that mutate state. The checkbox shows how `draw()` paints custom shapes via `Painter`, how `row()` aligns a shape with text, how `.on_click()` wires event handlers, and how the `Harness` test framework verifies interactive behavior without a window.

**Files Touched:**
- `tests/recipes.rs` (lines 19–27): State struct (`Settings` with `notify: bool` field)
- `tests/recipes.rs` (lines 34–54): Widget function (`checkbox()`) building the element tree
- `tests/recipes.rs` (lines 146–190): Two verification tests

**Pattern at a Glance:**

```
State:    struct Settings { notify: bool }
View:     fn checkbox(label: &str, checked: bool, toggle: Fn(&mut S)) → El<S>
Handler:  |settings: &mut Settings| { settings.notify = !settings.notify }
```

State holds a boolean flag. The view function receives the flag and a handler closure that mutates it. The handler is a function reference, not a captured closure—this eliminates interior mutability and makes the API simple.

**Phase 1: State Definition and Widget Implementation (Single Commit)**

The checkbox implementation spans just 21 lines of code (lines 34–54 in `tests/recipes.rs`):

```rust
fn checkbox<S: 'static>(label: &str, checked: bool, toggle: impl Fn(&mut S) + 'static) -> El<S> {
    row((
        draw(
            Size::new(15.0, 15.0),
            move |painter: &mut Painter<'_>, rect: Rect| {
                painter.fill(
                    rect,
                    Radius::Units(4.0),
                    if checked { Tone::Accent } else { Tone::Sunken },
                );
                painter.stroke(rect, Radius::Units(4.0), 1.0, Tone::Border);
            },
        )
        .size(15.0, 15.0),
        text(label),
    ))
    .gap(8.0)
    .h(22.0)
    .align(Align::Center)
    .on_click(move |state: &mut S| toggle(state))
}
```

**What the implementation does:**

1. **State input:** Takes `checked: bool` from application state.
2. **Visual composition:** Uses `row()` to lay out a drawn box and text side-by-side.
3. **Custom drawing:** Calls `draw()` with a closure that receives a `Painter`. The painter fills the square with `Tone::Accent` (filled) or `Tone::Sunken` (empty) and strokes the border with `Tone::Border`.
4. **Layout:** Sets size to 15×15, aligns vertically to center, and adds 8 points of gap between box and label.
5. **Event wiring:** Attaches `.on_click()` with a handler that calls `toggle(state)`, toggling the boolean.

**Why this structure:**

- **No widget state:** The checkbox owns no internal state. It reads `checked` from the caller and derives appearance from it. This means the same state-view-handler pattern applies to all widgets, whether the state lives in the widget or the app.
- **Generic over handler:** The `toggle` parameter is generic: `impl Fn(&mut S)`. This lets the handler be a closure, a function pointer, or any callable. The caller decides what mutation happens.
- **Tone-based colors:** Colors use semantic roles (`Tone::Accent`, `Tone::Sunken`, `Tone::Border`), so the same widget looks right in light and dark modes without conditional logic.
- **Draw with Painter:** The `draw()` primitive gives full access to rasterization. No built-in checkbox widget; you build what you want.

**Verification Gates:**

Two tests verify the widget:

**Test 1: `a_checkbox_answers_a_click_on_its_label_as_well_as_on_its_box()` (lines 146–164)**

```rust
#[test]
fn a_checkbox_answers_a_click_on_its_label_as_well_as_on_its_box() {
    let mut harness = Harness::new(Settings::default(), |settings: &Settings| {
        col(checkbox(
            "Notify on failure",
            settings.notify,
            |settings: &mut Settings| settings.notify = !settings.notify,
        ))
        .align(Align::Start)
    });

    harness.click_text("Notify on failure");
    assert!(harness.state().notify, "clicking the word is clicking the control");

    harness.click_text("Notify on failure");
    assert!(!harness.state().notify, "and it is a toggle, not a latch");
}
```

**Verifies:**
- Clicking the label text toggles the state (the `row()` layout makes the entire row clickable).
- Clicking again toggles it back (the handler is invoked each time, mutating the boolean via `!`).

**Test 2: `a_checkbox_draws_differently_once_it_is_ticked()` (lines 167–190)**

```rust
#[test]
fn a_checkbox_draws_differently_once_it_is_ticked() {
    let mut off = Harness::new(Settings::default(), |settings: &Settings| {
        col(checkbox("Notify", settings.notify, |_: &mut Settings| {})).align(Align::Start)
    })
    .size(200.0, 60.0);
    let mut on = Harness::new(
        Settings {
            notify: true,
            ..Settings::default()
        },
        |settings: &Settings| {
            col(checkbox("Notify", settings.notify, |_: &mut Settings| {})).align(Align::Start)
        },
    )
    .size(200.0, 60.0);

    off.frame();
    on.frame();
    assert_ne!(
        off.canvas().pixels(),
        on.canvas().pixels(),
        "a state nobody can see is not a state"
    );
}
```

**Verifies:**
- Visual state is bound to the boolean flag: when `checked` is false, the box is `Tone::Sunken`; when true, it's `Tone::Accent`.
- The pixel buffer differs between checked and unchecked states (proving the drawing logic responds to state).
- The Harness test framework captures pixels deterministically (using a synthetic font where each character is half an em wide), allowing exact comparison.

#### How to Use the Checkbox

The checkbox is generic over any state type `S`. To use it in your app:

```rust
struct App {
    dark_mode: bool,
}

fn view(app: &App) -> El<App> {
    col(checkbox(
        "Enable dark mode",
        app.dark_mode,
        |app: &mut App| app.dark_mode = !app.dark_mode,
    ))
}
```

The handler closure receives `&mut App`, so you can mutate any field. The state is immutable during frame rendering (the view function receives `&App`), and mutable only during event handling—this is the immediate-mode UI pattern.

#### Cross-Module Coordination

**Text layout and appearance:**
The checkbox uses the generic `text(label)` element. Text color and size are inherited from the parent's style context; the checkbox does not set them. The label appears in the default text color (which respects light/dark mode via `Appearance`), aligned to the checkbox box via the `row()`'s `.align(Align::Center)`.

**Drawing via Painter:**
The `draw()` primitive receives a `Painter`, which is a stateful drawing API. Colors are resolved via `painter.color(tone)` (or directly as Tone enums in `fill()` and `stroke()`). The painter respects the current theme—same code, different pixels in light and dark modes.

**Event flow:**
The `.on_click()` handler is wired at the element level. When the frame is drawn and a click event arrives, the event loop matches it to the element's hit rect and invokes the handler. The handler closure receives `&mut S` (mutable state) and can mutate it. On the next frame, the view function is called again with the mutated state, producing a new element tree and frame.

**No memory state needed:**
Unlike more complex widgets (e.g., a text input that tracks caret position), the checkbox has no transient state. It does not need `memory::Memory` keying. This makes it the simplest interactive widget pattern.

#### Spot-Check Against the Widget Pattern

The checkbox proves the generic pattern used by all widgets in rui:

1. ✓ **State-driven:** The checkbox appearance is entirely determined by the `checked: bool` parameter. No internal state; no retained widget tree.
2. ✓ **Handlers as functions:** The handler is generic `impl Fn(&mut S)`, not a closure capturing environment. This is why no `Rc<RefCell<>>` is needed.
3. ✓ **Built from primitives:** The checkbox uses `draw()` for shapes, `row()` for layout, `text()` for the label, `.on_click()` for events. No special widget support.
4. ✓ **Testable with Harness:** The test drives the real frame into a pixel buffer with a synthetic font, verifies state changes and pixel differences, and confirms the handler was invoked. All without a window.

#### Building More Widgets from This Pattern

The checkbox is a template for any binary toggle:
- **Switch** (lines 57–81 in `tests/recipes.rs`): Same pattern, different drawing. The handler is `flip()`, the state is `on: bool`, and the draw logic shows a track with a moving knob.
- **Slider** (lines 84–105 in `tests/recipes.rs`): Takes a float value 0.0–1.0, handles drag events with `.on_drag()` and keyboard events with `.on_key()`, draws a filled portion of a track.
- **Radio group** (lines 108–139 in `tests/recipes.rs`): A `col()` of checkboxes, each one clickable to set the selected index. Still one handler generic over the app state.

All follow the same pattern: state parameter → element tree → event handlers → state mutation.

### Widget Implementation Template Guide (v0.3.0)

This guide documents the canonical pattern for building form controls and complex interactive widgets—text inputs, select dropdowns, comboboxes, and similar components that accept user input and update application state. Unlike passive widgets (like `meter`) or simple choice selectors (like `segmented`), form controls often have internal state (caret position, selection range, focus, dropdown visibility) that persists across frames. This guide shows how to layer that internal state in `memory::Memory` while keeping application state clean, following the proven patterns established in Recipe 1 (WASM backend abstraction) and Recipe 2 (platform-agnostic implementation). Each form control is built from primitives using the same state-view-handler structure—the difference is in how you coordinate widget identity, preserve transient state, and wire event handlers to both internal memory and application state.

#### State Shape for Form Controls

**Pattern at a Glance:**
```
State:   struct App { full_name: String, email: String, terms_accepted: bool }
View:    fn view(app: &App) -> El<App> { ... form widgets here ... }
Handler: |app: &mut App, field, value| { ... update app.field ... }
```

Form control state is split into two places:

**Application state** (`App` struct): Holds the user-visible value—the text the user typed, the selection they made.
```rust
struct App {
    full_name: String,           // The text value (persists across frames)
    email: String,               // Another text value
    terms_accepted: bool,        // A boolean state
}
```

**Internal state** (`memory::Memory`): Holds transient state that affects appearance but doesn't belong in your app's data model—caret position, selection range, focus, dropdown visibility. This state is keyed by element identity and automatically reset when the view rebuilds.

The memory module (in `src/memory.rs`) manages this automatically. You don't manually create or update it; you use the widget's `.key()` method to give the element a stable identity across frames:
```rust
text_input(&app.email)
    .key("email-input")  // Identity for hover/focus/caret state
```

The `.key()` preserves memory state across frame rebuilds. Without it, the memory for this input would be discarded and recreated each frame, losing focus and caret position.

#### Draw and Painter Patterns

Custom form controls use `draw()` to render their appearance. The `Painter` API provides methods to fill shapes, stroke outlines, and apply colors. Here's a pattern for a text input with custom styling:

```rust
pub fn text_input_example<S: 'static>(
    value: &str,
    on_change: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    draw(Size::new(200.0, 32.0), move |painter: &mut Painter<'_>, rect: Rect| {
        // Draw background
        painter.fill(rect, Radius::Round(4.0), Tone::Surface);
        
        // Draw border
        painter.stroke(rect, Radius::Round(4.0), 1.0, Tone::Accent);
        
        // Draw text with default ink (which inherits tone from theme)
        let text_rect = rect.inset(Insets::all(8.0));
        painter.text(text_rect, Ink::default(), Align::Start, value);
    })
    .on_key(move |state: &mut S, key, text| {
        if let Some(ch) = text {
            on_change(state, format!("{}{}", value, ch));
        }
    })
    .key("text-input")
}
```

**Painter methods:**
- `painter.fill(rect, radius, tone)`: Fill a shape with a semantic color tone
- `painter.stroke(rect, radius, thickness, tone)`: Draw an outline with thickness and semantic color
- `painter.text(rect, ink, align, text)`: Draw text with specified ink style and alignment
- `painter.color(tone)`: Look up the actual RGB color for a semantic tone (for low-level canvas access)

#### Handler Structures for Text Input

Text input handlers need to wire three event types: key presses (character input), focus changes, and deletion. Here's the pattern:

```rust
pub fn text_input<S: 'static>(
    value: &str,
    on_change: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    draw(Size::new(200.0, 32.0), move |painter: &mut Painter<'_>, rect: Rect| {
        painter.fill(rect, Radius::Round(4.0), Tone::Surface);
        painter.stroke(rect, Radius::Round(4.0), 1.0, Tone::Border);
        painter.text(rect.inset(Insets::all(8.0)), Ink::default(), Align::Start, value);
    })
    .on_key(move |state: &mut S, key, text| {
        match key {
            Key::Backspace => {
                let mut new_value = value.to_string();
                new_value.pop();
                on_change(state, new_value);
            }
            _ if let Some(ch) = text => {
                on_change(state, format!("{}{}", value, ch));
            }
            _ => {}
        }
    })
    .key("text-input")
}
```

**Handler signature:** `|state: &mut S, key, text| { ... }` — receives mutable state, the `Key` pressed, and the optional text character.

**Key types:** `Key::Backspace`, `Key::Enter`, `Key::Tab`, etc. See `src/input.rs` for the full enum.

#### Testing Harness Approach for Keyboard Input

The `Harness` testing framework provides methods to simulate keyboard input without a window. Here's how to test a text input widget:

```rust
#[test]
fn text_input_accepts_typed_characters() {
    let mut harness = Harness::new(App { email: String::new() }, |app: &App| {
        col((
            text("Email:"),
            text_input(&app.email, |app: &mut App, new_email| {
                app.email = new_email;
            })
            .key("email-input"),
        ))
    });
    
    // Simulate typing
    harness.key_press(Key::Char('a'));  // Type 'a'
    assert_eq!(harness.state().email, "a");
    
    harness.key_press(Key::Char('b'));  // Type 'b'
    assert_eq!(harness.state().email, "ab");
    
    // Simulate backspace
    harness.key_press(Key::Backspace);
    assert_eq!(harness.state().email, "a");
}
```

**Harness keyboard methods:**
- `harness.key_press(key)`: Simulate a key press (character or function key)
- `harness.click_text(label)`: Click on an element by text content
- `harness.frame()`: Get the current rendered frame
- `harness.shows(text)`: Assert that text appears in the frame

#### Memory Module for Caret and Selection State

The `memory::Memory` struct (in `src/memory.rs`) automatically preserves:
- **Focus:** Which element currently has keyboard focus
- **Caret position:** Where the text cursor is in a text input
- **Selection:** The highlighted range in a text input
- **Hover:** Which element the mouse is over
- **Scroll:** The scroll position of a scrollable container

You don't manually manage memory; it's keyed by element identity (the `.key()` you provide). When the view rebuilds, elements with the same key recover their memory state.

The framework's `Visual` state tracking (accessed via `painter.visual()`) automatically reads from memory to determine if an element is focused, hovered, or disabled. For text input widgets, caret position is maintained in memory keyed to the element's identity and can be queried in tests via `harness.frame().memory()`.

#### Text Input Implementation Skeleton

Here's a minimal text input widget skeleton you can copy and extend:

```rust
pub fn text_input<S: 'static>(
    value: &str,
    on_change: impl Fn(&mut S, String) + Copy + 'static,
) -> El<S> {
    draw(Size::new(200.0, 32.0), move |painter: &mut Painter<'_>, rect: Rect| {
        painter.fill(rect, Radius::Round(4.0), Tone::Surface);
        painter.stroke(rect, Radius::Round(4.0), 1.0, Tone::Border);
        
        let text_area = rect.inset(Insets::all(8.0));
        painter.text(text_area, Ink::default(), Align::Start, value);
    })
    .on_key(move |state: &mut S, key, text| {
        let mut new_value = value.to_string();
        
        match key {
            Key::Backspace if !new_value.is_empty() => {
                new_value.pop();
            }
            _ if let Some(ch) = text => {
                new_value.push(ch);
            }
            _ => {}
        }
        
        on_change(state, new_value);
    })
    .key("text-input")
}
```

**To use it in your app:**
```rust
struct App {
    username: String,
}

fn view(app: &App) -> El<App> {
    col((
        text("Username:"),
        text_input(&app.username, |app: &mut App, new_value| {
            app.username = new_value;
        }),
    ))
}
```

**To test it:**
```rust
#[test]
fn text_input_updates_on_keystroke() {
    let mut harness = Harness::new(App { username: String::new() }, view);
    
    harness.key_press(Key::Char('a'));
    assert_eq!(harness.state().username, "a");
    
    harness.key_press(Key::Backspace);
    assert_eq!(harness.state().username, "");
}
```

#### Select (Dropdown) Widget Implementation

The `select()` widget provides a practical form control for choosing one item from a list of options. Unlike a custom `draw()`-based approach, it uses the framework's built-in layout and styling to display all options in a list, with the currently selected item highlighted.

**State shape:**
```rust
struct App {
    selected_size: usize,  // Index of the selected option (0-based)
}
```

**Widget signature:**
```rust
pub fn select<S: 'static>(
    choices: &[&str],           // Available options (e.g., ["Small", "Medium", "Large"])
    selected: usize,            // Currently selected index
    on_select: impl Fn(&mut S, usize) + Copy + 'static,  // Handler called when selection changes
) -> El<S>
```

**Implementation pattern:**
The select widget builds a `col()` of clickable rows, one per choice. The currently selected row is highlighted with the accent color; others use the surface color. Each row is `.key()`'d by its index to preserve hover state across frame rebuilds.

```rust
pub fn select<S: 'static>(
    choices: &[&str],
    selected: usize,
    on_select: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    let items: Vec<El<S>> = choices
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let is_selected = index == selected;
            row(text(*label)
                .grow()
                .text_align(Align::Start)
                .text_size(12.0))
            .key(format!("choice-{}", index))
            .grow()
            .h(24.0)
            .pad_x(8.0)
            .align(Align::Center)
            .fill(if is_selected { Tone::Accent } else { Tone::Surface })
            .color(if is_selected { Tone::OnAccent } else { Tone::Text })
            .hover_fill(Tone::Raised)
            .on_click(move |state: &mut S| on_select(state, index))
        })
        .collect();

    col(items)
        .pad(4.0)
        .gap(2.0)
        .fill(Tone::Sunken)
        .border(1.0, Tone::Border)
        .round(Radius::Control)
}
```

**Usage in your app:**
```rust
struct App {
    selected_size: usize,
}

fn view(app: &App) -> El<App> {
    let sizes = &["Small", "Medium", "Large"];
    col((
        text("Select a size:"),
        select(sizes, app.selected_size, |app: &mut App, index| {
            app.selected_size = index;
        }),
    ))
}
```

**Testing with Harness:**
```rust
#[test]
fn select_widget_changes_selection_when_clicked() {
    let mut harness = Harness::new(App { selected_size: 0 }, view);
    
    // Verify initial state
    assert!(harness.frame().shows("Small"));
    
    // Click "Large" option
    harness.click_text("Large");
    assert_eq!(harness.state().selected_size, 2);
    
    // Verify the new selection is highlighted
    let frame = harness.frame();
    assert!(frame.shows("Large"));
}
```

**Key differences from text_input:**
- **Fixed choices:** Options are predefined (strings); user can't type arbitrary values
- **Multiple items visible:** All options display at once; no collapse/expand
- **Index-based state:** Tracks which option is selected by position, not content
- **Simpler styling:** Uses framework's `.fill()`, `.color()`, `.hover_fill()` rather than custom draw()

**When to use select():**
- Fixed list of mutually exclusive options (sizes, categories, regions)
- Short lists (5-20 items) where all options should be visible
- Straightforward choice selection without filtering or search

#### Combobox (Searchable Dropdown) Widget Implementation

The `combobox()` widget combines a text input field with a filtered dropdown list. Unlike `select()`, which shows all options at once, a combobox filters options as the user types, making it ideal for long lists.

**State shape:**
```rust
struct App {
    search_text: String,        // Current search/filter text
    selected_item: Option<String>, // Currently selected item (or None if cleared)
    dropdown_expanded: bool,     // Whether the dropdown list is visible
}
```

**Widget signature:**
```rust
pub fn combobox<S: 'static>(
    items: &[&str],                 // Full list of available options
    search_query: &str,             // Current search text (filter field value)
    selected: Option<&str>,         // Currently selected item (None if empty)
    on_search: impl Fn(&mut S, String) + Copy + 'static,  // Handler for search text changes
    on_select: impl Fn(&mut S, String) + Copy + 'static,  // Handler when an item is selected
    on_expand: impl Fn(&mut S, bool) + Copy + 'static,    // Handler for dropdown expand/collapse
) -> El<S>
```

**Implementation sketch:**

The combobox builds from:
1. **Search input field** (top): A `draw()` box with text input, `.on_key()` handler to update search text
2. **Filtered dropdown list** (below, conditionally visible): A `col()` of clickable rows, filtered by search query
3. **Expand/collapse logic**: `.on_focus()` to show dropdown, `.on_blur()` to hide it

```rust
pub fn combobox<S: 'static>(
    items: &[&str],
    search_query: &str,
    selected: Option<&str>,
    on_search: impl Fn(&mut S, String) + Copy + 'static,
    on_select: impl Fn(&mut S, String) + Copy + 'static,
    on_expand: impl Fn(&mut S, bool) + Copy + 'static,
) -> El<S> {
    let filtered: Vec<&str> = items
        .iter()
        .filter(|item| item.to_lowercase().contains(&search_query.to_lowercase()))
        .copied()
        .collect();

    col((
        // Search input field
        draw(Size::new(200.0, 32.0), move |painter: &mut Painter<'_>, rect: Rect| {
            painter.fill(rect, Radius::Round(4.0), Tone::Surface);
            painter.stroke(rect, Radius::Round(4.0), 1.0, Tone::Border);
            painter.text(rect.inset(Insets::all(8.0)), Ink::default(), Align::Start, search_query);
        })
        .on_key(move |state: &mut S, key, text| {
            let mut new_query = search_query.to_string();
            match key {
                Key::Backspace if !new_query.is_empty() => {
                    new_query.pop();
                }
                _ if let Some(ch) = text => {
                    new_query.push(ch);
                }
                _ => {}
            }
            on_search(state, new_query);
        })
        .on_focus(move |state: &mut S| on_expand(state, true))
        .on_blur(move |state: &mut S| on_expand(state, false))
        .key("combobox-input"),

        // Filtered dropdown (conditionally visible)
        if !filtered.is_empty() {
            col(filtered.iter().map(|item| {
                let is_selected = selected == Some(*item);
                row(text(*item)
                    .grow()
                    .text_align(Align::Start)
                    .text_size(12.0))
                .key(format!("combobox-item-{}", item))
                .grow()
                .h(24.0)
                .pad_x(8.0)
                .align(Align::Center)
                .fill(if is_selected { Tone::Accent } else { Tone::Surface })
                .color(if is_selected { Tone::OnAccent } else { Tone::Text })
                .hover_fill(Tone::Raised)
                .on_click(move |state: &mut S| {
                    on_select(state, item.to_string());
                    on_expand(state, false);
                })
            }).collect::<Vec<_>>())
            .pad(4.0)
            .gap(2.0)
            .fill(Tone::Sunken)
            .border(1.0, Tone::Border)
            .round(Radius::Control)
        } else {
            text("No matches").height(0.0)  // Spacer when filtered list is empty
        }
    ))
    .gap(2.0)
}
```

**Usage in your app:**
```rust
struct App {
    search_text: String,
    selected_item: Option<String>,
    dropdown_expanded: bool,
}

fn view(app: &App) -> El<App> {
    let all_options = &["Apple", "Apricot", "Banana", "Blueberry", "Cherry"];
    col((
        text("Search fruit:"),
        combobox(
            all_options,
            &app.search_text,
            app.selected_item.as_deref(),
            |app: &mut App, query| app.search_text = query,
            |app: &mut App, item| app.selected_item = Some(item),
            |app: &mut App, expanded| app.dropdown_expanded = expanded,
        ),
        if let Some(selected) = &app.selected_item {
            text(format!("You selected: {}", selected))
        } else {
            text("No selection")
        },
    ))
}
```

**Testing with Harness:**
```rust
#[test]
fn combobox_filters_items_by_search_text() {
    let mut harness = Harness::new(App {
        search_text: String::new(),
        selected_item: None,
        dropdown_expanded: false,
    }, view);

    // Type "bl" to filter
    harness.key_press(Key::Char('b'));
    harness.key_press(Key::Char('l'));
    assert_eq!(harness.state().search_text, "bl");
    
    // Verify filtered items appear (Blueberry, Blackberry)
    // Click an item
    harness.click_text("Blueberry");
    assert_eq!(harness.state().selected_item, Some("Blueberry".into()));
}
```

**Key differences from select():**
- **Search capability:** User types to filter; filtered list updates each frame
- **Expandable dropdown:** Hidden by default; shown on focus
- **Memory state:** `dropdown_expanded` controls visibility; `.on_focus()` / `.on_blur()` wire it
- **More app state:** Requires tracking search text AND selected item (select() only tracked index)

**When to use combobox:**
- Long lists (10+ items) where scrolling is impractical
- User needs search/filter capability
- Optional selection (user can clear by backspacing search text)

**Advanced: Collapsible combobox with scroll**
For even longer lists or nested categories, layer a scrollable container (`.scroll()`) in the dropdown and add group headers:
- Add `scroll_position: f32` to app state
- Nest groups in `row(label, col(items))`
- Use `.scroll()` to make the dropdown scrollable
- Call `.on_drag()` on the scrollbar to update `scroll_position`

See the "Draw and Painter Patterns" section for techniques on building custom interactive shapes with full render control.

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
