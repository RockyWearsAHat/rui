# rui — Declarative Rust UI Library

rui is a declarative interface library for Rust with **zero external dependencies**. It unifies structure (layout), style (appearance), and behavior (interaction) into a single Rust expression, rendered by its own TrueType parser, glyph rasterizer, and platform-specific window backends.

**Build with `cargo build`, test with `cargo test`, and run examples via `cargo run -p rui --example <name>`.**

## Core Design Principles

- **View is a pure function of state.** The `view` function rebuilds the entire UI description from application data each frame—no retained widget tree.
- **Handlers are functions of state, not closures.** `on_click(|app: &mut App| …)` receives mutable state as an argument, eliminating `Rc`, `RefCell`, and interior mutability.
- **Roles, not values.** Colors are named by semantic role (`Tone::Surface`, `Tone::Accent`), so the same description works in light and dark modes without conditional logic.
- **Foundations, not a catalogue.** The library provides primitives (`draw`, `on_drag`, `on_key`, `layer`) for building custom controls—no built-in checkbox or slider that constrains design.

## Quick Start

### Installation

Ensure you have Rust 1.85 or later:
```bash
rustc --version  # Should be 1.85 or newer
rustup update    # Update if needed
```

Clone or download the repository:
```bash
git clone <repository-url>
cd rui
```

### Your First App

Run the counter example:
```bash
cargo run -p rui --example counter
```

A window appears showing increment/decrement buttons. Click to interact. This is the simplest app—a single state field, a view function, and a handler closure.

## Features & Capabilities

### Platforms

| Platform | Backend | Status |
|----------|---------|--------|
| macOS | Cocoa (AppKit) | ✅ Fully supported |
| Windows | WinAPI | ✅ Fully supported |
| Linux (X11) | Xlib + XPutImage | ✅ Fully supported |
| Linux (Wayland) | libwayland-client | ✅ Fully supported (v0.2.0) |
| Browser | WebAssembly + Canvas | ✅ Fully supported |

### Rendering

- **Zero dependencies:** Complete rendering pipeline (TrueType font parsing, glyph rasterization, shape drawing) built from scratch.
- **Light/Dark modes:** Automatic theme switching via system appearance detection. Test both modes without code changes.
- **Pixel-perfect accuracy:** WASM backend renders pixel-for-pixel identical frames to native backends (verified by parity tests).
- **Semantic colors:** Use `Tone::Surface`, `Tone::Accent`, `Tone::Muted`, etc.—the theme resolves them to RGB based on light/dark mode.

### Interaction

- **Immediate-mode UI:** View rebuilds every frame from state. State mutations only happen in handlers, never during rendering.
- **Event handling:** Keyboard, mouse (click, drag, wheel), and focus management. All events flow through a unified `Input` queue.
- **Memory & persistence:** Focus, hover, scroll position, and caret location persist across frame rebuilds via the `Memory` module (automatically keyed by element identity).
- **Animations:** Implicit animations (fade, slide, scale) driven by the frame loop. State animations and transitions built with primitives.

### Widgets & Layout

- **Flexbox-like layout:** Rows, columns, gap, alignment, grow, and shrink. Single-axis stacking with line wrapping via `flow()`.
- **Scroll containers:** `.scroll()` to make any element scrollable. Scroll position persists across frames.
- **Custom drawing:** `draw()` primitive with `Painter` API for complete render control. Fill shapes, stroke outlines, draw text—all with semantic colors.
- **Accessibility:** Optional `accessible_name`, `accessible_role`, and `accessible_description` fields on every element for screen reader support.

## Common Commands

### Build
```bash
cargo build                         # Debug build
cargo build --release              # Optimized build
```

### Run Examples
```bash
cargo run -p rui --example counter              # Interactive counter app
cargo run -p rui --example controls             # Control showcase
cargo run -p rui --example gallery -- .         # Render all elements to PNG
cargo run -p rui --example segmented            # Choice selector exemplar (copy & modify)
cargo run -p rui --example meter                # Progress bar exemplar (passive widget)
cargo run -p rui --example calculator           # Numeric input and layout
cargo run -p rui --example theme_switcher       # Light/dark mode support
cargo run -p rui --example todo_app             # List rendering and state management
cargo run -p rui --example form_example         # Forms with text input, select, checkbox
cargo run -p rui --example parity -- target/parity  # Build reference frame for WASM parity test
```

### Test
```bash
cargo test                          # Run all tests (unit + integration)
cargo test --lib                    # Unit tests only
cargo test --test interaction -- --nocapture  # Run one test file with output
cargo test --test recipes -- widget_name      # Run specific widget test
```

### Format & Lint
```bash
cargo fmt                           # Auto-format all code
cargo clippy                        # Run linter
```

