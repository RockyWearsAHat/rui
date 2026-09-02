# STEP 4: Widget-to-Constant Mapping

**Status**: Complete (with corrections)
**Date**: 2026-09-01
**Last Updated**: 2026-09-01 (Added missing UNMATCHED comments for tabs/segmented text sizes; corrected comment for section widget)
**Task**: Map affected widgets and document which widget functions reference hardcoded constants found in STEP 3

## Acceptance Criteria
- ✓ `grep -n "^pub fn \|^fn " src/widgets.rs | wc -l` = 27
- ✓ Count matches documented widgets
- ✓ Verification tests written and passing (tests/step_4_widget_mapping.rs)

## Executive Summary

All 27 widget functions in `src/widgets.rs` have been mapped to their constant usages and hardcoded numeric duplicates identified. The mapping reveals:
- **25 public widget constructors** (re-exported via lib.rs) + **1 public but internal widget** (scrollbar) + **1 private helper** (word_for) = 27 total functions
- **4 widget-specific constants** defined at file top (BODY_SIZE, HEADING_TRACKING, TAG_HEIGHT, FIELD_ROW_LABEL_WIDTH)
- **14 theme constants** imported from `theme.rs` (Metrics, type scale sizes)
- **6 widgets with hardcoded literals** not yet extracted to constants (meter, tabs, segmented, star_rating, section, scrollbar)
- **15 duplicate constants/values** identified: 6 theme-defined constants (used by 15 widgets) + 9 hardcoded numeric duplicates (used across 36+ widget references)

## Widget Function Registry

### Layout Primitives (4 functions)
1. **col** (line 59) — Vertical stack; no constants
2. **row** (line 70) — Horizontal stack; no constants
3. **spacer** (line 79) — Empty box; no constants
4. **draw** (line 308) — Custom drawing surface; no constants

### Typography (8 functions)
5. **text** (line 84) — Plain text; no constants
6. **title** (line 89) — Window/pane heading; uses TITLE_SIZE
7. **heading** (line 100) — Section label; uses HEADING_SIZE (line 102), HEADING_TRACKING (line 103)
8. **caption** (line 109) — Aside/timestamp; uses CAPTION_SIZE (line 111)
9. **micro** (line 116) — Smallest annotation; uses MICRO_SIZE (line 118)
10. **figure** (line 124) — Large readable count; uses FIGURE_SIZE (line 125)
11. **code** (line 129) — Machine-produced text; uses CODE_SIZE (line 130)
12. **paragraph** (line 134) — Wrapped text block; no constants

### Containers (2 functions)
13. **panel** (line 144) — Floating surface with shadow; uses Metrics::DEFAULT.padding (line 150)
14. **divider** (line 154) — Hairline separator; uses Metrics::DEFAULT.hairline (line 156)

### Interactive Controls (13 functions)

#### Buttons & Fields
15. **button** (line 175) — Raised clickable surface
    - Uses Metrics::DEFAULT.control_height (line 177)
    - Uses Metrics::DEFAULT.padding (line 178)

16. **field** (line 192) — Text input, monospace
    - Uses Metrics::DEFAULT.control_height (line 197)
    - Uses Metrics::DEFAULT.gap (line 198)
    - Uses CODE_SIZE (line 199)

#### Status & Tags
17. **tag** (line 215) — Status badge with tint
    - Uses TAG_HEIGHT (line 218)
    - Uses Metrics::DEFAULT.gap (line 217)
    - Uses HEADING_SIZE (line 221)
    - Hardcoded tracking value: 0.4 (line 222)

18. **dot** (line 234) — Status indicator circle
    - No constants (uses radius parameter, line 234)

#### Visualizations
19. **meter** (line 275) — Progress bar
    - **Hardcoded 80.0** (line 279) — bar width
    - **Hardcoded 6.0** (line 279) — bar height
    - **Hardcoded 6.0** (line 296) — alternative height

#### Selectors
20. **tabs** (line 325) — Tab row selector
    - **Hardcoded 12.5** (line 336) — text size (UNMATCHED comment added)
    - **Hardcoded 26.0** (line 337) — tab row height
    - **Hardcoded 2.0** (line 340) — indicator bar height
    - Uses Metrics::DEFAULT.padding (line 338)
    - Uses Metrics::DEFAULT.control_height (line 354)

21. **segmented** (line 361) — Multi-button choice
    - **Hardcoded 12.0** (line 374) — text size (UNMATCHED comment added)
    - **Hardcoded 3.0** (line 390) — internal padding
    - **Hardcoded 2.0** (line 391) — button gap
    - Uses Metrics::DEFAULT.row_height (line 377)
    - Uses Metrics::DEFAULT.control_height (line 389)

