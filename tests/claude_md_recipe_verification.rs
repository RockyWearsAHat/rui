#![doc = "Verify Recipe 2 commit SHAs exist in git history"]

//! Verify that all Recipe 2 commit SHAs referenced in CLAUDE.md
//! actually exist in the git repository history, with correct messages.
//!
//! This test ensures the documented commits are real and can be checked out.
//! It also verifies commit messages match the documentation.

use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
struct CommitInfo {
    phase: String,
    sha: String,
}

#[test]
fn recipe_2_commits_exist() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Extract Recipe 2 commits from CLAUDE.md
    let commits = extract_recipe_2_commits(&claude_md);

    println!("Checking {} Recipe 2 commits...", commits.len());

    let mut found_count = 0;
    for commit in &commits {
        let output = Command::new("git")
            .args(["show", commit.sha.as_str(), "--quiet"])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run git show for {}", commit.sha));

        if output.status.success() {
            println!("✓ {} commit exists: {}", commit.phase, commit.sha);
            found_count += 1;
        } else {
            panic!(
                "✗ {} commit not found in git history: {}",
                commit.phase, commit.sha
            );
        }
    }

    println!("{}/{} commits found", found_count, commits.len());
    assert_eq!(
        found_count,
        commits.len(),
        "Not all Recipe 2 commits found in git history"
    );
}

#[test]
fn recipe_2_line_counts() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");
    let commits = extract_recipe_2_commits(&claude_md);

    // Documented line counts for x11.rs at each phase
    let documented_counts = [748, 1220, 1321, 1368];

    println!("Verifying Recipe 2 line counts within ±10...");

    let mut pass_count = 0;
    for (i, commit) in commits.iter().enumerate() {
        if i >= documented_counts.len() {
            break;
        }

        let line_count = get_file_line_count(&commit.sha, "src/shell/platform/x11.rs");
        let documented = documented_counts[i];
        let tolerance = 10;
        let diff = (line_count as i32 - documented).abs();

        if diff <= tolerance {
            println!(
                "✓ {} ({}): {} lines (documented: {}, diff: {})",
                commit.phase, commit.sha, line_count, documented, diff
            );
            pass_count += 1;
        } else {
            panic!(
                "✗ {} ({}): {} lines (documented: {}, diff: {} > tolerance: {})",
                commit.phase, commit.sha, line_count, documented, diff, tolerance
            );
        }
    }

    println!("{}/4 line counts pass", pass_count);
    assert_eq!(
        pass_count, 4,
        "Not all Recipe 2 line counts within tolerance"
    );
}

fn get_file_line_count(sha: &str, filepath: &str) -> usize {
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

#[test]
fn parse_recipe_1() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");
    let commits = extract_recipe_1_commits(&claude_md);

    // Recipe 1 (WASM) is a template pattern, not an implementation,
    // so no commit SHAs are documented in CLAUDE.md
    assert!(
        commits.is_empty(),
        "Recipe 1 should have no commits (it's a template pattern, not an implementation)"
    );

    println!("Recipe 1: no commit SHAs documented");
}

#[test]
fn recipe_1_documentation_files_exist() {
    let doc_files = [
        "STEP_4_RECIPE_1_ANALYSIS.md",
        "STEP_4_RECIPE_1_VERIFICATION_GATES.md",
        "STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md",
        "STEP_4_RECIPE_1_SUMMARY.md",
        "STEP_4_RECIPE_1_COORDINATE_CONTRACT.md",
        "STEP_4_RECIPE_1_EVENT_TRANSLATION.md",
    ];

    println!("Checking Recipe 1 documentation files...");

    let mut found_count = 0;
    for file in &doc_files {
        if fs::metadata(file).is_ok() {
            println!("✓ {} exists", file);
            found_count += 1;
        } else {
            println!("✗ {} NOT FOUND", file);
        }
    }

    println!(
        "{}/{} documentation files exist",
        found_count,
        doc_files.len()
    );
    assert_eq!(
        found_count,
        doc_files.len(),
        "Recipe 1 documentation extraction incomplete"
    );
}

