# Contributing to rui-native

Thank you for your interest in contributing to **rui-native**! This document outlines the process and guidelines for contributing code, documentation, and ideas.

## Getting Started

### Prerequisites

- **Rust 1.85+** — Verify with `rustc --version`. Update with `rustup update` if needed.
- **Git** — For version control
- **A modern terminal** — For running build commands

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/RockyWearsAHat/rui.git
   cd rui
   ```

2. **Verify your setup:**
   ```bash
   cargo test --test setup
   ```
   This confirms your Rust version and pre-commit hook are configured.

3. **Run the full test suite:**
   ```bash
   cargo test
   ```
   All 347+ tests should pass.

4. **Try an example:**
   ```bash
   cargo run --example counter
   ```

## Development Workflow

### Before You Start

1. **Check existing issues** — Look for [open issues](https://github.com/RockyWearsAHat/rui/issues) to avoid duplicate work.
2. **Discuss large changes** — Open an issue to discuss major features before implementing.
3. **Keep commits small** — One logical change per commit makes review easier.

### Making Changes

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Write tests first** (TDD encouraged):
   ```rust
   #[test]
   fn your_feature_works() {
       // Test setup
       let result = your_function();
       // Assertions
       assert_eq!(result, expected);
   }
   ```

3. **Implement your feature:**
   - Add code to the appropriate module in `src/`
   - Follow the existing code style (see Code Style below)
   - Keep changes minimal—don't refactor unrelated code

4. **Format and lint:**
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   ```
   The pre-commit hook enforces this automatically.

5. **Run tests:**
   ```bash
   cargo test
   ```
   All tests must pass before committing.

6. **Commit with a clear message:**
   ```bash
   git commit -m "feat: Add your feature description

   More detailed explanation if needed. Reference any related issues."
   ```

### Commit Message Format

Follow the Conventional Commits style:

- **`feat:`** — A new feature
- **`fix:`** — A bug fix
- **`docs:`** — Documentation changes
- **`test:`** — Adding or updating tests
- **`refactor:`** — Code restructuring without behavior change
- **`perf:`** — Performance improvements
- **`chore:`** — Build, CI, or dependency updates

**Examples:**
```
feat: Add checkbox widget
fix: Correct Rect::contains coordinate handling
docs: Expand getting-started guide
test: Add backend consistency tests
```

### Before Submitting a PR

1. **Rebase on main:**
   ```bash
   git fetch origin
   git rebase origin/main
   ```

2. **Run the full test suite one more time:**
   ```bash
   cargo test
   ```

3. **Verify no new warnings:**
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

4. **Check documentation builds:**
   ```bash
   cargo doc --no-deps
   ```

5. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request** on GitHub with a clear description of what you changed and why.

## Code Style

### Naming Conventions

- **Structs/Types:** `PascalCase` — `Counter`, `Rect`, `Painter`
- **Functions/Methods:** `snake_case` — `view`, `on_click`, `text_size`
- **Constants:** `SCREAMING_SNAKE_CASE` — `DEFAULT_SIZE`, `MAX_WIDTH`
- **Private/Internal:** Prefix with underscore if needed — `_private_helper`

### Code Patterns

#### State Management

Always follow the **State → View → Handler** pattern:

```rust
// State: A simple Rust struct
struct App {
    counter: i32,
    name: String,
}

// View: A function that returns an element tree
fn view(app: &App) -> El<App> {
    col((
        text(&app.name),
        button("Increment").on_click(|app: &mut App| app.counter += 1),
    ))
}

// Handler: Functions receive mutable state as an argument
// (defined inline in the handler, shown above)
```

#### Element Building

Use method chaining for clarity:

```rust
button("Click")
    .text_size(16.0)
    .fill(Tone::Accent)
    .on_click(|app: &mut App| app.handle_click())
```

#### Comments

- Only add comments for **why**, not **what**
- Self-documenting code is preferred
- Comments should explain non-obvious constraints or workarounds

**Good:**
```rust
// Clamp fraction to 0..1 because meters don't display values outside this range
let clamped = fraction.max(0.0).min(1.0);
```

**Avoid:**
```rust
// Clamp fraction to 0..1
let clamped = fraction.max(0.0).min(1.0);
```

#### Unsafe Code

Unsafe code is **confined to platform modules only** (`src/shell/platform/*.rs`). All rendering, layout, text, and element logic is safe Rust. If you find yourself writing unsafe code elsewhere, you're probably solving the problem the wrong way.

### Documentation

Add documentation comments to public APIs:

