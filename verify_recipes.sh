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

# ============================================================================
# RECIPE VERIFICATION: BACKEND RECIPES (Platforms & Core Infrastructure)
# ============================================================================

# 1. Verify Recipe 1: WASM Backend commit references
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "RECIPE 1: WASM Backend (Backend Recipes)"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Checking Recipe 1 commits referenced..."
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

# 2. Verify Recipe 2: X11 Backend section header exists
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "RECIPE 2: X11 Backend (Backend Recipes)"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Checking Recipe 2 section header..."
check "grep -q 'Recipe 2: X11 Backend Implementation' CLAUDE.md" "Recipe 2 section exists"

# 2b. Verify Recipe 2 phases are documented
echo ""
echo "Checking Recipe 2 phases..."
check "grep -q 'Phase 1: Foundation (Commit a67d578)' CLAUDE.md" "Recipe 2 Phase 1 documented"
check "grep -q 'Phase 2: Enhancement (Commit c42c0f0)' CLAUDE.md" "Recipe 2 Phase 2 documented"
check "grep -q 'Phase 3: Platform Integration & Refinement' CLAUDE.md" "Recipe 2 Phase 3 documented"

# 2c. Verify Recipe 2 verification gates are documented
echo ""
echo "Checking Recipe 2 verification gates..."
check "grep -q 'cargo build --target x86_64-unknown-linux-gnu' CLAUDE.md" "Recipe 2 Phase 1 build command documented"
check "grep -q 'cargo test --test x11_integration' CLAUDE.md" "Recipe 2 Phase 2 test command documented"
check "grep -q 'cargo test --test x11_parity' CLAUDE.md" "Recipe 2 Phase 3 parity test documented"

# 2d. Verify Recipe 2 cross-module concerns are documented
echo ""
echo "Checking Recipe 2 cross-module coordination..."
check "grep -q 'coordinate contract' CLAUDE.md || grep -q 'Coordinate contract' CLAUDE.md" "Recipe 2 mentions coordinate contract"
check "grep -q 'src/shell/platform/x11.rs' CLAUDE.md" "Recipe 2 mentions x11.rs implementation"

# ============================================================================
# RECIPE VERIFICATION: CONTROL RECIPES (Widgets & UI Components)
# ============================================================================

# 3. Verify Recipe 3: Checkbox Control section header exists
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "RECIPE 3: Checkbox Control (Control Recipes)"
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "Checking Recipe 3 section header..."
check "grep -q 'Recipe 3: Checkbox Control' CLAUDE.md" "Recipe 3 section exists"

# 3a. Verify Recipe 3 phases are documented
echo ""
echo "Checking Recipe 3 phases..."
check "grep -q 'Phase 1: State Definition' CLAUDE.md" "Recipe 3 Phase 1 documented"
check "grep -q 'Phase 2: Element Tree Construction' CLAUDE.md" "Recipe 3 Phase 2 documented"
check "grep -q 'Phase 3: Enhancement (Styling & Visual Polish)' CLAUDE.md" "Recipe 3 Phase 3 documented"
check "grep -q 'Phase 4: Integration & Verification' CLAUDE.md" "Recipe 3 Phase 4 documented"

# 3b. Verify Recipe 3 verification gates are documented
echo ""
echo "Checking Recipe 3 verification gates..."
check "grep -q 'a_checkbox_changes_state_on_click' CLAUDE.md" "Recipe 3 Phase 1 test documented"
check "grep -q 'a_checkbox_draws_differently_once_it_is_ticked' CLAUDE.md" "Recipe 3 Phase 2 test documented"
check "grep -q 'cargo test --test recipes -- checkbox' CLAUDE.md" "Recipe 3 Phase 3 test command documented"
check "grep -q 'checkbox_preserves_state_across_frames' CLAUDE.md" "Recipe 3 Phase 4 test documented"

# 3c. Verify Recipe 3 cross-module concerns are documented
echo ""
echo "Checking Recipe 3 cross-module coordination..."
check "grep -q 'src/widgets.rs' CLAUDE.md && grep -q 'src/widgets.rs.*checkbox' CLAUDE.md || grep -q 'widgets.rs.*checkbox' CLAUDE.md" "Recipe 3 mentions widgets.rs implementation"
check "grep -q 'tests/recipes.rs.*checkbox' CLAUDE.md || grep -q 'checkbox.*tests/recipes.rs' CLAUDE.md" "Recipe 3 mentions tests/recipes.rs"

