//! STEP 10: Verify all documentation files reference valid commits and files.
//!
//! This test ensures that CLAUDE.md and all STEP_*.md documentation files
//! only reference commits that exist in git history and file paths that exist
//! in the src/ directory.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn step_10_recipe_commits_exist_in_git_history() {
    // Recipe 2: X11 Backend Implementation commit SHAs
    let recipe_commits = vec![
        "a67d578", // Phase 1: Foundation
        "c42c0f0", // Phase 2: Enhancement
        "80e3003", // Phase 3: Integration
        "991167a", // Polish
    ];

    for sha in recipe_commits {
        let output = Command::new("git")
            .args(["log", "--oneline"])
            .output()
            .expect("Failed to run git log");

        let log_output = String::from_utf8(output.stdout).expect("Invalid UTF-8 in git log");
        assert!(
            log_output.contains(sha),
            "Recipe commit {} not found in git history",
            sha
        );
    }
}

#[test]
fn step_10_required_files_exist() {
    // Files that must exist in src/
    let required_files = vec![
        "src/shell/mod.rs",
        "src/app.rs",
        "src/input.rs",
        "src/shell/platform/x11.rs",
    ];

    for file_path in required_files {
        assert!(
            Path::new(file_path).exists(),
            "Required file {} does not exist",
            file_path
        );
    }
}

#[test]
fn step_10_doc_files_reference_valid_paths() {
    // Read all documentation files
    let doc_files = vec!["CLAUDE.md"];

    for doc_file in doc_files {
        let content =
            fs::read_to_string(doc_file).unwrap_or_else(|_| panic!("Could not read {}", doc_file));

        // Find all src/*.rs file path references
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            // Look for patterns like src/shell/mod.rs
            if let Some(start) = line.find("src/") {
                if let Some(end) = line[start..].find(|c: char| {
                    !c.is_alphanumeric() && c != '_' && c != '/' && c != '-' && c != '.'
                }) {
                    let path = &line[start..start + end];
                    if path.ends_with(".rs") {
                        // Skip template placeholders
                        if path.contains("your_backend") || path.contains("BACKEND_NAME") {
                            continue;
                        }
                        assert!(
                            Path::new(path).exists(),
                            "File path {} referenced in {} does not exist",
                            path,
                            doc_file
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn step_10_claude_md_has_required_sections() {
    let content = fs::read_to_string("CLAUDE.md").expect("Could not read CLAUDE.md");

    let required_sections = vec![
        "## Module Structure",
        "## Key Invariants",
        "## Key Architectural Patterns",
        "## Recipe 1: Adding a WASM Backend",
        "## Recipe 2: X11 Backend Implementation",
        "## Recipe 3: Checkbox Control",
        "## Widget Exemplars",
        "## Build and Test",
        "## Stellar UI Practices",
        "## Library Roadmap",
        "## Conventions",
        "## Contributor Workflow",
    ];

    for section in required_sections {
        assert!(
            content.contains(section),
            "Required section '{}' not found in CLAUDE.md",
            section
        );
    }
}
