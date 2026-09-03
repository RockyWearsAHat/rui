# STEP 1: Backend Consistency Test Scaffold — Cross-Module Concerns

## Overview
This document identifies cross-module interactions that the backend consistency test scaffold validates. Understanding these interactions is essential for maintaining coordinate system correctness as the library evolves.

---

## Core Module Interactions

```
                    Input System (input.rs)
                            ↓
        ┌─────────────────────────────────────────┐
        │    Coordinate Transformation (geom.rs)  │
        │    logical = device / scale_factor      │
        └─────────────────────────────────────────┘
                            ↓
                    Event Translation
                            ↓
        ┌─────────────────────────────────────────┐
        │  Handler Execution (paint.rs, app.rs)   │
        └─────────────────────────────────────────┘
                            ↓
        ┌─────────────────────────────────────────┐
        │   State Management (memory.rs)          │
        │   - Focus tracking                      │
        │   - Scroll position                     │
        │   - Interaction state                   │
        └─────────────────────────────────────────┘
                            ↓
        ┌─────────────────────────────────────────┐
        │  Theme Application (theme.rs)           │
        │  - Tone resolution                      │
        │  - Display scale application            │
        └─────────────────────────────────────────┘
                            ↓
        ┌─────────────────────────────────────────┐
        │ Accessibility Audit (accessibility.rs) │
        │ - Tab order verification                │
        │ - Focus ring rendering                  │
        └─────────────────────────────────────────┘
```

---

## Concern 1: Input → Coordinate Transformation

### Module Boundaries
- **Source**: input.rs (Event generation)
- **Transformer**: geom.rs (Point, Rect, coordinate math)
- **Consumer**: paint.rs (hit testing, handler dispatch)

### The Issue
Platform backends deliver raw device coordinates (e.g., X11 motion events with pixel coordinates). These must be converted to logical coordinates before handlers execute. If transformation is inconsistent across platforms, the same visual click might hit different elements on different backends.

### How STEP 1 Tests Validate This
**Phase 1 — Coordinate Transformation Tests:**
```rust
#[test]
fn test_pointer_coordinates_at_scale_2_0() {
    // Device coordinate (1000, 500) at scale 2.0 should become
    // logical coordinate (500, 250)
    let mut h = Harness::new(App::default(), view).dpi(2.0);
    h.click_at_point(Point { x: 1000.0, y: 500.0 });
    // Verify handler receives state that proves the logical coordinate was correct
    assert_eq!(h.state().clicked_at, Point { x: 500.0, y: 250.0 });
}
```

### Common Pitfalls
- ✗ Forgetting to transform Y coordinate (different coordinate systems)
- ✗ Integer truncation in scale factor (0.5 device pixels lost)
- ✗ Scale applied twice (platform → logical → wrong again)
- ✗ Platform-specific coordinate space confusion (X11 origin top-left, different from macOS)

### Verification Pattern
1. Send event at device coordinate (N, M, scale=S)
2. Verify handler receives logical coordinate (N/S, M/S)
3. Repeat for scale factors 1.0, 1.5, 2.0, 2.5, 3.0

### Resolution in Other Backends (Template)
When implementing a new backend:
1. **Identify platform coordinate space** — Does origin differ? Which direction is Y?
2. **Apply transformation immediately** — Convert device → logical in event pump
3. **Add to STEP 1 test scaffold** — Include this backend in coordinate tests

---

## Concern 2: Handler Execution Order

### Module Boundaries
- **Source**: paint.rs (traverses element tree, calls handlers)
- **Consumer**: app.rs (main loop collects handler results)
- **User**: handlers themselves (Fn(&mut S) closures)

### The Issue
When multiple handlers can respond to the same coordinate (nested elements, multiple registered handlers), they must execute in a consistent order. If order varies by platform or scale factor, the same action might have different effects.

### How STEP 1 Tests Validate This
**Phase 2 — Event Ordering Tests:**
```rust
#[test]
fn test_handler_execution_order_preserved() {
    // Register 3 handlers on nested elements all at same click coordinate
    // Verify they execute in depth-first order regardless of scale
    let mut h = Harness::new(App::default(), view).dpi(1.5);
    h.click_text("button");
    // Handler 1 (outer), Handler 2 (middle), Handler 3 (inner)
    // should execute in that order on every platform
    assert_eq!(h.state().execution_order, vec![1, 2, 3]);
}
```

