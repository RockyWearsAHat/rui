# STEP 4: Recipe 1 Cross-Module Concerns

**Purpose**: Identify where Recipe 1 (WASM backend) touches multiple modules and document the friction points, contracts, and data flow.

---

## Module Interaction Map

```
┌─────────────────────────────────────────────────────────┐
│              Platform-Agnostic Code                    │
│  (layout, paint, handlers, accessibility tree)         │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┴───────────┐
         │   Backend Trait       │
         │   (12 methods)        │
         └───────────┬───────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
   ┌────▼────────────────┐   ┌───▼──────────────┐
   │   src/shell/mod.rs  │   │  Platform Code   │
   │   (frame loop)      │   │  (wasm.rs)       │
   └────────────────────┘   └──────────────────┘
```

### Data Flow: Frame to Frame

1. **Frame Begin** (src/shell/mod.rs line 305)
   - Backend::pump() returns Vec<Event>
   - Time elapsed injected into Memory::begin_frame()

2. **View Function** (user code + src/widgets.rs)
   - Reads Memory (focus, scroll state)
   - Rebuilds element tree
   - No platform-specific code here

3. **Layout & Paint** (src/layout.rs, src/paint.rs)
   - Measures and places elements
   - Draws to canvas (CPU rasterizer)
   - All logical coordinates

4. **Present** (Backend::present)
   - WASM: blit_bgra to canvas context
   - Scale factor applied: logical → device pixels

5. **Event Translation** (src/input.rs)
   - DOM events converted to rui Events (wasm-specific)
   - Events → Input (platform-agnostic)
   - Handlers called with mutable state

6. **Memory Update** (src/memory.rs)
   - Focus, scroll, animation state persisted
   - Ready for next frame

---

## Critical Friction Points

### 1. Time Injection (src/shell/mod.rs ↔ src/memory.rs)

**Requirement**: Platform code must inject elapsed time; view code must never read wall clock.

**Contract**:
- Backend::pump() receives `timeout: Duration` parameter
- Platform doesn't call `Instant::now()` directly; instead receives elapsed time from event loop
- `Memory::begin_frame(elapsed)` receives injected Duration
- View code (widgets.rs, animations) reads time from Memory, never from Instant

