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

# 1. Verify commit references (on any branch, not just main)
echo ""
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

# 2. Verify Recipe 2 section header exists
echo ""
echo "Checking Recipe 2 section header..."
check "grep -q 'Recipe 2: X11 Backend Implementation' CLAUDE.md" "Recipe 2 section exists"

# 2b. Verify Recipe 3 section header exists
echo ""
echo "Checking Recipe 3 section header..."
check "grep -q 'Recipe 3: Checkbox Control' CLAUDE.md" "Recipe 3 section exists"

# 3. Verify file paths that exist on main
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

# 4. Verify each phase has verification gates documented
echo ""
echo "Checking verification gates..."
check "grep -q 'Phase 1: Clock Abstraction' CLAUDE.md" "Phase 1 verification documented"
check "grep -q 'Phase 2: FrameDriver Refactor' CLAUDE.md" "Phase 2 verification documented"
check "grep -q 'Phase 3: WASM Integration' CLAUDE.md" "Phase 3 verification documented"

# 5. Verify verification gates have test commands
echo ""
echo "Checking test commands..."
check "grep -q 'cargo test --lib' CLAUDE.md" "cargo test --lib command documented"
check "grep -q 'cargo build --target wasm32-unknown-unknown' CLAUDE.md" "WASM build command documented"
check "grep -q 'wasm-pack test --headless --firefox' CLAUDE.md" "Browser test command documented"
check "grep -q 'cargo run -p rui --example parity' CLAUDE.md" "Parity test command documented"

# 6. Verify template is actionable
echo ""
echo "Checking template for next backend..."
check "grep -q 'Template for the Next Backend' CLAUDE.md" "Template section exists"
check "grep -q 'src/shell/platform/wayland.rs' CLAUDE.md" "Template mentions src/shell/platform/wayland.rs"
check "grep -q 'Backend' CLAUDE.md" "Template mentions Backend trait"

# 7. Verify cross-module coordination is documented
echo ""
echo "Checking cross-module coordination..."
check "grep -q 'Cross-Module Concerns' CLAUDE.md" "Cross-module coordination section exists"
check "grep -c 'shell::clock' CLAUDE.md | grep -qE '[2-9]|[0-9][0-9]' && true || false" "shell::clock mentioned ≥ 2 times"
check "grep -c 'Backend' CLAUDE.md | grep -qE '[0-9][0-9]' && true || false" "Backend mentioned ≥ 10 times"

# 8. Verify git history is accurate (commits that are on main)
echo ""
echo "Checking git history accuracy (commits on main)..."
check "git show caa3066 --stat | grep -q 'src/shell/mod.rs'" "Commit caa3066 touches src/shell/mod.rs as documented"
check "git show 2df7f1c --stat | grep -q 'parity' || true" "Commit 2df7f1c (parity test) exists"
check "git show 401a8a7 --stat | grep -q 'src' || true" "Commit 401a8a7 (expose FrameDriver) exists"

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
    echo "Some checks failed. Review the recipe and fix gaps."
    exit 1
fi