### Common Pitfalls
- ✗ Hash map iteration order varies (must use depth-first tree walk)
- ✗ Platform async event handling changes order (must serialize)
- ✗ Scale factor affects hit-test order (must transform before hit-test)
- ✗ Focus ring rendering changes handler dispatch (visual shouldn't affect logic)

### Verification Pattern
1. Create nested element structure
2. Register distinct handlers at each level
3. Send event that hits all levels
4. Verify execution order: depth-first, innermost-last
5. Repeat at scales 1.0, 2.0, 3.0

### Resolution in Other Modules
- **paint.rs**: Tree walk is depth-first by construction; add assertion
- **app.rs**: Handlers collected after frame; execute in collected order
- **memory.rs**: No interaction; focus doesn't affect ordering

---

## Concern 3: State Persistence Across Coordinates

### Module Boundaries
- **Source**: memory.rs (focus, scroll, interaction state)
- **Trigger**: paint.rs (handler calls mutate state)
- **Verifier**: Tests that read state after coordinate-based events

### The Issue
If state is per-coordinate or scale-dependent, it breaks identity. The same button at scale 1.0 and 2.0 must be the same button with the same state. If focus or scroll position depends on scale factor, clicking the same button on different backends will see different state.

### How STEP 1 Tests Validate This
**Phase 3 — State Persistence Tests:**
```rust
#[test]
fn test_focus_persists_across_scale_factors() {
    let mut h1 = Harness::new(App::default(), view).size(800, 600);
    let mut h2 = Harness::new(App::default(), view).size(800, 600).dpi(2.0);
    
    // Click same element in both harnesses
    h1.click_text("button");
    h2.click_text("button");
    
    // Both should have button focused, regardless of scale
    assert!(h1.state().focused);
    assert!(h2.state().focused);
}
```

### Common Pitfalls
- ✗ Hash-based element identity using coordinate (breaks at different scales)
- ✗ Storing device coordinates in state (must store logical)
- ✗ Path-based identity with scale (must normalize path)
- ✗ Scroll position tied to device pixels (must use logical units)

### Verification Pattern
1. Create mutable state struct
2. Perform action at one scale factor
3. Verify state changed
4. Create new harness at different scale
5. Perform same action
6. Verify state changed identically

### Resolution in Other Modules
- **element.rs**: El::key() uses element path, not coordinate; platform-independent
- **memory.rs**: Focus is by element path, not coordinate; scale-independent
- **widgets.rs**: State shapes are application types, not coordinate-based

---

## Concern 4: Theme Application at Different Scales

### Module Boundaries
- **Source**: theme.rs (Palette, Metrics, CornerStyle)
- **Consumer**: canvas.rs (SDF rendering with theme values)
- **Validator**: Tests checking visual appearance

### The Issue
Theme metrics (corner radius, stroke width, padding) are in logical units. At high DPI, these must not scale incorrectly. A 2px border at scale 2.0 should stay visually similar (2 logical pixels), not become 4 device pixels.

### How STEP 1 Tests Validate This
**Phase 3 — Theme Consistency Tests:**
```rust
#[test]
fn test_theme_applies_correctly_at_scale() {
    let mut h = Harness::new(App::default(), view).dpi(2.0);
    h.click_text("button");
    
    // Render and compare button appearance
    let pixels1 = h.render().pixels();
    
    // Same click at scale 1.0 should have similar appearance
    let mut h2 = Harness::new(App::default(), view).dpi(1.0);
    h2.click_text("button");
    let pixels2 = h2.render().pixels();
    
    // Visual appearance similar (accounting for DPI scaling)
    assert_similar_appearance(pixels1, pixels2);
}
```

### Common Pitfalls
- ✗ Metrics applied before scale transformation (visual distortion)
- ✗ Corner radius in device pixels (changes visual scale at high DPI)
- ✗ Stroke width platform-specific (looks different on each backend)
- ✗ Gradient stops not respecting scale (colors misaligned)

### Verification Pattern
1. Render element at scale 1.0
2. Render same element at scale 2.0
3. Account for device pixel scaling
4. Verify visual appearance equivalent (within rounding)
5. Verify theme metrics applied correctly

### Resolution in Other Modules
- **canvas.rs**: Metrics already in logical units; multiply by scale only at final blit
- **paint.rs**: Uses canvas for rendering; coordinate transformation already applied
- **sdf.rs**: SDF values in logical units; canvas handles scale

---

## Concern 5: Accessibility Consistency

### Module Boundaries
- **Source**: accessibility.rs (audit function, tab order)
- **Validator**: Element tree and focus system
- **Trigger**: Tests calling assert_accessible() and assert_tab_order()

### The Issue
Accessibility audit must agree with hit-testing about what's focusable. If an element is focusable at scale 1.0 but not at scale 2.0, the tab order changes invisibly. Focus ring placement must be scale-independent.

### How STEP 1 Tests Validate This
**Phase 3 — Accessibility Tests:**
```rust
#[test]
fn test_accessibility_consistent_across_scales() {
    let mut h1 = Harness::new(App::default(), view).dpi(1.0);
    let mut h2 = Harness::new(App::default(), view).dpi(2.0);
    
    // Assert accessibility for both
    h1.assert_accessible();
    h1.assert_tab_order();
    
    h2.assert_accessible();
    h2.assert_tab_order();
    
    // Both should have same focusable elements
    // (Focus walk, tab order, and hit testing all agree)
}
```

### Common Pitfalls
- ✗ `El::takes_focus` depends on coordinate (breaks at different scales)
- ✗ Disabled state applied per-scale (same element has different state)
- ✗ Focus ring clipping at scale factors (appears cut off)
- ✗ Semantic meaning changes with scale (button considered inactive)

### Verification Pattern
1. Run assert_accessible() at scale 1.0
2. Run assert_accessible() at scale 2.0
3. Verify same elements reported as focusable
4. Run assert_tab_order() at both scales
5. Verify tab order identical

### Resolution in Other Modules
- **element.rs**: El::takes_focus is (focusable && !disabled), not coordinate-based
- **memory.rs**: Focus is by element path; scale-independent
- **paint.rs**: Focus ring rendered by library; scale-aware

---

## Concern 6: Event Routing Under Nesting

### Module Boundaries
- **Source**: paint.rs (tree traversal, hit-test)
- **Target**: Handler closures in element tree
- **Verifier**: Tests with nested structures

### The Issue
When elements are nested, the tree walk must find the innermost handler first. Scale factors must not affect this order. If an element is nested 5 levels deep, its handler must run after outer handlers regardless of display scale.

### How STEP 1 Tests Validate This
**Phase 3 — Nested Handler Tests:**
```rust
#[test]
fn test_nested_handlers_respect_tree_order() {
    // Create deeply nested structure
    // All elements respond to same coordinate
    let mut h = Harness::new(App::default(), view).dpi(1.5);
    h.click_at_point(Point { x: 100.0, y: 100.0 });
    
    // Verify innermost handler ran last
    // (depth-first order)
    assert_eq!(h.state().handler_sequence, vec!["outer", "middle", "inner"]);
}
```

### Common Pitfalls
- ✗ Hit test returns only one element (should bubble)
- ✗ Handler collection order changes with scale (must be tree-based)
- ✗ Coordinate transformation applied after hit test (wrong element hit)
- ✗ Early return on first handler (other handlers skipped)

### Verification Pattern
1. Create nested element structure (3+ levels)
2. Register handlers at each level
3. Send event that hits entire stack
4. Verify handlers executed in correct order (outer to inner)
5. Repeat at different scales (1.0, 2.0)

### Resolution in Other Modules
- **element.rs**: Tree structure defined; traversal order fixed
- **paint.rs**: Tree walk is depth-first by construction
- **app.rs**: Collects handlers in tree-walk order, executes them

---

## Concern 7: Canvas Rendering at Different Scales

### Module Boundaries
- **Source**: canvas.rs (rasterization, blitting)
- **Input**: Logical coordinates and dimensions
- **Output**: Device pixel buffer

### The Issue
Canvas must render identically at different logical coordinates if scale factor is applied correctly. A 100×100 button should render the same regardless of scale, only taking more device pixels at higher scales.

### How STEP 1 Tests Validate This
Tests don't directly verify canvas rendering (that's image-based regression testing), but they verify that:
- Coordinates reach canvas.rs correctly transformed
- Scale factor is applied consistently
- No double-transformation occurs