22. **star_rating** (line 398) — 1–5 star widget
    - **Hardcoded 16.0×16.0** (line 407) — star size
    - Uses Metrics::DEFAULT.gap_small (line 417)

#### Organizers
23. **section** (line 489) — Label with rule to edge
    - **Hardcoded 14.0** (line 498) — section header height
    - Uses Metrics::DEFAULT.hairline (line 493)
    - Uses Metrics::DEFAULT.gap (line 499)

24. **field_row** (line 514) — Label + value pair
    - Uses FIELD_ROW_LABEL_WIDTH (line 521)
    - Uses Metrics::DEFAULT.gap (line 522)

25. **field_group** (line 536) — Label + stacked values
    - Uses FIELD_ROW_LABEL_WIDTH (line 542)
    - Uses Metrics::DEFAULT.control_height (line 543)
    - Uses Metrics::DEFAULT.gap (line 547)

#### Scrolling
26. **scrollbar** (line 555) — Interactive scroll thumb
    - **Hardcoded 12.0** (line 562) — scrollbar width

### Utilities (1 private function)
27. **word_for** (line 260) — Status label lookup; no constants

## Constant Reference Summary

### Defined in widgets.rs (4 constants)
| Constant | Value | Used By | Line |
|----------|-------|---------|------|
| BODY_SIZE | 13.0 | ❌ NOT USED | 41 |
| HEADING_TRACKING | 0.9 | heading | 103 |
| TAG_HEIGHT | 18.0 | tag | 218 |
| FIELD_ROW_LABEL_WIDTH | 78.0 | field_row, field_group | 521, 542 |

### Imported from theme.rs (14 constants + Metrics)
| Constant | Used By | Line(s) |
|----------|---------|---------|
| TITLE_SIZE | title | 91 |
| HEADING_SIZE | heading, tag | 102, 221 |
| CAPTION_SIZE | caption | 111 |
| MICRO_SIZE | micro | 118 |
| FIGURE_SIZE | figure | 125 |
| CODE_SIZE | code, field | 130, 199 |
| Metrics::DEFAULT.padding | panel, button, tabs | 150, 178, 338 |
| Metrics::DEFAULT.control_height | button, field, tabs, segmented, field_group | 177, 197, 354, 389, 543 |
| Metrics::DEFAULT.gap | field, tag, field_row, field_group, section | 198, 217, 522, 547, 499 |
| Metrics::DEFAULT.gap_small | star_rating | 417 |
| Metrics::DEFAULT.hairline | divider, section | 156, 493 |
| Metrics::DEFAULT.row_height | segmented | 377 |

## Duplicate Constants Analysis

**Definition**: Constants or hardcoded values referenced by 2 or more widget functions.

### Theme-Defined Duplicate Constants

| Constant | Usage Count | Widget Functions | Line Numbers |
|----------|-------------|------------------|--------------|
| **CODE_SIZE** | 2 | code, field | 130, 199 |
| **HEADING_SIZE** | 2 | heading, tag | 102, 221 |
| **Metrics::DEFAULT.hairline** | 2 | divider, section | 156, 493 |
| **Metrics::DEFAULT.padding** | 3 | panel, button, tabs | 150, 178, 338 |
| **Metrics::DEFAULT.gap** | 5 | field, tag, field_row, field_group, section | 198, 217, 522, 547, 499 |
| **Metrics::DEFAULT.control_height** | 5 | button, field, tabs, segmented, field_group | 177, 197, 354, 389, 543 |

**Subtotal**: 6 theme-defined duplicates across 15 widget references

### Hardcoded Numeric Duplicates (same value across multiple widgets)

Analysis of hardcoded numeric literals reveals actual duplicates—same numbers appearing in multiple widgets that could potentially be extracted:

| Value | Occurrences | Widgets | Context | Line Numbers |
|-------|-------------|---------|---------|--------------|
| **1.0** | 8 | button, field, meter (2×), panel, scrollbar, segmented, star_rating | Stroke widths, opacity, unit values | 147, 180, 202, 277, 284, 393, 443, 578 |
| **2.0** | 8 | dot (4×), meter, segmented, tabs | Border thickness, indicator height, gaps | 237, 244, 245, 280, 292, 340, 391 |
| **0.0** | 6 | meter, scrollbar (2×), star_rating (2×) | Position offsets, start values | 277, 287, 431, 452, 570, 578 |
| **16.0** | 4 | star_rating (4×) | Icon size for star glyphs | 405, 407 |
| **6.0** | 3 | meter (3×) | Meter bar height and metrics | 278, 279, 296 |
| **12.0** | 2 | scrollbar, segmented | Text size / control dimensions | 374, 562 |
| **20.0** | 2 | field_group, field_row | Label area width / spacing | 523, 548 |
| **5** | 2 | segmented, star_rating | Rating count / item count | 397, 402 |
| **80.0** | 2 | meter (2×) | Meter width measurement | 278, 279 |

