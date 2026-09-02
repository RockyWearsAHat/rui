# STEP 19: R2 Motion Kit Audit — Test Execution Runbook

**Document**: Complete guide to running, interpreting, and extending STEP 19 audit tests  
**Audience**: Implementers, QA, reviewers  
**Last Updated**: 2026-09-02

---

## Quick Start: Verify Current Animation System

```bash
# Run all audit baseline tests
cargo test --test recipes -- --nocapture a_animation 2>&1 | head -100

# Run all integration tests
cargo test --lib memory -- --nocapture 2>&1 | grep -A 5 "animation"

# Check test count
cargo test --test recipes 2>&1 | grep "test result:"
```

**Expected Output**:
```
test result: ok. 27 passed; 0 failed
```

---

## Test Execution by Phase

### Phase 1: Verify Current Animation Primitives

**Objective**: Confirm 4 working primitives exist (ease, phase, defer, transitions)

```bash
# Run baseline animation tests
cargo test --lib memory::animation -- --nocapture 2>&1 | head -200
```

**What to look for**:
- ✅ `test_ease_animation_updates_value_each_frame` — PASS
- ✅ `test_phase_animation_updates_cycles` — PASS
- ✅ `test_defer_delays_handler` — PASS
- ✅ `test_transition_animates_between_states` — PASS

**If any fail**: Check src/memory.rs lines 245–350 (animation primitives).

---

### Phase 2: Verify Framework Storage

**Objective**: Confirm 5 storage locations exist in Memory

```bash
# Check Memory struct fields
grep -n "eased\|cycles\|deferred\|transitions\|accumulated_time" src/memory.rs | head -20
```

**Expected**:
```
src/memory.rs:82:    eased: HashMap<u64, EasedValue>,          // Gap 3 easing
src/memory.rs:83:    cycles: HashMap<u64, u32>,                 // Gap 2 phase cycles
src/memory.rs:84:    deferred: Vec<DeferredHandler>,            // Gap 1 defer queue
src/memory.rs:85:    transitions: HashMap<u64, Transition>,     // Gap 4 element transitions
src/memory.rs:87:    accumulated_time: Duration,                // Gap 5 lifecycle time
```

**Verification command**:
```bash
cargo test --lib memory::tests::memory_structure -- --nocapture 2>&1
```

**If fields missing**: The audit is out of date — run Phase 3 re-validation.

---

### Phase 3: Verify 7 Gaps (Test Evidence)

**Objective**: Each gap has test evidence proving absence of feature

**Gap 1: Springs with bounce control**

```bash
# Verify Spring type doesn't exist
grep -c "struct Spring\|enum Spring" src/memory.rs
# Expected: 0

# Check test stub
grep -A 5 "spring_animation_applies_damping" tests/recipes.rs
```

**Test evidence**: `acceptance_spring_animation_applies_damping` is #[ignore] in tests/recipes.rs line 1247

---

**Gap 2: Enter/exit transitions**

```bash
# Verify EnterExit enum doesn't exist
grep -c "enum EnterExit\|struct EnterExit" src/widgets.rs
# Expected: 0

# Check test stub
grep -A 5 "enter_exit_animation_fires_on_lifecycle" tests/recipes.rs
```

**Test evidence**: `acceptance_element_enter_exit_animations` is #[ignore] in tests/recipes.rs line 1259

---

**Gap 3: Easing enum support**

```bash
# Verify Easing enum is incomplete
grep -A 10 "enum Easing" src/memory.rs | head -20
# Expected: Only Custom, no Linear/EaseInOut/etc.

# Check test stub
grep -A 5 "easing_enum_supports_multiple_curves" tests/recipes.rs
```

**Test evidence**: `acceptance_easing_enum_curves` is #[ignore] in tests/recipes.rs line 1271

---

**Gap 4: 2-live-loop budget enforcement**

```bash
# Verify budget check doesn't exist
grep -n "budget\|max.*animation\|2.*live" src/app.rs | head -10
# Expected: No results or only comments

# Check test stub
grep -A 5 "animation_budget_prevents_third_animation" tests/recipes.rs
```

**Test evidence**: `acceptance_animation_budget_enforcement` is #[ignore] in tests/recipes.rs line 1283

---

**Gap 5: Metrics.motion=0 accessibility collapse**

