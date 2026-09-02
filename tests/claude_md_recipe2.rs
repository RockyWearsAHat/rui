#![doc = "Verify recipe references in CLAUDE.md are grounded in reality"]

//! Verify that recipe section references in CLAUDE.md
//! are grounded in real git history and actual codebase facts.
//!
//! This test ensures CLAUDE.md doesn't reference fabricated commits, files, or line numbers.
//! It guards against documentation drift by checking key claims against the source of truth.
//!
//! Note on Recipe 1 (WASM) and Recipe 3 (Checkbox):
//! These recipes are documented as templates/patterns without specific commit lists.
//! Their purpose is to exemplify implementation patterns that are replicable.
//! Only Recipe 2 (X11 Backend) has a "Commit list" section with real git SHAs to verify.
use std::fs;
use std::path::Path;

#[test]
fn recipe_2_claude_md_references_existing_x11_backend() {
    // Recipe 2 claims to reference X11 backend implementation
    let x11_path = "src/shell/platform/x11.rs";
    assert!(
        Path::new(x11_path).exists(),
        "Recipe 2 references X11 backend at src/shell/platform/x11.rs, but file not found"
    );
}

#[test]
fn recipe_2_claude_md_mentions_backend_trait_methods() {
    // Recipe 2 claims Backend trait has 12 core methods and references src/shell/mod.rs line 183+
    let shell_mod =
        fs::read_to_string("src/shell/mod.rs").expect("Failed to read src/shell/mod.rs");

    // Verify Backend trait exists
    assert!(
        shell_mod.contains("trait Backend"),
        "Recipe 2 references Backend trait in src/shell/mod.rs, but trait not found"
    );

    // Verify key Backend trait methods exist
    let required_methods = [
        "fn open",
        "fn pump",
        "fn surface",
        "fn appearance",
        "fn present",
        "fn is_open",
        "fn is_fullscreen",
        "fn set_fullscreen",
        "fn clipboard_text",
        "fn set_clipboard_text",
        "fn set_composition_area",
        "fn update_accessibility",
    ];

    for method in &required_methods {
        assert!(
            shell_mod.contains(method),
            "Recipe 2 claims Backend trait has method '{}', but not found in src/shell/mod.rs",
            method
        );
    }
}

#[test]
fn recipe_2_claude_md_references_correct_verification_tests() {
    // Recipe 2 mentions cargo test commands for verification gates
    // Verify the test files referenced actually exist

    // Phase 1: Basic compilation should work
    assert!(
        Path::new("src").exists(),
        "Recipe 2 references cargo build, but src/ directory not found"
    );

    // Phase 2: x11_integration tests should exist or be achievable
    // This test verifies the structure is in place for such tests
    assert!(
        Path::new("tests").exists(),
        "Recipe 2 references test files, but tests/ directory not found"
    );
}

#[test]
fn recipe_2_claude_md_coordinate_contract_is_documented() {
    // Recipe 2 claims coordinate transformation: logical = device / scale_factor
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        claude_md.contains("logical_x = device_x / scale_factor"),
        "Recipe 2 documents coordinate transformation, should contain logical_x = device_x / scale_factor"
    );
}

#[test]
fn recipe_2_claude_md_event_translation_is_documented() {
    // Recipe 2 claims X11 event types are translated to rui Events
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let required_translations = [
        "X11 MotionNotify",
        "ButtonPress",
        "ButtonRelease",
        "KeyPress",
        "ConfigureNotify",
    ];

    for event_type in &required_translations {
        assert!(
            claude_md.contains(event_type),
            "Recipe 2 documents event translation for {}, but not found",
            event_type
        );
    }
}

#[test]
fn recipe_2_claude_md_phases_are_clearly_separated() {
    // Recipe 2 should document all three phases clearly
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let phases = [
        "### Phase 1: Foundation",
        "### Phase 2: Enhancement",
        "### Phase 3: Integration",
    ];

    for phase in &phases {
        assert!(
            claude_md.contains(phase),
            "Recipe 2 should document all three phases; missing: {}",
            phase
        );
    }
}

