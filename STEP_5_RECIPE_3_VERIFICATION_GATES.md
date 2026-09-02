# Recipe 3: Checkbox Control — Verification Gates

## Phase-by-Phase Acceptance Criteria

Each phase must pass all gates before merging. Gates prevent regressions and ensure implementation quality.

---

## Phase 1: State Definition

**Goal**: Verify state struct compiles and basic test passes.

**Acceptance criteria**:
- [ ] State struct `App { checked: bool }` compiles without error
- [ ] Test runs: `cargo test --test recipes -- a_checkbox_changes_state_on_click`
- [ ] Test passes (state toggle logic is correct)
- [ ] No compiler warnings or clippy errors
- [ ] Code is formatted: `cargo fmt --check`

**Gate commands**:
```bash
# Compile check
cargo build --tests

# Run state test
cargo test --test recipes -- a_checkbox_changes_state_on_click --nocapture

# Format and lint
cargo fmt --check
cargo clippy -- -D warnings
```

**Expected output**:
```
test a_checkbox_changes_state_on_click ... ok

test result: ok. 1 passed; 0 failed
```

**When gate fails**:
- If compile error: Fix syntax in state struct
- If test fails: Verify state toggle logic is correct (bool should flip)
- If clippy error: Ensure struct derives Debug/Clone if needed

---

## Phase 2: Element Tree Construction

**Goal**: Verify checkbox renders correctly and responds to clicks.

**Acceptance criteria**:
- [ ] Checkbox constructor compiles and builds
- [ ] Example runs without crash: `cargo run -p rui --example controls`
- [ ] Visual output shows checkbox box (15x15 px) with label
- [ ] Clicking checkbox toggles the box appearance
- [ ] Handler is called when clicked (verify with debug print if needed)
- [ ] Box renders filled when checked, empty when unchecked
- [ ] No panics or rendering errors

**Gate commands**:
```bash
# Compile
cargo build --examples

# Run visual example
cargo run -p rui --example controls

# Run rendering test
cargo test --test recipes -- a_checkbox_renders --nocapture

# Verify no new warnings
cargo clippy -- -D warnings
```

**Expected behavior**:
- Checkbox visible in controls example
- Click checkbox: box fills/empties
- Behavior is responsive (no lag)
- Styling matches theme (light/dark mode)

**When gate fails**:
- If example crashes: Check exception in stderr; verify draw closure is correct
- If checkbox doesn't respond: Check `on_click` handler is wired; verify toggle closure captures state correctly
- If appearance is wrong: Check `checked` parameter in draw closure; verify painter calls are correct
- If styling mismatches: Check Tone roles are resolving against Theme

---

## Phase 3: Enhancement (Styling & Visual Polish)

**Goal**: Verify focus ring, hover, and disabled states work correctly.

**Acceptance criteria**:
- [ ] Focus ring appears when focused (keyboard Tab)
- [ ] Hover highlight shows on mouse over
- [ ] Disabled checkbox renders at 0.38 alpha
- [ ] Focus ring is 3:1 contrast minimum
- [ ] Light mode rendering passes contrast check (≥4.5 secondary)
- [ ] Dark mode rendering passes contrast check (≥4.5 secondary)
- [ ] Text contrast is ≥7 in both modes
- [ ] No new compiler warnings

**Gate commands**:
```bash
# Visual inspection test
cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover --nocapture

# Accessibility audit
cargo test --lib theme::tests::the_battery_rejects_an_illegible_palette

# Compile check
cargo build --all-features

# Full lint
cargo clippy -- -D warnings
```

**Expected output**:
```
test a_checkbox_displays_visual_feedback_on_hover ... ok
test the_battery_rejects_an_illegible_palette ... ok

test result: ok. 2 passed; 0 failed
```

**Verification steps** (manual):
1. Run `cargo run -p rui --example controls`
2. Tab to checkbox (focus ring should appear, distinct from selection)
3. Hover over checkbox (highlight should change)
4. Use `.disabled(true)` in code, reload, verify 0.38 alpha rendering
5. Swap theme to dark mode (if supported), verify contrast still passes