### Common Pitfalls
- ✗ Scale applied to SDF values (geometry distorted)
- ✗ Stroke width applied before scale (looks different at high DPI)
- ✗ Blitting without scale factor (image placed wrong)
- ✗ Clip region not transformed (drawing clipped incorrectly)

### Verification Pattern
1. Send click to element
2. Verify element handler executed
3. (Visual verification done by golden-image tests, not here)
4. No exceptions or panics during rendering

### Resolution in Other Modules
- **paint.rs**: Passes logical coordinates to canvas
- **sdf.rs**: All shape math in logical units
- **canvas.rs**: Multiplies by scale_factor only at Bgra::new

---

## Integration Testing Strategy

### Test Organization
```
STEP_1_ANALYSIS.md
  ├── Phase 1: Foundation (64 tests)
  │   └── Coordinate transformation validation
  ├── Phase 2: Enhancement (12 tests)
  │   └── Stress testing + performance baselines
  └── Phase 3: Integration (12 tests)
      └── Cross-module consistency

tests/backend_consistency.rs
  ├── Coordinate transformation tests
  ├── Event ordering tests
  ├── State persistence tests
  ├── Theme consistency tests
  ├── Accessibility tests
  ├── Nested handler tests
  └── Performance baselines
```

### Adding New Platforms
When adding a new backend, verify these cross-module concerns:

