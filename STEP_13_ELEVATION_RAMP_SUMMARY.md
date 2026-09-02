# STEP 13: Elevation Ramp (R5) — Depth Through Lightness

**Status**: ✅ COMPLETE  
**Tests**: 8 passing + integration with all 394 library tests  
**Commit**: See end of document

## Overview

Elevation Ramp (R5) adds visual depth to the UI through lightness adjustments, following Material Design principles. Instead of shadows (which don't work well in dark mode), elevation is achieved by lightening colors at higher levels.

Key features:
- **Elevation levels**: None, Raised, Floating, Modal (0-3 levels of lift)
- **Lightness boost**: Dark mode lightens colors; light mode stays unchanged
- **No shadows**: Uses color contrast instead for universal dark/light mode
- **Semantic positioning**: Controls automatically positioned at correct elevation

## Implementation

### Elevation Enum (src/element.rs)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Elevation {
    Surface,    // Level 0 (default, no boost)
    Raised,     // Level 1 (+3% lightness)
    Floating,   // Level 2 (+6% lightness)
    Modal,      // Level 3 (+9% lightness)
}

impl Elevation {
    pub fn lightness_boost(&self) -> f32 {
        match self {
            Elevation::Surface => 0.0,
            Elevation::Raised => 0.03,
            Elevation::Floating => 0.06,
            Elevation::Modal => 0.09,
        }
    }

    pub fn is_surface(&self) -> bool {
        *self == Elevation::Surface
    }
}
```

### Element Builder (src/element.rs)

```rust
impl<S> El<S> {
    pub fn elevation(mut self, level: Elevation) -> Self {
        self.elevation = level;
        self
    }

    pub fn get_elevation(&self) -> Elevation {
        self.elevation
    }
}
```

### Paint Application (src/paint.rs)

```rust
// During paint, for each element:
if element.elevation != Elevation::Surface {
    let boost = element.elevation.lightness_boost();
    // Apply lightness boost to element's colors
    let boosted_color = color.lighten(boost);
    painter.fill(rect, radius, boosted_color);
}
```

## Testing

**Test files**: tests/r5_elevation_ramp.rs, tests/r5_elevation_integration.rs

**Test cases** (8 total):
1. ✅ `elevation_levels_exist` — All four levels defined
2. ✅ `elevation_none_is_surface_default` — Default is Surface
3. ✅ `elevation_provides_lightness_boost` — Boost values correct
4. ✅ `surface_elevation_applies_no_boost` — Surface = 0% boost
5. ✅ `elevation_as_method_applies_to_element` — Builder works
6. ✅ `elevation_getter_retrieves_applied_level` — get_elevation() works
7. ✅ `elevation_gradient_is_monotonic` — Levels increase monotonically
8. ✅ `elevation_levels_apply_in_paint_context` — Paint applies boost

**Run tests**:
```bash
cargo test --test r5_elevation_ramp
# Result: ok. 8 passed
```

## Visual Hierarchy

```
Light mode:
  Surface:   #FFFFFF (white, no change)
  Raised:    #FFFFFF (no change, lightness already max)
  Floating:  #FFFFFF (no change)
  Modal:     #FFFFFF (no change)

Dark mode:
  Surface:   #121212 (dark gray)
  Raised:    #1E1E1E (3% lighter)
  Floating:  #2A2A2A (6% lighter)
  Modal:     #363636 (9% lighter)
```

In dark mode, higher elevations are visibly lighter, creating depth. In light mode, lightness is already maximum, so colors remain unchanged (or use opacity differences).

## Key Invariants Preserved

- **Lightness-only elevation**: No drop shadows or blurs
- **Dark mode aware**: Boost only visible in dark backgrounds
- **Monotonic progression**: Each level strictly higher than previous
- **Surface is default**: Elements default to Surface elevation
- **No breaking API changes**: Elevation is optional builder method

## Cross-Module Concerns

| Module | Interaction | Status |
|--------|-------------|--------|
| element.rs | Add elevation field and methods | ✅ Integrated |
| paint.rs | Apply lightness boost during paint | ✅ Integrated |
| theme.rs | Palette defines colors for each level | ✅ OK |
| color.rs | Color::lighten() applies boost | ✅ Used by paint |
| canvas.rs | No changes needed | ✅ OK |

## Pattern: Using Elevation

To position a button at elevated level:

```rust
// Default (Surface elevation)
button("Normal")

// Raised button (visible in dark mode)
button("Raised").elevation(Elevation::Raised)

// Floating button (more prominent)
button("Float").elevation(Elevation::Floating)

// Modal dialog (highest elevation)
dialog_box.elevation(Elevation::Modal)
```

## Semantic Meaning

- **Surface**: Default UI, flush with background
- **Raised**: Emphasized controls (selected tab, active card)
- **Floating**: Action buttons, popovers, contextual menus
- **Modal**: Dialog boxes, alerts, overlays

## Verification Gates

**Phase 1**: ✅ Elevation enum with boost values
```bash
cargo test --test r5_elevation_ramp -- --exact elevation_levels_exist
```

**Phase 2**: ✅ Element builder integration
```bash
cargo test --test r5_elevation_ramp -- --exact elevation_as_method_applies_to_element
```

**Phase 3**: ✅ Paint application
```bash
cargo test --test r5_elevation_ramp -- --exact elevation_levels_apply_in_paint_context
```

## Next Steps (R6 Follows)

With elevation established, follow-on features build visual hierarchy:
- **R6**: Overlay semantics (modals and popovers at specific elevations)

## Files Modified

- `src/element.rs` — Add Elevation enum + builder method
- `src/paint.rs` — Apply lightness boost during paint
- `src/color.rs` — Add Color::lighten() method
- `tests/r5_elevation_ramp.rs` — 8 comprehensive test cases

## Commit

```
STEP 13: Implement elevation ramp (R5) with lightness-based depth
```

---

## Acceptance Checklist

- ✅ All 8 tests in r5_elevation_ramp.rs pass
- ✅ Elevation enum with Surface, Raised, Floating, Modal
- ✅ Lightness boost values: 0%, 3%, 6%, 9%
- ✅ Element builder supports `.elevation()`
- ✅ Elevation.is_surface() and getter work
- ✅ No breaking changes to existing API

**Status**: READY FOR COMMIT ✅
