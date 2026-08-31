# X11 Backend Module Dependencies & Data Flow

## Overview

The X11 backend integrates with rui's frame loop through a precise module dependency chain. This document maps how data flows from X11 events through to rendered pixels, and identifies the module boundary contracts.

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│ APPLICATION LAYER                                               │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ src/app.rs - App::run()                                     │ │
│ │ Owns: state<S>, view fn, event loop driver selection        │ │
│ └──────────────────────────┬──────────────────────────────────┘ │
│                            │ calls                               │
│                            ▼                                     │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ src/shell/mod.rs - Surface::draw() + turn()                │ │
│ │ Owns: frame loop, state machine, animation driver          │ │
│ └──────────────────────────┬──────────────────────────────────┘ │
└───────────────────────────┼──────────────────────────────────────┘
                            │ calls
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ FRAME LOGIC (PLATFORM-AGNOSTIC)                                 │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Backend trait (6 methods)                                   │ │
│ │ - open(&WindowOptions) -> Result<Self, Error>              │ │
│ │ - pump(&mut, Duration, events, redraw) -> Result<(), Err> │ │
│ │ - surface() -> (u32, u32, f32)                             │ │
│ │ - appearance() -> Appearance                                │ │
│ │ - present(&Canvas) -> Result<(), Error>                    │ │
│ │ - is_open() -> bool                                         │ │
│ └──────────────────────────┬──────────────────────────────────┘ │
│                            │ implemented by                      │
│                            ▼                                     │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Platform selector (cfg gates in src/shell/mod.rs)          │ │
│ │ if linux: use x11::Window                                   │ │
│ │ if windows: use windows::Window                             │ │
│ │ if macos: use macos::Window                                 │ │
│ │ if wasm: use wasm::Window                                   │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                            │ implement
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ X11 PLATFORM LAYER                                              │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ src/shell/platform/x11.rs - Window struct + Backend impl   │ │
│ │ ┌────────────────────────────────────────────────────────┐  │ │
│ │ │ Data:                                                  │  │ │
│ │ │ - display: *mut c_void (XDisplay*)                     │  │ │
│ │ │ - window: c_ulong (X11 window ID)                      │  │ │
│ │ │ - context: *mut c_void (graphics context)              │  │ │
│ │ │ - visual: *mut c_void (XVisual*)                       │  │ │
│ │ │ - depth: c_uint (color depth)                          │  │ │
│ │ │ - delete_window: Atom (WM_DELETE_WINDOW)              │  │ │
│ │ │ - open: bool (window state)                            │  │ │
│ │ │ - size: (u32, u32) (device pixels)                     │  │ │
│ │ │ - scale: f32 (DPI scale factor)                        │  │ │
│ │ └────────────────────────────────────────────────────────┘  │ │
│ │                                                              │ │
│ │ Backend::open() [Phase 1 Foundation]                        │ │
│ │   └─ XOpenDisplay() → display handle                        │ │
│ │   └─ XDefaultScreen() → screen index                        │ │
│ │   └─ density_scale() → f32 DPI factor                       │ │
│ │   └─ XCreateSimpleWindow() → window ID                      │ │
│ │   └─ XSelectInput() → register event mask                   │ │
│ │   └─ XSetWMProtocols() → WM_DELETE_WINDOW                   │ │
│ │   └─ XMapWindow() → show window                             │ │
│ │                                                              │ │
│ │ Backend::pump() [Phase 1 Foundation]                        │ │
│ │   └─ XPending() → check buffered events                      │ │
│ │   └─ poll() → wait on X connection FD (timeout)             │ │
│ │   └─ XNextEvent() → collect one event                       │ │
│ │   └─ translate() → convert X11 event to rui Event           │ │
│ │   └─ refresh_geometry() → check window size change          │ │
│ │                                                              │ │
│ │ Backend::surface() [Phase 1 Foundation]                     │ │
│ │   └─ return (width, height, scale)                          │ │
│ │                                                              │ │
│ │ Backend::appearance() [Phase 2 Enhancement]                 │ │
│ │   └─ GTK_THEME env var → light/dark                         │ │
│ │   └─ QT_STYLE_OVERRIDE env var → light/dark                 │ │
│ │   └─ SELFHOST_APPEARANCE env var → light/dark               │ │
│ │   └─ fallback to Light                                       │ │
│ │                                                              │ │
│ │ Backend::present() [Phase 2 Enhancement]                    │ │
│ │   ├─ take Canvas pixel buffer                               │ │
│ │   ├─ XCreateImage() → wrap pixels as XImage                 │ │
│ │   ├─ XPutImage() → blit to window                           │ │
│ │   └─ handle zero-sized canvas (no-op)                       │ │
│ │                                                              │ │
│ │ Backend::is_open() [Phase 1 Foundation]                     │ │
│ │   └─ return self.open (set by WM_DELETE_WINDOW)             │ │
│ │                                                              │ │
│ │ Helpers:                                                    │ │
│ │   density_scale(display, screen) -> f32                     │ │
│ │   modifiers_of(state) -> Modifiers                          │ │
│ │   key_for_symbol(keysym) -> Option<Key>                     │ │
│ │   translate(&event, events_out)                             │ │
│ │   refresh_geometry()                                        │ │
│ └────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            │ reads
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│ RENDERING OUTPUT LAYER                                          │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ src/canvas.rs - Canvas buffer                              │ │
│ │ Vec<u32> (XRGB pixels, little-endian)                      │ │
│ │                                                              │ │
│ │ src/paint.rs - Painter API                                 │ │
│ │ Rasterizes shapes, lines, text to Canvas                   │ │
│ │                                                              │ │
│ │ src/text.rs - Font rendering                               │ │
│ │ TrueType parser, glyph rasterizer                          │ │
│ │                                                              │ │
│ │ src/color.rs - Color & sRGB gamma                          │ │
│ │                                                              │ │
│ │ src/image.rs - PNG export (for parity tests)               │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                            ▲
                            │ written by
                            │