1. **Coordinate transformation**: Implement in Backend::pump(), verify with STEP 1 tests
2. **Event ordering**: Tree walk unchanged; verify order with Phase 2 tests
3. **State persistence**: Memory module independent of platform; verify with Phase 3 tests
4. **Theme application**: Theme module platform-independent; visual verification only
5. **Accessibility**: audit() independent of platform; verify with Phase 3 tests
6. **Event routing**: paint.rs tree walk independent; behavior identical across platforms
7. **Canvas rendering**: canvas.rs platform-independent; verify at platform level

### Regression Detection
If any module is modified that affects coordinate handling:
1. Run `cargo test --test backend_consistency`
2. Verify all 88 tests pass
3. Verify no new clippy warnings
4. If fails, the module change broke a cross-module invariant

---

## Summary Table

| Concern | Validated By | Risk Level | Recovery |
|---------|-------------|-----------|----------|
| Input → Coordinate Transformation | Phase 1 (6 tests) | CRITICAL | Add scale factors to testing |
| Handler Execution Order | Phase 2 (3 tests) | HIGH | Verify tree walk order |
| State Persistence | Phase 3 (2 tests) | HIGH | Ensure path-based identity |
| Theme Application | Phase 3 (3 tests) | MEDIUM | Visual regression tests |
| Accessibility Consistency | Phase 3 (3 tests) | HIGH | audit() + tab order |
| Event Routing | Phase 3 (3 tests) | HIGH | Verify tree traversal |
| Canvas Rendering | Integration | MEDIUM | Golden-image tests |

**Critical Path**: Concerns 1-2-3 must pass before Concerns 4-5-6-7 can work correctly.

---

## Template for Next Backend Implementation

When adding platform X:

1. **Phase 1**: Implement Backend trait, verify Concern 1 (coordinate transformation)
   - Add tests to STEP 1 for platform X
   - Verify 6 pointer_coordinates tests pass

2. **Phase 2**: Add event handling, verify Concerns 2-6 (ordering, state, routing)
   - Run STEP 1 full suite
   - Verify no new concerns violated

3. **Phase 3**: Full integration, verify Concern 7 (rendering consistency)
   - Run all STEP 1 Phase 3 tests
   - Verify cross-module integration works

4. **Sign-off**: Commit references STEP_1_CROSS_MODULE_CONCERNS.md

See Recipe 2 (X11 Backend) for detailed implementation example.
