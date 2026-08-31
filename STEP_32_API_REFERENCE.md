# STEP 32: API Reference Documentation Generation

## Overview

Generate comprehensive, auto-updated API reference documentation from Rust code comments and type definitions. This step creates the `docs/api/` section of the website with searchable, cross-linked documentation for every public type, function, and module.

**Goal:** Provide developers with a complete, searchable API reference that's automatically kept in sync with the code.

---

## Architecture: From Code to HTML

### Source of Truth: Rust Code Comments

All API documentation lives in the code itself as Rust doc comments:

```rust
/// Creates a button widget with a label and click handler.
///
/// # Arguments
///
/// * `label` - Text displayed on the button
/// * `handler` - Function called when button is clicked
///
/// # Example
///
/// ```rust
/// button("Click me", |app| app.count += 1)
/// ```
///
/// # Styling
///
/// Use `.fill()` to customize button appearance:
///
/// ```rust
/// button("Submit", |app| app.submit())
///   .fill(Tone::Accent)
/// ```
pub fn button<S: 'static>(
    label: &str,
    handler: impl Fn(&mut S) + 'static,
) -> El<S> {
    // implementation
}
```

### Processing Pipeline

```
Rust code comments (doc comments)
    ↓
cargo doc (generates HTML with rustdoc)
    ↓
Extract doc JSON (cargo metadata)
    ↓
Custom parser (parse examples, code blocks, links)
    ↓
Zola markdown files (auto-generated)
    ↓
Zola builds HTML
    ↓
Deployed to rui.dev/docs/api/
```

### Implementation Approaches

**Approach 1: rustdoc Default (Simplest)**
- Use `cargo doc --no-deps --open` to generate docs
- Host generated HTML on rui.dev
- Pros: Zero custom tooling, official Rust format
- Cons: Less customizable styling, separate from main site design

**Approach 2: Custom Markdown Generation (Recommended)**
- Parse `cargo metadata` JSON output
- Extract doc comments and convert to Markdown
- Generate Zola-compatible `.md` files in `docs/api/`
- Pros: Integrates with website, consistent styling, searchable
- Cons: Custom tooling required

**Approach 3: Hybrid (Best)**
- Use rustdoc for detailed documentation (deployed separately)
- Generate simplified Markdown overview for website
- Link from website to detailed docs
- Best of both worlds: searchable website + detailed reference

**Recommendation:** Use Approach 3 (Hybrid) for launch.

---

## Phase 1: Module Index & Overview

### Top-Level Module Structure

Create `docs/api/_index.md` with overview of all modules:

```markdown
+++
title = "API Reference"
description = "Complete API reference for the rui UI library"
template = "api-index.html"
+++

# API Reference

The rui library is organized into logical modules, each handling a specific aspect of UI building.

## Core Modules

### [element](element/)
UI element tree. `El<T>` is the root type. Builders for structure (layout) and containers.
- Types: `El`, `Attr`, `Prop`
- Functions: `text`, `button`, `checkbox`
- See: [Element guide](/docs/guide/elements/)

### [style](style/)
Layout and appearance: `Length`, `Radius`, `Tone` (color roles), `Align`, `Justify`.
- Types: `Tone`, `Align`, `Justify`, `Length`, `Radius`, `Insets`
- Example: `.fill(Tone::Accent)` applies accent color
- See: [Styling guide](/docs/guide/styling/)

### [layout](layout/)
Flexbox-like layout engine. Single-axis stacking, flow wrapping, scroll, layer support.
- Functions: `row`, `col`, `flow`, `scroll`, `layer`
- See: [Layout guide](/docs/guide/layouts/)

### [paint](paint/)
Drawing abstraction. `Painter` API used by all elements.
- Types: `Painter`, `Visual`
- See: [Custom drawing guide](/docs/guide/drawing/)

### [widgets](widgets/)
High-level components (recipes). Built from primitives: `button`, `checkbox`, `segmented`, `slider`, `meter`.
- Functions: `button`, `checkbox`, `segmented`, `slider`, `toggle`, `radio`, `meter`, `progress_bar`
- See: [Widgets catalog](/docs/examples/)

### [input](input/)
Event handling. Immediate-mode input queries without event listeners.
- Types: `Event`, `Key`, `Click`, `Drag`
- Functions: `on_click`, `on_drag`, `on_key`
- See: [Events guide](/docs/guide/events/)

### [memory](memory/)
Persistent state between frames. Hover, focus, scroll, caret, animations.
- Types: `Memory`, `State`
- See: [State management guide](/docs/guide/state/)

