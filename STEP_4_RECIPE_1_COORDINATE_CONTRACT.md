# Recipe 1: WASM Backend — Coordinate Transformation Contract

**Document**: STEP_4_RECIPE_1_COORDINATE_CONTRACT.md  
**Purpose**: Define the exact coordinate transformation rules for the WASM backend  
**Scope**: All coordinate systems from browser input to rui logical layout  
**Audience**: WASM backend implementer, platform verification tests

## Overview

The WASM backend runs in a browser environment where pixel coordinates come from:
- **Mouse events**: `MouseEvent.clientX/Y` (viewport pixels)
- **Touch events**: `Touch.clientX/Y` (viewport pixels)
- **Canvas size**: `canvas.width/height` (device pixels, including `devicePixelRatio` scaling)
- **Display scale**: `window.devicePixelRatio` (1.0 on regular displays, 2.0 on Retina, etc.)

The rui library operates in **logical units**, where all layout, hit-testing, and rendering assume a coordinate system independent of display scale.

## Coordinate Systems

### 1. Browser Client Coordinates
**Source**: DOM events (Mouse, Touch, Keyboard)  
**Range**: 0 to `window.innerWidth` (X), 0 to `window.innerHeight` (Y)  
**Unit**: CSS pixels (device-independent, always 1:1 with viewport)  
**Scaling**: NOT affected by `devicePixelRatio`

```javascript
// Example: Regular 1080p display
window.innerWidth = 1920
window.innerHeight = 1080
devicePixelRatio = 1.0

// Retina display (2x)
window.innerWidth = 1920
window.innerHeight = 1080
devicePixelRatio = 2.0
// Canvas still sized to 1920×1080 CSS pixels, but rendered at 3840×2160 device pixels
```

### 2. Canvas Device Pixels
**Source**: `canvas.width * devicePixelRatio`, `canvas.height * devicePixelRatio`  
**Range**: 0 to `canvas.width * scale_factor`, 0 to `canvas.height * scale_factor`  
**Unit**: Physical device pixels  
**Purpose**: Rasterization target (where CPU rasterizer writes BGRA words)

```javascript
// Regular display (scale_factor = 1.0)
canvas.width = 1920
canvas.height = 1080
device_pixels_width = 1920 * 1.0 = 1920

// Retina display (scale_factor = 2.0)
canvas.width = 1920
canvas.height = 1080
device_pixels_width = 1920 * 2.0 = 3840
```

### 3. Logical Coordinates (rui internal)
**Source**: All rui layout, hit-testing, element positioning  
**Range**: 0 to `canvas.width`, 0 to `canvas.height`  
**Unit**: Logical units (scale-independent)  
**Purpose**: Element trees, handlers, text measurement, layout

```rust
// Same interface at any devicePixelRatio
fn view(app: &App) -> El<App> {
    col((
        text("Logical 400×300"),  // Always 400×300 logical units
    )).size(400.0, 300.0)
}
// On regular display: draws at 400×300 pixels
// On Retina (2x): draws at 800×600 device pixels, but still called 400×300 logical
```

## Transformation Rules

### Rule 1: Input Events → Logical Coordinates

**Browser mouse click at viewport pixel (960, 540)**

**Step 1**: Normalize client coordinates to canvas coordinates
```
canvas_x = clientX * (canvas.width / window.innerWidth)
canvas_y = clientY * (canvas.height / window.innerHeight)
```

**Example (1920×1080 display, 1:1 ratio)**:
```
canvas_x = 960 * (1920 / 1920) = 960
canvas_y = 540 * (1080 / 1080) = 540
```

**Example (1920×1080 display with CSS scaling, e.g., 80% zoom)**:
```
window.innerWidth = 2400 (1920 / 0.8)
window.innerHeight = 1350 (1080 / 0.8)
canvas_x = 960 * (1920 / 2400) = 768
canvas_y = 540 * (1080 / 1350) = 432
```

**Step 2**: Convert device pixels to logical units
```
logical_x = canvas_x / scale_factor
logical_y = canvas_y / scale_factor
```

**Where**:
```
scale_factor = window.devicePixelRatio
             = canvas_device_width / canvas_logical_width
             = canvas_device_height / canvas_logical_height
```

**Example (Retina, scale_factor = 2.0)**:
```
logical_x = 960 / 2.0 = 480
logical_y = 540 / 2.0 = 270
```

### Rule 2: Rendering → Device Pixels