**When gate fails**:
- If focus ring missing: Check `.takes_focus(true)` is set
- If hover doesn't work: Check on_pointer_move handler or hover state in theme
- If disabled looks wrong: Check .disabled() builder and alpha application in draw()
- If contrast fails: Adjust fill/stroke colors to meet WCAG AA (4.5:1 for secondary)

---

## Phase 4: Integration & Persistence

**Goal**: Verify multiple instances, state persistence, and correct identity handling.

**Acceptance criteria**:
- [ ] Multiple checkboxes render with independent state
- [ ] State persists across 10+ frame rebuilds
- [ ] Reordering checkboxes (with `.key()`) preserves state
- [ ] Focus state is maintained in Memory correctly
- [ ] Accessibility tree includes all instances
- [ ] Tab order is correct (document order)
- [ ] Memory module tests pass (focus, scroll, interaction state)
- [ ] All library tests pass (379 tests)

**Gate commands**:
```bash
# Multiple instance test
cargo test --test recipes -- checkbox_preserves_state_across_frames --nocapture
cargo test --test recipes -- checkbox_works_with_multiple_instances --nocapture

# Memory module verification
cargo test --lib memory -- --nocapture

# Full test suite
cargo test --lib

# Accessibility audit
cargo test --test interaction
```

**Expected output**:
```
test checkbox_preserves_state_across_frames ... ok
test checkbox_works_with_multiple_instances ... ok
test memory ... ok

test result: ok. N passed; 0 failed
```

**Verification with Harness** (code example):
```rust
let mut h = Harness::new(App { checkboxes: vec![false, false, false] }, view);

// Click first checkbox
h.click_at_rect(h.find_element("checkbox-0").unwrap().bounds);
assert_eq!(h.state().checkboxes[0], true);
assert_eq!(h.state().checkboxes[1], false);

// Rebuild frames 10 times
h.frames(10);
assert_eq!(h.state().checkboxes[0], true, "State should persist");

// Reorder and verify identity
h.state_mut().checkboxes.reverse();
h.frames(1);
// With .key(), state should follow item, not position
```

**When gate fails**:
- If multiple instances share state: Check identity (path vs key); add `.key(id)` to override
- If state doesn't persist: Verify Memory struct in memory.rs handles checkbox state
- If reordering breaks: Use `.key(unique_id)` to fix identity
- If focus state lost: Check Memory::focus tracking
- If accessibility tree wrong: Verify El::takes_focus consistency

---

## Gate Sequencing

**Critical path**:
```
Phase 1 PASS → Phase 2 PASS → Phase 3 PASS → Phase 4 PASS → SHIP
```

Each phase depends on previous phase passing. Do not proceed to next phase if current phase gate fails.

**Parallel testing** (optional, before gating):
- Run `cargo test --lib` to verify no regressions in core
- Run `cargo clippy` to catch quality issues early
- Run `cargo fmt` to maintain consistency

---

## Command Summary (All Phases at Once)

To verify entire Recipe 3 implementation:

```bash
# Build everything
cargo build --all-features

# Run all gates in order
cargo test --test recipes -- checkbox --nocapture
cargo test --lib memory --nocapture
cargo test --test interaction --nocapture

# Full suite
cargo test --lib

# Lint and format
cargo fmt --check
cargo clippy -- -D warnings
```

**Expected result**: All tests pass, no warnings or errors.

---

## Quick Reference by Phase

| Phase | Key Gate Command | Pass Condition |
|-------|------------------|----------------|
| 1 | `cargo test --test recipes -- a_checkbox_changes_state_on_click` | Test passes, 1/1 |
| 2 | `cargo run -p rui --example controls` | No crash, checkbox visible and clickable |
| 3 | `cargo test --test recipes -- a_checkbox_displays_visual_feedback_on_hover` | Focus ring, hover, disabled all work |
| 4 | `cargo test --test recipes -- checkbox_preserves_state_across_frames` | Multiple instances, persistence, identity correct |

---

End of STEP_5_RECIPE_3_VERIFICATION_GATES.md