```bash
# Verify prefers-reduced-motion check doesn't exist
grep -n "prefers.*motion\|motion.*0" src/theme.rs
# Expected: No results

# Check test stub
grep -A 5 "accessibility_motion_disabled" tests/recipes.rs
```

**Test evidence**: `acceptance_accessibility_motion_preferences` is #[ignore] in tests/recipes.rs line 1295

---

**Gap 6: Velocity inheritance**

```bash
# Verify Velocity type doesn't exist
grep -c "struct Velocity\|type Velocity" src/memory.rs
# Expected: 0

# Check test stub
grep -A 5 "velocity_from_drag_animation" tests/recipes.rs
```

**Test evidence**: `acceptance_velocity_inheritance_from_drag` is #[ignore] in tests/recipes.rs line 1307

---

**Gap 7: Cleanup policy & Memory::after() sugar**

```bash
# Verify Memory::after() doesn't exist
grep -n "fn after\|pub after" src/memory.rs | head -10
# Expected: No results

# Check test stub
grep -A 5 "memory_after_sugar_defers_handler" tests/recipes.rs
```

**Test evidence**: `acceptance_memory_after_sugar` is #[ignore] in tests/recipes.rs line 1319

---

## Running All Audit Tests

```bash
# Full audit baseline suite (27 tests)
cargo test --test recipes -- --nocapture a_animation 2>&1
```

**Expected output structure**:
```
running 27 tests

test a_animation_ease_updates_value_each_frame ... ok
test a_animation_phase_updates_cycles ... ok
... [25 more tests] ...

test result: ok. 27 passed; 0 failed; 0 ignored
```

---

## Acceptance Test Stubs (Ready for Phase Implementation)

All 12 acceptance test stubs are located in `tests/recipes.rs` lines 1247–1331 and marked `#[ignore]`.

### Phase 1 Acceptance Tests (3 stubs — to activate in STEP 20)

```bash
# Currently ignored, will activate when implementing Phase 1
cargo test --test recipes -- --include-ignored acceleration_applies_metrics_motion 2>&1
cargo test --test recipes -- --include-ignored animation_budget_enforcement 2>&1
cargo test --test recipes -- --include-ignored accessibility_motion_preferences 2>&1
```

**Activation command** (STEP 20):
```bash
sed -i 's/#\[ignore\].*acceptance_phase_1//' tests/recipes.rs
```

### Phase 2 Acceptance Tests (2 stubs — to activate in STEP 21)

```bash
cargo test --test recipes -- --include-ignored spring_animation_applies_damping 2>&1
cargo test --test recipes -- --include-ignored element_enter_exit_animations 2>&1
```

### Phase 3 Acceptance Tests (2 stubs — to activate in STEP 22)

```bash
cargo test --test recipes -- --include-ignored easing_enum_curves 2>&1
cargo test --test recipes -- --include-ignored memory_after_sugar 2>&1
```

---

## Verification Gates Checklist

### Gate 1: Current State Documented

- [ ] Run `cargo test --test recipes -- a_animation` — all 27 pass
- [ ] Run `cargo test --lib memory` — zero regressions
- [ ] Verify 4 primitives in src/memory.rs lines 245–350
- [ ] Verify 5 storage fields in src/memory.rs lines 82–87
- [ ] **Expected result**: All baseline tests passing, current animation system fully documented

### Gate 2: Gaps Identified & Test Evidence Collected

- [ ] Run all 7 gap verification commands (see Phase 3 above)
- [ ] Confirm each gap returns 0 when checking for feature existence
- [ ] Verify each gap has test stub in tests/recipes.rs
- [ ] **Expected result**: All 7 gaps confirmed absent, all test stubs in place

### Gate 3: 3-Phase Roadmap Validated

- [ ] Read STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md Phase 1/2/3 sections
- [ ] Verify API Reference matches Analysis document
- [ ] Confirm implementation checklists align with verification gates
- [ ] **Expected result**: Implementation roadmap is clear and actionable

### Gate 4: No Regressions

- [ ] Run full test suite: `cargo test --lib`
- [ ] Verify result: `396 tests passing`
- [ ] Check git diff: `git status` shows clean (all committed)
- [ ] **Expected result**: Zero changes to library functionality

---

## Debugging Failed Tests

### Common Failure: "Animation primitives missing"

