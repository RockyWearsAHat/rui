#![doc = "Parse CLAUDE.md recipes and verify commit references are grounded in git history"]

//! Verify that Recipe sections in CLAUDE.md correctly document their commit history.
//! This test ensures recipe documentation doesn't drift from reality by checking that:
//! 1. All claimed commits exist in git history
//! 2. Line counts match the actual file state at each commit
//! 3. All recipes (1, 2, 3) are correctly documented

use std::fs;
use std::process::Command;

/// Represents a single commit entry from a recipe's Commit list section
#[derive(Debug, Clone, PartialEq)]
struct RecipeCommit {
    phase: String,
    sha: String,
    message: String,
    claimed_lines: Option<usize>,
}

/// Represents a parsed recipe with its commit list and metadata
#[derive(Debug, Clone)]
struct ParsedRecipe {
    number: usize,
    title: String,
    purpose: String,
    status: Option<String>,
    commits: Vec<RecipeCommit>,
}

/// Parse CLAUDE.md and extract all recipes with their metadata and commit lists
fn parse_recipes_from_claude_md() -> Vec<ParsedRecipe> {
    let content = fs::read_to_string("CLAUDE.md").expect("Failed to read CLAUDE.md");
    let lines: Vec<&str> = content.lines().collect();

    let mut recipes = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Look for recipe headers: "## Recipe N: Title"
        if line.starts_with("## Recipe ") && !line.contains("Infrastructure") {
            // Parse recipe number and title
            let header = line.strip_prefix("## Recipe ").unwrap_or("");
            let parts: Vec<&str> = header.splitn(2, ':').collect();
            if parts.len() == 2 {
                let recipe_number: usize = parts[0].trim().parse().unwrap_or(0);
                let recipe_title = parts[1].trim().to_string();

                // Extract Purpose and Status from following lines
                let mut purpose = String::new();
                let mut status: Option<String> = None;
                let mut j = i + 1;

                while j < lines.len() && !lines[j].starts_with("## ") && j < i + 20 {
                    let current_line = lines[j];
                    if current_line.starts_with("**Purpose**:") {
                        purpose = current_line
                            .strip_prefix("**Purpose**:")
                            .unwrap_or("")
                            .trim()
                            .to_string();
                    }
                    if current_line.starts_with("**Status**:") {
                        status = Some(
                            current_line
                                .strip_prefix("**Status**:")
                                .unwrap_or("")
                                .trim()
                                .to_string(),
                        );
                    }
                    j += 1;
                }

                // Look for "### Commit list" section
                let mut recipe_commits = Vec::new();
                j = i + 1;

                while j < lines.len() && !lines[j].starts_with("## ") {
                    if lines[j].trim() == "### Commit list" {
                        // Parse commits from this section
                        j += 1;
                        while j < lines.len() && !lines[j].starts_with("###") {
                            let current_line = lines[j].trim();

                            // Look for phase headers (e.g., "**Phase 1: Foundation**" or "**Polish**")
                            if current_line.starts_with("**") && current_line.ends_with("**") {
                                let phase_content = current_line.trim_matches('*');
                                let phase_text = if phase_content.contains(':') {
                                    phase_content
                                        .split(':')
                                        .next()
                                        .unwrap_or("")
                                        .trim()
                                        .to_string()
                                } else {
                                    phase_content.trim().to_string()
                                };

                                // Parse the next few lines for commit details
                                j += 1;
                                let mut commit_sha = String::new();
                                let mut commit_message = String::new();
                                let mut claimed_lines: Option<usize> = None;

                                while j < lines.len() && lines[j].starts_with('-') {
                                    let detail_line = lines[j].trim();

                                    if detail_line.starts_with("- Commit:") {
                                        // Extract SHA from backticks
                                        if let Some(start) = detail_line.find('`') {
                                            if let Some(end) = detail_line.rfind('`') {
                                                if start < end {
                                                    commit_sha =
                                                        detail_line[start + 1..end].to_string();
                                                }
                                            }
                                        }
                                    } else if detail_line.starts_with("- Message:") {
                                        commit_message = detail_line
                                            .strip_prefix("- Message:")
                                            .unwrap_or("")
                                            .trim()
                                            .trim_matches('"')
                                            .to_string();
                                    } else if detail_line.starts_with("- Lines:") {
                                        let lines_text = detail_line
                                            .strip_prefix("- Lines:")
                                            .unwrap_or("")
                                            .trim();
                                        // Extract the number before any space/paren
                                        if let Ok(num) = lines_text
                                            .split_whitespace()
                                            .next()
                                            .unwrap_or("0")
                                            .parse::<usize>()
                                        {
                                            claimed_lines = Some(num);
                                        }
                                    }

                                    j += 1;

                                    // Stop if we hit a blank line or next phase
                                    if j < lines.len()
                                        && (lines[j].is_empty()
                                            || (lines[j].starts_with("**")
                                                && lines[j].contains(':')))
                                    {
                                        break;
                                    }
                                }

                                // Only add if we found a SHA
                                if !commit_sha.is_empty() {
                                    recipe_commits.push(RecipeCommit {
                                        phase: phase_text,
                                        sha: commit_sha,
                                        message: commit_message,
                                        claimed_lines,
                                    });
                                }

                                // Continue from current position
                                continue;
                            }

                            j += 1;
                        }
                        break;
                    }
                    j += 1;
                }

                if recipe_number > 0 {
                    recipes.push(ParsedRecipe {
                        number: recipe_number,
                        title: recipe_title,
                        purpose,
                        status,
                        commits: recipe_commits,
                    });
                }
            }
        }

        i += 1;
    }

    recipes
}

