# X11 Coordinate Contract: Specification & Verification

## Overview

The X11 backend must translate between two coordinate systems:
- **Device pixels**: Physical screen coordinates reported by X11 (XEvent motion_x, button_x, etc.)
- **Logical pixels**: DPI-independent coordinates used by rui's frame loop

### Specification: The Coordinate Translation Formula

**Forward Transform (Device → Logical):**
```
logical_x = device_x / dpi_scale_factor
logical_y = device_y / dpi_scale_factor
```

**Reverse Transform (Logical → Physical):**
```
device_x = logical_x * dpi_scale_factor
device_y = logical_y * dpi_scale_factor
```

## DPI Scale Factor Derivation

```rust
let width_pixels = XDisplayWidth(display, screen);       // in device pixels
let width_mm = XDisplayWidthMM(display, screen);         // physical width in mm
let dpi = (width_pixels as f32) / (width_mm as f32 * MM_PER_INCH);
let scale_factor = dpi / 96.0;  // 96 DPI is the baseline (1:1 scale)
```

## Critical Invariants

### Invariant 1: Click Event Coordinate Translation

**Property:** When a Click event is generated at device coordinate (dx, dy), the frame loop receives it at logical coordinate (dx/scale, dy/scale).

### Invariant 2: Widget Hit Testing

**Property:** A widget at logical rect (lx, ly, lw, lh) occupies device rect (lx*scale, ly*scale, lw*scale, lh*scale). A click in the device rect hits the widget.

### Invariant 3: Drag Deltas Preserve Scale

**Property:** A drag motion from device (x1, y1) to device (x2, y2) produces a delta that matches the logical delta.

### Invariant 4: Window Size Consistency

**Property:** If the window's device size is (device_w, device_h) and scale is s, the logical size is (device_w/s, device_h/s).

## Phase-by-Phase Contract Verification

### Phase 1: Foundation (Commit a67d578)
**Gate:** Coordinate translation compiles; scale factor defaults to 1.0; device→logical math is correct.

### Phase 2: Enhancement (Commit c42c0f0)
**Gate:** DPI detection from XDisplayWidth/XDisplayWidthMM; scale factor within [0.5, 2.5] for typical monitors.

### Phase 3: Integration (Commits 80e3003–84ade0e)
**Gate:** Click events at device (x, y) hit correct logical widget; drag deltas accurate; window resize adjusts logical dimensions.

## Testing the Contract on Real Hardware

To verify coordinate contract on an actual X11 system:

```bash
# 1. Build the X11 backend
cargo build --target x86_64-unknown-linux-gnu

# 2. Run coordinate contract tests
cargo test --test x11_backend_phases -- coordinate_contract

# 3. Manual verification: Click known logical positions
DISPLAY=:0 cargo run -p rui --example counter
```

## Regression Prevention

```bash
# After changes to coordinate translation:
cargo test --test x11_backend_phases -- coordinate_contract
cargo test --test recipe_1_verification -- click_event_coordinate
```

## Summary Table: Contract Verification Points

| Invariant | Phase | Acceptance |
|-----------|-------|-----------|
| Click event translation | 1, 3 | Device (dx, dy) → Logical (dx/s, dy/s) |
| Widget hit testing | 2, 3 | Logical hit ⟺ Device hit |
| Drag delta preservation | 1, 3 | Delta_device / s = Delta_logical |
| Window size consistency | 1, 3 | Window_logical = Window_device / s |
| DPI detection | 2 | Scale ∈ [0.5, 2.5] for common monitors |
| Event loop enforcement | 3 | All events translated before reaching frame |