```rust
/// Adds padding inside the element.
///
/// # Arguments
///
/// * `padding` — The padding in window-logical units (DPI-adjusted).
///
/// # Returns
///
/// Self, for method chaining.
///
/// # Coordinate System Contract
///
/// The padding is specified in window-logical units. At a 2x DPI scale,
/// a padding of 16 logical units becomes 32 device pixels.
pub fn pad(self, padding: f32) -> Self {
    // Implementation
}
```

For coordinate-related functions, explicitly document the coordinate system:

```rust
/// Returns whether this rect contains the given point.
///
/// # Coordinate System Contract
///
/// Both the rect and point are expected to be in window-logical units
/// (DPI-adjusted, not device pixels).
pub fn contains(&self, point: Point) -> bool {
    // Implementation
}
```

## Testing Guidelines

### Unit Tests

Place in the same module, at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_points_inside_bounds() {
        let rect = Rect::new(10.0, 10.0, 50.0, 50.0);
        assert!(rect.contains(Point::new(25.0, 25.0)));
        assert!(!rect.contains(Point::new(5.0, 5.0)));
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/custom_widget.rs
use rui_native::testing::Harness;

#[test]
fn custom_widget_responds_to_input() {
    let mut harness = Harness::new(App::default(), view);
    harness.click_text("Button");
    assert_eq!(harness.state().clicked, true);
}
```

### Test Names

Use descriptive names that explain what's being tested:

- ✅ `rect_contains_interior_points` — Clear what is being tested
- ❌ `test1` — Unclear what this tests

### Coverage

Aim for:
- **Unit tests** for individual functions/methods
- **Integration tests** for widgets and interactions
- **Platform tests** (optional) for platform-specific backends

Existing tests should remain passing. New tests should accompany new features.

## Platform Support

### macOS

- No special setup required beyond Xcode Command Line Tools
- Test with: `cargo test`

### Windows

- No special setup required beyond MSVC toolchain (installed with Rust)
- Test with: `cargo test`

### Linux

- Requires X11 development headers:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libx11-dev

  # Fedora/RHEL
  sudo yum install libX11-devel
  ```
- Test with: `cargo test`
- Headless systems: `xvfb-run -a cargo test`

### WebAssembly

- Install wasm-pack: `curl https://rustwasm.org/wasm-pack/installer/init.sh -sSf | sh`
- Test with: `wasm-pack test --headless --firefox`
- Requires Firefox

## Performance Considerations

- **Avoid allocations in `view()`** — It's called every frame
- **Use `&str` instead of `String`** when possible for text literals
- **Cache computed values** — Store expensive results in state, not view
- **Profile with release builds** — Debug builds are 10x slower

```bash
# Profile your app
cargo build --release
# Use profiling tools: flamegraph, perf, Instruments (macOS)
```

## Documentation

### Code Documentation

- All public items should have documentation comments
- Examples in doc comments are encouraged
- Reference CLAUDE.md for complex patterns

### User-Facing Documentation

- Update [GETTING_STARTED.md](GETTING_STARTED.md) for new user-facing features
- Update [CLAUDE.md](CLAUDE.md) for architectural or pattern changes
- Add recipes to `tests/recipes.rs` for new widgets

## Bug Reports

If you find a bug:

1. **Check if it's already reported** — Search [existing issues](https://github.com/RockyWearsAHat/rui/issues)
2. **Create a minimal reproduction** — The smallest code that demonstrates the bug
3. **Include platform and Rust version** — `rustc --version`, `uname -s`
4. **Describe expected vs. actual behavior** — What should happen vs. what does

## Feature Requests

Have an idea? Great! Please:

1. **Check existing issues** — Look for similar requests
2. **Describe the use case** — Why do you need this feature?
3. **Show examples** — Code snippets or pseudocode help
4. **Discuss design** — API design is important; discuss before implementing

## Review Process

When you open a PR:

1. **Automated checks run** — Tests, linting, documentation build
2. **Code review** — Maintainers review your changes
3. **Feedback** — You may receive suggestions or questions
4. **Revisions** — Address feedback by pushing new commits
5. **Merge** — Once approved, your PR is merged to main

## Recognition

Contributors are recognized in:

- **Commit history** — Your name is in git permanently
- **Release notes** — Major contributions are highlighted
- **Contributors list** — We maintain a list of contributors

## Questions?

- **Documentation:** See [CLAUDE.md](CLAUDE.md)
- **Examples:** Check `examples/` and `tests/recipes.rs`
- **Issues:** Open a discussion on GitHub

---

Thank you for contributing to rui-native! Your work makes the library better for everyone. 🙏
