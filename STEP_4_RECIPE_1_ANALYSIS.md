# STEP 4: Recipe 1 WASM Backend Analysis

**Status**: Template Pattern — No commits documented (abstract pattern for future implementers)

## Overview

Recipe 1 documents the three-phase pattern for implementing a new platform backend. Unlike Recipe 2 (X11 Backend Implementation), which documents a **completed implementation** with concrete commit SHAs, Recipe 1 is a **replicable template** showing how to structure the work.

The WASM backend for browser environments exemplifies this pattern. It allows identical UI code to run in a browser by implementing the `Backend` trait and using a shared frame-loop function instead of a platform-specific event pump.

**File**: Section in CLAUDE.md (lines 197–254)
**Status**: Template/Pattern documentation only (no implementation commits in this repository)
**Applicability**: Recipe 2 (X11) follows this pattern; Wayland and other backends should too

---

## The Three-Phase Pattern

### Phase 1: Foundation
**Goal**: Platform-specific window + event pump working

- **Scope**: Create `src/shell/platform/{backend}.rs` implementing all 12 `Backend` trait methods
- **Core methods**: `open()`, `pump()`, `surface()`, `appearance()`, `present()`
- **Verification**: `cargo build --target <target>` succeeds
- **Pattern example**: Recipe 2 Phase 1 commit `a67d578` (748 lines, X11 window setup)

**Key decisions**:
- Trait boundary separates platform-specific from platform-agnostic code
- All 12 methods must be implemented; no partial implementations
- Coordinate transformation established (device pixels ↔ logical units)

### Phase 2: Enhancement
**Goal**: Full feature parity with other backends

- **Scope**: DPI detection, event translation, keyboard support, clipboard
- **Sub-components**:
  - Display metrics and scale factor (DPI/resolution)
  - Keyboard event translation (KeyCode → Key with modifiers)
  - Pointer events (motion, pressed, released)
  - Clipboard read/write
  - IME composition area setup
  - Accessibility tree updates
- **Verification**: Platform-specific test suite passes; `cargo test --lib` succeeds
- **Pattern example**: Recipe 2 Phase 2 commit `c42c0f0` (1220 lines, full enhancement)

**Key decisions**:
- Scale factor range validation (1.0–4.0 typical; others fail gracefully)
- Event translation table: 11+ X11 event types → rui `Event` types
- Modifier masks (shift/control/alt/meta) extracted and normalized

### Phase 3: Integration
**Goal**: Seamless cross-platform operation

- **Scope**: Frame loop wiring, cross-module coordination, parity validation
- **Sub-components**:
  - Time injection into `Memory::begin_frame()` (never wall-clock reads)
  - Event flow from pump → draw → handlers
  - Focus and interaction state management
  - Accessibility node creation and updates
  - Coordinate transformation in event handlers
- **Verification**: Parity tests pass; visual output identical to other backends; `cargo build --all-features` succeeds
- **Pattern example**: Recipe 2 Phase 3 commit `80e3003` (1321 lines, integration)

**Key decisions**:
- Platform branching via `#[cfg(...)]` in `src/shell/mod.rs` and `src/app.rs`
- Shared `draw()` function called by all backends
- Platform-specific `run()` function implementation

---

## Cross-Module Concerns

The three phases span multiple module boundaries:

1. **Time injection** (src/shell/mod.rs)
   - Platform loop injects elapsed time via `Memory::begin_frame()`
   - Never reads wall clock; tests inject exact time values

2. **Backend trait** (src/shell/mod.rs line 183+)
   - All backends implement identical 12-method interface
   - Trait boundary separates platform-specific from platform-agnostic

3. **Generic draw() function** (src/shell/mod.rs line 305+)
   - Called by all backends (native and WASM)
   - Takes `&mut Backend` and state; returns painted frame

4. **Event flow**
   - Native backends: `pump()` returns `Vec<Event>` (blocking event loop)
   - WASM: Collects from DOM listeners; both produce same `Event` types
   - Converted to `Input` by platform-agnostic code

5. **State persistence** (src/memory.rs)
   - `Memory` holds hover, focus, scroll, animation state
   - Persists across frames; identity is path-based
   - Queried by both native and WASM render loops

6. **Platform branching** (src/app.rs)
   - `App::run()` calls `shell::run()` with implementation gated by `#[cfg(target_arch)]`
   - Native backends: OS-specific event loop
   - WASM: JavaScript event listeners + requestAnimationFrame