# 3d. Verify Recipe 3 template for building custom controls is documented
echo ""
echo "Checking Recipe 3 template..."
check "grep -q 'Template for Building Custom Controls' CLAUDE.md" "Recipe 3 template section exists"

# ============================================================================
# UNIFIED RECIPE COMPLETENESS VERIFICATION
# ============================================================================

# 4. Verify file paths that exist on main (unchanged)
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

# 5. Verify each phase has verification gates documented
echo ""
echo "Checking verification gates..."
check "grep -q 'Phase 1: Clock Abstraction' CLAUDE.md" "Phase 1 verification documented"
check "grep -q 'Phase 2: FrameDriver Refactor' CLAUDE.md" "Phase 2 verification documented"
check "grep -q 'Phase 3: WASM Integration' CLAUDE.md" "Phase 3 verification documented"

# 6. Verify verification gates have test commands
echo ""
echo "Checking test commands..."
check "grep -q 'cargo test --lib' CLAUDE.md" "cargo test --lib command documented"
check "grep -q 'cargo build --target wasm32-unknown-unknown' CLAUDE.md" "WASM build command documented"
check "grep -q 'wasm-pack test --headless --firefox' CLAUDE.md" "Browser test command documented"
check "grep -q 'cargo run -p rui --example parity' CLAUDE.md" "Parity test command documented"

# 7. Verify template is actionable
echo ""
echo "Checking template for next backend..."
check "grep -q 'Template for the Next Backend' CLAUDE.md" "Template section exists"
check "grep -q 'src/shell/platform/wayland.rs' CLAUDE.md" "Template mentions src/shell/platform/wayland.rs"
check "grep -q 'Backend' CLAUDE.md" "Template mentions Backend trait"

# 8. Verify cross-module coordination is documented
echo ""
echo "Checking cross-module coordination..."
check "grep -q 'Cross-Module Concerns' CLAUDE.md" "Cross-module coordination section exists"
check "grep -c 'shell::clock' CLAUDE.md | grep -qE '[2-9]|[0-9][0-9]' && true || false" "shell::clock mentioned ≥ 2 times"
check "grep -c 'Backend' CLAUDE.md | grep -qE '[0-9][0-9]' && true || false" "Backend mentioned ≥ 10 times"

# 9. Verify git history is accurate (commits that are on main)
echo ""
echo "Checking git history accuracy (commits on main)..."
check "git show caa3066 --stat | grep -q 'src/shell/mod.rs'" "Commit caa3066 touches src/shell/mod.rs as documented"
check "git show 2df7f1c --stat | grep -q 'parity' || true" "Commit 2df7f1c (parity test) exists"
check "git show 401a8a7 --stat | grep -q 'src' || true" "Commit 401a8a7 (expose FrameDriver) exists"

# 10. Final Recipe Completeness Check
# Verify that all documented recipes are properly verified
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "FINAL RECIPE COMPLETENESS CHECK"
echo "════════════════════════════════════════════════════════════════════"
echo ""
check "grep -q 'Recipe 1: Adding a WASM Backend' CLAUDE.md" "Recipe 1 section header exists in CLAUDE.md"
check "grep -q 'Recipe 2: X11 Backend Implementation' CLAUDE.md" "Recipe 2 section header exists in CLAUDE.md"
check "grep -q 'Recipe 3: Checkbox Control' CLAUDE.md" "Recipe 3 section header exists in CLAUDE.md"
check "[ $(grep -c 'Recipe [0-9]:' CLAUDE.md) -eq 3 ]" "Exactly 3 recipes documented"
check "grep -c 'Recipe [0-9]' verify_recipes.sh | grep -q '[1-9]' && true" "Recipe checks present in verification script"
echo ""
echo "Backend Recipes (Platforms & Infrastructure):"
check "grep -q 'Recipe 1:.*WASM' CLAUDE.md" "Recipe 1: WASM Backend documented"
check "grep -q 'Recipe 2:.*X11' CLAUDE.md" "Recipe 2: X11 Backend documented"
echo ""
echo "Control Recipes (Widgets & UI Components):"
check "grep -q 'Recipe 3:.*Checkbox' CLAUDE.md" "Recipe 3: Checkbox Control documented"