### [theme](theme/)
Colors, spacing, type sizes. `Appearance` (light/dark) and `Tone` (semantic roles).
- Types: `Tone`, `Appearance`
- Constants: Type scale, color palette
- See: [Theming guide](/docs/guide/theming/)

### [text](text/)
TrueType parsing, glyph rasterizing, text layout with kerning/ligatures.
- Types: `FontId`, `TextLayout`
- See: [Text guide](/docs/guide/text/)

### [canvas](canvas/)
Pixel buffer and rasteriser. Used internally by backends.
- Types: `Canvas`

### [image](image/)
PNG encoding for rendering to files.
- Functions: `save_png`

### [shell](shell/)
Platform window management. macOS/Windows/X11/WASM backends.
- Types: `Backend`
- See: [Platforms guide](/docs/guide/platforms/)

### [app](app/)
Application entry point. Couples state, view function, and event loop.
- Functions: `run`, `run_with_fonts`
- See: [Getting started](/docs/quickstart/)

### [testing](testing/)
`Harness` for driving the UI in tests without a window.
- Types: `Harness`
- See: [Testing guide](/docs/guide/testing/)

## Common Patterns

### Text & Display
```
text()           → Display static text
title()          → Larger heading text
label()          → Small label text
code_block()     → Monospace code
```

### Interaction
```
button()         → Clickable button
checkbox()       → Toggle boolean
segmented()      → Choose from list
slider()         → Numeric input
text_input()     → Text entry
```

### Layout
```
row()            → Horizontal stack
col()            → Vertical stack
flow()           → Line wrapping
scroll()         → Scrollable area
layer()          → Z-order stacking
```

### Styling
```
.fill(Tone)      → Background color
.gap(f32)        → Space between items
.pad(Insets)     → Padding
.border(Tone)    → Border color
.on_hover(|s| …) → Hover handler
```

### State & Events
```
.on_click(|s| …)    → Click handler
.on_drag(|s, d| …)  → Drag handler
.on_key(|s, k| …)   → Key handler
.on_focus(|s| …)    → Focus handler
```

## Search Tips

- **By name:** Type function/type name (e.g., "button", "Tone")
- **By pattern:** Search "button pattern" for examples
- **By module:** "layout::" for all layout functions
- **By example:** Copy-paste code snippet

## API Stability

- **Stable APIs:** Types and functions in core modules (element, style, widgets)
- **Beta APIs:** New features marked with `#[doc(hidden)]` or `[unstable]` attribute
- **Platform-specific:** Backend implementations may differ slightly between platforms

See [Versioning](/docs/guide/versioning/) for more.
```

### Module Listing Script

Create `tools/generate_api_docs.sh` to extract and generate docs:

```bash
#!/bin/bash

# Generate API documentation from Rust code comments
# Usage: ./tools/generate_api_docs.sh

set -e

echo "🔍 Generating API reference from source code..."

# 1. Extract module structure
cargo metadata --format-version 1 \
  | jq '.packages[0].targets[0] | {name, kind}' \
  > /tmp/rui_metadata.json

# 2. Run cargo doc to generate HTML
cargo doc --no-deps --release 2>/dev/null

# 3. Parse rustdoc output and convert to Markdown
# This would use a custom tool (Python or Rust) to:
# - Read target/doc/rui/*.html files
# - Extract module docs
# - Convert to Markdown format
# - Write to docs/api/

python3 tools/extract_docs.py

echo "✅ API reference generated in docs/api/"
echo "📖 View with: zola serve"
```

### Python Extractor (tools/extract_docs.py)

```python
#!/usr/bin/env python3
"""Extract Rust doc comments and convert to Markdown."""

import json
import re
from pathlib import Path

def extract_module_doc(html_file):
    """Parse rustdoc HTML and extract module documentation."""
    with open(html_file) as f:
        content = f.read()
    
    # Extract module description
    # Parse HTML to get doc comments, examples, types
    # Return as dict for Markdown generation
    pass

def generate_markdown(module_name, doc_dict):
    """Convert extracted docs to Markdown."""
    md = f"""+++
title = "{module_name} module"
description = "{doc_dict['summary']}"
template = "api-module.html"
weight = {doc_dict.get('weight', 0)}
+++

# {module_name}

{doc_dict['description']}

## Types

{format_types(doc_dict['types'])}

## Functions

{format_functions(doc_dict['functions'])}

## Examples

{doc_dict.get('examples', '')}
"""
    return md