┌─────────────────────────────────────────────────────────────────┐
│ LAYOUT & RENDERING LAYER                                        │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ src/element.rs - Element tree builder                       │ │
│ │ src/layout.rs - Flexbox-like layout engine                 │ │
│ │ src/widgets.rs - Buttons, text, panels, etc.               │ │
│ │ src/style.rs - Length, Tone, Align, etc.                  │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow: Event → Frame → Pixels

### Path 1: X11 Event Collection (Backend::pump)

```
1. X11 server sends events to X11 window
   ↓
2. pump() calls XPending()
   - Returns count of buffered events
   - If 0, calls poll() on X connection FD with timeout
   ↓
3. poll() blocks until:
   - X11 event arrives, OR
   - timeout milliseconds elapse
   ↓
4. XNextEvent() receives one event struct (XEvent union)
   ↓
5. translate() examines event type and translates to rui Event
   - XKeyEvent → Event::Key (with character, key code, modifiers)
   - XButtonEvent → Event::Pointer {state: Press/Release}
   - XMotionEvent → Event::Pointer {state: Move}
   - Expose → Event::Redraw
   - ConfigureNotify → call refresh_geometry()
   - ClientMessage (WM_DELETE_WINDOW) → set self.open = false
   ↓
6. Event added to events_out vector
   ↓
7. Surface::draw() consumes events_out and updates state via handlers
```

### Path 2: Canvas Rendering (Backend::present)

```
1. Surface::draw() calls view(&state) → El<S> (element tree)
   ↓
2. layout() engine measures and positions elements
   ↓
3. paint() engine rasterizes to Canvas
   - Fill rectangles with colors
   - Draw text (TrueType glyphs)
   - Apply shadows, SDF effects
   ↓
4. Canvas holds pixel buffer: Vec<u32> in XRGB format
   - Width × Height pixels
   - Each pixel: u32 with XRGB bytes (little-endian: B G R X)
   ↓
5. Backend::present(canvas) is called
   ↓
6. XCreateImage() wraps canvas.pixels() as XImage struct
   - display: self.display
   - visual: self.visual
   - depth: self.depth
   - width: canvas.width()
   - height: canvas.height()
   - data: canvas.pixels().as_ptr() (no copy)
   ↓
7. XPutImage() blits XImage to window
   - Sends graphics command to X server
   - X server blits pixels to the displayed window
   ↓
8. Window on screen is updated
```

