#![doc = "Verify Recipe 2 commit SHAs exist in git history"]

//! Verify that all Recipe 2 commit SHAs referenced in CLAUDE.md
//! actually exist in the git repository history.
//!
//! This test ensures the documented commits are real and can be checked out.

use std::fs;
use std::process::Command;

#[test]
fn recipe_2_commits_exist() {
    let claude_md = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");

    // Extract Recipe 2 commits from CLAUDE.md
    let commits = extract_recipe_2_commits(&claude_md);

    println!("Checking {} Recipe 2 commits...", commits.len());

    let mut found_count = 0;
    for (phase, sha) in &commits {
        let output = Command::new("git")
            .args(["rev-parse", sha])
            .output()
            .unwrap_or_else(|_| panic!("Failed to run git rev-parse for {}", sha));

        if output.status.success() {
            println!("✓ {} commit exists: {}", phase, sha);
            found_count += 1;
        } else {
            panic!("✗ {} commit not found in git history: {}", phase, sha);
        }
    }

    println!("{}/{} commits found", found_count, commits.len());
    assert_eq!(
        found_count,
        commits.len(),
        "Not all Recipe 2 commits found in git history"
    );
}

fn extract_recipe_2_commits(text: &str) -> Vec<(String, String)> {
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
        if line.contains("- Commit:") && line.contains("`") {
            let mut sha = String::new();

            // Extract SHA from backticks
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    sha = line[start + 1..start + 1 + end].to_string();
                }
            }

            if !sha.is_empty() {
                commits.push((phases[phase_index].to_string(), sha));
                phase_index += 1;
            }
        }
        i += 1;
    }

    commits
}
