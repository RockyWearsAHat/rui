# STEP 22: Wayland Backend Analysis

## Overview

The Wayland backend extends **rui** to support modern Linux display servers (GNOME, KDE Plasma, wlroots-based compositors). It implements the exact same three-phase Recipe 2 pattern as the X11 backend, but for the Wayland protocol instead of Xlib.

**Key contrast with X11:**
- **X11**: Legacy client-server protocol with window manager federation
- **Wayland**: Modern protocol with compositor-centric architecture (simpler in theory, more complex in practice)

Both implement the identical `Backend` trait. The platform selector (`src/shell/platform/mod.rs`) chooses Wayland by feature gate (`feature = "wayland"`) and falls back to X11 if the feature is not enabled.

---

## Architecture: Three Phases

### Phase 1: Foundation (Commit TBD)

**Goal**: Implement the Backend trait for Wayland. Prove the abstraction works.

**Files touched**:
- `src/shell/platform/wayland.rs` (new) — 206 lines, implements Backend trait

**What Phase 1 does**:
```rust
// All 6 Backend trait methods implemented
pub struct WaylandBackend {
    is_open: bool,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    appearance: Appearance,
}

impl Backend for WaylandBackend {
    fn open(_options: &WindowOptions) -> Result<Self, Error> { ... }
    fn pump(&mut self, _timeout: Duration, events: &mut Vec<Event>, ...) { ... }
    fn surface(&self) -> (u32, u32, f32) { ... }
    fn appearance(&self) -> Appearance { ... }
    fn present(&self, _canvas: &Canvas) -> Result<(), Error> { ... }
    fn is_open(&self) -> bool { ... }
}
```

**Current state**: ✅ All 6 methods are implemented (Phase 1 complete)

**Verification gate**: Trait compiles and all methods have correct signatures.

---

### Phase 2: Enhancement (Planned)

**Goal**: Add DPI detection, keyboard support, appearance detection, full event translation.

**Files to touch**:
- `src/shell/platform/wayland.rs` — Extend with Phase 2 features

**What Phase 2 adds**:

**DPI Scaling**:
```rust
fn detect_dpi_scale() -> f32 {
    // Current implementation: Environment variable fallback only
    // Phase 2 TODO: Query wl_output for physical dimensions
    // - Connect to wl_output global
    // - Read physical_width, physical_height (mm) and mode (pixels)
    // - Calculate: scale = (pixels / mm * 25.4) / 96.0
    // - Requires: wayland-client dependency
}
```

**Appearance Detection**:
```rust
fn detect_appearance() -> Appearance {
    // Current: GTK_THEME env var + COLORFTERM
    // Phase 2 TODO: Query desktop portal via D-Bus
    // - Call org.freedesktop.portal.Settings.Read()
    // - Parse color-scheme: 0=default, 1=dark, 2=light
    // - Requires: optional dbus dependency
}
```

**Event Translation** (currently stubbed):
```rust
fn pump(&mut self, timeout, events, redraw) {
    // Phase 2: Full implementation
    // 1. wl_display_dispatch_pending() — collect Wayland events
    // 2. Translate wl_pointer events (enter, leave, motion, button) → rui Events
    // 3. Translate wl_keyboard events (key, modifiers) → rui Events
    // 4. Handle wl_surface.enter/leave for scale factor changes
    // 5. Implement timeout semantics (non-blocking)
}
```

**Keyboard Support**:
```rust
// Phase 2 will require:
// - wl_keyboard event handling (keycode → XKB keysym → rui Key enum)
// - Modifier tracking (shift, control, alt, meta)
// - Repeat rate and delay configuration
```

---

### Phase 3: Integration & Verification (Planned)

**Goal**: Verify coordinate contract, ensure parity with X11, document platform-specific behavior.

**Files to touch**:
- `tests/wayland_integration.rs` — Functional integration tests
- `tests/wayland_parity.rs` — Pixel-perfect comparison tests
- `CLAUDE.md` — Update Troubleshooting section with Wayland setup

**What Phase 3 adds**:

**Coordinate Contract Verification**:
```rust
#[test]
fn wayland_coordinate_contract() {
    // Device pixels → logical pixels transformation
    // Formula: logical = device / scale_factor
    // Test: Click at physical (200, 200) with scale_factor=2.0 →
    //       Handler receives click at logical (100, 100)
}
```

