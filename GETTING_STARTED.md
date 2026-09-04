# Getting Started with rui-native

Welcome to **rui-native**, a declarative interface library for Rust with zero dependencies. This guide will help you build your first interactive application.

## Installation

Add rui-native to your `Cargo.toml`:

```bash
cargo add rui-native
```

Or manually:

```toml
[dependencies]
rui-native = "0.1"
```

## Your First App: Counter

Here's the simplest rui-native application:

```rust
use rui_native::{El, button, col, title};

struct Counter {
    count: i32,
}

fn view(counter: &Counter) -> El<Counter> {
    col((
        title(format!("{}", counter.count)).text_size(56.0).bold(),
        button("Increment").on_click(|counter: &mut Counter| counter.count += 1),
        button("Decrement").on_click(|counter: &mut Counter| counter.count -= 1),
    ))
    .gap(16.0)
    .pad(32.0)
    .center()
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("Counter", Counter { count: 0 }, view)
}
```

Run it with:

```bash
cargo run
```

## The Three Core Concepts

### 1. **State** — Your Application Data

```rust
struct Counter {
    count: i32,
}
```

This is just a Rust struct. It holds all the data your app needs. Nothing magical.

### 2. **View** — A Function That Describes the UI

```rust
fn view(counter: &Counter) -> El<Counter> {
    // Return an element tree built from state
}
```

The `view` function rebuilds the entire UI description from your state on every frame. It's **not** a callback that runs once—it runs continuously, and the UI always reflects the current state.

### 3. **Handlers** — Functions That Mutate State

```rust
button("Increment").on_click(|counter: &mut Counter| {
    counter.count += 1;
})
```

Handlers receive mutable state as an argument, so you can update it freely. No `Rc<RefCell<>>`, no interior mutability tricks—just pure Rust.

## Building Your First Widget

Let's create a custom button that displays state:

```rust
use rui_native::{El, button, text, row};

struct App {
    clicks: u32,
}

fn view(app: &App) -> El<App> {
    row((
        button("Click me").on_click(|app: &mut App| app.clicks += 1),
        text(format!("Clicks: {}", app.clicks)),
    ))
    .gap(16.0)
    .pad(16.0)
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("Click Counter", App { clicks: 0 }, view)
}
```

## Layout

rui-native uses a flexbox-like layout system. Arrange elements with:

- **`col()`** — Stack elements vertically
- **`row()`** — Stack elements horizontally
- **`.gap()`** — Add spacing between children
- **`.pad()`** — Add padding inside the element

```rust
col((
    title("Welcome"),
    row((
        button("OK"),
        button("Cancel"),
    )).gap(8.0),
))
.pad(16.0)
.gap(12.0)
```

## Styling

Apply colors, text sizes, and appearance with method calls:

```rust
button("Click")
    .text_size(18.0)
    .bold()
    .fill(Tone::Accent)
```

Use semantic color roles (light/dark mode aware):

- `Tone::Surface` — Background color
- `Tone::Muted` — Disabled or secondary text
- `Tone::Accent` — Interactive highlights
- `Tone::Ok` — Success (green)
- `Tone::Warn` — Warning (orange)
- `Tone::Bad` — Error (red)

Colors automatically adapt to light and dark modes.

## Event Handling

### Click Events

```rust
button("Save").on_click(|app: &mut App| {
    app.data.save();
})
```

### Drag Events

```rust
draw(Size::new(100.0, 100.0), |painter, rect| {
    painter.fill(rect, Radius::Pill, Tone::Accent);
})
.on_drag(|app: &mut App, drag| {
    app.position = drag.position;
})
```

### Keyboard Events

```rust
input_field()
    .on_key(|app: &mut App, key, mods| {
        if key == Key::Return {
            app.submit();
        }
    })
```

## Examples

The repository includes several examples:

```bash
# Simple counter
cargo run --example counter

# Showcase of all controls
cargo run --example controls

# Render every element to PNG
cargo run --example gallery -- .

# Exemplar: minimal choice selector (33 lines)
cargo run --example segmented

# Exemplar: passive progress bar
cargo run --example meter

# Check platform compatibility (renders reference frames)
cargo run --example parity -- target/parity
```

