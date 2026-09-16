# Recipe 2: X11 Backend — Coordinate Contract

Reference document for Recipe 2 (X11 Backend Implementation). This file documents the coordinate transformation contract between X11 device pixels and rui logical units.

## The Contract

**Every coordinate transformation is applied exactly once:**

```
Device Pixels (X11)
    ↓ ÷ scale_factor (in pump())
Logical Units (rui)
    ↑ × scale_factor (in canvas.rs::present())
Device Pixels (X11 display)
```

**Contract statement:**
```
logical_x = device_x / scale_factor
logical_y = device_y / scale_factor
device_x = logical_x × scale_factor
device_y = logical_y × scale_factor
```

## Why This Matters

- **Layout** uses only logical units; never sees device pixels
- **Rendering** builds in logical units; translated at presentation
- **Events** arrive in device pixels; immediately translated to logical
- **DPI changes** only affect scale_factor; layout doesn't need recalculation

The contract guarantees:
1. No scale factor applied twice (common bug)
2. No mixed units in a single function
3. Deterministic behavior across DPI values

## Scale Factor Detection (Phase 2)

**Query sequence:**

```rust
// Step 1: Use Xrandr to query physical monitor dimensions
let resources = XrandrGetScreenResourcesCurrent(dpy, screen);
let output = resources.outputs[0];  // Primary output
let crtc = get_crtc_for_output(output);

// Step 2: Compare physical dimensions (mm) with pixel dimensions
let physical_width_mm = output.physical_width_mm;
let pixel_width = crtc.width;

// Step 3: Compute DPI
let dpi = (pixel_width as f32 * 25.4) / physical_width_mm as f32;
// 96 DPI = 1.0 scale, 192 DPI = 2.0 scale
let scale_factor = dpi / 96.0;

// Step 4: Clamp to valid range
let scale_factor = scale_factor.max(1.0).min(4.0);
```

**Fallback sequence (if Xrandr unavailable):**

```rust
// Try environment variable first
if let Ok(val) = std::env::var("GDK_SCALE") {
    if let Ok(scale) = val.parse::<f32>() {
        return scale.max(1.0).min(4.0);
    }
}

// Fall back to assuming 1.0 (96 DPI standard)
1.0
```

**Valid range:** scale_factor ∈ [1.0, 4.0]
- 1.0 = 96 DPI (standard)
- 1.5 = 144 DPI (laptop screen)
- 2.0 = 192 DPI (high-DPI display)
- 4.0 = 384 DPI (theoretical max for accessibility)

## Pointer Event Translation (Phase 2)

**Input: X11 MotionNotify event (device pixels)**
```c
XMotionEvent {
    int x;      // device pixel coordinate
    int y;      // device pixel coordinate
    int x_root; // screen-relative device pixel
    int y_root;
}
```

**Output: rui Pointer event (logical units)**
```rust
Event::Pointer {
    x: device_x / scale_factor,    // convert to logical
    y: device_y / scale_factor,
    // ... other fields
}
```

**Implementation in x11.rs::pump():**
```rust
match event {
    XEvent::MotionNotify(motion) => {
        events.push(Event::Pointer {
            x: motion.x as f32 / self.scale_factor,
            y: motion.y as f32 / self.scale_factor,
            ..Default::default()
        });
    }
}
```

**Verification:**
- Compile: `cargo build --target x86_64-unknown-linux-gnu`
- Test: `cargo test --test x11_integration -- pointer_events`
- Visual: `cargo run -p rui --example controls` at 1.0x and 2.0x scale

## Window Dimensions and Resizing (Phase 1 & 2)

**X11 ConfigureNotify event (device pixels):**
```c
XConfigureEvent {
    int width;   // device pixels
    int height;  // device pixels
}
```

**rui Backend::surface() returns (logical width, logical height, scale):**
```rust
fn surface(&self) -> (u32, u32, f32) {
    let width_logical = self.window_width as u32 / self.scale_factor as u32;
    let height_logical = self.window_height as u32 / self.scale_factor as u32;
    (width_logical, height_logical, self.scale_factor)
}
```

**When window is resized:**
1. X11 sends ConfigureNotify with new device pixel dimensions
2. x11.rs stores `window_width`, `window_height` in device pixels
3. On next pump(), Backend::surface() is called
4. layout.rs receives (logical_width, logical_height) and re-flows
5. paint.rs renders at logical size
6. canvas.rs presents at device size using scale_factor

**Verification:**
- Resize window: layout should reflow smoothly
- Check for double-scaling: rendered text should read clearly at 2.0x scale
- Verify no off-by-one errors in dimension rounding

## Rendering Pipeline (Phase 3)

**Frame rendering flow:**