### Path 3: Coordinate Translation (Phase 3 Integration)

```
1. X11 event arrives with coordinates in DEVICE PIXELS
   e.g., XButtonEvent.x = 400 (physical pixels at 2x DPI)
   ↓
2. translate() applies coordinate contract:
   logical_x = device_x / scale
   logical_y = device_y / scale
   ↓
3. Event is added to events_out with logical coordinates
   ↓
4. Surface::draw() uses logical coordinates for hit testing
   - Element tree uses logical units (CSS-like)
   - Handlers receive logical (x, y)
   ↓
5. Handlers update state (e.g., app.selected = new_index)
   ↓
6. Next frame, view() is called with updated state
   ↓
7. Layout engine positions elements in logical units
   ↓
8. Paint engine rasterizes at 1:1 to canvas (logical == canvas units)
   ↓
9. Backend::present() applies scale during display
   - Canvas is width×height in logical pixels
   - X11 window is (width × scale) × (height × scale) in device pixels
   - XPutImage stretches/scales the blit as needed (via X server)
   
   OR (more common):
   - Canvas is pre-rendered at device scale (in open())
   - window size = (width * scale) × (height * scale)
   - Canvas matches window device pixels 1:1
   - XPutImage blits without scaling
```

## Module Boundary Contracts

### Backend Trait (src/shell/mod.rs) ← → x11.rs

**Caller (src/shell/mod.rs) provides to Backend:**
- `WindowOptions { title, width, height, min_width, min_height }`
- `Duration` (timeout for event waiting)
- `&mut Vec<Event>` (output buffer for collected events)
- `&Canvas` (pixel buffer to render)

**Backend provides to Caller:**
- `Result<Self, Error>` (window resource or error)
- `(u32, u32, f32)` (width, height, DPI scale)
- `Appearance` (light or dark mode)
- `bool` (is_open)

**Invariants:**
- Width and height are in device pixels, not logical pixels
- Scale factor is ≥ 1.0 (high-DPI displays have scale > 1.0)
- Events are in logical pixel coordinates (after coordinate translation)
- Canvas buffer is in XRGB little-endian format (matching X11 expectations)

### Canvas (src/canvas.rs) ← → x11.rs

**X11 backend accesses Canvas:**
- `canvas.width() -> u32` (logical pixels)
- `canvas.height() -> u32` (logical pixels)
- `canvas.pixels() -> &[u32]` (XRGB buffer)

**Invariant:**
- Buffer size must be >= width × height × 4 bytes
- Pixels are XRGB little-endian (matching XImage expectations)

### Input Events (src/input.rs) ← → x11.rs

**X11 backend creates Events:**
- `Event::Key { character, key, modifiers }` from XKeyEvent
- `Event::Pointer { position, button, state, modifiers }` from XButtonEvent / XMotionEvent
- `Event::Redraw` from Expose
- `Event::Wheel` (if supported)

**Invariants:**
- Coordinates are in logical pixels (device pixels ÷ scale)
- Modifiers include Shift, Control, Alt, Super
- Button state is Press, Release, or Move
- Key codes match rui's Key enum (Return, Escape, Tab, etc.)

### Memory (src/memory.rs) ← → x11.rs

**X11 backend does NOT directly access Memory.**

The frame loop (Surface::draw) manages Memory:
- Stores hover, focus, scroll state between frames
- Persists across X11 event processing
- Survives window resize (ConfigureNotify)

Memory is indirectly affected by X11 events:
- Click event triggers focus update in Memory
- Mousemove event triggers hover update in Memory

### Appearance (src/theme.rs) ← → x11.rs

**Backend::appearance() returns Appearance (enum):**
- `Appearance::Light`
- `Appearance::Dark`

**X11 backend reads from:**
- Environment variables: `GTK_THEME`, `QT_STYLE_OVERRIDE`, `SELFHOST_APPEARANCE`
- Fallback: Light (conservative default)

**Frame loop uses Appearance to:**
- Select color palette (Tone::Surface, Tone::OnSurface, etc.)
- Re-render UI in light or dark mode

