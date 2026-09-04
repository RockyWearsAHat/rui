---
name: Bug Report
about: Report a bug or unexpected behavior
title: "[BUG] "
labels: bug
assignees: ""
---

## Description

A clear and concise description of what the bug is.

## Steps to Reproduce

A minimal code example that reproduces the issue:

```rust
use rui_native::{El, button, col, text};

struct App {
    // your state here
}

fn view(app: &App) -> El<App> {
    // your view here
}

fn main() -> Result<(), rui_native::Error> {
    rui_native::run("Example", App { /* ... */ }, view)
}
```

## Expected Behavior

What should happen?

## Actual Behavior

What actually happens? Include error messages if applicable:

```
error: ...
```

## Environment

- **Rust version:** `rustc --version`
- **OS:** macOS / Windows / Linux
- **rui-native version:** 0.1.0 (or commit hash if using git)
- **Target:** Native / WASM

## Additional Context

Any other context that might help debug this issue (screenshots, platform-specific info, etc).
