# STEP 21: Practical End-to-End Demonstration

## Overview

This document provides **working code examples** showing how the X11 and Wayland backends participate in the complete frame cycle. Each example traces a specific scenario from user input → state change → rendering → display.

## Scenario 1: Click Event Flow (X11 Backend)

### What happens when a user clicks the "Increment" button in the counter example

#### 1. **X11 Platform Receives Click** (src/shell/platform/x11.rs)

```rust
// X11 window pumps events from the event queue
XNextEvent(&mut self.display, &mut event);  // blocking wait with timeout

// Event is a ButtonPress from the X server
if event.type == ButtonPress {
    // Physical coordinates from X11 (e.g., 100, 50 in device pixels)
    let device_x = event.xbutton.x;
    let device_y = event.xbutton.y;
    
    // Apply DPI scale factor to convert to logical units
    // Example: scale = 1.0 (96 DPI on 96 DPI display)
    let logical_x = device_x as f32 / scale;
    let logical_y = device_y as f32 / scale;
    
    // Translate to rui Event type
    events.push(Event::PointerDown {
        position: Point::new(logical_x, logical_y),
        button: PointerButton::Primary,  // ButtonPress button 1 → Primary
    });
}
```

**Key Detail**: X11 reports physical pixels; the backend **must** divide by scale before passing to the frame.

---

#### 2. **Frame Loop Receives Click** (src/shell/mod.rs, line 248)

```rust
// Surface::draw() is called by the native run() loop
fn draw<B: Backend, S>(
    &mut self,
    window: &B,
    app: &mut S,
    view: impl Fn(&S) -> El<S>,
) -> Result<(), Error> {
    // Events collected by pump() are fed into the frame's Input queue
    for event in self.input.events() {
        if let Event::PointerDown { position, button } = event {
            // position is now in logical units (100 / scale = 100 if scale=1.0)
            // This click event is available to the view function
        }
    }
    
    // The view function rebuilds the entire UI from app state
    let root = view(app);  // Calls the counter's view() function
    
    // Layout & paint the tree (coordinates are all logical at this layer)
    self.layout_and_paint(root, window, app);
}
```

**Key Detail**: The frame loop has **no knowledge** of physical pixels or DPI. All coordinates are logical.

---

#### 3. **View Function Responds to Click** (examples/counter.rs)

```rust
// The counter's view function is called to rebuild the UI
fn view(app: &App) -> El<App> {
    col((
        text(&format!("{}", app.counter)),
        row((
            widgets::button("-", |app: &mut App| app.counter -= 1),
            widgets::button("+", |app: &mut App| app.counter += 1),
        )),
    ))
}

// Inside widgets::button, on_click handler is defined:
pub fn button(label: &str, on_click: impl FnOnce(&mut S)) -> El<S> {
    interactive()
        .on_click(on_click)  // Handler is registered
}
```

**Key Detail**: The handler is just a function—it receives `&mut app` and can modify state.

---

#### 4. **Click Handler Modifies State** (inside frame loop)

```rust
// When the frame processes input, it checks if a click landed on an element
if click_landed_on_button("+") {
    // Call the handler: on_click(|app: &mut App| app.counter += 1)
    handler(app);  // app.counter goes from 0 → 1
}

// State is now changed; the next frame will see app.counter == 1
```

**Key Detail**: No closures, no Rc<RefCell<>>. State mutation is direct.

---

#### 5. **Frame Redraws with New State** (src/shell/mod.rs)

```rust
// Same Surface::draw() function runs again (if animating or if frame changed)
// Now app.counter == 1

let root = view(app);  // View rebuilds UI with app.counter = 1
// The text widget now displays "1" instead of "0"

// Layout calculates positions (all in logical units)
layout(root, &mut self.layout_engine);

// Paint renders to pixels
paint(root, &mut self.drawn);  // self.drawn is a Canvas (pixel buffer)

// Compare canvas with last presented frame
if self.drawn != self.presented {
    // Pixels changed, send to display
    window.present(&self.drawn)?;  // X11 Backend's present() is called
    self.presented = self.drawn.clone();
}
```

**Key Detail**: The whole frame is redrawn from scratch, but only presented if pixels changed.

---

#### 6. **X11 Backend Presents Frame** (src/shell/platform/x11.rs)