### WebAssembly
```bash
# Build WASM target
cargo build --target wasm32-unknown-unknown -p rui --example counter

# Generate wasm-bindgen glue and serve
wasm-pack build --target web --release --out-dir pkg
python3 -m http.server 8731 --bind 127.0.0.1

# Test in headless browser (Firefox required)
wasm-pack test --headless --firefox

# Pixel-perfect parity verification
cargo run -p rui --example parity -- target/parity
# Then open http://127.0.0.1:8731/examples/parity.html
```

## Examples

All examples can be run with `cargo run -p rui --example <name>`:

| Example | Purpose | Learning focus |
|---------|---------|-----------------|
| `counter` | Increment/decrement with state persistence | **Start here.** Single state field, view function, click handler. |
| `segmented` | Choice selector with 3+ options | Exemplar for building custom controls. Copy and modify. |
| `meter` | Passive progress bar | Build read-only widgets that display state without user interaction. |
| `controls` | Showcase of all widgets | See button, checkbox, slider, segmented control, radio, tooltip in one place. |
| `gallery` | Render every UI element to PNG | Verify visual appearance without launching windows. Used for regression testing. |
| `calculator` | Numeric input and button grids | Multi-step computation and button layouts. |
| `theme_switcher` | Light/dark mode support | Flow appearance preferences through the entire UI. |
| `todo_app` | List rendering and state management | Create, toggle, and delete list items. State updates flow through the view. |
| `form_example` | Comprehensive form | Text input, select dropdown, checkbox, and form control integration. |
| `parity` | Native reference frame | Builds frames for pixel-perfect WASM backend comparison. |
| `icon` | macOS `.iconset` and `.icns` generation | Draw and export app icons at all required sizes. |

**Learning Path:** Start with `counter` (simplest), then `segmented` (understand handlers), then `meter` (passive widgets). Continue with `calculator` (numeric input and multi-step computation), `theme_switcher` (appearance and semantic colors), `todo_app` (list rendering), and `form_example` (form controls). Use `controls` to see all available widgets. The `gallery` example renders all elements to PNG for visual verification.

## Architecture Overview

### Module Structure

| Module | Purpose |
|--------|---------|
| `element` | UI element tree—`El<T>` is the root type; builders for structure (`col`, `row`). |
| `widgets` | Recipe implementations: `button`, `text`, `checkbox`, `slider`, `segmented`, etc. All built from primitives. |
| `style` | Layout and appearance: `Length`, `Radius`, `Tone`, `Align`, `Justify`. |
| `layout` | Flexbox-like layout engine; single-axis stacking, line wrapping, scroll, layers. |
| `paint` | Drawing abstraction: `Painter` API for shapes, outlines, and text. Used by all elements. |
| `canvas` | Pixel buffer and rasterizer; root drawing operation. |
| `text` | TrueType parser, glyph rasterizing, text layout with kerning/ligatures. |
| `color` | RGB(A) colors and sRGB gamma handling. |
| `theme` | Colors, spacing, and type sizes; `Appearance` (light/dark) and `Tone` (semantic roles). |
| `input` | Input events and per-frame event view. Translates raw `Event` stream to immediate-mode queries. |
| `memory` | Stateful interaction data (hover, focus, scroll, caret, animations) keyed by element identity. |
| `shell` | Platform window management. `Backend` trait implementation for macOS, Windows, X11, Wayland, WASM. |
| `app` | Application entry point; couples state, view function, and event loop. |
| `testing` | `Harness`: drives the real frame into a buffer with synthetic font for deterministic testing. |

### Key Pattern: View is a Function of State

```rust
struct App {
    counter: i32,
}

fn view(app: &App) -> El<App> {
    col((
        text(format!("{}", app.counter)),
        button("Increment", |app: &mut App| app.counter += 1),
        button("Decrement", |app: &mut App| app.counter -= 1),
    ))
}

fn main() {
    rui::app::run(App { counter: 0 }, view);
}
```

State is a struct. View rebuilds the element tree every frame from state. Handlers receive `&mut App` and mutate state. On the next frame, the new state produces a new element tree and render. No retained widget tree, no closures capturing shared references, no interior mutability needed.

### Event Loop (Platform-Agnostic)

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

Platform-specific backends (macOS, Windows, X11, Wayland, WASM) implement a `Backend` trait with six methods: `open()`, `pump()`, `surface()`, `appearance()`, `present()`, `is_open()`. All layout, painting, and state logic is platform-agnostic above that line.

## Testing Strategy

### Unit Tests
Fast, isolated tests of individual modules:
```bash
cargo test --lib
```

### Integration Tests
Full-stack tests using the `Harness` testing framework. Tests create a `Harness` with app state and view function, then simulate user interactions and assert state changes:
```bash
cargo test --test recipes
```