---

## Template for Next Backend Implementation

When implementing a new backend (e.g., Wayland, DirectX, game engine), follow this checklist:

### Phase 1: Foundation
- [ ] Understand the platform's window/event model
- [ ] Create src/shell/platform/{backend}.rs
- [ ] Implement all 12 `Backend` trait methods
- [ ] Establish coordinate transformation (device ↔ logical)
- [ ] Verify compilation: `cargo build --target <target>`
- [ ] Verify no clippy warnings: `cargo clippy --target <target> -- -D warnings`

### Phase 2: Enhancement
- [ ] Extract platform-specific event loop differences
- [ ] Implement keyboard and input event translation
- [ ] Add DPI/scale factor detection and validation
- [ ] Implement clipboard read/write
- [ ] Set up IME composition support
- [ ] Write integration tests in tests/{backend}_integration.rs
- [ ] Verify: `cargo test --lib`

### Phase 3: Integration
- [ ] Wire into src/shell/mod.rs platform selector with `#[cfg(...)]`
- [ ] Create platform-specific `run()` function
- [ ] Verify time injection works (no `Instant::now()` in view code)
- [ ] Write parity tests in tests/{backend}_parity.rs
- [ ] Test cross-platform behavior consistency
- [ ] Verify visual output identical at same resolution/DPI
- [ ] Document in CLAUDE.md

### Verification Gates per Phase

**Phase 1 Gate (Foundation)**:
```bash
cargo build --target <target>
cargo clippy --target <target> -- -D warnings
grep -n "fn open\|fn pump\|fn surface\|fn appearance\|fn present\|fn is_open\|fn is_fullscreen\|fn set_fullscreen\|fn clipboard_text\|fn set_clipboard_text\|fn set_composition_area\|fn update_accessibility" src/shell/platform/{backend}.rs
# Verify all 12 methods present
cargo fmt --check
cargo test --lib  # Verify no test regressions
```

**Phase 2 Gate (Enhancement)**:
```bash
cargo build --release
cargo test --test {backend}_integration
cargo test --lib  # Full suite (379+ tests)
```

**Phase 3 Gate (Integration)**:
```bash
cargo test --test {backend}_parity
cargo build --all-features
# Verify visual parity: same frame rendered at different DPIs should look identical
```

---

## Key Pattern Proof Points

1. **Backend trait as abstraction boundary**
   - All backends implement same 12 methods
   - Platform-agnostic code above trait boundary
   - Platform-specific code below (shell/platform/)

2. **Shared draw() function**
   - Works for any backend implementing the trait
   - Single frame loop for all platforms
   - Proof: X11 and macOS/Windows backends all call `draw()`

3. **Event translation from platform to rui types**
   - Platform-specific event → `Event` (backend trait)
   - `Event` → `Input` (src/input.rs, platform-agnostic)
   - Handlers receive `Input` type (platform-independent)

4. **Coordinate transformation at boundary**
   - Device pixels ↔ logical units only in platform-specific code
   - All layout/paint in logical units
   - Scale factor applied only when reaching device pixels

5. **Identity is path-based**
   - Reordered list items preserve state via `El::key()`
   - No platform-specific identity mechanism needed
   - Works identically on all backends

---

## Next Steps for WASM Implementation

If implementing Recipe 1 (WASM backend):

1. **Research Phase**: Understand wasm32-unknown-unknown target and web APIs
   - Canvas API for rendering
   - requestAnimationFrame for frame loop
   - Event listeners (mousemove, click, keydown, etc.)

2. **Phase 1 implementation**:
   - Create src/shell/platform/wasm.rs
   - Implement Backend trait for wasm32 target
   - Get canvas rendering working (blit_bgra → Canvas drawing)

3. **Phase 2 implementation**:
   - Add pointer event translation (DOM events → rui Event)
   - Add keyboard event translation
   - Test keyboard input (letters, navigation keys, modifiers)

4. **Phase 3 implementation**:
   - Wire into shell::run() with wasm-specific event loop
   - Test parity with native backends
   - Document coordinate transformation for web coordinates

---

## References

- **CLAUDE.md Recipe 1 section**: Lines 197–254
- **Recipe 2 (X11)**: Completed implementation following this pattern
- **Backend trait**: src/shell/mod.rs line 183
- **Frame loop**: src/shell/mod.rs line 305+ (`draw()` function)
- **Event translation**: src/input.rs
- **Memory/state persistence**: src/memory.rs
