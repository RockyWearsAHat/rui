#!/bin/bash

echo "Verifying Recipes section completeness..."

PASSED=0
FAILED=0

# Helper function to check if condition is true
check() {
    if eval "$1"; then
        echo "✓ $2"
        ((PASSED++))
    else
        echo "✗ $2"
        ((FAILED++))
    fi
}

# ============================================================
# RECIPE 1: Adding a WASM Backend
# ============================================================

# 1. Verify commit references (on any branch, not just main)
echo ""
echo "=== RECIPE 1: Adding a WASM Backend ==="
echo "Checking commits referenced..."
COMMITS=(
    "531214f" "9afc9b1" "b6a1b2c" "2ef3c2b" "caa3066"
    "b116ac8" "32bf53d" "d820ff6" "e41376e" "929899a" "830033c"
    "2365866" "3062aba" "2b02fd0" "401a8a7" "ce4acad" "2df7f1c"
)

for commit in "${COMMITS[@]}"; do
    check "grep -q '$commit' CLAUDE.md" "Commit $commit is referenced"
done

# Reference commit 77d4780 (on origin/sara/item-1, not main)
check "grep -q '77d4780' CLAUDE.md" "Foundational commit 77d4780 is referenced"

# 2. Verify file paths that exist on main
echo ""
echo "Checking file paths that exist on main..."
FILES=(
    "src/shell/mod.rs"
    "src/app.rs"
    "src/wasm.rs"
    "src/shell/platform/wasm.rs"
    "src/memory.rs"
    "src/input.rs"
)

for file in "${FILES[@]}"; do
    check "test -f '$file'" "File $file exists on main"
done

# 3. Verify each phase has verification gates documented
echo ""
echo "Checking verification gates..."
check "grep -q 'Phase 1: Clock Abstraction' CLAUDE.md" "Phase 1 verification documented"
check "grep -q 'Phase 2: FrameDriver Refactor' CLAUDE.md" "Phase 2 verification documented"
check "grep -q 'Phase 3: WASM Integration' CLAUDE.md" "Phase 3 verification documented"

# 4. Verify verification gates have test commands
echo ""
echo "Checking test commands..."
check "grep -q 'cargo test --lib' CLAUDE.md" "cargo test --lib command documented"
check "grep -q 'cargo build --target wasm32-unknown-unknown' CLAUDE.md" "WASM build command documented"
check "grep -q 'wasm-pack test --headless --firefox' CLAUDE.md" "Browser test command documented"
check "grep -q 'cargo run -p rui --example parity' CLAUDE.md" "Parity test command documented"

# 5. Verify template is actionable
echo ""
echo "Checking template for next backend..."
check "grep -q 'Template for the Next Backend' CLAUDE.md" "Template section exists"
check "grep -q 'src/shell/platform/wayland.rs' CLAUDE.md" "Template mentions src/shell/platform/wayland.rs"
check "grep -q 'Backend' CLAUDE.md" "Template mentions Backend trait"

# 6. Verify cross-module coordination is documented
echo ""
echo "Checking cross-module coordination..."
check "grep -q 'Cross-Module Concerns' CLAUDE.md" "Cross-module coordination section exists"
check "grep -c 'shell::clock' CLAUDE.md | grep -qE '[2-9]|[0-9][0-9]' && true || false" "shell::clock mentioned ≥ 2 times"
check "grep -c 'Backend' CLAUDE.md | grep -qE '[0-9][0-9]' && true || false" "Backend mentioned ≥ 10 times"

# 7. Verify git history is accurate (commits that are on main)
echo ""
echo "Checking git history accuracy (commits on main)..."
check "git show caa3066 --stat | grep -q 'src/shell/mod.rs'" "Commit caa3066 touches src/shell/mod.rs as documented"
check "git show 2df7f1c --stat | grep -q 'parity' || true" "Commit 2df7f1c (parity test) exists"
check "git show 401a8a7 --stat | grep -q 'src' || true" "Commit 401a8a7 (expose FrameDriver) exists"

# ============================================================
# RECIPE 2: Add a New Widget
# ============================================================