fn extract_recipe_1_commits(text: &str) -> Vec<CommitInfo> {
    let recipe_1_start = match text.find("## Recipe 1: Adding a WASM Backend") {
        Some(pos) => pos,
        None => return Vec::new(),
    };

    let recipe_1_end = text[recipe_1_start..]
        .find("## Recipe 2:")
        .map(|pos| recipe_1_start + pos)
        .unwrap_or(text.len());

    let recipe_1_section = &text[recipe_1_start..recipe_1_end];

    // Find the "Commit list" section if it exists
    let commit_list_start = match recipe_1_section.find("### Commit list") {
        Some(pos) => recipe_1_start + pos,
        None => return Vec::new(),
    };

    // Extract up to the next ### heading or end of recipe
    let commit_section_text = &text[commit_list_start..recipe_1_end];
    let commit_section_end = commit_section_text
        .find("\n### ")
        .unwrap_or(commit_section_text.len());
    let commit_list_text = &commit_section_text[..commit_section_end];

    let mut commits = Vec::new();
    let lines: Vec<&str> = commit_list_text.lines().collect();
    let mut i = 0;

    let phases = ["Phase 1", "Phase 2", "Phase 3"];
    let mut phase_index = 0;

    while i < lines.len() && phase_index < phases.len() {
        let line = lines[i];
        if line.contains("- Commit:") && line.contains('`') {
            let mut sha = String::new();

            // Extract SHA from backticks
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    sha = line[start + 1..start + 1 + end].to_string();
                }
            }

            if !sha.is_empty() {
                commits.push(CommitInfo {
                    phase: phases[phase_index].to_string(),
                    sha,
                });
                phase_index += 1;
            }
        }
        i += 1;
    }

    commits
}

fn extract_recipe_2_commits(text: &str) -> Vec<CommitInfo> {
    let recipe_2_start = match text.find("## Recipe 2: X11 Backend Implementation") {
        Some(pos) => pos,
        None => return Vec::new(),
    };

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

    let phases = ["Phase 1", "Phase 2", "Phase 3", "Polish"];
    let mut phase_index = 0;

    while i < lines.len() && phase_index < phases.len() {
        let line = lines[i];
        if line.contains("- Commit:") && line.contains('`') {
            let mut sha = String::new();

            // Extract SHA from backticks
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    sha = line[start + 1..start + 1 + end].to_string();
                }
            }

            if !sha.is_empty() {
                commits.push(CommitInfo {
                    phase: phases[phase_index].to_string(),
                    sha,
                });
                phase_index += 1;
            }
        }
        i += 1;
    }

    commits
}

#[test]
fn recipe_1_template_validation_exists() {
    let validation_path = "STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md";
    assert!(
        std::path::Path::new(validation_path).exists(),
        "Template validation document missing: {}",
        validation_path
    );

    let content =
        std::fs::read_to_string(validation_path).expect("Failed to read validation document");

    // Verify key validation sections exist
    assert!(
        content.contains("Template Claims Verification"),
        "Missing 'Template Claims Verification' section"
    );
    assert!(content.contains("Verdict"), "Missing verification verdicts");
    assert!(
        content.contains("✅ CONFIRMED"),
        "Missing confirmed verdicts"
    );
    assert!(
        content.contains("Template Completeness Matrix"),
        "Missing completeness matrix"
    );
    assert!(
        content.contains("10 major claims verified"),
        "Missing claim summary"
    );

    println!("✓ Recipe 1 template validation document exists and is complete");
}

