# STEP 19: R2 Motion Kit Audit — Quick Reference

## One-Sentence Summary

STEP 19 audits the current animation system, documents 4 working primitives and 5 framework storage spots, identifies 7 implementation gaps, and provides a 3-phase roadmap with test stubs for R2 Motion Kit.

---

## What's Working (4 Primitives)

| Primitive | Code | Use Case |
|-----------|------|----------|
| **Easing** | `painter.ease("key", target, seconds)` | Smooth value transitions |
| **Phase** | `painter.phase("key", period)` | Looping 0→1→0 cycles |
| **Defer** | `memory.defer(id, seconds)` | One-time delayed actions |
| **Transitions** | `memory.start_transition(id, duration)` | Linear state progression |

---

## What's Needed (7 Gaps)

| Gap | Impact | Complexity | Phase |
|-----|--------|------------|-------|
| **Springs with bounce** | Elastic interactions | Moderate | 2 |
| **Enter/exit transitions** | Choreography | Low | 2 |
| **Memory::after()** | Auto-dismiss, cascading | Low | 3 |
| **2-live-loop budget** | Animation safety | Very Low | 1 |
| **Metrics.motion=0** | Accessibility | Very Low | 1 |
| **Velocity inheritance** | Smooth retargeting | Low | 1 |
| **Cleanup policy** | Memory safety | Moderate | 3 |

---

## Test Files

**Main Test File**: `tests/r2_motion_kit_audit.rs`
- 27 baseline tests (passing) — confirm current state works
- 12 acceptance stubs (ignored) — R2 feature tests, ready to uncomment

**Run Tests**:
```bash
cargo test --test r2_motion_kit_audit -- --nocapture
# Shows: 27 passing, 12 ignored
# Output: CURRENT STATE with 4 primitives + 7 gaps
```

---

## Documentation Files

| File | Purpose | Audience |
|------|---------|----------|
| **STEP_19_R2_MOTION_KIT_AUDIT.md** | High-level overview | Everyone (start here) |
| **STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md** | Detailed breakdown | Implementers |
| **STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md** | Testing checklist | QA, reviewers |
| **STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md** | Integration points | Architects |
| **STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md** | Phase-by-phase code changes | Developers |
| **STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md** | This file | Quick lookup |

---

## Key Findings

### What's in Memory Right Now

```rust
struct Memory {
    eased: HashMap<Id, Eased>,              // Easing animations
    cycles: HashMap<Id, Cycle>,             // Looping animations
    deferred: HashMap<Id, f32>,             // Scheduled actions
    transitions: HashMap<Id, (f32, f32)>,   // State transitions
    accumulated_time: f32,                  // Time accumulator
    animating: bool,                        // Request redraw?
}
```

### What's Missing

1. `springs: HashMap<Id, Spring>` ← Physics-based animations
2. `callbacks: HashMap<Id, (f32, Box<dyn Fn()>)>` ← Callback scheduling
3. Phase tracking in transitions (enter/live/exit)
4. Velocity field in Eased (for smooth retargeting)
5. Metrics.motion check in all animation methods
6. 2-live-loop budget enforcement
7. Explicit cleanup documentation

---

## Implementation Order

**Phase 1 (Foundation)**: 3 quick wins, no dependencies
- Gap 5: Metrics.motion=0 (accessibility)
- Gap 6: Velocity inheritance (smooth feel)
- Gap 4: 2-live-loop budget (safety)

**Phase 2 (Core)**: High-impact features, depends on Phase 1
- Gap 1: Springs with bounce (organic motion)
- Gap 2: Enter/exit transitions (choreography)

**Phase 3 (Polish)**: Quality-of-life features, depends on 1+2
- Gap 3: Memory::after() sugar (auto-dismiss)
- Gap 7: Cleanup policy (robustness)

---

## Code Locations (Quick Lookup)

| What | Where | Line |
|------|-------|------|
| Painter::ease | src/paint.rs | 147–153 |
| Memory::ease | src/memory.rs | 259–285 |
| Painter::phase | src/paint.rs | 155–178 |
| Memory::phase | src/memory.rs | 287–310 |
| Memory.eased field | src/memory.rs | 213 |
| Memory.cycles field | src/memory.rs | 215 |
| Memory.deferred field | src/memory.rs | 247 |
| Memory.transitions field | src/memory.rs | 251 |
| begin_frame (cleanup) | src/memory.rs | 325+ |

---

## How to Read the Docs

### I'm new to this project
1. Read this summary (2 min)
2. Read STEP_19_R2_MOTION_KIT_AUDIT.md (10 min)
3. Skim STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md (15 min)

### I'm implementing Phase 1
1. Read STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md (Phase 1 section)
2. Refer to STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md for integration points
3. Use STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md to verify each step

### I'm reviewing a PR
1. Read STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md (Gate checklist)
2. Run tests and verify output matches acceptance criteria
3. Check regression tests: `cargo test --lib` should show 396+ passing

### I'm debugging a test failure
1. Check test output against STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md (Debugging Checklist)
2. Consult STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md for interaction risks
3. Review STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md (specific gap section)

