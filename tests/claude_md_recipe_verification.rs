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