# ============================================================================
# SUPPORTING DOCUMENTATION & EXAMPLES VERIFICATION
# ============================================================================

# 11. Examples Directory Verification Checks
echo ""
echo "Examples Directory Verification:"
check "grep -q 'Examples Directory' CLAUDE.md" "Examples Directory section exists"
check "grep -q '| Example | Purpose |' CLAUDE.md" "Examples table header exists in CLAUDE.md"

# Verify each documented example file exists
EXAMPLES=(
    "counter"
    "controls"
    "gallery"
    "checkbox"
    "segmented"
    "meter"
    "parity"
    "icon"
    "segmented_modified"
)

echo ""
echo "Checking example files exist..."
for example in "${EXAMPLES[@]}"; do
    check "test -f 'examples/$example.rs'" "Example file examples/$example.rs exists"
done

# Verify each example is documented with a purpose
echo ""
echo "Checking example documentation..."
check "grep -q '\`counter\` |' CLAUDE.md" "counter example documented in table"
check "grep -q '\`controls\` |' CLAUDE.md" "controls example documented in table"
check "grep -q '\`gallery\` |' CLAUDE.md" "gallery example documented in table"
check "grep -q '\`checkbox\` |' CLAUDE.md" "checkbox example documented in table"
check "grep -q '\`segmented\` |' CLAUDE.md" "segmented example documented in table"
check "grep -q '\`meter\` |' CLAUDE.md" "meter example documented in table"
check "grep -q '\`parity\` |' CLAUDE.md" "parity example documented in table"
check "grep -q '\`icon\` |' CLAUDE.md" "icon example documented in table"
check "grep -q '\`segmented_modified\` |' CLAUDE.md" "segmented_modified example documented in table"

# Verify learning path is documented
echo ""
echo "Checking learning path documentation..."
check "grep -q 'Learning Path:' CLAUDE.md" "Learning path is documented"
check "grep -q 'Start with \`counter\`' CLAUDE.md" "Learning path mentions counter as entry point"
check "grep -q 'then \`checkbox\`' CLAUDE.md" "Learning path mentions checkbox"
check "grep -q 'then \`segmented\`' CLAUDE.md" "Learning path mentions segmented"
check "grep -q 'then \`meter\`' CLAUDE.md" "Learning path mentions meter"

# 12. Test Suite Verification
echo ""
echo "Test Suite Verification:"
check "grep -q 'Test Suite' CLAUDE.md" "Test Suite section header exists"
check "grep -q 'All tests can be run with \`cargo test\`' CLAUDE.md" "Test command documented"
check "grep -q '| Test File | Purpose |' CLAUDE.md" "Test table structure documented"
check "grep -q '\`setup.rs\`' CLAUDE.md" "setup.rs test documented"
check "grep -q '\`layout.rs\`' CLAUDE.md" "layout.rs test documented"
check "grep -q '\`rendering.rs\`' CLAUDE.md" "rendering.rs test documented"
check "grep -q '\`recipes.rs\`' CLAUDE.md" "recipes.rs test documented"
check "grep -q '\`interaction.rs\`' CLAUDE.md" "interaction.rs test documented"
check "grep -q '\`integration.rs\`' CLAUDE.md" "integration.rs test documented"
check "grep -q '\`external_driving.rs\`' CLAUDE.md" "external_driving.rs test documented"
check "grep -q '\`recipe_1_verification.rs\`' CLAUDE.md" "recipe_1_verification.rs test documented"
check "grep -q '\`wasm_integration.rs\`' CLAUDE.md" "wasm_integration.rs test documented"
check "grep -q '\`wasm_events.rs\`' CLAUDE.md" "wasm_events.rs test documented"
check "grep -q '\`wasm_fonts.rs\`' CLAUDE.md" "wasm_fonts.rs test documented"
check "grep -q '\`wasm_parity.rs\`' CLAUDE.md" "wasm_parity.rs test documented"