---

## Acceptance Criteria (STEP 19 Audit)

- [x] 4 existing primitives documented (ease, phase, defer, transitions)
- [x] 5 framework storage locations documented (eased, cycles, deferred, transitions, accumulated_time)
- [x] 7 missing features identified with impact analysis
- [x] 27 baseline tests passing (prove current state works)
- [x] 12 acceptance test stubs ready for R2 (currently ignored)
- [x] 0 regressions in 396 library tests
- [x] 6 comprehensive documentation files
- [x] 3-phase implementation roadmap provided

**Status**: ✅ COMPLETE

---

## Next Steps (After STEP 19)

### STEP 20: Phase 1 Implementation
Implement the 3 foundation gaps (metrics.motion, velocity, 2-live-loop budget)
- Duration: ~4 commits
- Uncomment 3 acceptance tests
- Expected result: All Phase 1 tests pass

### STEP 21: Phase 2 Implementation
Implement springs and enter/exit transitions
- Duration: ~5 commits
- Uncomment 2 acceptance tests
- Expected result: All Phase 2 tests pass

### STEP 22: Phase 3 Implementation
Implement Memory::after() and cleanup policy
- Duration: ~3 commits
- Uncomment 2 acceptance tests
- Expected result: All Phase 3 tests pass + all 12 acceptance tests pass

### STEP 23: R2 Documentation
Update CLAUDE.md with R2 recipes and patterns
- Duration: ~1 commit
- Add animation patterns to "Widget Exemplars"
- Update "Library Roadmap" to mark R2 as landed

---

## Testing Quick Start

### Run All Audit Tests
```bash
cargo test --test r2_motion_kit_audit
# Expected: 27 pass, 12 ignored
```

### Run Specific Baseline Test
```bash
cargo test --test r2_motion_kit_audit -- primitives_ease_works --nocapture
```

### Run Specific Acceptance Test (when Phase is implemented)
```bash
cargo test --test r2_motion_kit_audit -- r2_acceptance_spring_integration
```

### Check for Regressions
```bash
cargo test --lib 2>&1 | grep "test result"
# Expected: ok. 396 passed; 0 failed
```

### Run Full Suite
```bash
cargo test 2>&1 | tail -5
# Expected: ok. 423 passed (396 lib + 27 audit); 12 ignored
```

---

## Common Questions

### Q: Why are acceptance tests ignored?
A: They test R2 features not yet implemented. When each feature lands, uncomment the test.

### Q: Can I implement multiple gaps out of order?
A: No. Follow the order: Phase 1 (metrics + velocity + budget), then Phase 2 (springs + enter/exit), then Phase 3 (after + cleanup). Phase 2 depends on velocity from Phase 1.

### Q: What if a baseline test fails?
A: The baseline tests confirm current state. If one fails, something broke the existing animation system. Check git diff for recent changes to memory.rs or paint.rs.

### Q: Do I need to update CLAUDE.md?
A: After Phase 3, yes. Add R2 patterns to the Widget Exemplars and Library Roadmap sections.

### Q: How do I know if my implementation is done?
A: Run the phase's acceptance tests. If they all pass and regression tests show 0 failures, you're done. Create a commit with clear message referencing the gaps closed.

---

## Critical Constraints (Don't Break These)

1. **Time is injected, never read**: No `Instant::now()` calls
2. **Identity is path-based**: Animations keyed by tree position or El.key()
3. **View is pure**: No side effects during draw()
4. **Handlers run after frame**: Deferred execution model
5. **Metrics must be respected**: Metrics.motion=0 disables all animation

---

## Performance Budget

- Animation loop must be < 1ms per frame
- 2-live-loop budget prevents runaway cycles
- Spring solver must not allocate per frame
- Callback invocation must be O(n) where n = fired callbacks

---

## Performance Regression Check

```bash
# Before/after implementation, measure frame time
cargo run --release --example gallery
# Visual inspection: smooth at 60fps?
# Check system monitor: CPU < 10%?
```

---

## Contact & Questions

For questions during implementation:
1. Check this summary first (likely answered)
2. Consult STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md for deep dives
3. Review acceptance test stubs for usage examples
4. Check git log for similar patterns in other STEPs

---

## File Checklist

Before marking STEP 19 complete:

- [x] tests/r2_motion_kit_audit.rs exists (27 baseline + 12 acceptance)
- [x] STEP_19_R2_MOTION_KIT_AUDIT.md exists
- [x] STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md exists
- [x] STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md exists
- [x] STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md exists
- [x] STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md exists
- [x] STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md exists (this file)
- [x] All tests passing
- [x] No regressions

**Total**: 7 files, 1 test file, 100% test pass rate

---

## Sign-Off

STEP 19 Audit Complete ✅

- Animation system baseline established
- 7 gaps precisely identified with impact analysis
- 27 baseline tests passing (prove current state)
- 12 acceptance stubs ready (guide R2 implementation)
- 3-phase roadmap with code blueprints
- 0 regressions in 396 library tests
- Ready to begin STEP 20 (Phase 1 implementation)