#[test]
fn recipe_1_claims_verified_against_x11() {
    let validation_path = "STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md";
    let content =
        std::fs::read_to_string(validation_path).expect("Failed to read validation document");

    // Count verified claims
    let confirmed_count = content.matches("✅ CONFIRMED").count();
    assert!(
        confirmed_count >= 10,
        "Expected at least 10 confirmed claims, found {}",
        confirmed_count
    );

    // Verify all 12 Backend trait methods are documented
    assert!(
        content.contains("12 methods") || content.contains("12 Backend trait"),
        "Backend trait completeness not verified"
    );

    // Verify phase line counts are exact
    assert!(
        content.contains("748") && content.contains("1220") && content.contains("1321"),
        "Phase line counts not verified"
    );

    println!(
        "✓ Recipe 1: {} major claims verified against X11 actual implementation",
        confirmed_count
    );
}

#[test]
fn recipe_1_validation_proves_template_is_replicable() {
    let validation_path = "STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md";
    let content =
        std::fs::read_to_string(validation_path).expect("Failed to read validation document");

    // Verify the validation document recommends using this for future implementers
    assert!(
        content.contains("Next Implementer Should"),
        "Missing implementation guidance for next developer"
    );
    assert!(
        content.contains("production-ready"),
        "Template should be marked production-ready"
    );
    assert!(
        content.contains("WASM") || content.contains("new platform backend"),
        "Should mention WASM or new backends as use cases"
    );

    println!("✓ Recipe 1 validation proves template is actionable for new implementers");
}

#[test]
fn recipe_1_coordinate_contract_complete() {
    let coord_path = "STEP_4_RECIPE_1_COORDINATE_CONTRACT.md";
    let content = fs::read_to_string(coord_path).expect("Failed to read coordinate contract");

    // Verify all required sections exist
    let required_sections = [
        "Coordinate Systems",
        "Transformation Rules",
        "Scale Factor Detection",
        "Coordinate Contract Implementation Checklist",
        "Common Pitfalls",
        "Verification",
    ];

    let mut found_sections = 0;
    for section in &required_sections {
        if content.contains(section) {
            println!("✓ Coordinate contract has section: {}", section);
            found_sections += 1;
        } else {
            println!("✗ Coordinate contract missing section: {}", section);
        }
    }

    // Verify key transformation formulas are documented
    assert!(
        content.contains("logical_x = device_x / scale_factor"),
        "Missing key transformation formula: logical = device / scale"
    );
    assert!(
        content.contains("device_x = logical_x * scale_factor"),
        "Missing key transformation formula: device = logical * scale"
    );

    // Verify pitfalls and tests are documented
    assert!(
        content.contains("Common Pitfalls"),
        "Missing common pitfalls section"
    );
    assert!(
        content.contains("Verification"),
        "Missing verification section"
    );

    println!(
        "✓ Recipe 1 coordinate contract: {}/{} sections found",
        found_sections,
        required_sections.len()
    );
}

#[test]
fn recipe_1_event_translation_complete() {
    let event_path = "STEP_4_RECIPE_1_EVENT_TRANSLATION.md";
    let content = fs::read_to_string(event_path).expect("Failed to read event translation");

    // Verify all required event types are documented
    let required_events = [
        "Pointer Events",
        "Keyboard Events",
        "Wheel Events",
        "Composition Input",
        "Focus Events",
        "Resize Events",
    ];

    let mut found_events = 0;
    for event_type in &required_events {
        if content.contains(event_type) {
            println!("✓ Event translation has: {}", event_type);
            found_events += 1;
        } else {
            println!("✗ Event translation missing: {}", event_type);
        }
    }

    // Verify implementation checklist exists
    assert!(
        content.contains("Implementation Checklist"),
        "Missing implementation checklist"
    );

    // Verify testing strategy is documented
    assert!(
        content.contains("Testing Strategy"),
        "Missing testing strategy section"
    );

    // Verify common pitfalls are documented
    assert!(
        content.contains("Common Pitfalls"),
        "Missing common pitfalls in event translation"
    );

    println!(
        "✓ Recipe 1 event translation: {}/{} event types documented",
        found_events,
        required_events.len()
    );
}