**Rui layout produces a 400×300 element at logical (0, 0)**

**Step 1**: Receive logical coordinates from layout
```rust
element_rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 400.0,
    h: 300.0
}
```

**Step 2**: Transform to device pixels in canvas.rs
```rust
device_x = logical_x * scale_factor
device_y = logical_y * scale_factor
device_w = logical_w * scale_factor
device_h = logical_h * scale_factor
```

**Example (scale_factor = 1.0)**:
```
device_x = 0.0 * 1.0 = 0
device_y = 0.0 * 1.0 = 0
device_w = 400.0 * 1.0 = 400
device_h = 300.0 * 1.0 = 300
```

**Example (scale_factor = 2.0)**:
```
device_x = 0.0 * 2.0 = 0
device_y = 0.0 * 2.0 = 0
device_w = 400.0 * 2.0 = 800
device_h = 300.0 * 2.0 = 600
```

### Rule 3: Hit-Testing

Hit-testing happens in **logical coordinates only**. No transformation needed.

```rust
// In paint.rs: one_tree walks elements and checks pointer position
pointer_in_logical_coords = Point { x: 480.0, y: 270.0 }
element_rect_logical = Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 }

if pointer_in_logical_coords.x >= element_rect.x
   && pointer_in_logical_coords.x < element_rect.x + element_rect.w { 
    // Hit
}
```

### Rule 4: Text Measurement and Rendering

Text measurement uses **logical coordinates only**. Font size, line height, advance width all operate in logical units.

```rust
// Measurement (logical units)
let advance = font.measure_text("Hello", font_size: 14.0);
// Result: advance = 70.0 logical pixels

// Drawing (canvas.rs multiplies by scale_factor)
let device_x = logical_x * scale_factor;
let device_advance = advance * scale_factor;
// On Retina (2.0): device_advance = 70.0 * 2.0 = 140.0 device pixels
```

## Scale Factor Detection

### Method 1: Direct Read (preferred)
```javascript
const scale_factor = window.devicePixelRatio;
```

**Advantages**:
- Simple, accurate, no polling needed
- Works on all browsers and OSes
- Updates automatically if display changes (e.g., monitor hot-plug)

**Caveats**:
- May not be exact on some browsers (e.g., 1.5 on some Android devices)
- Rounding to nearest 0.25 is safe for common cases (1.0, 1.25, 1.5, 2.0, etc.)

### Method 2: Canvas Size Inference (fallback)
```javascript
const canvas = document.getElementById('canvas');
const scale_factor = canvas.width / canvas.clientWidth;
// or
const scale_factor = canvas.height / canvas.clientHeight;
```

**When to use**: If `devicePixelRatio` is unavailable (unlikely in modern browsers)

**Example**:
```javascript
// Canvas logical size: 1920×1080
// Canvas device size: 3840×2160 (Retina)
const scale_factor = 3840 / 1920 = 2.0
```

### Method 3: Media Query (informational only)
```javascript
const scale_factor = window.matchMedia('(resolution: 2dppx)').matches ? 2.0 : 1.0;
```

**Note**: Less accurate than `devicePixelRatio`; use only for feature detection (e.g., "is this a Retina display?").

## Coordinate Contract Implementation Checklist

### Phase 1: Foundation
- [ ] Store canvas and scale_factor in Backend struct
- [ ] Read scale_factor from `window.devicePixelRatio` on startup
- [ ] Return scale_factor in `Backend::surface()` → `(width, height, scale_factor)`
- [ ] Verify scale_factor is 1.0 ≤ scale ≤ 4.0 (clamp if outside)
- [ ] No coordinate transformation in Phase 1 (defer to Phase 2)

### Phase 2: Enhancement
- [ ] Implement input event coordinate transformation:
  - [ ] clientX/Y → canvas coordinates (normalize by canvas/window size ratio)
  - [ ] canvas coordinates → logical coordinates (divide by scale_factor)
  - [ ] Write test: click at (0, 0) in browser → logical (0, 0) in handler
  - [ ] Write test: click at (1920, 1080) → logical (canvas.w, canvas.h)
- [ ] Handle window resize: update canvas size, preserve scale_factor
- [ ] Handle DPI change: detect new scale_factor, invalidate layout
- [ ] Test on displays with scale_factor = 1.0, 1.5, 2.0

### Phase 3: Integration
- [ ] Verify no device pixels leak into logical coordinates
  - [ ] `grep -n "device_pixel\|window.innerWidth\|clientX" src/` — should be in platform/wasm.rs only