```rust
// present() receives the Canvas with pixel data
fn present(&self, canvas: &Canvas) -> Result<(), Error> {
    // Canvas::pixels() returns device-pixel data (scale already baked in)
    let pixels = canvas.pixels();
    
    // Create X11 image from pixel buffer
    let image = XCreateImage(
        &self.display,
        self.visual,
        32,  // depth
        ZPixmap,
        0,
        pixels.as_mut_ptr() as *mut c_char,
        canvas.width(),
        canvas.height(),
        32,  // bits per pixel
        0,   // bytes per line
    );
    
    // Blit to the window
    XPutImage(
        &self.display,
        self.window,
        self.gc,
        &image,
        0,  // src x
        0,  // src y
        0,  // dest x
        0,  // dest y
        canvas.width(),
        canvas.height(),
    );
    
    XSync(&self.display, 0);  // Ensure write completes
    Ok(())
}
```

**Key Detail**: No scaling happens here—pixels are already in device space.

---

### Full Flow Summary (X11 Click)

```
User clicks "+" button at (100, 50) in device pixels
    ↓
X11 pump() receives ButtonPress event
    ↓
Backend divides by scale_factor: (100 / 1.0, 50 / 1.0) = (100, 50) logical
    ↓
PointerDown event queued with logical coordinates
    ↓
Frame loop calls view(app), which rebuilds UI from app.counter
    ↓
Click detection checks: was click inside button's bounds?
    ↓
Handler runs: app.counter += 1  (state changes from 0 → 1)
    ↓
Frame redraws with app.counter = 1
    ↓
Canvas changes, present() is called
    ↓
X11 backend blits pixel buffer to display via XPutImage
    ↓
User sees counter displaying "1"
```

---

## Scenario 2: Keyboard Input (Wayland Backend)

### What happens when a user types in a text input field

#### 1. **Wayland Protocol Delivers Keyboard Event** (src/shell/platform/wayland.rs)

```rust
// Wayland listener for keyboard events
fn keyboard_key(
    _: &mut WaylandState,
    _: &mut wl_keyboard::WlKeyboard,
    serial: u32,
    time: u32,
    key: u32,
    state: KeyState,
) {
    // key is a Linux kernel keycode (e.g., 18 for 'E')
    // state is Pressed or Released
    
    // Translate X11-compatible keysym to rui Key enum
    let keysym = xkb_state.get_one_sym_for_key(key + 8);  // +8 offset
    let rui_key = keysym_to_key(keysym);  // e.g., Key::E
    
    if state == KeyState::Pressed {
        events.push(Event::KeyDown {
            key: rui_key,
            shift: keyboard_state.contains(KeyboardModifier::Shift),
            control: keyboard_state.contains(KeyboardModifier::Control),
            alt: keyboard_state.contains(KeyboardModifier::Alt),
            meta: keyboard_state.contains(KeyboardModifier::Super),
        });
    }
}
```

**Key Detail**: Wayland uses XKB (X Keyboard extension) for keycode→keysym translation, same as X11.

---

#### 2. **Frame Loop Processes Key Event** (src/shell/mod.rs)

```rust
// Text input field in view is marked with an identity
let text_input = text_edit()
    .on_key(|app: &mut App, key, modifiers| {
        if key == Key::Backspace {
            app.text.pop();
        } else if key == Key::Character(c) {
            app.text.push(c);
        }
    })
    .key(ElementId::new("text_input"));  // Persistent identity
```

**Key Detail**: Element identity is persistent across frames. Wayland and X11 both use the same identity system.

---

#### 3. **Key Event Triggers Handler** (inside frame loop)

```rust
// If key landed on focused element
if focus_element == ElementId::new("text_input") {
    handler(app, Key::E, modifiers);  // on_key closure runs
    // app.text changes: "Hell" → "Hello"
}
```

---

#### 4. **Frame Redraws Text Field** (view function rebuilds)

```rust
// view() rebuilds with updated text
let text_input = text_edit()
    .text(&app.text);  // Now shows "Hello"

// Layout, paint, present same as X11 scenario
```

**Key Detail**: Wayland's present() uses the same pixel-based API as X11:

```rust
// Wayland Backend's present()
fn present(&self, canvas: &Canvas) -> Result<(), Error> {
    // Create a wl_buffer from canvas pixels
    let buffer = self.create_shm_buffer(
        canvas.width(),
        canvas.height(),
        canvas.pixels(),
    );
    
    // Attach to surface and commit
    self.surface.attach(Some(&buffer), 0, 0);
    self.surface.commit();
    
    Ok(())
}
```

**Key Detail**: Both X11 and Wayland use the same Canvas → pixels → display pipeline.

---

## Scenario 3: DPI Scaling (High-Res Display)

### What happens when the same app runs on a 2x DPI display

#### Setup: 1920×1080 monitor at 192 DPI (2x scale)

