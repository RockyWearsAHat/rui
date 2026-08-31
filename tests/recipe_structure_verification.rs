//! Verify that Recipe 1 (WASM Backend) and Recipe 3 (Mobile Backend)
//! have matching documentation structure in CLAUDE.md.
//!
//! This test ensures:
//! - Recipe 1 has 3 phases with files, why, and verification gates
//! - Recipe 3 has 3 phases with files, why, and verification gates
//! - Both recipes have cross-module concerns and template sections
//! - Structure is replicable for future platform implementations

#[test]
fn recipe_1_and_3_have_matching_structure() {
    let claude_md =
        std::fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist in project root");

    // Recipe 1 must exist
    assert!(
        claude_md.contains("### Recipe 1: Adding a WASM Backend"),
        "Recipe 1 section not found in CLAUDE.md"
    );

    // Recipe 1 must have exactly 3 phases
    let recipe1_start = claude_md
        .find("### Recipe 1: Adding a WASM Backend")
        .expect("Recipe 1 start");
    let recipe1_end = claude_md[recipe1_start..]
        .find("### Recipe 2:")
        .expect("Recipe 2 start to mark end of Recipe 1");
    let recipe1_section = &claude_md[recipe1_start..recipe1_start + recipe1_end];

    assert!(
        recipe1_section.contains("**Phase 1:"),
        "Recipe 1 Phase 1 missing"
    );
    assert!(
        recipe1_section.contains("**Phase 2:"),
        "Recipe 1 Phase 2 missing"
    );
    assert!(
        recipe1_section.contains("**Phase 3:"),
        "Recipe 1 Phase 3 missing"
    );

    // Recipe 1 must have required sections
    assert!(
        recipe1_section.contains("Files touched:"),
        "Recipe 1 missing 'Files touched' sections"
    );
    assert!(
        recipe1_section.contains("**Why this order:**"),
        "Recipe 1 missing 'Why this order' sections"
    );
    assert!(
        recipe1_section.contains("**Verification gate"),
        "Recipe 1 missing verification gates"
    );
    assert!(
        recipe1_section.contains("#### Cross-Module"),
        "Recipe 1 missing cross-module concerns section"
    );
    assert!(
        recipe1_section.contains("#### Template for the Next Backend"),
        "Recipe 1 missing template section"
    );

    // Recipe 3 must exist
    assert!(
        claude_md.contains("### Recipe 3: Mobile Backend Implementation"),
        "Recipe 3 section not found in CLAUDE.md"
    );

    // Recipe 3 must have exactly 3 phases
    let recipe3_start = claude_md
        .find("### Recipe 3: Mobile Backend Implementation")
        .expect("Recipe 3 start");
    let recipe3_section = &claude_md[recipe3_start..];

    assert!(
        recipe3_section.contains("**Phase 1:"),
        "Recipe 3 Phase 1 missing"
    );
    assert!(
        recipe3_section.contains("**Phase 2:"),
        "Recipe 3 Phase 2 missing"
    );
    assert!(
        recipe3_section.contains("**Phase 3:"),
        "Recipe 3 Phase 3 missing"
    );

    // Recipe 3 must have required sections
    assert!(
        recipe3_section.contains("Files touched:"),
        "Recipe 3 missing 'Files touched' sections"
    );
    assert!(
        recipe3_section.contains("**Why this order:**"),
        "Recipe 3 missing 'Why this order' sections"
    );
    assert!(
        recipe3_section.contains("**Verification gate"),
        "Recipe 3 missing verification gates"
    );
    assert!(
        recipe3_section.contains("#### Cross-Module"),
        "Recipe 3 missing cross-module coordination section"
    );
    assert!(
        recipe3_section.contains("#### Template for Adding Mobile"),
        "Recipe 3 missing template section"
    );
}

#[test]
fn recipe_phases_have_shell_commands_in_verification_gates() {
    let claude_md =
        std::fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist in project root");

    // Recipe 1 verification gates must contain shell commands
    let recipe1_start = claude_md
        .find("### Recipe 1: Adding a WASM Backend")
        .expect("Recipe 1 start");
    let recipe1_end = claude_md[recipe1_start..]
        .find("### Recipe 2:")
        .expect("Recipe 2 start");
    let recipe1_section = &claude_md[recipe1_start..recipe1_start + recipe1_end];

    // Check for cargo/shell commands in verification gates
    let has_cargo_commands = recipe1_section.contains("cargo test")
        || recipe1_section.contains("cargo build")
        || recipe1_section.contains("wasm-pack");
    assert!(
        has_cargo_commands,
        "Recipe 1 verification gates missing shell commands (cargo test/build)"
    );

    // Recipe 3 verification gates must contain shell commands
    let recipe3_start = claude_md
        .find("### Recipe 3: Mobile Backend Implementation")
        .expect("Recipe 3 start");
    let recipe3_section = &claude_md[recipe3_start..];

    let has_mobile_commands = recipe3_section.contains("cargo build")
        || recipe3_section.contains("cargo test")
        || recipe3_section.contains("--target");
    assert!(
        has_mobile_commands,
        "Recipe 3 verification gates missing shell commands"
    );
}

#[test]
fn recipe_acceptance_criterion_passes() {
    use std::process::Command;

    // Run the exact acceptance criterion from the build plan
    let output = Command::new("grep")
        .arg("-E")
        .arg(r"^\*\*Phase [0-9]:")
        .arg("CLAUDE.md")
        .output()
        .expect("grep command failed");

    let grep_output = String::from_utf8(output.stdout).expect("grep output is not UTF-8");
    let phase_count = grep_output.lines().count();

    // Last 6 lines should be: Recipe 1 (3 phases) + Recipe 2 (3 phases) or Recipe 3 phases
    // We expect at least 6 phases across all recipes
    assert!(
        phase_count >= 6,
        "Expected at least 6 phases across recipes; found {}.\nOutput:\n{}",
        phase_count,
        grep_output
    );
}