Example:
```rust
#[test]
fn counter_increments_on_click() {
    let mut harness = Harness::new(App { counter: 0 }, view);
    harness.click_text("Increment");
    assert_eq!(harness.state().counter, 1);
}
```

### Platform Tests
Browser-specific tests for WASM backend (requires Firefox and `wasm-pack`):
```bash
wasm-pack test --headless --firefox
```

### Parity Tests
Pixel-perfect verification that WASM and native backends render identically:
```bash
cargo test --test wasm_parity
```

## Getting Started for New Developers

1. **Run the counter example:**
   ```bash
   cargo run -p rui --example counter
   ```
   Understand the three-part pattern: state struct, view function, handler closures.

2. **Copy the segmented exemplar:**
   ```bash
   cp examples/segmented.rs examples/my_control.rs
   cargo run -p rui --example my_control
   ```
   Modify the state, view, and handler to build your own widget.

3. **Write a test:**
   ```rust
   #[test]
   fn my_widget_works() {
       let mut harness = Harness::new(App { value: 0 }, view);
       harness.click_text("Label");
       assert_eq!(harness.state().value, 1);
   }
   ```
   Run `cargo test my_widget_works` to verify.

4. **Read the recipes:** Look at `src/widgets.rs` and `tests/recipes.rs` for worked examples of common controls (checkbox, switch, slider, radio, tooltip). Each follows the same state-view-handler structure.

5. **Explore the theme system:** Run `cargo run -p rui --example theme_switcher` to see light/dark mode in action. All `Tone` colors (Surface, Accent, Muted, etc.) adapt automatically.

## Building Custom Controls

The library provides primitives (draw, on_click, on_drag, on_key, col, row) for building any control from scratch. No built-in checkbox or slider exists because those would constrain design. Instead:

```rust
// Example: Custom checkbox
fn checkbox<S: 'static>(
    label: &str,
    checked: bool,
    toggle: impl Fn(&mut S) + 'static,
) -> El<S> {
    row((
        draw(Size::new(15.0, 15.0), move |painter: &mut Painter<'_>, rect: Rect| {
            painter.fill(rect, Radius::Small, 
                if checked { Tone::Accent } else { Tone::Sunken });
            painter.stroke(rect, Radius::Small, 1.0, Tone::Border);
        }),
        text(label),
    ))
    .gap(8.0)
    .on_click(move |state: &mut S| toggle(state))
}
```

Use `draw()` for custom shapes, `Painter` for control over rendering, and `.on_*()` event handlers to wire interaction. See `CLAUDE.md` and `tests/recipes.rs` for more examples.

## Documentation

- **CLAUDE.md** — Comprehensive codebase guide: module structure, testing strategy, architecture patterns, platform-specific guidance, and step-by-step recipes for adding features.
- **index.dx** — Project index and recipe documentation (opened with `dx_read` tool).
- **dev.dx** — Build plan and verification checklist.
- **Inline code documentation** — Each module (in `src/`) has a module-level comment explaining its purpose and public API.

Run `cargo doc --no-deps --open` to generate and browse the Rust API documentation.

## Troubleshooting

### Build fails with "Rust version too old"
```bash
rustc --version
rustup update
```
Minimum Rust version is 1.85 (Edition 2021). Update with `rustup update`.

### "error: could not compile rui"
```bash
cargo clean
cargo build
```
Clean build artifacts and retry.

### Tests fail
```bash
cargo test --lib
cargo test --test setup
```
Run unit tests first to isolate failures. Check the failure message—it tells you exactly what is broken.

### Examples won't run on Linux
Ensure X11 or Wayland is available:
```bash
echo $DISPLAY  # Should output :0 or similar for X11
# or
echo $WAYLAND_DISPLAY  # For Wayland
```
On headless systems, use Xvfb: `xvfb-run -a cargo run -p rui --example counter`

### WASM examples show blank canvas
Open browser developer tools (F12) and check the Console for JavaScript errors. Serve locally with `python3 -m http.server 8000`, not `file://` URLs.

## Contributing

1. Ensure `cargo test` passes.
2. Run `cargo fmt` and `cargo clippy` to check formatting and lints. The pre-commit hook enforces both.
3. Commit with a clear message describing what changed and why.
4. Open a pull request with a summary of the changes and the motivation.

## License

See LICENSE file in the repository.

## Acknowledgments

rui was built from scratch with zero external dependencies, including:
- Complete TrueType font parser and glyph rasterizer
- Cross-platform window management (macOS, Windows, X11, Wayland, WebAssembly)
- Immediate-mode rendering pipeline with light/dark mode support
- Comprehensive test suite covering layout, rendering, and interaction

The design prioritizes simplicity and control: state-driven views, function-based handlers, semantic color roles, and primitives over a predefined widget catalogue.
