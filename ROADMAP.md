# rui-native Roadmap

This document outlines the planned direction for rui-native over the coming quarters. Items are listed by priority and estimated timeline.

## Vision

rui-native aims to be the declarative, zero-dependency UI library for Rust, providing:

- **One codebase, all platforms** — Write once, compile to macOS, Windows, Linux, and WebAssembly
- **No dependencies** — Ship lightweight binaries with no external crates to audit
- **Simple mental model** — State → View → Handler, no closures or interior mutability
- **Platform-native rendering** — Pixel-perfect, hardware-accelerated on each platform

## Current Status (v0.1.0)

✅ **Complete:**
- Core element system (column, row, layers, drawing)
- Flexbox-like layout engine with auto-sizing
- Text rendering with TrueType font loading (zero-dependency)
- Color system with light/dark mode support
- Event handling (click, drag, keyboard, scroll)
- Testing framework (Harness)
- macOS, Windows, X11, and WASM backends
- 347+ tests (100% pass rate)
- Comprehensive documentation

## v0.2.0: Platform Completeness (Q4 2026)

### Wayland Support

- Implement native Wayland backend for Linux
- Maintain X11 support alongside Wayland
- Auto-detect and use appropriate backend

**Why:** X11 is aging; Wayland is the future for Linux. Supporting both ensures compatibility across distributions.

**Estimated effort:** 200 LOC platform module

### Accessibility (a11y)

- Screen reader support (VoiceOver on macOS, Narrator on Windows, Orca on Linux)
- Keyboard-only navigation
- ARIA-like annotations for semantic meaning

**Why:** Many users rely on accessibility features. This is table-stakes for a production UI library.

**Estimated effort:** 500 LOC for core framework + platform-specific hooks

### Mobile Backends (iOS/Android)

- Investigate iOS (UIKit) backend
- Investigate Android (JNI) backend
- Share 90% of rendering code between desktop and mobile

**Why:** Rust is growing on mobile. A truly multi-platform UI library should target mobile.

**Estimated effort:** 3000+ LOC (one backend per OS)

**Status:** Exploration phase; awaiting community feedback on priority.

## v0.3.0: Widget Library (Q1 2027)

### Built-in Widgets

Expand beyond primitives with commonly-needed widgets:

- **Form controls:** `text_input()`, `select()`, `combobox()`
- **Data display:** `table()`, `list()`, `tree()`
- **Navigation:** `menu_bar()`, `context_menu()`, `popover()`
- **Feedback:** `progress()` (improvements), `spinner()`, `toast()`
- **Dialogs:** `dialog()`, `alert()`, `file_picker()`

**Why:** Users shouldn't need to build every widget from scratch. These are high-frequency patterns.

**Constraint:** All widgets built from primitives; no external graphics libraries.

**Estimated effort:** 1500 LOC for core widgets + 500 LOC tests each

### CSS-Like Styling

- Support for style sheets (optional, not required)
- Uniform styling system across themes
- Animation support (transitions, keyframes)

**Why:** Complex applications need flexible styling without repeating code.

**Status:** Awaiting design input; should coordinate with community.

## v0.4.0: Performance & Optimization (Q2 2027)

### Rendering Optimization

- Dirty region tracking (only redraw changed areas)
- Geometry caching (cache computed layouts)
- Text shaping optimization (cache glyph metrics)
- GPU-accelerated rendering (optional, not required)

**Why:** Large UIs (500+ elements) can bottleneck on layout and painting.

**Estimated effort:** 500 LOC for framework + 200 LOC per optimization

### Bundle Size Reduction

- Tree-shake unused code with feature flags
- Optimize WASM bundle size (current: 1.2M)
- Support for linking system fonts instead of embedding

**Why:** WASM deployment needs smaller bundles. Native apps need faster startup.

**Estimated effort:** 300 LOC configuration + platform-specific optimizations

## v0.5.0: Ecosystem & Tooling (Q3 2027)

### Development Tools

- Hot reload (change code, see UI update without restart)
- Inspector/debugger (inspect element tree, state, styling)
- Theme designer (visual tool for creating themes)

**Why:** Faster iteration is key to developer satisfaction.

**Estimated effort:** 800 LOC for framework + 500 LOC per tool

### Package Registry

- Curated registry of third-party widgets and themes
- Standard packaging/distribution format
- Dependency resolution

**Why:** Ecosystem growth requires easy package discovery.

**Status:** Awaiting sufficient third-party contributions.

### Documentation Expansion

- Video tutorials
- Architecture deep-dives
- Porting guide (converting from immediate-mode, retained-tree systems)
- Platform-specific guides

**Estimated effort:** 5000+ words of content + 3-5 videos

## Future Considerations

### No-op Rendering (Immediate Mode)

Support "immediate-mode" UI where the view function has side effects:

```rust
// Proposed (currently not supported)
fn view(app: &App, ctx: &mut DrawContext) {
    ctx.label("Count: {}", app.count);
    if ctx.button("Increment") {
        // Handle click here
    }
}
```

**Decision:** Deferred pending API design discussion. Current state-driven model is simpler for most use cases.

### Native Look & Feel

Support native platform themes automatically:

```rust
// Auto-use platform defaults: Aqua on macOS, Fluent on Windows, etc.
let appearance = Appearance::native();
```

**Decision:** Low priority for v0.1-v0.3; investigate in v0.4.

### Embedded/Headless Targets

Support building UIs for embedded systems (e.g., Raspberry Pi displays).

**Decision:** Blocked on platform support; investigate after mobile backends.

### Game Engine Integration

Native bindings for game engines (Bevy, Godot, etc.) so games can use rui-native for UI.

**Decision:** High-value but complex; scope for v0.5+ only after core is stable.

---

## How to Influence the Roadmap

1. **Vote with issues** — Star and comment on features you care about
2. **Contribute** — Implement features from the roadmap and submit PRs
3. **Discuss design** — Open issues to propose changes or refinements
4. **Feedback** — Report gaps or pain points you encounter

## Release Cadence

- **Major versions (v0.X → v1.0):** Annually or on significant milestone
- **Minor versions (v0.X.0 → v0.X.1):** Quarterly or as features stabilize
- **Patches:** As-needed for bug fixes

**Note:** Semantic versioning will be more strict post-v1.0.

## Stability Guarantee

All APIs in v0.1.0+ are considered **stable**. Breaking changes will only occur in major versions (e.g., v1.0) and will be announced with a migration guide.

Unstable/experimental APIs will be marked with `#[unstable]` or equivalent, allowing us to iterate on new features without breaking user code.

---

**Questions?** Open an issue or start a discussion on [GitHub](https://github.com/RockyWearsAHat/rui).