## Cross-Module Coordination Points

### 1. Window Initialization (Phase 1)

- `App::run()` calls `shell::run()`
- `shell::run()` calls `Backend::open()`
- `x11::Backend::open()` creates X11 window, registers events, maps window
- Returns `Ok(Window)` or error

**Invariant:** Window is ready for `pump()` immediately after `open()` succeeds.

### 2. Event Loop (Phase 1)

- Frame loop calls `backend.pump(timeout, &mut events, redraw_fn)`
- X11 `pump()` collects events and adds to events vector
- Frame loop consumes events and calls handlers
- Handlers update state

**Invariant:** Events are in order received; coordinate translation is applied per-event.

### 3. Frame Rendering (Phase 2)

- Frame loop calls `view(&state)` → `El<S>`
- Layout engine measures and positions
- Paint engine rasterizes to Canvas
- Frame loop calls `backend.present(&canvas)`
- X11 `present()` blits Canvas to window

**Invariant:** Canvas pixel format matches X11 expectations (XRGB); no format conversion needed.

### 4. Coordinate Contract (Phase 3)

- X11 event arrives with device coordinates
- `translate()` applies coordinate formula: logical = device / scale
- Frame loop uses logical coordinates for hit testing and element positioning
- Canvas is rendered in logical units
- X11 window is sized in device pixels (width × scale, height × scale)

**Invariant:** Clicks register on the correct UI element regardless of DPI.

### 5. Appearance Switching (Phase 2)

- X11 `appearance()` reads environment variables
- Frame loop calls `backend.appearance()` once per frame
- If appearance changes (Light ↔ Dark), UI is re-rendered with new colors
- Canvas is re-rasterized with new palette

**Invariant:** Appearance switch is seamless (no state loss, no animation glitches).

### 6. Window Lifecycle (Phase 1)

- User clicks close button (or sends WM_DELETE_WINDOW)
- X11 `translate()` receives ClientMessage with WM_DELETE_WINDOW
- `translate()` sets `self.open = false`
- Frame loop calls `backend.is_open()` → false
- Event loop exits, process terminates

**Invariant:** App terminates cleanly; no resource leaks.

## Testing Strategy

### Unit Tests (src/shell/platform/x11.rs)

- Compile-time verification: Backend trait is implemented
- FFI bindings are present and declared correctly

### Integration Tests (tests/x11_integration.rs, if present)

- Cannot run without X11 display
- Would test:
  - Window creation succeeds on X11 systems
  - Events arrive and are translated correctly
  - Coordinate contract holds at various DPI scales

### Parity Tests (tests/x11_parity.rs, if present)

- Build native reference frame (src/examples/parity.rs)
- Build WASM parity renderer
- Compare X11 render output to native reference
- Verify pixel-perfect identical rendering

### Platform-Agnostic Tests

- `cargo test --lib` (unit tests) pass on all platforms
- `cargo test --test recipes` (widget examples) pass on all platforms
- These tests use Harness (in-memory rendering) and don't require X11

## Summary

The X11 backend integrates into rui's frame loop through the **Backend trait abstraction**. The trait defines six methods that encapsulate all platform-specific logic:

1. **open()** - Create window and register for events
2. **pump()** - Collect events with timeout
3. **surface()** - Report window dimensions and DPI scale
4. **appearance()** - Read light/dark mode preference
5. **present()** - Blit canvas pixels to window
6. **is_open()** - Report window open state

All X11-specific code (Xlib FFI bindings, event translation, coordinate contract) is confined to `src/shell/platform/x11.rs`. The frame loop in `src/shell/mod.rs` uses only the Backend trait interface, ensuring that adding a new platform (Wayland, etc.) requires only a new Backend implementation, with no changes to frame loop logic.

The three-phase architecture (Foundation, Enhancement, Integration) ensures that each phase builds on the previous:
- **Phase 1** establishes the Backend trait and basic window/event infrastructure
- **Phase 2** adds full rendering, appearance detection, and event translation
- **Phase 3** verifies coordinate contracts, cross-module coordination, and EventLoop compatibility

This structure makes the X11 backend maintainable, testable, and a template for future platform additions.