#[test]
fn recipe_2_claude_md_verification_gates_section_exists() {
    // Recipe 2 should document verification gates for each phase
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        claude_md.contains("### Verification Gates"),
        "Recipe 2 should have Verification Gates section"
    );

    // Verify it mentions the three phases
    assert!(
        claude_md.contains("Phase 1: Compilation Verification"),
        "Verification Gates should include Phase 1: Compilation Verification"
    );
    assert!(
        claude_md.contains("Phase 2: Integration Verification"),
        "Verification Gates should include Phase 2: Integration Verification"
    );
    assert!(
        claude_md.contains("Phase 3: Parity Verification"),
        "Verification Gates should include Phase 3: Parity Verification"
    );
}

#[test]
fn recipe_2_claude_md_key_contracts_documented() {
    // Recipe 2 should document coordinate transformation and event translation contracts
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        claude_md.contains("### Key Contracts"),
        "Recipe 2 should have Key Contracts section documenting transformation and translation rules"
    );
}

#[test]
fn recipe_2_claude_md_cross_module_concerns_identified() {
    // Recipe 2 should identify where platform interacts with other modules
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    assert!(
        claude_md.contains("### Cross-Module Concerns"),
        "Recipe 2 should identify cross-module concerns"
    );

    // Verify it mentions key modules
    let concerns = [
        "app.rs",       // Backend trait boundary
        "shell/mod.rs", // Platform selection
        "memory.rs",    // Focus and interaction state
        "input.rs",     // Event translation
        "paint.rs",     // Pixel buffer
    ];

    for module in &concerns {
        assert!(
            claude_md.contains(module),
            "Recipe 2 cross-module concerns should mention {}",
            module
        );
    }
}

#[test]
fn parse_recipe_2_commits() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let commits = parse_recipe_2_commits_from_claude_md(&claude_md);

    println!("parsed {} commits:", commits.len());
    for (i, commit) in commits.iter().enumerate() {
        println!("  Phase {} ({}): {} lines", i + 1, commit.sha, commit.lines);
    }

    assert_eq!(
        commits.len(),
        4,
        "Recipe 2 should have exactly 4 commits (Phase 1, 2, 3, Polish), found {}",
        commits.len()
    );

    // Verify all commits have SHAs
    for commit in &commits {
        assert!(
            !commit.sha.is_empty(),
            "Recipe 2 commit should have a SHA, but found empty"
        );
        assert!(
            commit.lines > 0,
            "Recipe 2 commit should have line count > 0, but found {}",
            commit.lines
        );
    }
}

#[test]
fn recipe_2_phase_1_line_count_matches_git_history() {
    let sha = "a67d578";
    let claimed_lines = 748;
    let actual = get_file_line_count(sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual, claimed_lines,
        "Phase 1 commit {} claims {} lines but git shows {}",
        sha, claimed_lines, actual
    );
}

#[test]
fn recipe_2_phase_2_line_count_matches_git_history() {
    let sha = "c42c0f0";
    let claimed_lines = 1220;
    let actual = get_file_line_count(sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual, claimed_lines,
        "Phase 2 commit {} claims {} lines but git shows {}",
        sha, claimed_lines, actual
    );
}

#[test]
fn recipe_2_phase_3_line_count_matches_git_history() {
    let sha = "80e3003";
    let claimed_lines = 1321;
    let actual = get_file_line_count(sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual, claimed_lines,
        "Phase 3 commit {} claims {} lines but git shows {}",
        sha, claimed_lines, actual
    );
}

#[test]
fn recipe_2_polish_line_count_matches_git_history() {
    let sha = "991167a";
    let claimed_lines = 1368;
    let actual = get_file_line_count(sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual, claimed_lines,
        "Polish commit {} claims {} lines but git shows {}",
        sha, claimed_lines, actual
    );
}