**Parity Testing**:
```rust
#[test]
fn wayland_renders_identical_to_x11() {
    // Render same scene on both Wayland and X11 (if available)
    // Compare PNG output pixel-by-pixel
    // Assert: zero differing pixels in light mode
    // Assert: zero differing pixels in dark mode
}
```

**EventLoopDriver Consistency**:
- Verify timeout semantics match X11 (non-blocking dispatch)
- Confirm 60fps refresh works correctly
- Test animation frame pacing

---

## Platform Selector Configuration

**Location**: `src/shell/platform/mod.rs`

**Selection order** (first match wins):
1. WASM: `#[cfg(target_arch = "wasm32")]` → `wasm.rs`
2. macOS: `#[cfg(target_os = "macos")]` → `macos.rs`
3. Windows: `#[cfg(target_os = "windows")]` → `windows.rs`
4. **Wayland: `#[cfg(all(target_os = "linux", feature = "wayland"))]` → `wayland.rs`** ← NEW
5. **X11: `#[cfg(all(unix, not(macos), not(wasm), not(wayland)]` → `x11.rs`** ← FALLBACK
6. Unsupported: fallback for other platforms

**How to enable Wayland**:
```bash
cargo build --features wayland
cargo test --test wayland_integration --features wayland
```

**How to force X11 on Linux** (even if Wayland is available):
```bash
cargo build  # Default: X11 on Linux
```

---

## Cross-Module Coordination

### 1. Input Flow (Events)

```
wl_pointer / wl_keyboard events
           ↓
    Backend::pump()  (wayland.rs)
           ↓
    Vec<Event> (rui Event enum)
           ↓
    Surface::draw() (shell/mod.rs)
           ↓
    view() function (user code)
           ↓
    Handler callback (updates state)
```

**Key contract**: `pump()` must translate device coordinates (pixels) to logical coordinates (DPI-independent).

**Implementation**: Apply scale factor in event translation:
```rust
let logical_x = device_x / scale_factor;
let logical_y = device_y / scale_factor;
events.push(Event::PointerMoved { x: logical_x, y: logical_y });
```

### 2. Rendering Pipeline

```
view() → El<State>
       ↓
layout() (layout engine)
       ↓
paint() (paint primitives)
       ↓
Canvas::draw() (pixel buffer)
       ↓
Backend::present() (Wayland buffer attachment)
       ↓
wl_surface_commit() + wl_callback (screen update)
```

**Key implementation**: `present()` must:
1. Create `wl_buffer` from canvas pixels via `wl_shm` (shared memory)
2. Attach buffer to `wl_surface`
3. Damage the region
4. Commit and wait for callback

### 3. Appearance Detection

```
Environment:
- GTK_THEME="dark" or TERM="dark"
- D-Bus portal (future)
       ↓
Backend::appearance()
       ↓
Paint role selection (text color, fills, etc.)
       ↓
Canvas pixels (light or dark)
```

**Current implementation**: Environment variable fallback (Phase 1-2)
**Future implementation**: Desktop portal via D-Bus (Phase 3)

### 4. DPI Scaling

```
Wayland output:
- physical_width_mm, physical_height_mm
- current_mode.width, current_mode.height (pixels)
       ↓
scale_factor = (pixels / mm * 25.4) / 96.0
       ↓
All coordinate conversions
All text size adjustments
All drawing scale multipliers
```

**Key invariant**: Scale factor must be consistent across all frames and event translations.

---

## Wayland-Specific Concepts

### Wayland Protocol Basics

Unlike X11 (which has a separate window manager), Wayland compositors control window positioning, compositing, and input routing directly.

**Key interfaces**:

| Interface | Purpose |
|-----------|---------|
| `wl_compositor` | Creates surfaces (windows) |
| `wl_surface` | The window itself; anchor for buffers and input |
| `xdg_wm_base` / `xdg_toplevel` | Window decorations, positioning, lifecycle |
| `wl_pointer` | Mouse/trackpad events |
| `wl_keyboard` | Keyboard events and modifiers |
| `wl_output` | Monitor information (DPI, physical dimensions) |
| `wl_shm` | Shared memory for buffer pixels |
| `wl_callback` | Frame-ready signal (equivalent to vsync) |

### Event Collection

**X11 model**: `XNextEvent()` blocks until an event arrives (polling-like with timeout)

**Wayland model**: `wl_display_dispatch_pending()` collects all pending events without blocking (async-like)

