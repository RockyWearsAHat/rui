# STEP 15: Palette::derive for Theme Generation (R13)

**Status**: ✅ COMPLETE  
**Commits**: 1  
**Tests Added**: 8  
**Total Tests Passing**: 751 (394 library + 357 feature tests)

## Overview

Implemented `Palette::derive(accent: Color, appearance: Appearance) -> Self`, enabling dynamic theme palette generation from a base accent color. Applications can now create complete custom palettes without hardcoding all 25 color values.

## Implementation Details

### Core Method
```rust
pub fn derive(accent: Color, appearance: Appearance) -> Self
```

Takes a single accent color and an appearance (Light/Dark), returning a fully-formed Palette that:
- Generates appropriate surface colors (background, surface, raised, sunken)
- Creates text colors (primary, muted, on-accent)
- Derives accent variants (darker for controls, lighter for highlights)
- Generates status colors (ok, warn, bad, idle) at appropriate hues
- Ensures all pairings meet WCAG contrast requirements

### Helper Methods
1. **`derive_light(accent: Color) -> Self`**
   - Creates light-appearance palette with neutral greys
   - Surface hierarchy: background < surface < raised
   - Text is dark (low luminance)

2. **`derive_dark(accent: Color) -> Self`**
   - Creates dark-appearance palette with ascending-value greys
   - Surface hierarchy: background_deep < background < surface < raised (by luminance)
   - Text is light (high luminance)

3. **`darken_for_contrast(color, factor, background)`**
   - Darkens a color by mixing with black
   - Ensures minimum 4.5:1 contrast ratio

4. **`lighten_for_contrast(color, factor, foreground)`**
   - Lightens a color by mixing with white
   - Ensures minimum 4.5:1 contrast ratio

5. **`ensure_focus_contrast(color, surface)`**
   - Guarantees focus ring achieves 3:1 contrast
   - Adjusts accent brightness/saturation if needed

## Test Coverage

✅ **derive_creates_a_light_palette_from_a_base_accent**
- Verifies accent matches input
- Confirms palette passes contrast audit
- Validates surface hierarchy

✅ **derive_creates_a_dark_palette_from_a_base_accent**
- Confirms ascending value hierarchy (dark theme requirement)
- Validates accent color is preserved
- Ensures legibility

✅ **derive_generates_all_status_colors**
- Verifies ok/warn/bad/idle colors are distinct from accent
- Confirms all pairings are legible

✅ **derive_accent_variants_are_distinct**
- Confirms accent_deep is darker than accent
- Confirms accent_light is lighter than accent

✅ **derive_respects_appearance_in_text_color**
- Light theme: text luminance < 0.5
- Dark theme: text luminance > 0.5

✅ **derive_red_accent_creates_distinct_status_colors**
- Tests with red accent (different from standard blue)
- Ensures status colors remain distinct

✅ **derive_green_accent_maintains_contrast**
- Tests with green accent (historically low contrast)
- Validates focus ring adjustment mechanism

✅ **multiple_derives_are_independent**
- Confirms multiple colors can be derived without interference
- Both palettes legible and independent

## Architecture Pattern

### Design Pattern
```rust
// Before: hardcode all 25 colors
let palette = Palette { background, background_deep, surface, ... };

// After: derive from one accent
let palette = Palette::derive(Color::rgb(0x25, 0x63, 0xd4), Appearance::Light);
```

### Invariant Maintained
- **Legibility**: Every derived palette passes `assert_legible()`
- **Contrast Hierarchy**: Dark palettes maintain ascending value; light palettes neutral
- **Status Distinction**: All four status hues remain visually distinct
- **Focus Accessibility**: Focus ring always meets 3:1 minimum contrast

## Integration Points

1. **Color module** (`src/color.rs`)
   - Uses existing `mix()` for color interpolation
   - Uses existing `luminance()` and `contrast_ratio()` for validation

2. **Theme module** (`src/theme.rs`)
   - New `Palette::derive()` public method
   - Private helpers for light/dark derivation
   - Reuses existing `assert_legible()` for validation

3. **Testing** (`tests/r13_palette_derive.rs`)
   - No dependencies on other R-feature tests
   - Tests derivation for both appearances
   - Tests corner cases (low-contrast accents)

## API Additions

```rust
impl Palette {
    pub fn derive(accent: Color, appearance: Appearance) -> Self;
}
```

Public, non-breaking addition. Existing code unaffected.

## Use Cases

1. **Branding**: Apps use corporate color as accent, derive full palette
2. **Theming**: Users choose a color, app generates complete theme
3. **Accessibility**: Different users get colors derived from their preferences
4. **Testing**: Generate varied palettes to test contrast compliance programmatically

## Verification

```bash
cargo test --test r13_palette_derive          # All 8 tests pass
cargo test --lib                              # All 394 library tests pass
cargo test                                    # All 751 tests pass
```

## Next Steps

- **R12** (Golden-image regression net): Capture reference pixels for visual regression
- **R11-R14**: Document existing features (caret blink, pointer move, contrast, shutdown)
- **R7, R8**: Polish and document elevation/overlay features
- **Extended roadmap**: Palette modulation, animation curves, layout inspector

## Related Documentation

- `rui.dx`: Library roadmap, full R13 entry
- `CLAUDE.md`: Recipe infrastructure, testing patterns
- `src/theme.rs`: Complete implementation with inline comments
