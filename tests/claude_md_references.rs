//! Verify that all commit SHAs and file paths referenced in CLAUDE.md are valid.
//!
//! This test module ensures documentation stays synchronized with actual git history
//! and filesystem state. It catches references to commits that have been rewritten
//! or deleted, and file paths that have moved or been removed.

#[test]
fn claude_md_references_are_valid() {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    // Read CLAUDE.md
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // The key commits that must exist (from Recipe 2: X11 Backend)
    let required_shas = vec![
        "a67d578", // Phase 1: Foundation
        "c42c0f0", // Phase 2: Enhancement
        "80e3003", // Phase 3: Integration
        "991167a", // Polish
    ];

    // Verify all required commits exist in git log
    let output = Command::new("git")
        .args(["log", "--oneline"])
        .output()
        .expect("Failed to run git log");

    let log = String::from_utf8_lossy(&output.stdout);

    for sha in required_shas {
        assert!(
            log.contains(sha),
            "Commit {} referenced in CLAUDE.md not found in git history",
            sha
        );
    }

    // Expected files that should exist based on CLAUDE.md references
    let required_files = vec![
        "src/lib.rs",
        "src/element.rs",
        "src/style.rs",
        "src/layout.rs",
        "src/geom.rs",
        "src/color.rs",
        "src/canvas.rs",
        "src/sdf.rs",
        "src/image.rs",
        "src/paint.rs",
        "src/text.rs",
        "src/theme.rs",
        "src/widgets.rs",
        "src/memory.rs",
        "src/input.rs",
        "src/accessibility.rs",
        "src/app.rs",
        "src/shell/mod.rs",
        "src/shell/platform/x11.rs",
        "src/reload.rs",
        "src/testing/mod.rs",
        "examples/controls.rs",
        "examples/gallery.rs",
        "tests/recipes.rs",
    ];

    // Verify all required files exist
    for file in required_files {
        assert!(
            Path::new(file).exists(),
            "File {} referenced in CLAUDE.md not found",
            file
        );
    }

    // Spot-check: verify recipes mentioned in CLAUDE.md have documentation
    assert!(
        claude_md.contains("## Recipe 1:"),
        "Recipe 1 documentation missing"
    );
    assert!(
        claude_md.contains("## Recipe 2:"),
        "Recipe 2 documentation missing"
    );
    assert!(
        claude_md.contains("## Recipe 3:"),
        "Recipe 3 documentation missing"
    );

    // Verify key sections exist
    let required_sections = vec![
        "## Module Structure",
        "## Key Invariants",
        "## Key Architectural Patterns",
        "## View-State-Handler Pattern",
        "## Backend Trait Pattern",
        "## Testing with Harness",
        "## Recipe Infrastructure",
        "## Build and Test",
        "## Stellar UI Practices",
        "## Widget Exemplars",
        "## Conventions",
        "## Common Patterns and Edge Cases",
        "## Contributor Workflow",
        "## Verification and Quality Checks",
    ];

    for section in required_sections {
        assert!(
            claude_md.contains(section),
            "Required section '{}' not found in CLAUDE.md",
            section
        );
    }
}

// Helper to validate a single commit SHA exists
fn commit_exists_in_history(sha: &str) -> bool {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline"])
        .output()
        .expect("Failed to run git log");

    let log = String::from_utf8_lossy(&output.stdout);
    log.contains(sha)
}

#[test]
fn recipe_2_x11_commits_are_valid() {
    // Recipe 2: X11 Backend Implementation
    // Verify the 4 key commits from the X11 backend exemplar
    assert!(
        commit_exists_in_history("a67d578"),
        "Recipe 2 Phase 1 commit a67d578 not found"
    );
    assert!(
        commit_exists_in_history("c42c0f0"),
        "Recipe 2 Phase 2 commit c42c0f0 not found"
    );
    assert!(
        commit_exists_in_history("80e3003"),
        "Recipe 2 Phase 3 commit 80e3003 not found"
    );
    assert!(
        commit_exists_in_history("991167a"),
        "Recipe 2 Polish commit 991167a not found"
    );
}

#[test]
fn recipe_2_x11_files_exist() {
    use std::path::Path;

    // Recipe 2 references these files
    let files = vec![
        "src/shell/platform/x11.rs",
        "src/shell/mod.rs",
        "src/app.rs",
        "src/input.rs",
        "src/accessibility.rs",
        "src/memory.rs",
        "src/paint.rs",
    ];

    for file in files {
        assert!(
            Path::new(file).exists(),
            "Recipe 2 file {} does not exist",
            file
        );
    }
}