```
layout.rs computes at logical units
    ↓ (logical Rect, Size, etc.)
paint.rs draws into Canvas at logical units
    ↓ (Rect { x: 100, y: 100 }, logical units)
Canvas::blit_bgra()
    ↓ applies scale_factor
canvas.rs device pixel offset: { x: 200, y: 200 } (if scale=2.0)
    ↓
XPutImage() to X11 drawable
```

**Key: Canvas knows the scale factor and applies it once during present()**

Example from canvas.rs::present():
```rust
pub fn present(&mut self, scale_factor: f32, drawable: Window) {
    // Translate device pixel buffer with blit_bgra
    // which respects scale_factor in coordinate calculations
    for y in 0..height {
        for x in 0..width {
            let device_pixel = self.buffer[y * stride + x];
            // Device pixel coordinates already account for scale
            put_pixel_to_x11(x, y, device_pixel);
        }
    }
}
```

## Common Pitfalls

### Pitfall 1: Applying scale_factor Twice
**Wrong:**
```rust
// In pump(): x_logical = device_x / scale_factor
let x_logical = motion.x as f32 / self.scale_factor;

// Then later in render: apply scale_factor again
let x_device = x_logical * self.scale_factor;  // ❌ back to original by accident
```

**Right:**
```rust
// In pump(): convert once
let x_logical = motion.x as f32 / self.scale_factor;

// Never touch it again until presentation
// canvas.rs handles presentation scaling
```

### Pitfall 2: Mixing Units in One Calculation
**Wrong:**
```rust
// Using device dimensions with logical coordinates
let rect = Rect {
    x: logical_x,
    y: logical_y,
    w: device_width / 2,  // ❌ mixing units
    h: device_height / 2,
};
```

**Right:**
```rust
// Use logical units consistently
let rect = Rect {
    x: logical_x,
    y: logical_y,
    w: device_width as f32 / self.scale_factor / 2.0,
    h: device_height as f32 / self.scale_factor / 2.0,
};
```

### Pitfall 3: Rounding Errors Accumulate
**Problem:** Converting 1.0x to 2.0x and back loses precision
```rust
let logical = device / 2.0;  // 100 / 2 = 50
let device_again = logical * 2.0;  // 50 * 2 = 100 ✓ OK so far

// But with 3 conversions:
let logical2 = device_again / 2.0;  // 100 / 2 = 50
let device_again2 = logical2 * 2.0;  // 50 * 2 = 100 ✓ still OK

// With fractional scale (1.5):
let logical = 100 / 1.5;  // = 66.667
let device_again = 66.667 * 1.5;  // = 100.0005 (rounding error!)
```

**Solution:** Convert once at boundaries (input/output), never in the middle of calculations.

### Pitfall 4: Forgetting Logical Coordinates in Tests
**Wrong:**
```rust
#[test]
fn test_layout() {
    let mut h = Harness::new(state, view);
    h.size(800, 600);  // ❌ Is this device or logical?
    h.click_at(100, 100);  // ❌ Are these device or logical coordinates?
}
```

**Right:**
```rust
#[test]
fn test_layout() {
    // Harness always uses logical units
    let mut h = Harness::new(state, view);
    h.size(800, 600);  // logical units, scale_factor=1.0
    h.click_at(100, 100);  // logical units
}
```

## Verification Across Platforms

**All platforms must render identically:**

```bash
# macOS (built-in 1.0x or 2.0x scale)
cargo run -p rui --example gallery

# Windows (1.0x or 2.0x via display settings)
cargo run -p rui --example gallery

# X11 (manually set scale via environment or Xrandr)
GDK_SCALE=1 cargo run -p rui --example gallery
GDK_SCALE=2 cargo run -p rui --example gallery
```

**Expected:** Text, colors, button sizes, spacing all identical despite DPI.

**Debugging:** If text is blurry or clipped:
- Check scale_factor in window title: `XFetchName()` debug output
- Verify pointer events are in logical units
- Ensure canvas.rs multiplies by scale_factor before presentation

## Reference

**Recipe 2 commits that implement coordinate contract:**

- Phase 1 (748 lines): a67d578eea41560c26fd7a6548c0d089223f3d70 — Foundation (basic window)
- Phase 2 (1220 lines): c42c0f05b3d75976665377a16257c36c472debc1 — DPI detection + coordinate translation
- Phase 3 (1321 lines): 80e3003563c26952e4d63c52d8eb8f5052cb463c — Integration + parity validation

Read these commits to see coordinate handling in practice:
```bash
git show c42c0f05b3d75976665377a16257c36c472debc1 -- src/shell/platform/x11.rs | grep -A5 -B5 "scale_factor"
```