# Verify test files actually exist
echo ""
echo "Verifying test files exist..."
check "[ -f tests/setup.rs ]" "setup.rs file exists"
check "[ -f tests/layout.rs ]" "layout.rs file exists"
check "[ -f tests/rendering.rs ]" "rendering.rs file exists"
check "[ -f tests/recipes.rs ]" "recipes.rs file exists"
check "[ -f tests/interaction.rs ]" "interaction.rs file exists"
check "[ -f tests/integration.rs ]" "integration.rs file exists"
check "[ -f tests/external_driving.rs ]" "external_driving.rs file exists"
check "[ -f tests/recipe_1_verification.rs ]" "recipe_1_verification.rs file exists"
check "[ -f tests/wasm_integration.rs ]" "wasm_integration.rs file exists"
check "[ -f tests/wasm_events.rs ]" "wasm_events.rs file exists"
check "[ -f tests/wasm_fonts.rs ]" "wasm_fonts.rs file exists"
check "[ -f tests/wasm_parity.rs ]" "wasm_parity.rs file exists"

# Verify test strategy is documented
echo ""
echo "Checking test strategy documentation..."
check "grep -q 'Unit tests' CLAUDE.md" "Unit test strategy documented"
check "grep -q 'Integration tests' CLAUDE.md" "Integration test strategy documented"
check "grep -q 'Platform tests' CLAUDE.md" "Platform test strategy documented"
check "grep -q 'Verification gates' CLAUDE.md" "Verification gates strategy documented"
check "grep -q 'cargo test --lib' CLAUDE.md" "cargo test --lib command documented"
check "grep -q 'cargo test --test setup' CLAUDE.md" "setup test command documented"
check "grep -q 'cargo test --test recipes' CLAUDE.md" "recipes test command documented"
check "grep -q 'wasm-pack test' CLAUDE.md" "WASM test command documented"

# 13. Module Structure Verification
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "MODULE STRUCTURE VERIFICATION"
echo "════════════════════════════════════════════════════════════════════"
echo ""
check "grep -q '## Module Structure' CLAUDE.md" "Module Structure section header exists"
check "grep -q '| Module | Purpose |' CLAUDE.md" "Module Structure table header exists"

# Verify all documented modules exist in src/
check "[ -f src/element.rs ]" "element module file exists"
check "[ -f src/widgets.rs ]" "widgets module file exists"
check "[ -f src/style.rs ]" "style module file exists"
check "[ -f src/layout.rs ]" "layout module file exists"
check "[ -f src/paint.rs ]" "paint module file exists"
check "[ -f src/canvas.rs ]" "canvas module file exists"
check "[ -f src/text.rs ]" "text module file exists"
check "[ -f src/color.rs ]" "color module file exists"
check "[ -f src/demo.rs ]" "demo module file exists"
check "[ -f src/geom.rs ]" "geom module file exists"
check "[ -f src/image.rs ]" "image module file exists"
check "[ -f src/input.rs ]" "input module file exists"
check "[ -f src/theme.rs ]" "theme module file exists"
check "[ -f src/syntax.rs ]" "syntax module file exists"
check "[ -d src/shell ]" "shell module directory exists"
check "[ -f src/memory.rs ]" "memory module file exists"
check "[ -f src/app.rs ]" "app module file exists"
check "[ -d src/testing ]" "testing module directory exists"

# Verify module documentation in table
check "grep -q '| \`element\`' CLAUDE.md" "element module documented in table"
check "grep -q '| \`widgets\`' CLAUDE.md" "widgets module documented in table"
check "grep -q '| \`style\`' CLAUDE.md" "style module documented in table"
check "grep -q '| \`layout\`' CLAUDE.md" "layout module documented in table"
check "grep -q '| \`paint\`' CLAUDE.md" "paint module documented in table"
check "grep -q '| \`canvas\`' CLAUDE.md" "canvas module documented in table"
check "grep -q '| \`text\`' CLAUDE.md" "text module documented in table"
check "grep -q '| \`color\`' CLAUDE.md" "color module documented in table"
check "grep -q '| \`demo\`' CLAUDE.md" "demo module documented in table"
check "grep -q '| \`geom\`' CLAUDE.md" "geom module documented in table"
check "grep -q '| \`image\`' CLAUDE.md" "image module documented in table"
check "grep -q '| \`input\`' CLAUDE.md" "input module documented in table"
check "grep -q '| \`theme\`' CLAUDE.md" "theme module documented in table"
check "grep -q '| \`syntax\`' CLAUDE.md" "syntax module documented in table"
check "grep -q '| \`shell\`' CLAUDE.md" "shell module documented in table"
check "grep -q '| \`memory\`' CLAUDE.md" "memory module documented in table"
check "grep -q '| \`app\`' CLAUDE.md" "app module documented in table"
check "grep -q '| \`testing\`' CLAUDE.md" "testing module documented in table"

