# Recipe 2: X11 Backend — Quick Reference and Navigation

Reference document for Recipe 2 (X11 Backend Implementation). This file provides a quick-reference summary and navigation guide for all Recipe 2 extracted documentation.

## What Is Recipe 2?

Recipe 2 is a detailed example of implementing a platform backend for the rui UI library. It documents how to add X11 backend support for Linux systems by implementing the Backend trait (12 methods) and translating X11 events to rui's platform-agnostic event model.

**Status:** Complete and production-ready (1368 lines across 4 commits)
**Platforms:** Linux with X11 protocol
**Dependencies:** libx11, libxcb, libxkb (system libraries, not Rust crates)

## When to Use This Document

- **"I need to add a platform backend (Wayland, DirectX, game engine)"** → Start with STEP_5_RECIPE_2_SUMMARY.md (this file), then read STEP_5_RECIPE_2_ANALYSIS.md
- **"I need to implement window creation and event pump"** → Read STEP_5_RECIPE_2_ANALYSIS.md Phase 1 section
- **"I need to handle keyboard events and DPI scaling"** → Read STEP_5_RECIPE_2_EVENT_TRANSLATION.md and STEP_5_RECIPE_2_COORDINATE_CONTRACT.md
- **"I need to know how X11 backend integrates with the rest of rui"** → Read STEP_5_RECIPE_2_CROSS_MODULE_CONCERNS.md
- **"I need to verify my implementation is complete"** → Use STEP_5_RECIPE_2_VERIFICATION_GATES.md
- **"Is this pattern replicable for other backends?"** → Read STEP_5_RECIPE_2_TEMPLATE_VALIDATION.md

## Three-Phase Pattern Overview

Recipe 2 follows a proven three-phase implementation pattern:

### Phase 1: Foundation (748 lines)
**Goal:** Implement the Backend trait with basic window and event pump

**Scope:**
- Create src/shell/platform/x11.rs
- Implement 12 Backend trait methods
- Get window creation, event queue, and rendering pipeline working
- No keyboard/DPI handling yet; basic window only

**Verification:**
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo test --lib  # Should pass (379 tests)
```

**Key insight:** Start minimal; every Backend method must exist, but can be skeletal.

### Phase 2: Enhancement (1220 lines)
**Goal:** Add platform-specific features (DPI, keyboard, event translation)

**Scope:**
- Detect DPI and compute scale_factor
- Translate X11 event types to rui Events
- Implement keyboard event handling with XKB
- Add modifier key support (shift, control, alt)

**Verification:**
```bash
cargo test --test x11_integration
cargo test --lib  # Full suite (379 tests)
```

**Key insight:** Event translation is the biggest work; this phase is where 50% of backend logic lives.

### Phase 3: Integration (1321 lines)
**Goal:** Wire into shared systems and achieve cross-platform parity

**Scope:**
- Frame loop integration in app.rs
- Focus and IME handling
- Accessibility (AT-SPI2 on X11)
- Clipboard support
- Cross-platform parity testing

**Verification:**
```bash
cargo test --test x11_parity
cargo test --test interaction
```

**Key insight:** Integration surfaces friction points with other modules; resolve them here.

### Phase 4: Polish (1368 lines)
**Goal:** Refinement, documentation, examples

**Scope:**
- Documentation of cross-module interactions
- Example widgets or features using the backend
- Performance tuning
- Accessibility audit

## Documentation Files and Their Purpose

| File | Purpose | Read If | Time to Read |
|------|---------|---------|--------------|
| **STEP_5_RECIPE_2_ANALYSIS.md** | Three-phase breakdown with commit list and line counts | You're implementing Phase 1, 2, or 3 | 15 min |
| **STEP_5_RECIPE_2_VERIFICATION_GATES.md** | Acceptance criteria and test commands per phase | You need to know if a phase is complete | 10 min |
| **STEP_5_RECIPE_2_CROSS_MODULE_CONCERNS.md** | Module interactions and friction point resolutions | You're hitting cross-module bugs or designing interactions | 20 min |
| **STEP_5_RECIPE_2_COORDINATE_CONTRACT.md** | Coordinate transformation (device → logical) | You're implementing pointer events or layout | 15 min |
| **STEP_5_RECIPE_2_EVENT_TRANSLATION.md** | X11 event types mapped to rui Events | You're translating events or debugging event handling | 20 min |
| **STEP_5_RECIPE_2_TEMPLATE_VALIDATION.md** | Proof that pattern is replicable for Wayland, etc. | You're implementing a new backend and want confidence the template works | 10 min |
| **STEP_5_RECIPE_2_SUMMARY.md** | This file — quick reference and navigation | You just arrived and need to orient | 5 min |

## Key Concepts

### Backend Trait
A 12-method interface that all platform backends implement. Abstracts window, input, rendering, clipboard, accessibility.

```rust
pub trait Backend: Sized {
    fn open(options: &WindowOptions) -> Result<Self, Error>;
    fn pump(&mut self, timeout: Duration, events: &mut Vec<Event>, ...) -> Result<(), Error>;
    fn surface(&self) -> (u32, u32, f32);  // width, height, scale_factor
    fn appearance(&self) -> Appearance;    // light or dark theme
    fn present(&self, canvas: &Canvas) -> Result<(), Error>;
    fn is_open(&self) -> bool;
    fn is_fullscreen(&self) -> bool;
    fn set_fullscreen(&self, filling: bool) -> Result<(), Error>;
    fn clipboard_text(&self) -> Result<Option<String>, Error>;
    fn set_clipboard_text(&self, text: &str) -> Result<(), Error>;
    fn set_composition_area(&self, area: Option<Rect>) -> Result<(), Error>;
    fn update_accessibility(&self, update: &AccessUpdate) -> Result<(), Error>;
}
```

### Event Translation
The process of converting protocol-specific events (X11 MotionNotify, ButtonPress, KeyPress) to rui's platform-agnostic Event enum.

**Example:** X11 ButtonPress(button=1, x=100, y=200) → Event::Pointer { button: Primary, x: 100.0, y: 200.0, pressed: true }

### Coordinate Transformation
Converting between device pixels (what X11 uses) and logical units (what rui uses).

**Contract:** `logical = device / scale_factor`

### Cross-Module Friction
Where the X11 backend interacts with other rui modules:
- app.rs (frame loop)
- shell/mod.rs (platform selection)
- input.rs (event resolution)
- paint.rs (rendering)
- memory.rs (focus, scroll state)
- accessibility.rs (semantic tree)

## Verification Checklist

Before claiming a phase is complete, verify:

**Phase 1 Complete:**
- [ ] src/shell/platform/x11.rs exists with all 12 Backend trait methods
- [ ] `cargo build` succeeds with no warnings
- [ ] `cargo test --lib` passes (379 tests)
- [ ] Window opens and closes without crashing

**Phase 2 Complete:**
- [ ] DPI detection works (test with GDK_SCALE=2)
- [ ] Keyboard events translate to rui Key (test Tab, Enter, arrows)
- [ ] Mouse events work (click, drag, scroll)
- [ ] Modifiers (shift, control, alt) are correctly decoded
- [ ] `cargo test --test x11_integration` passes

**Phase 3 Complete:**
- [ ] Focus ring appears when using Tab key
- [ ] Text input works (characters appear in text fields)
- [ ] IME composition area follows cursor
- [ ] Accessibility tree works with screen reader
- [ ] Visual output matches macOS/Windows (`cargo run -p rui --example gallery`)
- [ ] `cargo test --test x11_parity` passes

## Commit Reference

All Recipe 2 commits are in git history; verify documentation accuracy:

```bash
# View Phase 1: Basic window and event pump (748 lines)
git show a67d578eea41560c26fd7a6548c0d089223f3d70 -- src/shell/platform/x11.rs

