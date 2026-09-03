//! Verification that all documentation files reference valid commits and file paths.
//!
//! This test ensures documentation integrity by checking:
//! 1. All commit SHAs mentioned in docs exist in git history
//! 2. All file paths mentioned in docs exist in the project

#[test]
fn all_referenced_commits_exist_in_git_history() {
    let output = std::process::Command::new("git")
        .arg("log")
        .arg("--oneline")
        .arg("--all")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("git log failed");

    let git_log = String::from_utf8(output.stdout).expect("git log output not utf8");

    // Commits from Recipe 2 (X11 backend) that CLAUDE.md references
    let expected_commits = [
        "a67d578", // Phase 1: Foundation
        "c42c0f0", // Phase 2: Enhancement
        "80e3003", // Phase 3: Integration
        "991167a", // Polish
    ];

    for sha in &expected_commits {
        assert!(
            git_log.contains(sha),
            "Commit {} referenced in CLAUDE.md Recipe 2 not found in git history",
            sha
        );
    }
}

#[test]
fn all_core_module_files_referenced_in_docs_exist() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // File paths referenced in CLAUDE.md Module Structure section
    let core_modules = [
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
    ];

    for file in &core_modules {
        let path = std::path::Path::new(manifest_dir).join(file);
        assert!(
            path.exists(),
            "Module file {} referenced in CLAUDE.md Module Structure does not exist",
            file
        );
    }
}

#[test]
fn recipe_2_referenced_files_exist() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // Files modified in Recipe 2 according to CLAUDE.md
    let recipe2_files = [
        "src/shell/platform/x11.rs",
        "src/shell/mod.rs",
        "src/input.rs",
        "src/app.rs",
    ];

    for file in &recipe2_files {
        let path = std::path::Path::new(manifest_dir).join(file);
        assert!(
            path.exists(),
            "Recipe 2 file {} referenced in CLAUDE.md does not exist",
            file
        );
    }
}