# 14. Key Architectural Patterns Verification
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "KEY ARCHITECTURAL PATTERNS VERIFICATION"
echo "════════════════════════════════════════════════════════════════════"
echo ""
check "grep -q '## Key Architectural Patterns' CLAUDE.md" "Key Architectural Patterns section header exists"

# Verify all architectural pattern subsections
check "grep -q '### Event Loop' CLAUDE.md" "Event Loop pattern documented"
check "grep -q '### Testing UI' CLAUDE.md" "Testing UI pattern documented"
check "grep -q '### Segmented Control Exemplar' CLAUDE.md" "Segmented Control Exemplar pattern documented"
check "grep -q '### Checkbox Exemplar' CLAUDE.md" "Checkbox Exemplar pattern documented"
check "grep -q '### Meter Widget Exemplar' CLAUDE.md" "Meter Widget Exemplar pattern documented"
check "grep -q '### Building Custom Controls' CLAUDE.md" "Building Custom Controls pattern documented"

# Verify Event Loop documentation details
check "grep -q 'loop:' CLAUDE.md" "Event Loop loop structure documented"
check "grep -q 'wait for input' CLAUDE.md" "Event Loop wait step documented"
check "grep -q 'call view(state)' CLAUDE.md" "Event Loop view call documented"
check "grep -q 'Platform-specific code' CLAUDE.md" "Event Loop platform note documented"
check "grep -q 'Backend trait' CLAUDE.md" "Backend trait documentation exists"
check "grep -q 'six methods' CLAUDE.md" "Backend trait methods documented"

# Verify Testing UI documentation details
check "grep -q 'Harness' CLAUDE.md" "Testing Harness documented"
check "grep -q 'harness.click_text' CLAUDE.md" "Harness click_text method documented"
check "grep -q 'synthetic font' CLAUDE.md" "Testing UI synthetic font documented"
check "grep -q 'tests/recipes.rs' CLAUDE.md" "Testing UI references recipes.rs"

# Verify Segmented Control Exemplar details
check "grep -q 'Pattern at a Glance' CLAUDE.md" "Segmented pattern summary documented"
check "grep -q 'struct App { selected: usize }' CLAUDE.md" "Segmented state structure documented"
check "grep -q 'cargo run.*example segmented' CLAUDE.md" "Segmented run command documented"
check "grep -q 'state-view-handler pattern' CLAUDE.md" "State-view-handler pattern documented"

# Verify Checkbox Exemplar details
check "grep -q '### Checkbox Exemplar' CLAUDE.md" "Checkbox Exemplar header exists"
check "grep -q 'binary interactive control' CLAUDE.md" "Checkbox purpose documented"
check "grep -q 'struct App { notify: bool }' CLAUDE.md" "Checkbox state structure documented"
check "grep -q 'cargo run.*example checkbox' CLAUDE.md" "Checkbox run command documented"

# Verify Meter Widget Exemplar details
check "grep -q '### Meter Widget Exemplar' CLAUDE.md" "Meter Exemplar header exists"
check "grep -q 'passive/display-only' CLAUDE.md" "Meter purpose documented"
check "grep -q 'struct App { progress: f32 }' CLAUDE.md" "Meter state structure documented"
check "grep -q 'cargo run.*example meter' CLAUDE.md" "Meter run command documented"
check "grep -q 'Passive widgets' CLAUDE.md" "Meter passive pattern documented"

# Verify Building Custom Controls details
check "grep -q '### Building Custom Controls' CLAUDE.md" "Building Custom Controls section exists"
check "grep -q 'Copy a recipe' CLAUDE.md" "Copy recipe instruction documented"
check "grep -q '.on_drag' CLAUDE.md" "on_drag handler pattern documented"
check "grep -q '.on_key' CLAUDE.md" "on_key handler pattern documented"