```bash
# If a_animation_ease_updates_value_each_frame fails:
grep -n "pub fn ease\|impl Memory" src/memory.rs | head -20
# Animation functions should exist at lines 245–350
```

**Fix**: Check that src/memory.rs was not accidentally modified.

### Common Failure: "Storage fields moved"

```bash
# If memory structure test fails:
grep -n "struct Memory" src/memory.rs
# Inspect the struct definition
cargo test --lib memory::tests::memory_structure -- --nocapture
```

**Fix**: Verify Memory fields have not been renamed or removed.

### Common Failure: "Acceptance tests not found"

```bash
# If test stub lookup fails:
grep -c "acceptance_spring_animation_applies_damping" tests/recipes.rs
# Should return: 1
```

**Fix**: Ensure tests/recipes.rs was written correctly (check last commit).

---

## Extending the Audit

### Adding a New Baseline Test

```rust
// In tests/recipes.rs, add a new test following this pattern:

#[test]
fn a_animation_new_primitive_works() {
    let mut h = Harness::new(App { value: 0.0 }, view);
    
    // Verify the new primitive animates correctly
    h.state_mut().animate(/* params */);
    h.frames(5);
    
    assert!(h.state().value > 0.0);
}
```

Then run:
```bash
cargo test --test recipes new_primitive_works
```

### Adding New Acceptance Test Stub

```rust
// In tests/recipes.rs, add a stub for a new gap:

#[test]
#[ignore]
fn acceptance_new_r2_feature_works() {
    let mut h = Harness::new(App::new(), view);
    
    // This test will be activated when R2 feature is implemented
    // Shows the expected API of the new feature
    h.state_mut().spring(/* expected API */);
    h.frames(10);
    
    assert_eq!(h.state().velocity, expected);
}
```

Activate for Phase 2 implementation:
```bash
sed -i '/acceptance_new_r2_feature_works/s/#\[ignore\]//' tests/recipes.rs
cargo test --test recipes new_r2_feature_works
```

---

## Performance Verification

### Frame Time Baseline

```bash
# Measure current animation system overhead
cargo run -p rui --release --example cost 2>&1 | grep -A 5 "Animation"
```

**Expected**: <1ms per frame during animation

### Memory Usage

```bash
# Check Memory struct size
cargo test --lib memory::tests::memory_size -- --nocapture 2>&1
```

**Expected**: <1KB overhead per 10 concurrent animations

---

## Integration Test Verification

### Test Harness Integration

```bash
# Verify Harness handles animations correctly
cargo test --test interaction animation 2>&1
```

**Should pass**:
- Animations don't require real wall-clock time
- Deterministic frame stepping works
- Multiple animations coexist without interference

### Cross-Module Integration

```bash
# Verify animation integrates with other systems
cargo test --lib element::tests -- animation 2>&1
cargo test --lib paint::tests -- animation 2>&1
cargo test --lib input::tests -- animation 2>&1
```

---

## Sign-Off Checklist

When moving from STEP 19 (audit) to STEP 20 (Phase 1 implementation):

- [ ] All 27 baseline tests passing
- [ ] All 7 gaps verified absent (test evidence collected)
- [ ] All 12 acceptance test stubs in place
- [ ] No regressions (396 library tests passing)
- [ ] Documentation suite complete (12 files, 7,941 lines)
- [ ] API Reference grounded in acceptance test stubs
- [ ] State Machine document matches current behavior
- [ ] Integration checklists reviewed and approved
- [ ] Phase 1 gap priorities confirmed (Metrics.motion, Velocity, 2-live-loop)
- [ ] Implementation blueprint reviewed by team

**Ready to begin STEP 20: Phase 1 Implementation** ✅

---

## Document Cross-References

**For understanding current tests**:
→ STEP_19_R2_MOTION_KIT_AUDIT_ANALYSIS.md (current state section)

**For running acceptance tests**:
→ STEP_19_R2_MOTION_KIT_AUDIT_VERIFICATION_GATES.md (acceptance test section)

**For Phase 1 implementation**:
→ STEP_19_R2_MOTION_KIT_AUDIT_IMPLEMENTATION_BLUEPRINT.md (Phase 1 code guide)

**For debugging integration**:
→ STEP_19_R2_MOTION_KIT_AUDIT_CROSS_MODULE_CONCERNS.md (friction point resolution)

---