## Testing UI Without a Window

Use the `Harness` testing framework to drive the real rendering without opening a window:

```rust
#[cfg(test)]
mod tests {
    use rui_native::testing::Harness;

    #[test]
    fn counter_increments() {
        let mut harness = Harness::new(Counter { count: 0 }, view);
        harness.click_text("Increment");
        assert_eq!(harness.state().count, 1);
    }
}
```

## Building Custom Controls

The library provides primitives for building custom widgets. Copy the `segmented` or `meter` examples and modify them:

1. **Define state** — What data does your control need?
2. **Write the view function** — Build an `El<S>` from primitives
3. **Add handlers** — Call `.on_click()`, `.on_drag()`, etc.
4. **Test with Harness** — Verify it works without a window
5. **Use it** — No registration or special support needed

See [CLAUDE.md](CLAUDE.md) for the complete pattern and `tests/recipes.rs` for working examples of `checkbox`, `switch`, `slider`, `radio`, and more.

## Platforms

rui-native runs on:

- **macOS** — Via Cocoa
- **Windows** — Via WinAPI
- **Linux** — Via X11 (Wayland support coming soon)
- **Browser** — Via WebAssembly (same code, no changes)

The same code compiles and runs identically on all platforms.

## Building for WebAssembly

To run your app in a browser:

```bash
# Install wasm-pack if needed
curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh

# Build and generate web package
wasm-pack build --target web --release

# Serve locally
python3 -m http.server 8000

# Open http://localhost:8000
```

## Documentation

- **[CLAUDE.md](CLAUDE.md)** — Complete reference, module structure, recipes, troubleshooting
- **[examples/](examples/)** — Runnable demonstrations
- **[tests/recipes.rs](tests/recipes.rs)** — Widget implementations and patterns
- **API docs** — `cargo doc --no-deps --open`

## Common Tasks

### Add a Button

```rust
button("Click me").on_click(|app: &mut App| {
    app.handle_click();
})
```

### Display Text

```rust
text("Hello, world!")
    .text_size(14.0)
    .fill(Tone::Muted)
```

### Create a Checkbox

See `tests/recipes.rs` line 132 for a working example. Copy and modify.

### Handle Drag

```rust
draw(Size::new(50.0, 50.0), |painter, rect| {
    painter.fill(rect, Radius::Pill, Tone::Accent);
})
.on_drag(|app: &mut App, drag| {
    if drag.started() {
        app.start_position = drag.position;
    }
    app.current_position = drag.position;
})
```

### Show/Hide Elements

```rust
if app.show_details {
    col((
        title("Details"),
        text("..."),
    ))
}
```

Conditional rendering is just Rust `if` statements—no special syntax.

## Troubleshooting

**"Cannot open display" on Linux**

Ensure you're in an X11 session:
```bash
echo $DISPLAY
```

Should output `:0` or similar. If empty, start an X11 server or use Xvfb:
```bash
xvfb-run -a cargo run --example counter
```

**"Instant::now() panics" on WASM**

Only happens if you're using `std::time::Instant` directly. Use rui-native's provided time abstractions instead.

**Tests fail with "Harness not found"**

Make sure you're importing from the right module:
```rust
use rui_native::testing::Harness;
```

## Next Steps

1. **Run the counter example** — Familiarize yourself with how state flows
2. **Build a custom widget** — Copy `examples/segmented.rs` and modify it
3. **Write tests** — Use `Harness` to verify your widget works
4. **Read CLAUDE.md** — Understand the architecture and patterns
5. **Explore examples** — Study `controls`, `gallery`, `meter` to learn techniques

## Getting Help

- Check [CLAUDE.md](CLAUDE.md) for detailed documentation
- Read [tests/recipes.rs](tests/recipes.rs) for working examples
- Look at git history (`git log --oneline`) to see how features were built
- Open an issue on [GitHub](https://github.com/RockyWearsAHat/rui)

---

**Happy building!** 🚀