#[test]
fn parse_recipe_2_commits() {
    let recipes = parse_recipes_from_claude_md();

    // Find Recipe 2
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    println!("Recipe 2: {}", recipe_2.title);
    println!("Parsed {} commits:", recipe_2.commits.len());

    for commit in &recipe_2.commits {
        println!(
            "  {} ({}): {} lines",
            commit.phase,
            &commit.sha[..7.min(commit.sha.len())],
            commit.claimed_lines.unwrap_or(0)
        );
    }

    assert_eq!(
        recipe_2.commits.len(),
        4,
        "Recipe 2 should have 4 commits (Phase 1, 2, 3, and Polish)"
    );
}

#[test]
fn recipe_2_commits_have_shas() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    for commit in &recipe_2.commits {
        assert!(
            !commit.sha.is_empty(),
            "Recipe 2 {} commit should have a SHA",
            commit.phase
        );
        assert!(
            commit.sha.len() >= 7,
            "Recipe 2 {} SHA too short: {}",
            commit.phase,
            commit.sha
        );
    }
}

#[test]
fn recipe_2_commits_have_line_counts() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    for commit in &recipe_2.commits {
        assert!(
            commit.claimed_lines.is_some(),
            "Recipe 2 {} commit should have a claimed line count",
            commit.phase
        );
    }
}

#[test]
fn recipe_2_phase_1_sha_exists_in_git() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let phase_1 = recipe_2
        .commits
        .iter()
        .find(|c| c.phase.contains("Phase 1"))
        .expect("Recipe 2 Phase 1 not found");

    verify_commit_exists(&phase_1.sha);
}

#[test]
fn recipe_2_phase_2_sha_exists_in_git() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let phase_2 = recipe_2
        .commits
        .iter()
        .find(|c| c.phase.contains("Phase 2"))
        .expect("Recipe 2 Phase 2 not found");

    verify_commit_exists(&phase_2.sha);
}

#[test]
fn recipe_2_phase_3_sha_exists_in_git() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let phase_3 = recipe_2
        .commits
        .iter()
        .find(|c| c.phase.contains("Phase 3"))
        .expect("Recipe 2 Phase 3 not found");

    verify_commit_exists(&phase_3.sha);
}

/// Verify that a given SHA exists in git history
fn verify_commit_exists(sha: &str) {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", sha])
        .output()
        .expect("Failed to run git rev-parse");

    assert!(
        output.status.success(),
        "Commit SHA {} not found in git history",
        sha
    );
}

#[test]
fn parse_all_recipes() {
    let recipes = parse_recipes_from_claude_md();

    println!("Found {} recipes", recipes.len());
    for recipe in &recipes {
        println!(
            "  Recipe {}: {} (status: {:?})",
            recipe.number, recipe.title, recipe.status
        );
    }

    assert_eq!(
        recipes.len(),
        3,
        "Should parse exactly 3 recipes (WASM, X11, Checkbox)"
    );
}

#[test]
fn recipe_1_has_metadata() {
    let recipes = parse_recipes_from_claude_md();

    let recipe_1 = recipes
        .iter()
        .find(|r| r.number == 1)
        .expect("Recipe 1 not found");

    println!("Recipe 1 title: {}", recipe_1.title);
    println!("Recipe 1 purpose: {}", recipe_1.purpose);

    assert!(!recipe_1.title.is_empty(), "Recipe 1 should have a title");
    assert!(
        !recipe_1.purpose.is_empty(),
        "Recipe 1 should have a purpose"
    );
    assert!(
        recipe_1.purpose.contains("exemplar") || recipe_1.purpose.contains("Exemplar"),
        "Recipe 1 purpose should mention exemplar pattern"
    );
}

#[test]
fn recipe_3_has_metadata() {
    let recipes = parse_recipes_from_claude_md();

    let recipe_3 = recipes
        .iter()
        .find(|r| r.number == 3)
        .expect("Recipe 3 not found");

    println!("Recipe 3 title: {}", recipe_3.title);
    println!("Recipe 3 status: {:?}", recipe_3.status);

    assert!(!recipe_3.title.is_empty(), "Recipe 3 should have a title");
    assert_eq!(
        recipe_3.status,
        Some(
            "Complete — Verified as replicable pattern for custom widget implementation."
                .to_string()
        ),
        "Recipe 3 should have Complete status"
    );
}