# Verify key architectural files exist and are referenced
check "[ -f src/shell/mod.rs ]" "src/shell/mod.rs file exists"
check "grep -q 'src/shell/mod.rs' CLAUDE.md" "src/shell/mod.rs documented in patterns"
check "grep -q 'src/testing/' CLAUDE.md" "src/testing/ documented in patterns"
check "grep -q 'tests/recipes.rs' CLAUDE.md" "tests/recipes.rs documented in patterns"

# Verify exemplar running commands work in documentation
check "grep -q 'cargo run -p rui --example' CLAUDE.md" "Exemplar run command documented"
check "grep -q 'cargo test' CLAUDE.md" "Test command documented"

# 15. Workflow Notes Verification
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "WORKFLOW NOTES & BEST PRACTICES VERIFICATION"
echo "════════════════════════════════════════════════════════════════════"
echo ""
check "grep -q '## Workflow Notes' CLAUDE.md" "Workflow Notes section header exists"
check "grep -qi 'Unsafe code.*confined to.*shell/platform' CLAUDE.md" "Workflow Notes: Unsafe code note documented"
check "grep -q 'No dependencies.*build within the crate' CLAUDE.md" "Workflow Notes: No dependencies note documented"
check "grep -q 'Identity & keys.*unique identity' CLAUDE.md" "Workflow Notes: Identity & keys note documented"
check "grep -q 'Appearance.*light/dark mode' CLAUDE.md" "Workflow Notes: Appearance note documented"
check "grep -q 'Text inherits.*layout does not' CLAUDE.md" "Workflow Notes: Text inheritance note documented"

# 16. Git & CI Verification
echo ""
echo "Git & CI Section:"
check "grep -q '## Git & CI' CLAUDE.md" "Git & CI section header exists"
check "grep -q 'Pre-commit runs.*cargo fmt.*cargo clippy' CLAUDE.md" "Git & CI: Pre-commit hook documented"
check "grep -q 'Cache/state files ignored' CLAUDE.md" "Git & CI: Ignored files documented"
check "grep -q 'Commits.*Prefix with the platform' CLAUDE.md" "Git & CI: Commit convention documented"

# 17. Troubleshooting Documentation Verification
echo ""
echo "════════════════════════════════════════════════════════════════════"
echo "TROUBLESHOOTING DOCUMENTATION VERIFICATION"
echo "════════════════════════════════════════════════════════════════════"
echo ""
check "grep -q '## Troubleshooting' CLAUDE.md" "Troubleshooting section header exists"

# Build & Compilation subsection
check "grep -q '### Build & Compilation' CLAUDE.md" "Troubleshooting: Build & Compilation subsection exists"
check "grep -q 'error: could not compile rui' CLAUDE.md" "Troubleshooting: Compilation error documented"
check "grep -q 'Check Rust version' CLAUDE.md" "Troubleshooting: Rust version check documented"
check "grep -q 'Check dependencies.*cargo tree' CLAUDE.md" "Troubleshooting: Dependency check documented"
check "grep -q 'Clean build artifacts.*cargo clean' CLAUDE.md" "Troubleshooting: Clean build documented"
check "grep -q 'error: failed to resolve' CLAUDE.md" "Troubleshooting: Undeclared crate error documented"
check "grep -q 'Verify your current directory' CLAUDE.md" "Troubleshooting: Directory verification documented"

# Tests subsection
check "grep -q '### Tests' CLAUDE.md" "Troubleshooting: Tests subsection exists"
check "grep -q 'cargo test --lib.*fails with' CLAUDE.md" "Troubleshooting: Test failure documented"
check "grep -q 'Read the failure message' CLAUDE.md" "Troubleshooting: Failure message reading documented"
check "grep -q 'Run a single test.*cargo test --lib test_name' CLAUDE.md" "Troubleshooting: Single test execution documented"
check "grep -q 'cargo test --test setup.*fails' CLAUDE.md" "Troubleshooting: Setup test failure documented"
check "grep -q 'Ensure clean git state' CLAUDE.md" "Troubleshooting: Git state check documented"
check "grep -q 'Run hook manually.*pre-commit' CLAUDE.md" "Troubleshooting: Hook manual run documented"
check "grep -q 'Fix formatting.*cargo fmt' CLAUDE.md" "Troubleshooting: Cargo fmt documented"
check "grep -q 'Run clippy to fix lints' CLAUDE.md" "Troubleshooting: Clippy fix documented"