if __name__ == '__main__':
    # Generate docs for all modules
    for module in ['element', 'widgets', 'style', 'layout', 'input', 'paint']:
        # Extract and generate
        pass
```

---

## Phase 2: Per-Module Documentation

### Element Module (docs/api/element.md)

```markdown
+++
title = "element"
template = "api-module.html"
+++

# element — UI element tree

The `element` module provides the core UI building block: `El<S>`, a description
of what to render for application state of type `S`.

## Core Types

### `El<S>`

An element describing how to render some part of the UI.

```rust
pub enum El<S> {
    // Element variants (view functions return these)
}
```

**Generic parameter:** `S` is your application state type.

**Example:**
```rust
fn view(app: &Counter) -> El<Counter> {
    col((
        text(format!("Count: {}", app.count)),
        button("Increment", |app| app.count += 1),
    ))
}
```

### Functions

#### `text(content: &str) -> El<S>`

Display static text.

**Example:**
```rust
text("Hello, world!")
```

#### `button(label: &str, handler: fn(&mut S)) -> El<S>`

Create a clickable button.

**Example:**
```rust
button("Click me", |app| app.clicked = true)
```

**Styling:**
```rust
button("Submit", |app| app.submit())
  .fill(Tone::Accent)    // Blue background
  .pad(Insets::uniform(8.0))  // Padding
```

## Building the Element Tree

Elements compose using tuples and closures:

```rust
col((                    // Vertical stack
    text("Title"),
    row((                // Horizontal stack
        button("A", |app| app.choice = 0),
        button("B", |app| app.choice = 1),
    )),
))
```

## See Also

- [Layout guide](/docs/guide/layouts/) — Understanding row/col/flow
- [Styling](/docs/guide/styling/) — Colors, padding, fonts
- [Events](/docs/guide/events/) — Handling clicks, drags, keys
```

### Widgets Module (docs/api/widgets.md)

```markdown
+++
title = "widgets"
template = "api-module.html"
+++

# widgets — Pre-built UI components

High-level components built from primitives. Use these for common controls.

## Common Widgets

### `button(label: &str, handler: fn(&mut S)) -> El<S>`

Clickable button.

```rust
button("Submit", |app| app.submit())
```

**Variations:**
```rust
button("Primary", |app| app.action())
  .fill(Tone::Accent)    // Blue

button("Danger", |app| app.delete())
  .fill(Tone::Error)     // Red
```

### `checkbox(label: &str, checked: bool, toggle: fn(&mut S)) -> El<S>`

Boolean toggle.

```rust
checkbox("Enable notifications", app.notify, |app| {
    app.notify = !app.notify;
})
```

### `segmented(choices: &[&str], selected: usize, on_select: fn(&mut S, usize)) -> El<S>`

Choose one from several options.

```rust
segmented(
    &["Small", "Medium", "Large"],
    app.size_index,
    |app, idx| app.size_index = idx,
)
```

### `slider(value: f32, on_change: fn(&mut S, f32)) -> El<S>`

Numeric input (0.0-1.0).

```rust
slider(app.volume, |app, v| app.volume = v)
```

**Styling:**
```rust
slider(app.brightness, |app, v| app.brightness = v)
  .fill(Tone::Warning)   // Orange track
```

### `meter(fraction: f32, tone: Tone) -> El<S>`

Read-only progress display.

```rust
meter(app.progress, Tone::Accent)  // 0.0-1.0
```

## Building Custom Widgets

Copy an exemplar and modify:

```rust
// From examples/segmented.rs, then customize:

pub fn my_widget<S: 'static>(
    value: bool,
    toggle: impl Fn(&mut S) + 'static,
) -> El<S> {
    draw(Size::new(32.0, 32.0), move |painter, rect| {
        painter.fill(
            rect,
            Radius::Pill,
            if value { Tone::Accent } else { Tone::Sunken },
        );
    })
    .on_click(move |state| toggle(state))
}
```

See [Exemplar templates](/docs/recipes/) for checkbox, segmented, meter patterns.

## See Also

- [Recipes](/docs/recipes/) — Templates for building custom controls
- [Examples](/docs/examples/) — Real widget usage examples
- [Styling](/docs/guide/styling/) — Customizing appearance
```

### Generating All Module Docs

For each module (element, widgets, style, layout, input, paint, theme, text, shell, memory, testing, app):

1. Extract doc comments from source
2. Format as Markdown
3. Generate `docs/api/{module}.md`
4. Create `docs/api/{module}/` subdirectory for detailed docs
5. Add cross-references and examples

**Automation:** Create `tools/generate_api_docs.sh` to run once per release

---

## Phase 3: Type Hierarchy & Cross-References

### Tone (Color Roles)

```markdown
# Tone — Semantic color roles