**Subtotal**: 9 hardcoded numeric duplicates across 36+ widget references

**Key insight:** Most duplicates (1.0, 2.0, 0.0) are generic values used for strokes, gaps, and offsets. Only larger values (12.0, 16.0, 20.0) are widget-specific. The frequent use of 1.0 and 2.0 suggests these could benefit from named constants (e.g., `HAIRLINE_WIDTH = 1.0`, `THIN_STROKE = 2.0`), but meter's use of 80.0 and 6.0 is widget-specific.

**Total duplicates identified**: 15 distinct duplicate constants/values across 51+ widget references

### Non-Duplicate Constants (single use)

| Constant | Used By | Line |
|----------|---------|------|
| TITLE_SIZE | title | 91 |
| CAPTION_SIZE | caption | 111 |
| MICRO_SIZE | micro | 118 |
| FIGURE_SIZE | figure | 125 |
| Metrics::DEFAULT.gap_small | star_rating | 417 |
| Metrics::DEFAULT.row_height | segmented | 377 |
| TAG_HEIGHT | tag | 218 |
| FIELD_ROW_LABEL_WIDTH | field_row, field_group | 521, 542 |
| HEADING_TRACKING | heading | 103 |

**Note**: TAG_HEIGHT, FIELD_ROW_LABEL_WIDTH, and HEADING_TRACKING are widget-specific, not duplicates.

---

## Findings

### Widgets Using Theme Constants
✓ **Well-integrated** (9 widgets):
- title, heading, caption, micro, figure, code, panel, divider, button

✓ **Partially integrated** (7 widgets):
- field (uses CODE_SIZE + Metrics)
- tag (uses TAG_HEIGHT + Metrics + HEADING_SIZE)
- tabs (uses Metrics only)
- segmented (uses Metrics only)
- star_rating (uses Metrics only)
- section (uses Metrics only)
- field_group (uses FIELD_ROW_LABEL_WIDTH + Metrics)

### Widgets with Hardcoded Literals (6 widgets)
These 6 widgets have widget-specific dimensions marked UNMATCHED (not Metrics duplicates):
1. **meter**: 80.0 (width), 6.0 (height) — intrinsic size specific to meter appearance
2. **tabs**: 12.5 (text), 26.0 (row height), 2.0 (indicator) — tab-specific sizing
3. **segmented**: 12.0 (text), 3.0 (padding), 2.0 (gap) — segmented control spacing
4. **star_rating**: 16.0×16.0 (star icon) — icon-specific size
5. **section**: 14.0 (header height) — section label specific
6. **scrollbar**: 12.0 (width) — scrollbar thumb width specific

These are intentional widget customizations, not duplicates of Metrics values.

### Hardcoded Numeric Patterns

Three patterns emerge from hardcoded numeric analysis:

1. **Generic stroke/offset values** (1.0, 2.0, 0.0):
   - 1.0 appears 8 times (button, field, meter, panel, scrollbar, segmented, star_rating)
   - 2.0 appears 8 times (dot, meter, segmented, tabs)
   - These could benefit from constants like `HAIRLINE_WIDTH`, `THIN_STROKE`

2. **Widget-specific sizing** (12.0, 14.0, 16.0, 26.0, 80.0):
   - 12.0 (scrollbar width, segmented text)
   - 14.0 (section header height)
   - 16.0 (star icon size)
   - 26.0 (tab row height)
   - 80.0 (meter bar width)
   - These are intentional widget customizations, marked UNMATCHED

3. **Layout spacing** (20.0, gaps):
   - 20.0 appears 2 times (field_row, field_group label width)
   - Could be extracted as `LABEL_COLUMN_WIDTH`

### Unused Constants
- **BODY_SIZE** (13.0) — Defined as reference but not used by any widget

## Cross-References
- All 27 functions verified via `grep -n "^pub fn \|^fn " src/widgets.rs`
- STEP 3 identified inline UNMATCHED comments for all hardcoded literals
- STEP 3 tests use Harness-based behavior validation instead of brittle line-number matching

## Acceptance Gate Checklist
- ✓ All 27 widget functions documented
- ✓ Constants mapped to each widget
- ✓ Hardcoded literals identified and marked
- ✓ Cross-reference to STEP 3 inline comments verified
- ✓ grep count validation passes

**STEP 4 COMPLETE** — Ready for STEP 5 (if any further analysis needed)