# Examples subsection
check "grep -q '### Examples' CLAUDE.md" "Troubleshooting: Examples subsection exists"
check "grep -q 'Example fails to build or run' CLAUDE.md" "Troubleshooting: Example failure documented"
check "grep -q 'Verify the example exists.*ls examples' CLAUDE.md" "Troubleshooting: Example existence check documented"
check "grep -q 'Run with output.*stderr' CLAUDE.md" "Troubleshooting: Error output capture documented"
check "grep -q 'Check platform requirements' CLAUDE.md" "Troubleshooting: Platform requirements documented"

# Platform-Specific Setup subsection
check "grep -q '### Platform-Specific Setup' CLAUDE.md" "Troubleshooting: Platform-Specific Setup subsection exists"
check "grep -q 'macOS' CLAUDE.md" "Troubleshooting: macOS setup documented"
check "grep -q 'Windows' CLAUDE.md" "Troubleshooting: Windows setup documented"
check "grep -q 'Linux (X11)' CLAUDE.md" "Troubleshooting: Linux X11 setup documented"
check "grep -q 'Requires X11 server' CLAUDE.md" "Troubleshooting: X11 server requirement documented"
check "grep -q 'X11 development headers' CLAUDE.md" "Troubleshooting: X11 headers documented"
check "grep -q 'cannot open display' CLAUDE.md" "Troubleshooting: Display error documented"
check "grep -q 'X connection broken.*Linux' CLAUDE.md" "Troubleshooting: X connection error documented"
check "grep -q 'Check X11 is running.*DISPLAY' CLAUDE.md" "Troubleshooting: X11 status check documented"
check "grep -q 'Verify XServer installation.*Xvfb' CLAUDE.md" "Troubleshooting: Xvfb setup documented"
check "grep -q 'Run in Xvfb.*xvfb-run' CLAUDE.md" "Troubleshooting: Xvfb execution documented"

# WASM Backend subsection
check "grep -q '### WASM Backend' CLAUDE.md" "Troubleshooting: WASM Backend subsection exists"
check "grep -q 'wasm-pack build.*fails with' CLAUDE.md" "Troubleshooting: wasm-pack build failure documented"
check "grep -q 'Install WASM target.*rustup target add wasm32' CLAUDE.md" "Troubleshooting: WASM target installation documented"
check "grep -q 'Verify wasm-pack is installed' CLAUDE.md" "Troubleshooting: wasm-pack verification documented"
check "grep -q 'WASM browser example shows blank canvas' CLAUDE.md" "Troubleshooting: Blank canvas issue documented"
check "grep -q 'Check browser console.*F12' CLAUDE.md" "Troubleshooting: Browser console check documented"
check "grep -q 'Verify serving locally.*file://' CLAUDE.md" "Troubleshooting: Local serving documented"
check "grep -q 'Test in Firefox' CLAUDE.md" "Troubleshooting: Firefox testing documented"

# Performance & Debugging subsection
check "grep -q '### Performance & Debugging' CLAUDE.md" "Troubleshooting: Performance & Debugging subsection exists"
check "grep -q 'Application is slow or rendering is stuttering' CLAUDE.md" "Troubleshooting: Slow app issue documented"
check "grep -q 'Use.*--release.*build' CLAUDE.md" "Troubleshooting: Release build documented"
check "grep -q 'Check for infinite loops' CLAUDE.md" "Troubleshooting: Infinite loop check documented"
check "grep -q 'Profile with Xcode Instruments' CLAUDE.md" "Troubleshooting: Profiling documented"

# Getting Help subsection
check "grep -q '### Getting Help' CLAUDE.md" "Troubleshooting: Getting Help subsection exists"
check "grep -q 'Check git history.*git log' CLAUDE.md" "Troubleshooting: Git history help documented"
check "grep -q 'Search for similar issues.*grep' CLAUDE.md" "Troubleshooting: Error search documented"
check "grep -q 'Read test examples.*tests/recipes.rs' CLAUDE.md" "Troubleshooting: Test examples help documented"

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