# View Phase 2: Event translation and DPI (1220 lines)
git show c42c0f05b3d75976665377a16257c36c472debc1 -- src/shell/platform/x11.rs

# View Phase 3: Integration and parity (1321 lines)
git show 80e3003563c26952e4d63c52d8eb8f5052cb463c -- src/shell/platform/x11.rs

# View Polish: Refinement (1368 lines)
git show 991167a3898d643199a6e0b9dfa461be31cae264 -- src/shell/platform/x11.rs
```

## Running Tests

```bash
# Library tests (should always pass)
cargo test --lib

# X11-specific tests
cargo test --test x11_integration
cargo test --test x11_parity
cargo test --test interaction

# Visual verification
cargo run -p rui --example gallery -- .
```

## For Next Backend Implementer

If you're implementing Wayland, DirectX, or another backend:

1. **Read this file** (5 min) to understand the pattern
2. **Read STEP_5_RECIPE_2_ANALYSIS.md** (15 min) to see how phases are structured
3. **Read STEP_5_RECIPE_2_TEMPLATE_VALIDATION.md** (10 min) to confirm the pattern applies to your backend
4. **Implement Phase 1** following the X11 Phase 1 section
5. **Run STEP_5_RECIPE_2_VERIFICATION_GATES.md** Phase 1 tests
6. **Repeat for Phase 2 and 3**

The pattern is identical; only protocol-specific details change.

## Architecture Overview

```
Your Application
    ↓
app.rs (frame loop)
    ├→ Backend::pump()  [X11 converts XEvents to rui Events]
    ├→ layout.rs        [layout engine]
    ├→ paint.rs         [call handlers, render to canvas]
    ├→ canvas.rs        [rasterize to device pixels, apply scale_factor]
    └→ Backend::present() [X11 pushes pixels to window]
```

**Key principle:** Platform backends are thin (event translation + window management); all UI logic is platform-agnostic.

## Common Mistakes

1. **Applying scale_factor twice** — Check coordinate transformation logic; divide in pump(), multiply in canvas.rs only
2. **Not translating keysyms** — X11 gives KeyCode (position); must use XKB to get Key (meaning)
3. **Forgetting Button 4/5** — X11 uses buttons 4 and 5 for scroll; translate to Event::Scroll, not Event::Pointer
4. **Mixing device and logical units** — Pick one per function; convert at boundaries (pump for input, present for output)
5. **Not handling scale factor changes** — Listen for ConfigureNotify or monitor XrandrOutputChangeNotify to detect DPI changes

## Questions?

- "How do I implement Phase 1?" → STEP_5_RECIPE_2_ANALYSIS.md section "Phase 1: Foundation"
- "What events do I need to translate?" → STEP_5_RECIPE_2_EVENT_TRANSLATION.md
- "How do modifiers work?" → STEP_5_RECIPE_2_EVENT_TRANSLATION.md section "Modifier decoding"
- "Where does my code fit?" → STEP_5_RECIPE_2_CROSS_MODULE_CONCERNS.md
- "Is my implementation complete?" → STEP_5_RECIPE_2_VERIFICATION_GATES.md
- "Will this pattern work for Wayland?" → STEP_5_RECIPE_2_TEMPLATE_VALIDATION.md

---

**Recipe 2 Status:** ✓ Complete, Production-Ready, Proven Replicable

Last validated: 2026-09-14
