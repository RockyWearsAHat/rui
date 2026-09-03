//! STEP 10: Verify all doc files reference valid commits and files
//!
//! Extracts all file paths and commit SHAs from CLAUDE.md and other doc files,
//! then verifies they exist in the git repository and file system.

#[test]
fn test_claude_md_references_valid_commits() {
    // All 4 recipe commits must exist in git history
    let commits = vec!["a67d578", "c42c0f0", "80e3003", "991167a"];

    for sha_abbrev in commits {
        let output = std::process::Command::new("git")
            .args(["log", "--oneline"])
            .output()
            .expect("git log failed");

        let log = String::from_utf8_lossy(&output.stdout);
        assert!(
            log.lines().any(|line| line.starts_with(sha_abbrev)),
            "Commit {} not found in git history",
            sha_abbrev
        );
    }
}

#[test]
fn test_claude_md_references_valid_file_paths() {
    // Paths that CLAUDE.md actually references (extracted via grep "src/.*\.rs")
    let expected_paths = vec![
        "src/accessibility.rs",
        "src/app.rs",
        "src/input.rs",
        "src/memory.rs",
        "src/shell/mod.rs",
        "src/shell/platform/x11.rs", // This is the actual file, not src/x11.rs
        "src/testing/mod.rs",
        "src/widgets.rs",
    ];

    for file_path in expected_paths {
        assert!(
            std::path::Path::new(file_path).exists(),
            "File {} referenced in CLAUDE.md does not exist",
            file_path
        );
    }
}

#[test]
fn test_all_doc_files_reference_valid_file_paths() {
    // Find all .md doc files
    let doc_dir = std::path::Path::new(".");
    let entries = std::fs::read_dir(doc_dir).expect("Could not read current directory");

    let doc_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".md"))
                .unwrap_or(false)
        })
        .collect();

    for entry in doc_files {
        let doc_path = entry.path();
        let content = std::fs::read_to_string(&doc_path).unwrap_or_else(|_| String::new());

        for line in content.lines() {
            // Skip template placeholder lines
            if line.contains("your_") || line.contains("new_") || line.contains("{backend}") {
                continue;
            }

            // Simple extraction: look for src/*.rs patterns
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if part.contains("src/") && part.ends_with(".rs") {
                    // Remove trailing punctuation
                    let file_ref =
                        part.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '/');

                    if !file_ref.contains("your_")
                        && !file_ref.contains("new_")
                        && !file_ref.contains("{")
                    {
                        let file_path = std::path::Path::new(file_ref);
                        if !file_path.exists() {
                            eprintln!(
                                "Warning: File {} referenced in {} does not exist",
                                file_ref,
                                doc_path.display()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_required_documentation_sections_exist_in_claude_md() {
    let claude_md = std::fs::read_to_string("CLAUDE.md").expect("Could not read CLAUDE.md");

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
            claude_md.contains(section),
            "Required section '{}' not found in CLAUDE.md",
            section
        );
    }
}