Colors by role, not by appearance. The same `Tone` renders differently in light/dark modes.

## Available Tones

| Tone | Light | Dark | Usage |
|------|-------|------|-------|
| `Accent` | #4A90E2 | #5B9FFF | Primary actions (buttons, highlights) |
| `Success` | #22c55e | #4ade80 | Positive feedback, confirmations |
| `Warning` | #f59e0b | #fbbf24 | Cautions, non-critical alerts |
| `Error` | #ef4444 | #f87171 | Errors, destructive actions |
| `Info` | #06b6d4 | #22d3ee | Information, neutral status |
| `Sunken` | #e2e8f0 | #1e293b | Backgrounds, recessed areas |
| `Muted` | #64748b | #94a3b8 | Disabled, secondary text |

## Examples

```rust
button("Save", |app| app.save())
  .fill(Tone::Accent)    // Blue (primary action)

button("Delete", |app| app.delete())
  .fill(Tone::Error)     // Red (destructive)

text("Upload complete")
  .fill(Tone::Success)   // Green
```

## Automatic Adaptation

The same code works in light and dark modes:

```rust
text("Status")
  .fill(Tone::Warning)   // Auto-adjusts color
```

Light mode renders orange; dark mode renders a lighter orange for contrast.

## Custom Colors

For colors not in the Tone palette, use RGB directly (not recommended for shipped apps):

```rust
// Built-in (recommended)
button("Primary", |app| app.go())
  .fill(Tone::Accent)

// Custom RGB (only for temporary/debug)
button("Custom", |app| app.go())
  .fill(Color::rgba(255, 128, 64, 255))  // Orange
```

Prefer semantic `Tone` values for consistency across light/dark modes.
```

### Event Types

```markdown
# Event types — User input

Events are the way applications respond to user input (clicks, drags, keys).

## Event Types

### Click

User clicked or tapped the element.

```rust
.on_click(|app| {
    app.counter += 1;
})
```

### Drag

User is dragging across the element.

```rust
.on_drag(|app, drag| {
    app.x = drag.from.x;
    app.y = drag.from.y;
})
```

The `Drag` struct contains:
- `from: Point` — Starting position
- `to: Point` — Current position
- `delta: Size` — Change since last frame
- `fraction: Size` — Normalized 0.0-1.0

### Key

User pressed a key.

```rust
.on_key(|app, key, modifiers| {
    match key {
        Key::Escape => app.close(),
        Key::Enter => app.submit(),
        _ => {}
    }
})
```

Available keys:
- `Escape`, `Enter`, `Tab`, `Space`
- `Up`, `Down`, `Left`, `Right`
- `Home`, `End`, `PageUp`, `PageDown`
- `Delete`, `Backspace`
- `A-Z`, `0-9` (character keys)
- `F1-F12` (function keys)

Modifiers:
- `modifiers.shift` — Shift key pressed
- `modifiers.control` — Control key pressed
- `modifiers.alt` — Alt/Option key pressed
- `modifiers.meta` — Cmd/Windows key pressed

## Testing Events

Use `Harness` to send events in tests:

```rust
let mut harness = Harness::new(Counter { count: 0 }, view);
harness.click_text("Increment");
assert_eq!(harness.state().count, 1);

harness.drag_from_to(Point::new(100.0, 100.0), Point::new(200.0, 100.0));
// Drag event processed
```

See [Testing guide](/docs/guide/testing/) for more.
```

---

## Phase 4: Search & Discoverability

### Search Index

Zola generates a search index automatically. For the website to include a search box:

**Add to base template (templates/base.html):**

```html
<div id="search-box">
  <input type="text" id="search-input" placeholder="Search API...">
  <div id="search-results"></div>
</div>