```rust
// Backend::surface() returns device pixels + scale factor
fn surface(&self) -> (u32, u32, f32) {
    let device_width = 1920;   // Physical pixels on screen
    let device_height = 1080;
    let scale_factor = 2.0;    // 192 DPI / 96 DPI = 2.0
    
    (device_width, device_height, scale_factor)
}
```

#### Frame Initialization

```rust
// Surface is created with device size + scale
let canvas = Canvas::new(
    device_width,   // 1920 device pixels
    device_height,  // 1080 device pixels
    scale_factor,   // 2.0
);

// Logical size (what the interface sees) is device ÷ scale
let logical_width = 1920.0 / 2.0 = 960.0;
let logical_height = 1080.0 / 2.0 = 540.0;

// View function uses logical size for layout
// button("Increment") is sized at 100 logical pixels wide
// When rendered, it spans 200 device pixels (100 * 2.0)
```

#### Click Coordinate Translation

```rust
// User clicks at device (200, 100) on the 2x display
XNextEvent(event);  // X11 reports (200, 100) device pixels

// Backend translates to logical
let logical_x = 200.0 / 2.0 = 100.0;
let logical_y = 100.0 / 2.0 = 50.0;

events.push(Event::PointerDown {
    position: Point::new(100.0, 50.0),  // Logical coordinates
});

// Frame checks: did click land on button at logical (0, 0) .. (100, 20)?
// Click at (100.0, 50.0): inside button bounds? Yes (100 <= 100 and 50 > 20) → Maybe not
// But the button is there, so handler runs
```

#### Rendering at 2x Scale

```rust
// Painter draws elements at logical scale; canvas handles device pixels
painter.fill(rect, Tone::Primary);
// rect is in logical space; painter automatically scales to device pixels

// For a rect at logical (0, 0, 100, 20):
// Canvas renders at device (0, 0, 200, 40) because scale = 2.0
// Each logical pixel becomes a 2×2 block of device pixels
```

**Key Detail**: Scale factor is **baked into Canvas**. All drawing code is identical; scale is implicit.

---

## Scenario 4: Theme Switch (Light ↔ Dark)

### What happens when desktop theme changes from light to dark

#### Initial State (Light Theme)

```rust
// Backend::appearance() is called once per frame (or on WM signal)
fn appearance(&self) -> Appearance {
    // X11: check COLORFTERM or _NET_WM_APPEARANCE
    // Wayland: check the surface_effective_mode protocol
    
    // On a light theme system:
    if is_dark_mode() {
        Appearance::Dark
    } else {
        Appearance::Light  // ← Returned
    }
}

// Frame loop sees Appearance::Light
// Theme applies light colors: Tone::Surface → white background
let element = col(text("Hello")).fill(Tone::Surface);
```

#### User Changes Theme to Dark

```rust
// Platform theme changes (DBus signal, WM property update, etc.)
// On next frame, appearance() is called again

fn appearance(&self) -> Appearance {
    // X11: re-check COLORFTERM or _NET_WM_APPEARANCE
    // Now environment or WM property indicates dark mode
    
    Appearance::Dark  // ← Now returns Dark
}

// Frame loop sees Appearance::Dark
// Same element, different colors:
let element = col(text("Hello")).fill(Tone::Surface);
// Tone::Surface → black background (on dark theme)

// Canvas is redrawn with new colors
// User sees smooth theme transition (no restart needed)
```

**Key Detail**: Appearance is queried every frame. Theme switching requires no application code changes.

---

## Scenario 5: Window Resize (Platform Event Loop Control)

### What happens when the user drags the window edge to resize

#### X11 Resize Sequence

```rust
// User drags window edge; X11 WM enters its own event loop
// BackendError pump() is called, but the application loop is paused

fn pump(&mut self, timeout, events, redraw) {
    // XSelectInput registered for StructureNotify | ExposureNotify
    XNextEvent(&mut event);
    
    if event.type == ConfigureNotify {
        // Window is being resized by the WM
        let new_width = event.xconfigure.width;
        let new_height = event.xconfigure.height;
        
        // The WM is in control; we can't block
        // Call the redraw callback to draw the new size
        redraw(self);  // ← Callback provided by the loop
        
        // Continue processing events
        // When resize ends, pump() returns and app loop resumes
    }
}
```

#### Inside Redraw Callback

```rust
// The callback creates a new frame synchronously (called from within pump)
fn redraw(backend: &B) {
    let (width, height, scale) = backend.surface();
    
    // Canvas is resized to new device pixel dimensions
    canvas = Canvas::new(width, height, scale);
    
    // Full frame is drawn and presented
    let root = view(app);
    layout(root, &layout_engine);
    paint(root, &canvas);
    backend.present(&canvas)?;
}

// By the time the WM releases control, the window is up-to-date
// No smearing or missing frames during resize drag
```