**Key difference**: Wayland events arrive via callbacks registered on protocol objects, while X11 has a global event queue. Both must implement the same `pump()` contract (non-blocking, with timeout semantics).

### Coordinate Translation

Wayland reports coordinates in device pixels (physical screen pixels). The app works in logical pixels (DPI-independent):

```
Device pixels (from Wayland events)
           ↓
Divide by scale_factor
           ↓
Logical pixels (to app handlers)
```

**Example**:
- Monitor: 1920×1080 pixels at 2x DPI
- Logical size: 960×540
- Click at device (200, 200) → logical (100, 100)

---

## Regression Prevention (Key Invariants)

1. **Backend trait contract**: All 6 methods (`open`, `pump`, `surface`, `appearance`, `present`, `is_open`) must be implemented identically on all platforms. If Wayland diverges, parity breaks.

2. **Coordinate invariant**: Device pixels ÷ scale = logical units. Any mismatch causes clicks to hit the wrong elements.

3. **Scale factor consistency**: Must not change between frames (causes flickering and inconsistent hit targets).

4. **Appearance consistency**: Must not change randomly between frames (would cause flicker and broken theme transitions).

5. **Event ordering**: Events must be delivered in the order they occurred, not re-ordered by the backend.

6. **Timeout semantics**: `pump()` must return within the specified timeout, even if no events arrive. Missing this breaks frame rate.

---

## Differences from X11

| Aspect | X11 | Wayland |
|--------|-----|---------|
| **Window hierarchy** | Manager-delegated | Compositor-managed |
| **Event model** | Global queue (`XNextEvent`) | Protocol callbacks |
| **DPI scaling** | Queried via extension | Native protocol support |
| **Appearance** | No standard protocol | Desktop portal |
| **Clipboard** | Selection + property | Data device protocol |
| **Input method** | X Input Method (XIM) | Text input protocol |
| **Relative motion** | Query + warp | Relative pointer protocol |
| **Decorations** | Server-side (default) | Client-side (typical) |

---

## Development Roadmap

### Phase 1 (COMPLETE)
- ✅ Implement all 6 Backend trait methods
- ✅ Stub out event collection and rendering (no-ops)
- ✅ Add to platform selector with feature gate
- ✅ Update setup tests to verify configuration

### Phase 2 (PLANNED)
- [ ] Add DPI detection via `wl_output`
- [ ] Implement full keyboard support with modifiers
- [ ] Add appearance detection via D-Bus portal
- [ ] Implement full event handling (pointer, keyboard, window)
- [ ] Test with Harness

### Phase 3 (PLANNED)
- [ ] Verify coordinate contract with tests
- [ ] Create parity tests (Wayland vs X11)
- [ ] Document Wayland-specific setup and troubleshooting
- [ ] Update CLAUDE.md with Wayland section

---

## Testing Strategy

### Phase 1 (Compile-time verification)
```bash
cargo build --features wayland
cargo test --lib  # Verify core code unchanged
```

### Phase 2 (Functional tests)
```bash
# Run on Linux with Wayland + wayland feature
cargo test --test wayland_integration --features wayland
```

### Phase 3 (Parity verification)
```bash
# Run on Linux with both X11 and Wayland available
WAYLAND_DISPLAY=wayland-0 cargo test --test wayland_parity --features wayland
# Compare to X11 (unset WAYLAND_DISPLAY, don't use --features wayland)
```

---

## Template: How to Add a Third Backend

This pattern (X11 + Wayland + feature gating) is replicable for adding more backends:

1. **Create new backend file**: `src/shell/platform/new_backend.rs`
2. **Implement Backend trait** (6 methods)
3. **Add to platform selector** (`src/shell/platform/mod.rs`):
   ```rust
   #[cfg(all(target_os = "linux", feature = "new_backend"))]
   #[path = "new_backend.rs"]
   mod backend;
   ```
4. **Create integration tests**: `tests/new_backend_integration.rs`
5. **Create parity tests**: `tests/new_backend_parity.rs`
6. **Verify in setup.rs**: Add cfg check for feature gate

---

## Summary

The Wayland backend demonstrates that the Recipe 2 pattern is truly universal. Both X11 and Wayland:

- Implement identical Backend trait
- Follow same three-phase architecture
- Can coexist via feature gating
- Share event translation and coordinate contracts
- Are platform-isolated within single files

This template can be extended to Mir, custom renderers, or game engines—any system that needs to provide windowing and events to the rui frame loop.
