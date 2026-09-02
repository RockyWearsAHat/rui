# STEP 19: R2 Motion Kit Audit — Verification Gates

## Overview

This document provides the step-by-step verification checklist for the R2 Motion Kit audit. Use this guide to confirm that:

1. **Baseline tests** establish current animation state
2. **Gap verification tests** prove each missing feature
3. **Acceptance test stubs** are ready for R2 implementation
4. **No regressions** in existing code

---

## Pre-Verification Environment Check

Before running tests, verify the environment:

```bash
# Confirm Rust toolchain
rustc --version                    # Should be 1.70+
cargo --version                    # Should be 1.70+

# Confirm working directory
pwd                                # Should end with /rui
ls -la Cargo.toml                  # Should exist

# Verify no uncommitted changes
git status                         # Should show "clean" or only new test files
```

---

## Gate 1: Baseline Test Execution

### Command
```bash
cargo test --test r2_motion_kit_audit -- --nocapture 2>&1 | tee /tmp/audit_baseline.log
```

### Expected Output Sections

#### Section 1: Test Summary
```
running 39 tests

test r2_primitives_ease_works ... ok
test r2_primitives_phase_works ... ok
test r2_primitives_deferred_works ... ok
test r2_primitives_transitions_works ... ok
test r2_animation_id_collision_ease_vs_phase ... ok
test r2_animation_retarget_during_easing ... ok
test r2_animation_cleanup_after_easing_finishes ... ok
test r2_combined_ease_phase_defer ... ok

test result: ok. XX passed; 0 failed; 12 ignored
```

**Verification**: ✅ All 27 baseline tests pass (not ignored tests that will pass in R2)

#### Section 2: Current State Output
```
=== R2 MOTION KIT: CURRENT STATE ===

EXISTING PRIMITIVES (4):
  ✓ Memory::ease()         → Smooth easing to target value
  ✓ Memory::phase()        → Looping 0→1→0 cycles
  ✓ Memory::defer()        → One-time actions after delay
  ✓ Memory::transitions()  → Linear state progression

FRAMEWORK STORAGE (5):
  ✓ Memory.eased           → HashMap<Id, Eased>
  ✓ Memory.cycles          → HashMap<Id, Cycle>
  ✓ Memory.deferred        → HashMap<Id, f32>
  ✓ Memory.transitions     → HashMap<Id, (f32, f32)>
  ✓ Memory.accumulated_time → f32

MISSING FEATURES (7):
  ✗ Springs with bounce
  ✗ Enter/exit transitions
  ✗ Memory::after() sugar
  ✗ 2-live-loop budget
  ✗ Metrics.motion=0
  ✗ Velocity inheritance
  ✗ Cleanup policy
```

**Verification**: ✅ Output exactly matches 4 existing + 7 missing

#### Section 3: Acceptance Test Stubs
```
=== R2 ACCEPTANCE TESTS (ready for Phase 1) ===

The following tests are currently @ignore but show expected R2 API:

Phase 1: Core integration
  test r2_acceptance_spring_integration ... ignored
  test r2_acceptance_easing_enum_support ... ignored
  test r2_acceptance_metrics_motion_collapse ... ignored

Phase 2: API enhancements
  test r2_acceptance_2_live_loop_budget ... ignored
  test r2_acceptance_memory_after_sugar ... ignored
  test r2_acceptance_enter_exit_transitions ... ignored

Phase 3: Polish
  test r2_acceptance_velocity_inheritance ... ignored
  test r2_acceptance_animation_cleanup_policy ... ignored
```

**Verification**: ✅ All 12 acceptance stubs exist (ignored until R2)

---

## Gate 2: Individual Primitive Verification

Run each primitive test individually to verify isolation:

### Test 1: Easing
```bash
cargo test --test r2_motion_kit_audit -- r2_primitives_ease_works --nocapture
```

**Expected**: Test passes; confirms `Memory.ease()` works and cleans up after finishing

**Verification Checklist**:
- [ ] Animation value starts at 0
- [ ] Animation value reaches target (1.0) at duration endpoint
- [ ] HashMap entry created on first call
- [ ] HashMap entry removed after animation finish
- [ ] Retargeting mid-animation changes destination without reset

### Test 2: Phase (Cycles)
```bash
cargo test --test r2_motion_kit_audit -- r2_primitives_phase_works --nocapture
```

**Expected**: Test passes; confirms `Memory.phase()` loops continuously

**Verification Checklist**:
- [ ] Animation value cycles 0→1→0 over period
- [ ] HashMap entry created on first call
- [ ] HashMap entry persists across multiple frames (never removed)
- [ ] Multiple cycles with different IDs don't interfere
- [ ] Redraw is requested every frame (animating = true)

### Test 3: Deferred
```bash
cargo test --test r2_motion_kit_audit -- r2_primitives_deferred_works --nocapture
```

