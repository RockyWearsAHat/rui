# STEP 2: TextRole/Space/Height State Extraction

**Date**: 2026-09-01  
**Task**: Read src/theme.rs and extract TextRole/Space/Height state with exact line numbers  
**Status**: COMPLETE

## Grep Results (Complete)

```
298:pub struct Metrics {
646:    pub fn spacing(&self, space: Space) -> f32 {
655:    pub fn text_size(&self, role: TextRole) -> f32 {
667:    pub fn control_height(&self, h: Height) -> f32 {
677:pub enum Space {
688:pub enum TextRole {
705:pub enum Height {
```

**All acceptance criteria elements found** ✓

## Element Locations and Content

### 1. Metrics Struct (Line 298)
**Status**: PRESENT ✓  
**Scope**: Lines 298–340  
**Contents**:
- `pub struct Metrics` with 15 fields:
  - `gap_small: f32` (line 300)
  - `gap: f32` (line 302)
  - `gap_large: f32` (line 304)
  - `padding: f32` (line 306)
  - `corner: f32` (line 311)
  - `corner_small: f32` (line 313)
  - `control_height: f32` (line 315)
  - `row_height: f32` (line 317)
  - `hairline: f32` (line 322)
  - `scrollbar: f32` (line 324)
  - `shadow: f32` (line 326)
  - `shadow_offset: f32` (line 332)
  - `motion: f32` (line 339)
- `Metrics::DEFAULT` constant (lines 354–368)

### 2. Space Enum (Line 677)
**Status**: PRESENT ✓  
**Scope**: Lines 677–684  
**Contents**:
```rust
pub enum Space {
    Small,      // Gap between closely-related items
    Normal,     // Standard gap between items in a list or row
    Large,      // Gap between sections
}
```
**Uses**: Passed to `Theme::spacing()` method (line 646)

### 3. TextRole Enum (Line 688)
**Status**: PRESENT ✓  
**Scope**: Lines 688–701  
**Contents**:
```rust
pub enum TextRole {
    Title,      // Prominent designations and window titles
    Heading,    // Section headings and group labels
    Body,       // Ordinary text body
    Caption,    // Secondary text, labels, and explanations
    Micro,      // Smallest readable text: numbers, units, ticks
    Code,       // Machine output and monospaced code
}
```
**Uses**: Passed to `Theme::text_size()` method (line 655)

### 4. Height Enum (Line 705)
**Status**: PRESENT ✓  
**Scope**: Lines 705–710  
**Contents**:
```rust
pub enum Height {
    Control,    // Height of a button or text field control
    Row,        // Height of one row in a list or table
}
```
**Uses**: Passed to `Theme::control_height()` method (line 667)

## Resolution Methods in Theme

### Theme::spacing() (Line 646)
**Status**: PRESENT ✓  
**Signature**: `pub fn spacing(&self, space: Space) -> f32`  
**Lines**: 646–652  
**Implementation**: Resolves Space enum to gap values from Metrics

### Theme::text_size() (Line 655)
**Status**: PRESENT ✓  
**Signature**: `pub fn text_size(&self, role: TextRole) -> f32`  
**Lines**: 655–664  
**Implementation**: Resolves TextRole enum to hardcoded font sizes
**Hardcoded sizes**:
- Title: 15.0
- Heading: 10.5
- Body: 13.0
- Caption: 11.5
- Micro: 9.5
- Code: 11.5

### Theme::control_height() (Line 667)
**Status**: PRESENT ✓  
**Signature**: `pub fn control_height(&self, h: Height) -> f32`  
**Lines**: 667–672  
**Implementation**: Resolves Height enum to control_height/row_height from Metrics

## Key Findings

### Duplication Detected

**Issue**: TextRole::text_size() has hardcoded font sizes (lines 657–663) that could be managed by Theme.
- Title: `15.0` (hardcoded) vs `TITLE_SIZE: 15.0` constant (line 372)
- Heading: `10.5` (hardcoded) vs `HEADING_SIZE: 10.5` constant (line 375)
- Body: `13.0` (hardcoded) vs not exposed as constant
- Caption: `11.5` (hardcoded) vs `CAPTION_SIZE: 11.5` constant (line 378)
- Micro: `9.5` (hardcoded) vs `MICRO_SIZE: 9.5` constant (line 381)
- Code: `11.5` (hardcoded) vs `CODE_SIZE: 11.5` constant (line 390)

### Inconsistencies with CLAUDE.md

Per CLAUDE.md R1 roadmap note:
> "widgets.rs's duplicate size constants (they already disagree with Theme: body 13.0 vs BODY_SIZE 13.5)"

**Finding**: In src/theme.rs, Theme::body() uses 13.0 (line 572), but there may be a BODY_SIZE constant somewhere.

### Roadmap R1 Requirements

From rui.dx#roadmap-list R1:
1. ✓ TextRole enum exists (all variants present)
2. ✓ Space enum exists (all variants present)
3. ✓ Height enum exists (all variants present)
4. ✓ Theme::spacing() resolution method exists
5. ✓ Theme::text_size() resolution method exists
6. ✓ Theme::control_height() resolution method exists
7. ⚠ Hardcoded sizes in text_size() should be extracted to Metrics or constants
8. ⚠ widgets.rs constants need audit (BODY_SIZE discrepancy noted in CLAUDE.md)

## Test Coverage

**Lines 725–1046**: Comprehensive test suite in src/theme.rs
- 25 test functions covering palette legibility, contrast, surface shading, corner shapes, etc.
- No tests yet for TextRole/Space/Height enum resolution (opportunity for new tests)

## Acceptance Criteria Met

✓ File read and analyzed  
✓ All target enums and structs located with exact line numbers  
✓ Resolution methods identified and documented  
✓ Duplication and inconsistencies noted  
✓ Findings captured into scratch notes  

## Next Steps (STEP 3)

1. Write failing test for hardcoded sizes → extract to Metrics struct
2. Audit widgets.rs for size constant discrepancies
3. Refactor Theme to use Metrics for all size resolution
4. Verify all tests pass and no hardcoded values remain in view code