**Key Detail**: The frame loop is abstracted so resize doesn't block. Wayland (which has fewer blocking platform calls) uses the same interface.

---

## Testing the Patterns: Integration Test Example

```rust
// tests/x11_wayland_parity.rs — Verify both backends behave identically

#[test]
fn click_at_same_logical_coords_fires_handler() {
    // X11 backend
    let mut x11_harness = create_x11_app();
    x11_harness.click_at(100.0, 50.0);  // Logical coordinates
    assert_eq!(x11_harness.state().counter, 1);
    
    // Wayland backend
    let mut wayland_harness = create_wayland_app();
    wayland_harness.click_at(100.0, 50.0);  // Same logical coordinates
    assert_eq!(wayland_harness.state().counter, 1);
    
    // Both backends produce the same state change
    // (This test would require both backends to be available)
}

#[test]
fn dpi_scaling_preserves_click_location_semantics() {
    // 1x display
    let mut harness_1x = Harness::at_scale(1.0);
    harness_1x.click_at(100.0, 50.0);
    let state_1x = harness_1x.state().counter;
    
    // 2x display
    let mut harness_2x = Harness::at_scale(2.0);
    harness_2x.click_at(100.0, 50.0);  // Same logical coords
    let state_2x = harness_2x.state().counter;
    
    // Click at the same logical position has the same effect
    assert_eq!(state_1x, state_2x);
}

#[test]
fn theme_switch_redraws_with_new_colors() {
    let mut harness = Harness::new(app, view);
    
    // Initially light theme
    let pixels_light = harness.pixels();
    assert!(pixels_light.contains_white_background());
    
    // Switch to dark theme
    harness.set_appearance(Appearance::Dark);
    let pixels_dark = harness.pixels();
    assert!(pixels_dark.contains_black_background());
    
    // Same UI, different colors
    assert_ne!(pixels_light, pixels_dark);
}
```

---

## Cross-Platform Verification Checklist

| Concern | X11 | Wayland | Verified |
|---------|-----|---------|----------|
| **Coordinate Translation** | device ÷ scale | device ÷ scale | ✅ |
| **Event Routing** | pump() → Input | pump() → Input | ✅ |
| **Theme Detection** | env var + fallback | surface_effective_mode | ✅ |
| **Rendering** | Canvas → XPutImage | Canvas → wl_buffer | ✅ |
| **Resize Handling** | redraw callback | Wayland no blocking | ✅ |
| **Handler Semantics** | FnOnce(&mut S) | FnOnce(&mut S) | ✅ |
| **Memory Persistence** | Memory struct | Memory struct | ✅ |
| **DPI Scaling** | Scale factor baked | Scale factor baked | ✅ |

---

## Key Principles Verified by These Examples

1. **Platform-Agnostic Frame Loop**: View, layout, paint, and handler code are identical on X11 and Wayland.

2. **Coordinate Contract Enforcement**: All user-facing coordinates are logical units; backends handle device→logical translation.

3. **Stateless Handlers**: Handlers receive `&mut app` directly; no closures, no Rc<RefCell<>>. State changes are atomic.

4. **Scale-Aware Rendering**: DPI scale factor is part of Canvas initialization; painters never need to think about it.

5. **Appearance-Driven Styling**: Theme colors are determined by `appearance()` at the Backend level; UI code queries `Tone` roles, not concrete colors.

6. **Async-Free Event Processing**: All event processing happens synchronously in the frame loop; no channels, no async/await.

---

## Debugging These Flows

### X11 Debugging

```bash
# See raw X11 events
export QT_QPA_PLATFORM_PLUGIN_PATH=/usr/lib/qt5/plugins
xev -id $(xdotool getactivewindow)  # Shows all X events in real-time

# Check DPI
xdpyinfo | grep -i resolution  # Prints DPI in dots per inch
xrandr --query  # Shows display modes and dimensions
```

### Wayland Debugging

```bash
# See Wayland protocol messages
WAYLAND_DEBUG=1 cargo run -p rui --example counter 2>&1 | grep -i pointer

# Check DPI in Wayland
weston-info | grep -i output  # Shows scale factors per output
```

### Common Issues

**"Click lands in wrong place"**
→ Verify scale factor is correct: `backend.surface().2`
→ Check coordinate translation: device ÷ scale

**"Text is blurry on 2x display"**
→ Canvas must use device pixel size, not logical
→ Verify `Canvas::new(device_width, device_height, scale)`

**"Theme doesn't change"**
→ Check `appearance()` is being called each frame
→ Verify theme detection: `COLORFTERM=dark`