- [ ] Verify coordinate transformation is invisible to upper layers:
  - [ ] Element trees use only logical coordinates
  - [ ] Handlers receive logical point (never device pixels)
  - [ ] Text measurement uses only logical units
- [ ] Cross-platform parity test: same UI at same logical size should render identically on any scale_factor
  - [ ] 400×300 element at (0, 0) measured in logical units
  - [ ] Hit test at logical (200, 150) should work on 1.0× and 2.0× displays
- [ ] Run `cargo test --test wasm_coordinate_parity`

## Common Pitfalls

### Pitfall 1: Forgetting to Transform Client Coordinates to Canvas Coordinates

**Wrong**:
```javascript
// Client coordinates are in viewport space, not canvas space!
pointer_x = event.clientX;
pointer_y = event.clientY;
// If canvas is scrolled or positioned off-screen, this is wrong
```

**Right**:
```javascript
const rect = canvas.getBoundingClientRect();
pointer_x = (event.clientX - rect.left) * (canvas.width / rect.width);
pointer_y = (event.clientY - rect.top) * (canvas.height / rect.height);
```

### Pitfall 2: Forgetting to Divide by Scale Factor

**Wrong**:
```rust
// Passing device pixels to rui
logical_x = device_x;  // Bug! This is device pixels, not logical
```

**Right**:
```rust
logical_x = device_x / scale_factor;
```

### Pitfall 3: Mixing Coordinate Systems in the Same Function

**Wrong**:
```rust
fn on_pointer_move(event: MouseEvent) {
    let logical_x = event.clientX / scale_factor;  // clientX is not device pixels!
    // ...
}
```

**Right**:
```rust
fn on_pointer_move(event: MouseEvent, scale_factor: f32) {
    // Step 1: clientX → canvas coordinates (normalize to canvas space)
    let canvas_x = event.clientX * (canvas.width / window.innerWidth);
    // Step 2: canvas coordinates → logical coordinates
    let logical_x = canvas_x / scale_factor;
    // ...
}
```

### Pitfall 4: Assuming scale_factor Doesn't Change

**Wrong**:
```javascript
// Read once at startup, assume it never changes
const scale_factor = window.devicePixelRatio;
// User moves window to different monitor (different DPI) → stale scale_factor!
```

**Right**:
```javascript
// Re-read on every frame (cheap operation)
let scale_factor = window.devicePixelRatio;
// Or listen to 'change' event on media query
window.matchMedia('(resolution: 2dppx)').addEventListener('change', () => {
    scale_factor = window.devicePixelRatio;
    invalidate_layout();
});
```

## Verification

### Test: Input Coordinate Transform

```rust
#[test]
fn wasm_input_coordinates_transform_correctly() {
    let scale_factor = 2.0;
    let canvas_w = 1920.0;
    let canvas_h = 1080.0;
    let window_w = 1920.0;  // CSS pixels
    let window_h = 1080.0;

    // User clicks at viewport (960, 540)
    let client_x = 960.0;
    let client_y = 540.0;

    // Transform to logical
    let canvas_x = client_x * (canvas_w / window_w);
    let canvas_y = client_y * (canvas_h / window_h);
    let logical_x = canvas_x / scale_factor;
    let logical_y = canvas_y / scale_factor;

    // Expected: (960 * 1.0) / 2.0 = 480, (540 * 1.0) / 2.0 = 270
    assert_eq!(logical_x, 480.0);
    assert_eq!(logical_y, 270.0);
}
```

### Test: Rendering Coordinate Transform

```rust
#[test]
fn wasm_rendering_coordinates_transform_correctly() {
    let scale_factor = 2.0;
    let logical_rect = Rect { x: 0.0, y: 0.0, w: 400.0, h: 300.0 };

    // Transform to device pixels
    let device_x = logical_rect.x * scale_factor;
    let device_y = logical_rect.y * scale_factor;
    let device_w = logical_rect.w * scale_factor;
    let device_h = logical_rect.h * scale_factor;

    // Expected: 0.0, 0.0, 800.0, 600.0
    assert_eq!(device_x, 0.0);
    assert_eq!(device_y, 0.0);
    assert_eq!(device_w, 800.0);
    assert_eq!(device_h, 600.0);
}
```

---

**Next document**: STEP_4_RECIPE_1_EVENT_TRANSLATION.md — How browser events map to rui Events