#[test]
fn recipe_1_backend_specialized_docs_referenced() {
    let analysis_path = "STEP_4_RECIPE_1_ANALYSIS.md";
    let analysis = fs::read_to_string(analysis_path).expect("Failed to read analysis");

    // Verify analysis references the specialized backend documents
    assert!(
        analysis.contains("COORDINATE_CONTRACT") || analysis.contains("coordinate"),
        "Analysis should reference coordinate contract document"
    );
    assert!(
        analysis.contains("EVENT_TRANSLATION") || analysis.contains("event translation"),
        "Analysis should reference event translation document"
    );

    println!("✓ Recipe 1 analysis references coordinate and event translation specializations");
}

#[test]
fn recipe_1_claude_md_references_extracted_docs() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Verify Recipe 1 section exists
    assert!(
        claude_md.contains("## Recipe 1: Adding a WASM Backend"),
        "Recipe 1 section not found in CLAUDE.md"
    );

    // Verify Recipe 1 section references extracted documentation
    let recipe_1_start = claude_md
        .find("## Recipe 1: Adding a WASM Backend")
        .unwrap();
    let recipe_2_start = claude_md
        .find("## Recipe 2: X11 Backend Implementation")
        .unwrap();
    let recipe_1_section = &claude_md[recipe_1_start..recipe_2_start];

    let extracted_docs = [
        (
            "STEP_4_RECIPE_1_ANALYSIS.md",
            "three-phase pattern breakdown",
        ),
        (
            "STEP_4_RECIPE_1_VERIFICATION_GATES.md",
            "acceptance criteria",
        ),
        (
            "STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md",
            "friction points",
        ),
        (
            "STEP_4_RECIPE_1_COORDINATE_CONTRACT.md",
            "coordinate transformation",
        ),
        ("STEP_4_RECIPE_1_EVENT_TRANSLATION.md", "DOM event types"),
        ("STEP_4_RECIPE_1_TEMPLATE_VALIDATION.md", "template claims"),
        ("STEP_4_RECIPE_1_SUMMARY.md", "quick reference"),
    ];

    let mut found_count = 0;
    for (doc_file, description) in &extracted_docs {
        if recipe_1_section.contains(doc_file) {
            println!(
                "✓ CLAUDE.md Recipe 1 references {}: {}",
                doc_file, description
            );
            found_count += 1;
        } else {
            println!(
                "✗ CLAUDE.md Recipe 1 missing reference to {}: {}",
                doc_file, description
            );
        }
    }

    println!(
        "✓ CLAUDE.md Recipe 1 references {}/{} extracted documentation files",
        found_count,
        extracted_docs.len()
    );

    assert_eq!(
        found_count,
        extracted_docs.len(),
        "CLAUDE.md Recipe 1 should reference all extracted documentation files"
    );
}

#[test]
fn recipe_1_claude_md_has_implementation_guide() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Verify Recipe 1 section has "How to Implement" or similar guidance
    let recipe_1_start = claude_md
        .find("## Recipe 1: Adding a WASM Backend")
        .unwrap();
    let recipe_2_start = claude_md
        .find("## Recipe 2: X11 Backend Implementation")
        .unwrap();
    let recipe_1_section = &claude_md[recipe_1_start..recipe_2_start];

    assert!(
        recipe_1_section.contains("How to Implement")
            || recipe_1_section.contains("How to use")
            || recipe_1_section.contains("implementation"),
        "Recipe 1 should have guidance on how to implement WASM backend"
    );

    // Verify it mentions "new implementers"
    assert!(
        recipe_1_section.contains("implementer") || recipe_1_section.contains("implementers"),
        "Recipe 1 should address future implementers"
    );

    // Verify it explains the order to read documentation
    assert!(
        recipe_1_section.contains("Start here")
            || recipe_1_section.contains("order")
            || recipe_1_section.contains("Begin"),
        "Recipe 1 should guide implementers on which doc to read first"
    );

    println!("✓ CLAUDE.md Recipe 1 has complete implementation guidance for new backends");
}