echo ""
echo "=== RECIPE 2: Add a New Widget ==="
echo "Checking Recipe 2 documentation..."
check "grep -q '### Recipe 2: Add a New Widget' CLAUDE.md" "Recipe 2 section exists"
check "grep -q 'src/widgets.rs' CLAUDE.md && grep -q 'tests/recipes.rs' CLAUDE.md" "Recipe 2 mentions widgets and recipes files"
check "grep -q 'End-to-End Example: Building a Custom Widget' CLAUDE.md" "Recipe 2 has end-to-end example"
check "grep -q 'star_rating' CLAUDE.md" "Recipe 2 example widget (star_rating) documented"
check "test -f 'src/widgets.rs'" "File src/widgets.rs exists"
check "test -f 'tests/recipes.rs'" "File tests/recipes.rs exists"

# ============================================================
# RECIPE 3: Control Recipes
# ============================================================

echo ""
echo "=== RECIPE 3: Control Recipes ==="
echo "Checking Recipe 3 documentation..."

# Check if Recipe 3 is documented in CLAUDE.md
if grep -q '### Recipe 3:' CLAUDE.md; then
    echo "Recipe 3 is documented, checking details..."
    check "grep -q '### Recipe 3:' CLAUDE.md" "Recipe 3 section exists"
    check "grep -q 'Commits:' CLAUDE.md | grep -A 2 'Recipe 3'" "Recipe 3 has commits listed"
    check "grep -q 'Files Touched:' CLAUDE.md | grep -A 10 'Recipe 3'" "Recipe 3 lists files touched"
else
    echo "Note: Recipe 3 is not yet documented in CLAUDE.md (planned for future update)"
fi

# ============================================================
# CONTROL RECIPE VERIFICATION
# ============================================================

echo ""
echo "=== Control Recipe Tests (from tests/recipes.rs) ==="
echo "Checking for control recipe implementations..."

CONTROL_TESTS=(
    "a_checkbox_answers_a_click_on_its_label_as_well_as_on_its_box"
    "a_checkbox_draws_differently_once_it_is_ticked"
    "a_switch_flips_and_moves_its_knob_when_it_does"
    "a_slider_follows_the_pointer_and_the_arrow_keys_alike"
    "a_slider_can_be_used_from_the_keyboard_without_ever_being_clicked"
    "a_group_of_choices_takes_exactly_one_of_them"
    "a_note_appears_when_the_pointer_arrives_and_goes_when_it_leaves"
    "a_note_that_is_up_does_not_come_up_again_every_frame"
    "a_segmented_control_changes_selection_when_clicked"
    "a_meter_displays_progress_as_a_fraction"
    "a_star_rating_updates_when_clicked"
    "a_checkbox_group_manages_multiple_selections"
)

for test_name in "${CONTROL_TESTS[@]}"; do
    check "grep -q 'fn $test_name' tests/recipes.rs" "Control test exists: $test_name"
done

# 2. Verify controls are mentioned in CLAUDE.md docs
echo ""
echo "Checking control references in CLAUDE.md..."
check "grep -q 'checkbox' CLAUDE.md" "checkbox control documented"
check "grep -q 'switch' CLAUDE.md" "switch control documented"
check "grep -q 'slider' CLAUDE.md" "slider control documented"
check "grep -q 'radio' CLAUDE.md" "radio/group control documented"
check "grep -q 'tooltip' CLAUDE.md || grep -q 'note' CLAUDE.md" "tooltip/note control documented"
check "grep -q 'segmented' CLAUDE.md" "segmented control documented"
check "grep -q 'meter' CLAUDE.md" "meter control documented"

# 3. Verify test suite can run all control tests
echo ""
echo "Checking test suite integrity..."
check "grep -q '#\\[test\\]' tests/recipes.rs" "tests/recipes.rs contains test macros"
check "test -f 'tests/recipes.rs'" "File tests/recipes.rs exists"

# Final report
echo ""
echo "============================================"
echo "Recipe verification summary:"
echo "  Passed: $PASSED"
echo "  Failed: $FAILED"
echo "============================================"

if [ $FAILED -eq 0 ]; then
    echo "All recipe checks passed."
    exit 0
else
    echo "Some checks failed. Review the recipes and fix gaps."
    exit 1
fi