fn parse_recipe_2_commits_from_claude_md(text: &str) -> Vec<RecipeCommit> {
    // Find Recipe 2 section
    let recipe_2_start = match text.find("## Recipe 2: X11 Backend Implementation") {
        Some(pos) => pos,
        None => return Vec::new(),
    };

    // Find the next recipe section or end of document
    let recipe_2_end = text[recipe_2_start..]
        .find("## Recipe 3:")
        .map(|pos| recipe_2_start + pos)
        .unwrap_or(text.len());

    let recipe_2_section = &text[recipe_2_start..recipe_2_end];

    // Find the "Commit list" section
    let commit_list_start = match recipe_2_section.find("### Commit list") {
        Some(pos) => recipe_2_start + pos,
        None => return Vec::new(),
    };

    // Extract up to the next ### heading or end of recipe
    let commit_section_text = &text[commit_list_start..recipe_2_end];
    let commit_section_end = commit_section_text
        .find("\n### ")
        .unwrap_or(commit_section_text.len());
    let commit_list_text = &commit_section_text[..commit_section_end];

    let mut commits = Vec::new();
    let lines: Vec<&str> = commit_list_text.lines().collect();
    let mut i = 0;

    // Parse commit entries: look for "- Commit: `SHA`" followed by "- Lines: NNN"
    while i < lines.len() {
        let line = lines[i];
        if line.contains("- Commit:") && line.contains("`") {
            let mut sha = String::new();
            let mut line_count = 0;

            // Extract SHA from backticks
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    sha = line[start + 1..start + 1 + end].to_string();
                }
            }

            // Look for the following "- Lines:" entry within the next 3 lines
            for check_line in &lines[(i + 1)..std::cmp::min(i + 4, lines.len())] {
                if check_line.contains("- Lines:") || check_line.contains("- Lines ") {
                    // Extract number from "- Lines: 748 ..." or "- Lines: 748 (...)"
                    let after_colon = if let Some(pos) = check_line.find(':') {
                        &check_line[pos + 1..]
                    } else {
                        check_line
                    };
                    let words: Vec<&str> = after_colon.split_whitespace().collect();
                    if !words.is_empty() {
                        if let Ok(n) = words[0].parse::<usize>() {
                            line_count = n;
                        }
                    }
                    break;
                }
            }

            if !sha.is_empty() && line_count > 0 {
                commits.push(RecipeCommit {
                    sha,
                    lines: line_count,
                });
            }
        }
        i += 1;
    }

    commits
}

struct RecipeCommit {
    sha: String,
    lines: usize,
}

#[test]
fn parse_recipe_1_has_no_commits() {
    // Recipe 1 (WASM) is documented as a template/pattern, not a concrete feature implementation.
    // It should have no "Commit list" section with real git SHAs.
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let recipe_1_start = match claude_md.find("## Recipe 1: Adding a WASM Backend") {
        Some(pos) => pos,
        None => return, // Recipe 1 doesn't exist is fine — test passes vacuously
    };

    let recipe_1_end = claude_md[recipe_1_start..]
        .find("## Recipe 2:")
        .map(|pos| recipe_1_start + pos)
        .unwrap_or(claude_md.len());

    let recipe_1_section = &claude_md[recipe_1_start..recipe_1_end];

    // Verify Recipe 1 has no "### Commit list" section (which would indicate real commits)
    assert!(
        !recipe_1_section.contains("### Commit list"),
        "Recipe 1 should be a template without a 'Commit list' section"
    );
}

#[test]
fn parse_recipe_3_has_no_commits() {
    // Recipe 3 (Checkbox) is documented as a control exemplar/pattern, not a concrete feature.
    // It should have no "Commit list" section with real git SHAs.
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    let recipe_3_start = match claude_md.find("## Recipe 3: Checkbox Control") {
        Some(pos) => pos,
        None => return, // Recipe 3 doesn't exist is fine — test passes vacuously
    };

    let recipe_3_end = recipe_3_start + (claude_md[recipe_3_start..].len());

    let recipe_3_section = &claude_md[recipe_3_start..recipe_3_end];

    // Verify Recipe 3 has no "### Commit list" section (which would indicate real commits)
    assert!(
        !recipe_3_section.contains("### Commit list"),
        "Recipe 3 should be an exemplar pattern without a 'Commit list' section"
    );
}

fn get_file_line_count(sha: &str, filepath: &str) -> usize {
    use std::process::Command;

    let output = Command::new("git")
        .args(["show", &format!("{}:{}", sha, filepath)])
        .output()
        .unwrap_or_else(|_| panic!("Failed to run git show for {}:{}", sha, filepath));

    if !output.status.success() {
        panic!(
            "git show failed for {}:{}: {}",
            sha,
            filepath,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let content = String::from_utf8(output.stdout).expect("git output not valid UTF-8");
    content.lines().count()
}