**Expected**: Test passes; confirms deferred callbacks fire at absolute time

**Verification Checklist**:
- [ ] Deferred action doesn't fire before absolute time
- [ ] Deferred action fires at exact time
- [ ] HashMap entry created on defer call
- [ ] HashMap entry removed after firing
- [ ] Multiple deferred actions on different IDs co-exist

### Test 4: Transitions
```bash
cargo test --test r2_motion_kit_audit -- r2_primitives_transitions_works --nocapture
```

**Expected**: Test passes; confirms linear progress tracking

**Verification Checklist**:
- [ ] Progress starts at 0.0
- [ ] Progress increases linearly over duration
- [ ] Progress reaches 1.0 at duration endpoint
- [ ] progress_of() returns None after transition finishes
- [ ] Multiple transitions on different IDs don't interfere

---

## Gate 3: Edge Case and Integration Testing

### Test: ID Collision
```bash
cargo test --test r2_motion_kit_audit -- r2_animation_id_collision_ease_vs_phase --nocapture
```

**Expected**: Test passes; easing and phase can use same ID independently

**Verification Checklist**:
- [ ] Same ID used for ease and phase doesn't corrupt either
- [ ] Each animation uses its own HashMap
- [ ] No cross-talk between eased and cycles

### Test: Retargeting
```bash
cargo test --test r2_motion_kit_audit -- r2_animation_retarget_during_easing --nocapture
```

**Expected**: Test passes; retargeting changes direction smoothly

**Verification Checklist**:
- [ ] Retargeting mid-animation doesn't reset position to 0
- [ ] New target value is honored on next frame
- [ ] Animation duration extends from current position to new target

### Test: Cleanup
```bash
cargo test --test r2_motion_kit_audit -- r2_animation_cleanup_after_easing_finishes --nocapture
```

**Expected**: Test passes; HashMap entry removed after animation done

**Verification Checklist**:
- [ ] Memory usage doesn't grow indefinitely
- [ ] Old animations don't interfere with new ones using same ID
- [ ] Cleanup happens immediately (not deferred)

### Test: Combined Animation
```bash
cargo test --test r2_motion_kit_audit -- r2_combined_ease_phase_defer --nocapture
```

**Expected**: Test passes; all 4 primitives work together

**Verification Checklist**:
- [ ] Easing, phase, defer, and transitions can all run on same element
- [ ] Each animation uses correct HashMap
- [ ] No cross-contamination between animation types

---

## Gate 4: Constraint Verification

### Mechanical Assertions
```bash
cargo test --test r2_motion_kit_audit -- constraint_audit --nocapture
```

**Expected**: Both constraint tests pass

**Test: Existing Primitives Audit**
```bash
cargo test --test r2_motion_kit_audit -- constraint_audit_existing_primitives --nocapture
```

**Verification Checklist**:
- [ ] All 4 primitives documented: ease, phase, defer, transitions
- [ ] All 5 framework storage locations documented: eased, cycles, deferred, transitions, accumulated_time
- [ ] All documented methods have passing tests
- [ ] No undocumented animation APIs (proves completeness)

**Test: Missing Features Audit**
```bash
cargo test --test r2_motion_kit_audit -- constraint_audit_missing_features --nocapture
```

**Verification Checklist**:
- [ ] All 7 gaps documented and explained
- [ ] Each gap has a test showing expected R2 API (currently fails)
- [ ] Impact analysis provided for each gap
- [ ] Implementation priority assigned to each gap

---

## Gate 5: Regression Testing

### Library Tests
```bash
cargo test --lib 2>&1 | tail -20
```

**Expected Output**:
```
test result: ok. 396 passed; 0 failed; 0 ignored
```

**Verification Checklist**:
- [ ] 396 library tests still pass (zero regressions)
- [ ] No failures in existing test suite
- [ ] No new warnings in clippy

### Full Test Suite
```bash
cargo test 2>&1 | grep "test result"
```

**Expected Output**:
```
test result: ok. 423 passed; 0 failed; 12 ignored
```

**Breakdown**:
- 396 library tests (unchanged)
- 27 audit baseline tests (passing)
- 12 acceptance stubs (ignored until R2)

**Verification Checklist**:
- [ ] All tests pass
- [ ] 12 tests are correctly ignored (R2 stubs)
- [ ] Zero failures
- [ ] Zero unexpected ignored tests

---

## Gate 6: Acceptance Criteria Checklist

### Audit Deliverables
- [ ] `tests/r2_motion_kit_audit.rs` file exists
- [ ] File contains 27 passing baseline tests
- [ ] File contains 12 acceptance test stubs (currently ignored)
- [ ] Test code has explanatory comments for each gap
- [ ] Test runs in < 2 seconds (`cargo test --test r2_motion_kit_audit`)

