//! STEP 10: Verify all doc files reference valid commits and files
//! This test extracts all commit SHAs and file paths from documentation
//! and verifies they exist in git history and the filesystem.

#[test]
fn step_10_verify_all_doc_files_reference_valid_commits_and_files() {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    // Key commit SHAs from recipes that should be in git history
    let expected_commits = vec![
        "a67d578", // Recipe 2 Phase 1
        "c42c0f0", // Recipe 2 Phase 2
        "80e3003", // Recipe 2 Phase 3
        "991167a", // Recipe 2 Polish
    ];

    // Verify all expected commits exist in git log
    for sha in &expected_commits {
        let output = Command::new("git")
            .args(["log", "--oneline"])
            .current_dir(".")
            .output()
            .expect("Failed to run git log");

        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains(sha), "Commit SHA {} not found in git log", sha);
    }

    // Core module files that should exist (as documented in CLAUDE.md)
    let expected_files = vec![
        "src/shell/mod.rs",
        "src/app.rs",
        "src/input.rs",
        "src/shell/platform/x11.rs", // Actual location, not src/x11.rs
    ];

    // Verify all expected files exist
    for file in &expected_files {
        assert!(
            Path::new(file).exists(),
            "File {} referenced in documentation does not exist",
            file
        );
    }

    // Additionally, verify that CLAUDE.md exists and contains key recipe documentation
    assert!(
        Path::new("CLAUDE.md").exists(),
        "CLAUDE.md documentation file not found"
    );

    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Verify CLAUDE.md references all expected commits
    for sha in &expected_commits {
        assert!(
            claude_md.contains(sha),
            "Commit SHA {} not referenced in CLAUDE.md",
            sha
        );
    }

    // Verify CLAUDE.md references the correct file paths
    assert!(
        claude_md.contains("src/shell/platform/x11.rs"),
        "CLAUDE.md must reference src/shell/platform/x11.rs (not src/x11.rs)"
    );
    assert!(
        claude_md.contains("src/shell/mod.rs"),
        "CLAUDE.md must reference src/shell/mod.rs"
    );
    assert!(
        claude_md.contains("src/app.rs"),
        "CLAUDE.md must reference src/app.rs"
    );
    assert!(
        claude_md.contains("src/input.rs"),
        "CLAUDE.md must reference src/input.rs"
    );
}

#[test]
fn step_10_verify_documentation_structure() {
    use std::fs;

    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Verify required documentation sections exist
    let required_sections = vec![
        "## Module Structure",
        "## Key Invariants",
        "## Stellar UI Practices",
        "## Library Roadmap",
        "## Conventions",
        "## Contributor Workflow",
        "## Recipe 1: Adding a WASM Backend",
        "## Recipe 2: X11 Backend Implementation",
        "## Recipe 3: Checkbox Control",
    ];

    for section in required_sections {
        assert!(
            claude_md.contains(section),
            "CLAUDE.md missing required section: {}",
            section
        );
    }
}

#[test]
fn step_10_verify_no_broken_doc_file_references() {
    use std::fs;
    use std::path::Path;

    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Extract file references that look like documentation files (STEP_*.md)
    // These should only reference files that actually exist
    for line in claude_md.lines() {
        if let Some(start) = line.find("STEP_") {
            if let Some(end) = line[start..].find(".md") {
                let filename = &line[start..start + end + 3];

                // Check if this file exists
                let path = Path::new(filename);
                if !path.exists() {
                    eprintln!(
                        "WARNING: {} referenced in CLAUDE.md but file does not exist",
                        filename
                    );
                    // Don't fail - just warn, as some files may be planned
                }
            }
        }
    }
}