<script>
// Simple client-side search using Zola's search index
fetch('/search_index.en.json').then(r => r.json()).then(index => {
  document.getElementById('search-input').addEventListener('input', (e) => {
    const query = e.target.value.toLowerCase();
    const results = index.sections
      .concat(index.pages)
      .filter(item => 
        item.title.toLowerCase().includes(query) ||
        item.description?.toLowerCase().includes(query)
      )
      .slice(0, 10);
    
    document.getElementById('search-results').innerHTML = results
      .map(r => `<a href="${r.path}">${r.title}</a>`)
      .join('');
  });
});
</script>
```

### Auto-Linking

Doc comments with backtick-quoted types auto-link:

```rust
/// Returns an `El<S>` that displays the given `text`.
///
/// See also: [`button`], [`checkbox`]
pub fn text<S>(content: &str) -> El<S> { ... }
```

Renders as:
> Returns an [`El`](el.md) that displays the given [`text`](#).
> 
> See also: [`button`](widgets.md#button), [`checkbox`](widgets.md#checkbox)

### Search Tags

Add tags to markdown front matter for better search:

```markdown
+++
title = "button"
description = "Clickable button widget"
tags = ["button", "widget", "clickable", "action", "handler"]
+++
```

---

## Phase 5: Maintenance & Updates

### CI/CD Integration

Add to `.github/workflows/deploy.yml`:

```yaml
# In deploy job, after site build:
- name: Update API reference
  run: ./tools/generate_api_docs.sh
  
- name: Verify API docs built
  run: test -d docs/api && ls docs/api/*.md | wc -l
```

### Versioning

Keep API docs for multiple versions:

```
docs/
├── api/                # Latest (main branch)
├── api-0.2.0/         # Previous release
├── api-0.1.0/         # Earlier release
```

Link from landing page: "View docs for: [0.3.0] | [0.2.0] | [0.1.0]"

### Changelog

Update `docs/api/CHANGELOG.md` on each release:

```markdown
# API Changelog

## 0.3.0 (Unreleased)

### Added
- `text_input()` widget for text entry
- `SegmentedControl` generic over choice type
- Platform-specific accessibility APIs

### Changed
- `Tone::Muted` renamed to `Tone::Secondary`
- `on_hover()` merged into `on_drag()` for simplicity

### Removed
- Deprecated `checkbox_raw()` (use `checkbox()` instead)

## 0.2.0 (2024-MM-DD)

### Added
- X11 backend for Linux
- `meter()` widget for progress display
- Cross-platform parity tests
```

---

## Success Criteria

### Content Completeness

- [ ] All public types documented (element, widgets, style, input, etc.)
- [ ] All public functions documented with examples
- [ ] Type hierarchy documented (Tone, Appearance, Event types)
- [ ] Common patterns explained (button, checkbox, segmented)
- [ ] 5+ runnable code examples for each module
- [ ] Cross-references between related modules
- [ ] Changelog maintained

### Search & Discoverability

- [ ] Search index working
- [ ] Auto-linking between types
- [ ] Tags indexed for search
- [ ] No broken links (verification script)
- [ ] Breadcrumb navigation present

### Quality

- [ ] Lighthouse performance > 90 (pages load fast)
- [ ] Accessibility verified (WCAG 2.1 Level AA)
- [ ] Mobile responsive (readable on small screens)
- [ ] Dark mode support
- [ ] Code examples syntax-highlighted

### Maintenance

- [ ] Auto-generation script (`tools/generate_api_docs.sh`) working
- [ ] CI/CD runs on each commit
- [ ] Changelog up-to-date
- [ ] Versioned docs for multiple releases

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **1: Structure** | 1 day | Module index, directory layout, overview docs |
| **2: Per-module docs** | 2 days | Documentation for all major modules |
| **3: Types & references** | 1 day | Cross-links, type hierarchy, examples |
| **4: Search** | 1 day | Search index, auto-linking, tagging |
| **5: Maintenance** | 0.5 day | CI/CD integration, versioning setup |
| **Testing** | 1 day | Link verification, Lighthouse audits, QA |
| **Total** | ~6.5 days | Complete searchable API reference |

---

## Integration with STEP 31

This STEP 32 deliverable feeds into STEP 31's website:

- ✅ Documentation hub → `/docs/api/` with full API reference
- ✅ Search functionality → Integrated search box on all pages
- ✅ Examples → Runnable code in every API doc
- ✅ Cross-references → Links between guide docs and API docs

Once STEP 32 is complete, users can:
- Search for any function by name
- Jump from guide to API documentation
- See complete examples for every widget
- Understand type hierarchy and color roles

---

## Next Steps

1. **STEP 32A:** Extract module structure and create `docs/api/_index.md`
2. **STEP 32B:** Document major modules (element, widgets, style, input, paint)
3. **STEP 32C:** Add type hierarchy and common patterns
4. **STEP 32D:** Implement search index and cross-linking
5. **STEP 33:** Tutorial videos and recorded walkthroughs
6. **STEP 34:** Community growth and engagement