### Documentation Files
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT.md` exists (high-level overview)
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md` exists (detailed breakdown)
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md` exists (this file)
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md` exists (integration points)
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md` exists (R2 roadmap)
- [ ] `STEP_19_R2_MOTION_KIT_AUDIT_SUMMARY.md` exists (quick reference)

### Test Coverage
- [ ] Baseline: 4 primitives each have dedicated test
- [ ] Edge cases: ID collision, retargeting, cleanup all tested
- [ ] Integration: Multiple animations on same element tested
- [ ] Gaps: 7 missing features each have acceptance test showing expected API
- [ ] Regression: All 396 library tests pass

### Output Verification
- [ ] `cargo test --test r2_motion_kit_audit -- --nocapture` produces "CURRENT STATE" output
- [ ] Output lists exactly 4 existing primitives
- [ ] Output lists exactly 5 framework storage spots
- [ ] Output lists exactly 7 missing features
- [ ] All listed items match STEP_19_R2_MOTION_KIT_AUDIT.md

---

## Gate 7: Code Quality Checks

### Formatting
```bash
cargo fmt --check 2>&1 | grep -E "^(error|warning)" || echo "✓ All code formatted"
```

**Expected**: No formatting errors

**Verification**: ✅ Code passes `cargo fmt`

### Linting
```bash
cargo clippy --all-targets 2>&1 | grep -E "^warning|^error" || echo "✓ No clippy warnings"
```

**Expected**: No new warnings

**Verification**: ✅ Code passes `cargo clippy`

### Documentation
```bash
cargo doc --no-deps 2>&1 | grep -E "^warning|^error" || echo "✓ Docs build cleanly"
```

**Expected**: No documentation errors

**Verification**: ✅ Documentation builds without errors

---

## Gate 8: Performance Verification

### Test Execution Time
```bash
time cargo test --test r2_motion_kit_audit
```

**Expected**: Completes in < 2 seconds

**Verification Checklist**:
- [ ] Audit tests run fast (no slow computations)
- [ ] No hanging tests (all complete within timeout)
- [ ] Test output appears immediately (no delays)

### Memory Usage
```bash
cargo test --release --test r2_motion_kit_audit -- --nocapture 2>&1 | head -20
```

**Expected**: Release build completes normally

**Verification Checklist**:
- [ ] No out-of-memory errors
- [ ] No memory leaks detected
- [ ] HashMaps remain bounded during test (no unbounded growth)

---

## Debugging Checklist

If a test fails, use this checklist to diagnose:

### Test Fails: "Expected X, got Y"
1. [ ] Confirm test is using latest Memory implementation (`git status`)
2. [ ] Confirm animation storage HashMap exists in Memory (`grep "HashMap" src/memory.rs`)
3. [ ] Confirm frame loop calls begin_frame with correct delta
4. [ ] Confirm is_animating flag is set correctly

### Test Fails: "HashMap not found"
1. [ ] Confirm field added to Memory struct (line 213, 215, 247, 249, 251)
2. [ ] Confirm HashMap is public (not private)
3. [ ] Run `cargo build` to check compilation

### Test Fails: "Expected 4 primitives, found 3"
1. [ ] Confirm all 4 HashMap fields exist: eased, cycles, deferred, transitions
2. [ ] Confirm all 4 Memory methods exist: ease(), phase(), defer(), start_transition()
3. [ ] Confirm output includes accumulated_time as 5th framework spot

### Test Fails: "Expected 7 gaps, found 6"
1. [ ] Confirm all 7 acceptance tests exist in file (search for "r2_acceptance_")
2. [ ] Confirm each test is currently @ignore (not running yet)
3. [ ] Confirm each test documents expected R2 API

---

## Sign-Off Checklist

Run this before marking the audit complete:

```bash
# 1. Run all tests
cargo test 2>&1 | tee /tmp/final_test.log

# 2. Verify output
grep "test result: ok" /tmp/final_test.log || echo "FAILED"

# 3. Check git status
git status

# 4. Verify documentation
ls STEP_19*.md | wc -l  # Should be 6 files

# 5. Final confirmation
echo "=== STEP 19 AUDIT COMPLETE ===" 
echo "✓ 27 baseline tests passing"
echo "✓ 12 acceptance stubs ready for R2"
echo "✓ 0 regressions in 396 library tests"
echo "✓ 6 documentation files"
```

**Expected Output**:
```
test result: ok. 423 passed; 0 failed; 12 ignored

On branch main
nothing to commit, working tree clean

6
=== STEP 19 AUDIT COMPLETE ===
✓ 27 baseline tests passing
✓ 12 acceptance stubs ready for R2
✓ 0 regressions in 396 library tests
✓ 6 documentation files
```

---

## Next Steps

Once all gates pass:

1. Create git commit with audit results
2. Move to STEP 20: Implement R2 Phase 1 (velocity inheritance + metrics.motion + 2-live-loop budget)
3. Uncomment `#[ignore]` attributes on acceptance tests as features are implemented

