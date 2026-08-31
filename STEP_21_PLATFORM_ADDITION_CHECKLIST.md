# STEP 21: Platform Addition Checklist

## Quick Reference: Adding a New Backend (Mir, custom renderer, game engine, etc.)

This checklist is a step-by-step guide to adding a new platform backend to rui, following the Recipe 2 pattern proven by X11 and Wayland.

---

## Pre-Implementation: Requirements Gathering (1–2 hours)

### ☐ **Platform Investigation**

- [ ] Research the target platform's display server / window system
  - What's the native API? (X11 has Xlib, Wayland has libwayland-client, Mir has MirCore, etc.)
  - How do you open a window?
  - How do you receive input events?
  - How do you get DPI / scale factor?
  - How do you present pixels to the display?

- [ ] Document platform-specific concepts
  - Coordinate system (screen space vs. window space)
  - Event model (event queues, callbacks, etc.)
  - Color theme detection (env var, config file, property, protocol, etc.)
  - Font loading (system directories, embedded, protocol)
  - Clipboard / drag-and-drop (if relevant)

- [ ] Identify platform constraints
  - Can the platform block on events? (X11, Wayland: yes; WASM: no; game engine: maybe)
  - Does the platform have a main loop? (Most do; WASM doesn't)
  - Does the platform require mutable state in callbacks? (Most need Rc<RefCell<>>; rui avoids this)
  - Can DPI change at runtime? (On Wayland, yes; on X11, rarely; know your platform)

### ☐ **Dependency Planning**

- [ ] Identify required crates (rui has zero; try to keep it that way)
  - **Unsafe FFI bindings?** Add them in `src/shell/platform/new_platform.rs` (confined to one file)
  - **Language FFI?** (C, C++, assembly) → Keep in `src/shell/platform/new_platform.rs`
  - **Need an external crate?** Evaluate whether to use it or implement inline

- [ ] Assess code size expectations
  - X11 implementation: ~800 lines
  - Wayland implementation: ~200 lines (protocol is newer, cleaner)
  - Game engine integration: might be smaller (engine provides more infrastructure)

### ☐ **Documentation & Testing Strategy**

- [ ] Plan where documentation will live
  - Backend implementation: comment in the code (explain platform quirks)
  - Coordinate contract: document device→logical transformation
  - Event translation: list which platform events map to which rui Events
  - Gotchas: edge cases, platform-specific bugs, workarounds

- [ ] Plan verification gates (similar to X11/Wayland phases)
  - Phase 1: Basic Backend trait implementation (can you open a window, receive events, present pixels?)
  - Phase 2: Feature completeness (DPI scaling, keyboard input, theme detection)
  - Phase 3: Integration & parity (cross-platform consistency tests)

---

## Phase 1: Foundation (4–8 hours)

### ☐ **Create Platform Module** (`src/shell/platform/new_platform.rs`)

```rust
// Minimal stub
use crate::shell::{Backend, Error, Event};
use std::time::Duration;

pub(crate) struct Window {
    // Platform-specific state (display connection, window handle, etc.)
    // Keep this struct opaque; don't expose platform types
}

impl Backend for Window {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        Err(Error::Unsupported)  // TODO
    }
    
    fn pump(&mut self, _timeout: Duration, _events: &mut Vec<Event>, _redraw: &mut dyn FnMut(&Self)) -> Result<(), Error> {
        Err(Error::Unsupported)  // TODO
    }
    
    fn surface(&self) -> (u32, u32, f32) {
        (0, 0, 1.0)  // TODO
    }
    
    fn appearance(&self) -> Appearance {
        Appearance::Light  // TODO
    }
    
    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        Err(Error::Unsupported)  // TODO
    }
    
    fn is_open(&self) -> bool {
        false  // TODO
    }
}
```

### ☐ **Implement `open()`**

```rust
fn open(options: &WindowOptions) -> Result<Self, Error> {
    // 1. Connect to the display server
    //    X11: XOpenDisplay()
    //    Wayland: wl_display_connect()
    //    Game engine: get global engine instance
    
    let display = new_platform::connect()
        .map_err(|e| Error::Platform(format!("Cannot connect: {}", e)))?;
    
    // 2. Calculate device size from logical size + scale
    //    Assume 96 DPI default (scale = 1.0) until we query the real DPI
    let scale = 1.0;
    let device_width = (options.width * scale).ceil() as u32;
    let device_height = (options.height * scale).ceil() as u32;
    
    // 3. Create the window
    let window = new_platform::create_window(
        display,
        device_width,
        device_height,
        &options.title,
    )?;
    
    // 4. Register for input events
    new_platform::select_input_events(window)?;
    
    Ok(Self {
        display,
        window,
        scale,
        is_open: true,
        // ... other state
    })
}
```

### ☐ **Implement `pump()`**

```rust
fn pump(&mut self, timeout: Duration, events: &mut Vec<Event>, _redraw: &mut dyn FnMut(&Self)) -> Result<(), Error> {
    // 1. Wait for events (up to timeout)
    let raw_event = new_platform::wait_event(self.window, timeout)
        .map_err(|e| Error::Platform(format!("Event wait failed: {}", e)))?;
    
    // 2. Translate platform events to rui Events
    match raw_event {
        new_platform::RawEvent::ButtonPress { x, y, button } => {
            let logical_x = x as f32 / self.scale;
            let logical_y = y as f32 / self.scale;
            events.push(Event::PointerDown {
                position: Point::new(logical_x, logical_y),
                button: translate_button(button),
            });
        }
        new_platform::RawEvent::KeyPress { keysym, modifiers } => {
            events.push(Event::KeyDown {
                key: keysym_to_key(keysym),
                shift: modifiers.contains(Shift),
                control: modifiers.contains(Ctrl),
                alt: modifiers.contains(Alt),
                meta: modifiers.contains(Super),
            });
        }
        new_platform::RawEvent::WindowClose => {
            self.is_open = false;
        }
        _ => {} // Ignore unhandled events
    }
    
    Ok(())
}
```

### ☐ **Implement `surface()`**

```rust
fn surface(&self) -> (u32, u32, f32) {
    // Return device pixel dimensions + scale factor
    // scale factor = DPI / 96.0
    
    // Query DPI from platform (may be hardcoded if platform doesn't expose it)
    let dpi = new_platform::get_dpi(self.window).unwrap_or(96.0);
    let scale = (dpi / 96.0).clamp(1.0, 3.0);
    
    (self.device_width, self.device_height, scale)
}
```

### ☐ **Implement `appearance()`**

```rust
fn appearance(&self) -> Appearance {
    // Check platform's theme setting
    // Fallback order:
    // 1. Environment variable (e.g., COLORFTERM, GTK_THEME)
    // 2. Platform-specific property (e.g., _NET_WM_APPEARANCE)
    // 3. Default to light
    
    if let Ok(theme) = std::env::var("COLORFTERM") {
        if theme.contains("dark") {
            return Appearance::Dark;
        }
    }
    
    if new_platform::is_dark_mode(self.window) {
        Appearance::Dark
    } else {
        Appearance::Light
    }
}
```

### ☐ **Implement `present()`**

```rust
fn present(&self, canvas: &Canvas) -> Result<(), Error> {
    // 1. Get pixel data from canvas
    let pixels = canvas.pixels();
    let width = canvas.width();
    let height = canvas.height();
    
    // 2. Create a platform-specific buffer from pixels
    let buffer = new_platform::create_buffer(
        self.display,
        width,
        height,
        pixels,
    )?;
    
    // 3. Display the buffer
    new_platform::show_buffer(self.window, buffer)?;
    
    // 4. Clean up (may not be needed if buffer is reference-counted)
    new_platform::free_buffer(buffer)?;
    
    Ok(())
}
```

### ☐ **Implement `is_open()`**

```rust
fn is_open(&self) -> bool {
    self.is_open
}
```

### ☐ **Wire into Platform Selector** (src/shell/mod.rs)

```rust
mod platform;

// At the top of shell/mod.rs (around line 55):
pub use platform::Window as Backend;  // Rename to avoid confusion

// Later, when selecting backends based on OS:
#[cfg(target_os = "new_platform_name")]
use crate::shell::platform::Window;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
use crate::shell::platform::Window;  // Fallback if needed
```

### ☐ **Verify Phase 1 Compiles**

```bash
cargo build --target new_platform_target
# Should compile without errors
# May have unimplemented methods (they return errors for now)
```

### ☐ **Test Phase 1**

Create a simple integration test:

```rust
// tests/new_platform_phase1.rs
#[test]
fn backend_trait_is_implemented() {
    // This test just verifies the backend compiles
    // If this passes, all 6 Backend methods exist
    let _ = std::any::type_name::<impl Backend>();
}

#[test]
fn window_can_open() {
    let options = WindowOptions::default();
    let window = Window::open(&options);
    
    #[cfg(target_os = "new_platform")]
    assert!(window.is_ok(), "Window should open on new platform");
    
    #[cfg(not(target_os = "new_platform"))]
    assert!(window.is_err(), "Window should fail on other platforms");
}
```

Run:
```bash
cargo test --test new_platform_phase1
```

### ☐ **Verification Gate: Phase 1 Complete**

- [ ] All 6 Backend trait methods implemented (may error for now)
- [ ] Compiles on target platform
- [ ] Window can open (or errors with descriptive message)
- [ ] No unsafe code outside `src/shell/platform/new_platform.rs`

---

## Phase 2: Enhancement (8–16 hours)

### ☐ **Coordinate Contract Documentation**

Document the device→logical transformation:

```rust
// In src/shell/platform/new_platform.rs header comment:

/// # Coordinate System
///
/// This backend uses the following coordinate transformation:
///
/// - **Device coordinates**: Raw pixels reported by platform (e.g., mouse at (100, 50))
/// - **Logical coordinates**: DPI-adjusted, platform-independent (e.g., (50, 25) at 2x scale)
/// - **Transformation**: logical = device / scale_factor
/// - **Applied to**: All PointerDown, PointerMoved, PointerUp, Scrolled events
///
/// The scale_factor is computed as: DPI / 96.0, clamped to [1.0, 3.0]
///
/// Example:
///   - Device mouse: (200, 100)
///   - Display DPI: 192 (scale = 2.0)
///   - Logical mouse: (200 / 2.0, 100 / 2.0) = (100, 50)
```

### ☐ **Complete Event Translation**

Add handlers for all event types:

```rust
// In pump(), add cases for:

// - ☐ PointerMoved (mouse move)
// - ☐ PointerDown (mouse button press)
// - ☐ PointerUp (mouse button release)
// - ☐ Scrolled (mouse wheel / trackpad)
// - ☐ KeyDown (physical key pressed)
// - ☐ KeyUp (physical key released)
// - ☐ Text (IME input or typed text)
// - ☐ ModifiersChanged (shift, ctrl, alt, meta)
// - ☐ WindowResized (window size changed)
// - ☐ WindowFocusChanged (window gained/lost focus)
// - ☐ WindowCloseRequested (user clicked close button)

// For each event type, document:
// 1. Platform event that triggers it
// 2. Coordinate translation (if applicable)
// 3. Modifier mapping (if applicable)
// 4. Any platform-specific quirks
```

### ☐ **DPI & Scale Factor Handling**

```rust
// Query DPI from platform and compute scale:
let dpi = new_platform::get_dpi(self.window).unwrap_or(96.0);
let scale = (dpi / 96.0).clamp(1.0, 3.0);

// Store scale in Window struct
self.scale = scale;

// Apply to all coordinate events:
let logical_x = device_x as f32 / self.scale;
let logical_y = device_y as f32 / self.scale;
```

### ☐ **Keyboard Input & Key Translation**

Create a key mapping function:

```rust
// In src/shell/platform/new_platform.rs
fn keysym_to_key(keysym: u32) -> Key {
    match keysym {
        XK_a => Key::A,
        XK_b => Key::B,
        // ... all letters
        XK_Return => Key::Enter,
        XK_BackSpace => Key::Backspace,
        XK_Tab => Key::Tab,
        XK_Escape => Key::Escape,
        XK_space => Key::Space,
        // ... all special keys
        _ => Key::Unknown,
    }
}

// Add to pump():
new_platform::RawEvent::KeyPress { keysym, modifiers } => {
    events.push(Event::KeyDown {
        key: keysym_to_key(keysym),
        shift: modifiers.shift,
        control: modifiers.control,
        alt: modifiers.alt,
        meta: modifiers.meta,
    });
}
```

### ☐ **Theme/Appearance Detection**

Implement proper appearance detection:

```rust
fn appearance(&self) -> Appearance {
    // Try environment variables first
    if let Ok(theme) = std::env::var("NEW_PLATFORM_THEME") {
        if theme.contains("dark") {
            return Appearance::Dark;
        }
    }
    
    // Try platform-specific properties
    if let Some(dark) = new_platform::get_theme_property(self.window) {
        return if dark { Appearance::Dark } else { Appearance::Light };
    }
    
    // Fallback to light (safest default)
    Appearance::Light
}
```

### ☐ **Font Loading**

Ensure fonts can be loaded on the platform:

```rust
// In src/shell/fonts.rs (if platform-specific logic needed)
#[cfg(target_os = "new_platform")]
pub fn load_system_fonts() -> Result<Vec<LoadedFonts>, Error> {
    // Check platform-specific font directories
    // Example: ~/.fonts, /usr/share/fonts, C:\Windows\Fonts, etc.
    
    let font_dirs = vec![
        PathBuf::from("~/.fonts"),
        PathBuf::from("/usr/share/fonts"),
    ];
    
    let mut fonts = Vec::new();
    for dir in font_dirs {
        if dir.exists() {
            // Load TrueType fonts from directory
            for entry in std::fs::read_dir(dir)? {
                // ... load .ttf files
            }
        }
    }
    
    Ok(fonts)
}
```

### ☐ **Full Keyboard Support with Modifiers**

Track shift/control/alt/meta across the frame:

```rust
// In Window struct:
modifiers: KeyModifiers,

// In pump():
new_platform::RawEvent::ModifiersChanged { shift, control, alt, meta } => {
    self.modifiers = KeyModifiers { shift, control, alt, meta };
    // Key events will use self.modifiers when reported
}

// When reporting key events:
Event::KeyDown {
    key,
    shift: self.modifiers.shift,
    control: self.modifiers.control,
    alt: self.modifiers.alt,
    meta: self.modifiers.meta,
}
```

### ☐ **Window Resize Handling**

```rust
// In pump(), handle window resize events:
new_platform::RawEvent::WindowResized { width, height } => {
    self.device_width = width;
    self.device_height = height;
    
    // No Event emitted; frame loop queries surface() on next frame
    // This is correct: the layout engine handles size changes
}
```

### ☐ **Write Enhanced Tests**

```rust
// tests/new_platform_phase2.rs

#[test]
fn coordinate_transformation_x11_backend() {
    // At 2x DPI, device (200, 100) should translate to logical (100, 50)
    let scale = 2.0;
    let device_x = 200.0;
    let device_y = 100.0;
    
    let logical_x = device_x / scale;
    let logical_y = device_y / scale;
    
    assert_eq!(logical_x, 100.0);
    assert_eq!(logical_y, 50.0);
}

#[test]
fn modifier_keys_are_reported() {
    // Simulate Ctrl+A keypress
    let event = Event::KeyDown {
        key: Key::A,
        control: true,
        shift: false,
        alt: false,
        meta: false,
    };
    
    // Handler should receive the modifiers
    let mut app = App::default();
    let handler = |app: &mut App, key, control: bool| {
        if key == Key::A && control {
            app.select_all();
        }
    };
    
    // Verify handler logic works
    handler(&mut app, Key::A, true);
    assert!(app.is_all_selected());
}

#[test]
fn appearance_detects_dark_mode() {
    std::env::set_var("NEW_PLATFORM_THEME", "dark");
    
    let window = Window::open(&WindowOptions::default()).unwrap();
    assert_eq!(window.appearance(), Appearance::Dark);
    
    std::env::remove_var("NEW_PLATFORM_THEME");
}
```

### ☐ **Verification Gate: Phase 2 Complete**

- [ ] Coordinate contract documented in code comment
- [ ] Device→logical transformation verified in tests
- [ ] All 11+ event types translated (or documented as not applicable)
- [ ] Keyboard input with full modifier support
- [ ] DPI scaling formula: `scale = dpi / 96.0`
- [ ] Theme/appearance detection working
- [ ] Font loading from platform directories
- [ ] Window resize events handled

---

## Phase 3: Integration & Cross-Platform Consistency (8–12 hours)

### ☐ **Create Parity Tests** (Like X11 and Wayland)

```rust
// tests/new_platform_parity.rs

#[test]
fn render_same_frame_as_other_backends() {
    // Render a test scene on new platform and existing platform
    let mut new_harness = create_harness::<NewPlatform>();
    let mut mac_harness = create_harness::<MacOS>();
    
    // Draw identical frame
    new_harness.render_frame();
    mac_harness.render_frame();
    
    // Get pixels from both
    let new_pixels = new_harness.frame().pixels();
    let mac_pixels = mac_harness.frame().pixels();
    
    // Pixels should be byte-for-byte identical
    assert_eq!(new_pixels, mac_pixels);
}

#[test]
fn click_at_same_logical_coordinate_same_result() {
    // Click at (100, 50) on all platforms
    let platforms = vec![
        ("new_platform", create_harness::<NewPlatform>()),
        ("macos", create_harness::<MacOS>()),
        ("windows", create_harness::<Windows>()),
    ];
    
    for (name, mut harness) in platforms {
        harness.click_at(100.0, 50.0);
        assert_eq!(harness.state().counter, 1, "Failed on {}", name);
    }
}

#[test]
fn dpi_scaling_maintains_click_semantics() {
    for scale in [1.0, 1.5, 2.0, 3.0] {
        let mut harness = create_harness_with_scale::<NewPlatform>(scale);
        harness.click_at(100.0, 50.0);
        assert_eq!(harness.state().counter, 1, "Failed at scale {}", scale);
    }
}

#[test]
fn light_and_dark_modes_render_differently() {
    let mut harness = create_harness::<NewPlatform>();
    
    harness.set_appearance(Appearance::Light);
    let light_pixels = harness.frame().pixels();
    
    harness.set_appearance(Appearance::Dark);
    let dark_pixels = harness.frame().pixels();
    
    // Should be visibly different
    assert_ne!(light_pixels, dark_pixels);
}
```

### ☐ **Document Platform-Specific Gotchas**

Create a troubleshooting section (see X11 analysis for template):

```rust
// In src/shell/platform/new_platform.rs comment:

/// # Known Gotchas
///
/// 1. **DPI Caching**: DPI may change at runtime but is cached. Listen for
///    ConfigureNotify / output changes to detect DPI updates.
///
/// 2. **Event Ordering**: Some platforms report events out of order (e.g., focus change
///    after key event). Always assume events may be reordered.
///
/// 3. **Coordinate Edge Cases**: Some platforms (e.g., Wayland) have ambiguous
///    coordinate systems near window edges. Test thoroughly.
///
/// 4. **Memory Cleanup**: Platform objects (display handles, windows) must be
///    freed in drop() to avoid resource leaks.
```

### ☐ **Module Dependency Verification**

Create a diagram showing how new platform fits in the architecture:

```
User Input (platform-specific)
    ↓
pump() [in new_platform.rs]
    ↓
Event (unified type)
    ↓
Frame Loop [in shell/mod.rs]
    ↓
View / Layout / Paint [platform-agnostic]
    ↓
Canvas (pixel buffer)
    ↓
present() [in new_platform.rs]
    ↓
Display Output (platform-specific)
```

Document that `Event` and `Canvas` are the only crossing points.

### ☐ **EventLoopDriver Abstraction (if needed)**

If the platform has special event loop requirements:

```rust
// In src/shell/mod.rs (if not already present)
pub trait EventLoopDriver {
    fn can_block() -> bool;  // Can pump() block?
    fn requires_callback() -> bool;  // Does platform require callback?
}

#[cfg(target_os = "new_platform")]
impl EventLoopDriver for NewPlatform {
    fn can_block() -> bool {
        true  // or false if like WASM
    }
    
    fn requires_callback() -> bool {
        false  // or true if like WASM
    }
}
```

### ☐ **Add Feature Gate (Optional)**

If the platform is optional:

```toml
# Cargo.toml
[features]
default = ["new_platform"]
new_platform = []

# Or for a feature not built by default:
exotic_platform = []
```

```rust
// src/shell/mod.rs
#[cfg(feature = "new_platform")]
pub use platform::Window;

#[cfg(not(feature = "new_platform"))]
compile_error!("At least one platform must be enabled");
```

### ☐ **Update CLAUDE.md Documentation**

Add new platform to:
1. Module structure table (list `src/shell/platform/new_platform.rs`)
2. Platform support section (update supported platforms list)
3. Troubleshooting: Add platform-specific section

```markdown
### Linux (NEW PLATFORM)

- **Requires NEW PLATFORM environment**: [instructions]
- **Development headers**: [package names]
- **Verify backend builds**: `cargo build --target [target]`
- **Common issues**: [FAQ]
```

### ☐ **Write Platform-Specific Test Exemplar**

```rust
// tests/new_platform_exemplar.rs — Full end-to-end example

use rui::*;

struct App {
    counter: i32,
}

fn view(app: &App) -> El<App> {
    col((
        text(&format!("{}", app.counter)),
        row((
            widgets::button("-", |app: &mut App| app.counter -= 1),
            widgets::button("+", |app: &mut App| app.counter += 1),
        )),
    ))
}

#[test]
#[cfg(target_os = "new_platform")]
fn counter_works_on_new_platform() {
    let mut harness = Harness::new(App { counter: 0 }, view);
    
    harness.click_text("+");
    assert_eq!(harness.state().counter, 1);
    
    harness.click_text("+");
    assert_eq!(harness.state().counter, 2);
    
    harness.click_text("-");
    assert_eq!(harness.state().counter, 1);
}
```

### ☐ **Verification Gate: Phase 3 Complete**

- [ ] Parity tests compare renders across platforms
- [ ] Parity tests verify click semantics
- [ ] DPI scaling tests pass at all supported scales
- [ ] Theme switching tests pass
- [ ] Platform gotchas documented
- [ ] Module dependencies documented
- [ ] CLAUDE.md updated
- [ ] Platform-specific test exemplar passes

---

## Final Sign-Off

### ☐ **Run Full Test Suite**

```bash
cargo test
# All tests should pass, including:
# - Library tests (262+)
# - Platform-specific tests
# - Parity tests
# - Exemplar tests
```

### ☐ **Run Clippy & Format**

```bash
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

### ☐ **Verify Compile on Target Platform**

```bash
cargo build --target [target]
cargo run -p rui --example counter  # Visual inspection
```

### ☐ **Update Memory & Project Docs**

Add to `STEP_22_*_ANALYSIS.md`:
- Platform name & identifier
- Backend commit range
- Phase 1-3 timeline
- Gotchas encountered
- Template improvements for next platform

### ☐ **Commit**

```bash
git add -A
git commit -m "Add NEW_PLATFORM backend following Recipe 2 pattern

Implements Backend trait for [platform name].
- Phase 1: Basic window, events, rendering (commit X)
- Phase 2: DPI scaling, keyboard, theme detection (commit Y)  
- Phase 3: Parity tests, documentation (commits Z+)

All 6 Backend trait methods implemented.
Coordinate contract verified: device / scale → logical units.
Parity tests: renders and click semantics match existing platforms.

Co-Authored-By: Claude Haiku 4.5 <noreply@anthropic.com>"
```

---

## Validation Checklist (Before "Done")

| Item | Status |
|------|--------|
| All 6 Backend trait methods implemented | ☐ |
| Compiles on target platform | ☐ |
| Coordinate contract documented | ☐ |
| Device→logical transformation in all events | ☐ |
| DPI scaling: `scale = dpi / 96.0` | ☐ |
| Theme/appearance detection | ☐ |
| Keyboard input with modifiers | ☐ |
| Window resize handling | ☐ |
| Parity tests pass (render + click) | ☐ |
| Platform gotchas documented | ☐ |
| CLAUDE.md updated | ☐ |
| All existing tests still pass | ☐ |
| Clippy clean, formatting correct | ☐ |

---

## Estimated Effort by Phase

| Phase | Duration | Key Milestone |
|-------|----------|---------------|
| **Phase 1** | 4–8 hours | Window opens, events flow, pixels present |
| **Phase 2** | 8–16 hours | Full feature parity (DPI, keyboard, theme) |
| **Phase 3** | 8–12 hours | Parity tests pass, documentation complete |
| **Total** | **20–36 hours** | Production-ready backend |

---

## Reference Implementations

Study these to understand the pattern:

- **X11** (`src/shell/platform/x11.rs`): Full reference, 786 lines
- **Wayland** (`src/shell/platform/wayland.rs`): Cleaner protocol, 206 lines
- **WASM** (Phase 1 alternative): See Recipe 1 in CLAUDE.md

All three implement the same Backend trait and follow the same three-phase pattern.

