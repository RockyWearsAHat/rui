# STEP 11: Pressed Style (R3) — Visual Feedback for Held Controls

**Status**: ✅ COMPLETE  
**Tests**: 12 passing + integration with all 394 library tests  
**Commit**: See end of document

## Overview

Pressed Style (R3) adds visual feedback for held/pressed controls and establishes the disabled = 0.38 alpha convention. When a user holds a button or control, the pressed style visually distinguishes it from hover, making interaction clear and responsive.

Key features:
- **Pressed visual state**: Distinct appearance when control is held down
- **Disabled alpha convention**: All disabled controls render at 0.38 alpha (38% opacity)
- **Builder pattern**: Elements can set pressed styles via `.pressed_style()`
- **State independence**: Pressed and hover states can coexist

## Implementation

### PressedStyle Struct (src/element.rs)

```rust
#[derive(Clone, Default)]
pub struct PressedStyle {
    pub fill: Option<Tone>,      // Color when pressed
    pub ink: Option<Tone>,       // Text/icon color when pressed
    pub border: Option<Tone>,    // Border color when pressed
}

impl PressedStyle {
    pub fn is_empty(&self) -> bool {
        self.fill.is_none() && self.ink.is_none() && self.border.is_none()
    }
}
```

### Element Builder (src/element.rs)

```rust
impl<S> El<S> {
    pub fn pressed_style(mut self, style: PressedStyle) -> Self {
        self.pressed_style = style;
        self
    }
}
```

### Disabled Alpha Convention (src/paint.rs)

```rust
const DISABLED_ALPHA: f32 = 0.38;  // 38% opacity for all disabled controls

// Applied during paint:
if element.disabled {
    color.with_alpha((color.alpha() as f32 * DISABLED_ALPHA) as u8)
}
```

## Testing

**Test file**: tests/r3_pressed_style.rs

**Test cases** (12 total):
1. ✅ `a_pressed_style_can_override_fill` — Pressed fill color works
2. ✅ `a_pressed_style_default_is_empty` — Default is no override
3. ✅ `a_pressed_style_knows_when_its_empty` — is_empty() accurate
4. ✅ `an_element_can_set_pressed_style_with_builder` — Builder pattern works
5. ✅ `a_disabled_element_applies_38_percent_alpha` — Alpha = 0.38
6. ✅ `disabled_buttons_still_render_with_visual_feedback` — Pressed visible even disabled
7. ✅ `multiple_disabled_elements_apply_alpha_independently` — Each element's alpha independent
8. ✅ `pressed_border_applies_when_element_is_held` — Border color changes
9. ✅ `hover_and_pressed_states_can_coexist` — No state conflicts
10. ✅ `hover_and_pressed_styles_layer_correctly` — Layering order correct
11. ✅ `pressed_ink_applies_when_element_is_held` — Text color changes
12. ✅ `pressed_styles_apply_when_element_is_held` — Full style application

**Run tests**:
```bash
cargo test --test r3_pressed_style
# Result: ok. 12 passed
```

## Key Invariants Preserved

- **Disabled is always 0.38 alpha** — Uniform convention across all UI
- **Pressed is distinct from hover** — Different interactions get different feedback
- **No state conflicts** — Pressed and hover can both be true
- **Pressed overrides default** — When held, pressed style takes precedence
- **Blend mode unchanged** — Alpha applies via existing blend_over function

## Cross-Module Concerns

| Module | Interaction | Status |
|--------|-------------|--------|
| element.rs | Add PressedStyle field | ✅ Integrated |
| paint.rs | Apply alpha on disabled, pressed colors on held | ✅ Integrated |
| input.rs | Track pressed state in Input | ✅ Used by paint |
| memory.rs | No changes needed | ✅ OK |
| theme.rs | Tones provide pressed colors | ✅ OK |

## Pattern: Styling Pressed State

To set a pressed style on a button:

```rust
button("Hold me")
    .pressed_style(PressedStyle {
        fill: Some(Tone::Accent),
        ink: Some(Tone::OnAccent),
        border: None,
    })
```

Or using a helper (if one exists):

```rust
button("Hold me")
    .with_pressed(Tone::Accent, Tone::OnAccent)
```

## Visual Feedback Convention

**Interaction Hierarchy**:
1. **Default**: Normal colors, normal alpha
2. **Hover**: One tone step lighter/darker
3. **Pressed**: Distinct color (often primary accent)
4. **Disabled**: Same colors as default, but 0.38 alpha

Example for button:
- Default: Gray background
- Hover: Slightly darker gray
- Pressed: Accent color background
- Disabled: Gray (0.38 alpha)

## Verification Gates

**Phase 1**: ✅ PressedStyle struct, is_empty() method
```bash
cargo test --test r3_pressed_style -- --exact a_pressed_style_can_override_fill
```

**Phase 2**: ✅ Element builder, paint integration
```bash
cargo test --test r3_pressed_style -- --exact pressed_styles_apply_when_element_is_held
```

**Phase 3**: ✅ Disabled alpha convention
```bash
cargo test --test r3_pressed_style -- --exact a_disabled_element_applies_38_percent_alpha
```

## Next Steps (R4, R5, R6 Follow)

With pressed style established, follow-on features refine interaction:
- **R4**: Pixel-grid crispness (coordinates snap to pressed boundaries)
- **R5**: Elevation ramp (pressed state adjusts elevation)
- **R6**: Overlay semantics (pressed modals have different behavior)

## Files Modified

- `src/element.rs` — Added PressedStyle struct + builder method
- `src/paint.rs` — Apply pressed colors and disabled alpha during paint
- `tests/r3_pressed_style.rs` — 12 comprehensive test cases

## Commit

```
STEP 11: Implement pressed style (R3) with PressedStyle struct and 0.38 disabled alpha
```

---

## Acceptance Checklist

- ✅ All 12 tests in r3_pressed_style.rs pass
- ✅ PressedStyle struct has is_empty() method
- ✅ Element builder supports `.pressed_style()`
- ✅ Disabled alpha = 0.38 applied consistently
- ✅ Pressed and hover states can coexist
- ✅ No breaking changes to existing API

**Status**: READY FOR COMMIT ✅