**WASM Implementation Challenge**:
- requestAnimationFrame provides DOMHighResTimeStamp (milliseconds since page load)
- Must convert to Duration and inject into Memory::begin_frame
- Cannot use Instant::now() (would read browser's clock)

**Verification**:
```bash
grep "Instant::now" src/shell/platform/wasm.rs
# Expected: 0 matches (time injection verified)
```

### 2. Backend Trait Completeness (src/shell/mod.rs line 183)

**Requirement**: All backends implement identical 12-method interface.

**Contract**:
```rust
pub trait Backend: Sized {
    fn open(options: &WindowOptions) -> Result<Self, Error>;
    fn pump(&mut self, timeout: Duration, events: &mut Vec<Event>, ...) -> Result<(), Error>;
    fn surface(&self) -> (u32, u32, f32);  // width, height, scale_factor
    fn appearance(&self) -> Appearance;
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

**WASM Implementation Challenge**:
- WASM runs in browser; no native window system
- `open()` must create canvas element (or use provided)
- `pump()` never blocks (browser event loop is asynchronous)
- `is_fullscreen()` maps to document.fullscreenElement
- `clipboard_text()` uses async Clipboard API (requires wrapping in sync Result)
- IME composition doesn't exist in browser (optional)

**Verification**:
```bash
grep -c "fn " src/shell/platform/wasm.rs | \
  grep "impl Backend for"
# Expected: 12 methods
```

### 3. Coordinate Transformation (src/canvas.rs ↔ platform wasm.rs)

**Requirement**: Logical units ↔ device pixels transformation only at platform boundary.

**Contract**:
```
Logical coordinates: used in all layout, paint, and event handling
Device coordinates: only in Backend::present() when blit_bgra

Transformation:
  logical_x = device_x / scale_factor
  device_x = logical_x * scale_factor
```

**WASM Implementation Challenge**:
- Canvas CSS pixels ≠ canvas bitmap pixels on high-DPI displays
- Canvas context scaling: ctx.scale(dpr, dpr)
- Event coordinates (e.clientX, e.clientY) in CSS pixels
- Must normalize to logical units with scale factor

**Verification**:
- Test: pointer event at device (100, 100) on 2.0 DPI canvas
  - Logical: (50, 50) → should hit element at (50, 50)
  - Verify: handlers receive correct logical coordinates

### 4. Event Translation (wasm events → rui Events)

**Requirement**: Platform-specific events converted to platform-agnostic rui Event type.

**Event Mapping** (WASM → rui):
| Browser Event | rui Event Type | Details |
|---|---|---|
| mousemove | Pointer(moved) | clientX/clientY → logical coords, scale_factor applied |
| mousedown | Pointer(pressed) | button: 0=left, 1=middle, 2=right |
| mouseup | Pointer(released) | matches preceding down |
| mouseenter | — | ignored (use pointer_moved + tracking) |
| mouseleave | — | ignored |
| wheel | Pointer(scroll) | deltaY → units per frame |
| keydown | Key(...) | KeyboardEvent.key → rui Key enum |
| keyup | Key(released) | KeyboardEvent.key → rui Key enum |
| focus | — | triggers focus ring (from memory) |
| blur | — | clears focus ring |

**WASM Implementation Challenge**:
- Browser keycodes differ from X11/Windows
- Modifier keys: event.shiftKey, event.ctrlKey, event.altKey, event.metaKey
- Numpad vs regular number keys distinguished by location
- IME composition (optional in browser)

**Verification**:
```bash
cargo test --test wasm_integration -- event_translation --nocapture
```
Expected: All event types translated correctly; modifiers extracted.

### 5. State Persistence (src/memory.rs ↔ frame loop)

**Requirement**: Memory holds focus, scroll, easing state across frames; persists between DOM events.

**Contract**:
- `Memory` is created once and lives for app lifetime
- Between frames: `Memory::begin_frame()` called with elapsed time
- After view render: `Memory` state updated (focus, scroll position, easing phase)
- Next frame: previous Memory state available to view function

**WASM Implementation Challenge**:
- No native event loop; events arrive asynchronously from browser
- Memory must survive across event callbacks
- No blocking pump(); instead: event listener → handler → frame
- requestAnimationFrame coordinates frame timing

**Verification**:
- Test: scroll position persists across 10 frames
  - Frame 1: on_scroll(dy=100) → Memory.scroll_y = 100
  - Frame 2–10: should still see scroll_y = 100
  - Memory not cleared between events

### 6. Platform Branching (src/shell/mod.rs, src/app.rs)

**Requirement**: WASM-specific code gated with `#[cfg(target_arch = "wasm32")]`.

**Contract**:
- `run()` function has two implementations:
  - Native (macOS/Windows/X11): blocking event loop
  - WASM: async event loop with requestAnimationFrame
- Shared `draw()` function used by both
- Feature gates ensure clean separation

**Files to modify**:
- src/shell/mod.rs: Add `#[cfg(target_arch = "wasm32")]` and `#[cfg(not(target_arch = "wasm32"))]` blocks
- src/app.rs: Backend trait object could use monomorphization or dynamic dispatch
- Cargo.toml: Optional wasm feature with wasm-bindgen dependency

**Verification**:
```bash
cargo build --target wasm32-unknown-unknown
cargo build --target x86_64-unknown-linux-gnu
# Both should compile (feature gates correct)
```

### 7. Focus Management (src/accessibility.rs ↔ src/element.rs)

**Requirement**: `El::takes_focus` is the single source of truth for focusability.

**Contract**:
```rust
// Single point of truth:
impl El {
    fn takes_focus(&self) -> bool {
        self.focusable && !self.disabled
    }
}

// All focus logic queries this:
- Focus walk (tab navigation)
- Focus ring rendering
- Accessibility audit
```

**WASM Implementation Challenge**:
- Browser has native focus management (tabindex, focus() method)
- Must sync rui focus state with browser document.activeElement
- Tab key must follow rui's focus order, not browser's

**Verification**:
- Test: tab order matches declaration order in element tree
  - Elements in order: [button A, button B, field C]
  - Tab navigation: A → B → C → (wrap) → A
  - Verify: browser's focus synced with rui's Memory::focus

---

## Dependency Graph

```
src/shell/platform/wasm.rs (Backend trait impl)
  ├─→ src/shell/mod.rs (platform selector, draw() fn)
  │    ├─→ src/app.rs (run() function)
  │    ├─→ src/input.rs (Event → Input translation)
  │    ├─→ src/memory.rs (focus, scroll state)
  │    └─→ src/paint.rs (canvas rendering)
  │
  ├─→ src/memory.rs (time injection, state persistence)
  │    └─→ user view function
  │         ├─→ src/widgets.rs (button, field, etc.)
  │         └─→ src/element.rs (El::takes_focus)
  │
  ├─→ src/canvas.rs (blit_bgra, present)
  │    └─→ coordinate transformation (logical → device)
  │
  ├─→ src/accessibility.rs (tree updates)
  │    └─→ src/element.rs (El::takes_focus check)
  │
  └─→ src/input.rs (Event → Input)
       └─→ modifier key extraction, pointer position
```

---

## Design Principles

### Principle 1: Platform Transparency
**Goal**: View code is identical on all platforms.
**Implementation**: Backend trait hides platform differences; generic draw() works for all.
**Violation**: View code that reads `cfg!(target_arch = "wasm32")`.

### Principle 2: Time Determinism
**Goal**: Tests can step time exactly.
**Implementation**: No wall-clock reads; time injected via Memory::begin_frame().
**Violation**: Calling Instant::now() in view or platform code.

### Principle 3: Identity Stability
**Goal**: Reordered list items preserve state.
**Implementation**: Element identity = path through tree (or `El::key()` override).
**Violation**: Platform-specific state lookup by ID or name.

### Principle 4: Single Event Dispatch
**Goal**: One handler for click (whether mouse or keyboard activation).
**Implementation**: Accessibility activation and mouse click both call same handler.
**Violation**: Platform-specific event handling with different handlers per event type.

---

## Testing Strategy per Concern

| Concern | Module Pair | Test Name | Verification |
|---------|---|---|---|
| Time injection | shell/mod.rs ↔ memory.rs | test_time_not_read_from_clock | No Instant::now() grep |
| Backend trait | wasm.rs ↔ shell/mod.rs | test_backend_trait_completeness | 12 methods implemented |
| Coord transform | wasm.rs ↔ canvas.rs | test_device_to_logical | Pointer hit-test at scaled DPI |
| Event translation | wasm.rs ↔ input.rs | test_event_translation_complete | All event types handled |
| State persist | memory.rs ↔ frame loop | test_memory_persists_across_frames | Scroll position survives 10+ frames |
| Platform branch | shell/mod.rs ↔ app.rs | test_wasm_and_native_compile | Both targets build |
| Focus management | accessibility.rs ↔ element.rs | test_takes_focus_single_source | Audit and walk agree |

---

## Sign-off Checklist

- [ ] All 12 Backend trait methods implemented and tested
- [ ] Time injection verified (no Instant::now in wasm.rs)
- [ ] Coordinate transformation tested (logical ↔ device correct)
- [ ] Event translation complete and tested
- [ ] State persistence verified (10+ frames)
- [ ] Platform branching correct (cfg gates in place)
- [ ] Focus management synchronized (rui ↔ browser)
- [ ] All friction points documented and tested
- [ ] Cross-module integration tests pass

---

## References

- **Backend trait**: src/shell/mod.rs line 183
- **Frame loop**: src/shell/mod.rs line 305
- **Memory state**: src/memory.rs
- **Event types**: src/input.rs
- **Canvas present**: src/canvas.rs
- **Accessibility**: src/accessibility.rs
- **Element focus**: src/element.rs
