#![doc = "Parse CLAUDE.md recipes and verify commit references are grounded in git history"]

//! Verify that Recipe sections in CLAUDE.md correctly document their commit history.
//! This test ensures recipe documentation doesn't drift from reality by checking that:
//! 1. All claimed commits exist in git history
//! 2. Line counts match the actual file state at each commit
//! 3. All recipes (1, 2, 3) are correctly documented

use rui::recipe_verification::parse_recipes_from_claude_md;
use std::process::Command;

#[test]
fn parse_recipe_2_commits() {
    let recipes = parse_recipes_from_claude_md();

    // Find Recipe 2
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    println!("Recipe 2: {}", recipe_2.title);
    println!("parsed {} commits:", recipe_2.commits.len());

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
fn recipe_structure_complete() {
    let recipes = parse_recipes_from_claude_md();

    println!("Validating recipe structure completeness:");
    println!("  Found {} recipes", recipes.len());

    for recipe in &recipes {
        println!(
            "  Recipe {}: {} (status: {:?})",
            recipe.number,
            recipe.title,
            recipe.status.as_ref().map(|s| &s[..20.min(s.len())])
        );

        assert!(
            !recipe.title.is_empty(),
            "Recipe {} has empty title",
            recipe.number
        );

        // Recipe 1 and 2 should have purposes; Recipe 3 may have status instead
        if recipe.number <= 2 {
            assert!(
                !recipe.purpose.is_empty(),
                "Recipe {} should have a purpose",
                recipe.number
            );
        }

        // Only Recipe 2 has actual commit history in CLAUDE.md
        if recipe.number == 2 {
            assert!(
                !recipe.commits.is_empty(),
                "Recipe 2 should have commits documented"
            );

            for commit in &recipe.commits {
                assert!(
                    !commit.sha.is_empty(),
                    "Recipe 2 {} commit should have SHA",
                    commit.phase
                );
                assert!(
                    commit.claimed_lines.is_some(),
                    "Recipe 2 {} commit should have line count",
                    commit.phase
                );
            }
        }
    }

    assert_eq!(
        recipes.len(),
        3,
        "CLAUDE.md should document exactly 3 recipes"
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
        "Expected to find all 3 recipes (WASM, X11, Checkbox)"
    );
}

#[test]
fn recipe_1_has_metadata() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_1 = recipes
        .iter()
        .find(|r| r.number == 1)
        .expect("Recipe 1 not found");

    assert!(!recipe_1.title.is_empty(), "Recipe 1 should have a title");
    assert!(
        recipe_1.title.contains("WASM"),
        "Recipe 1 should be about WASM"
    );
}

#[test]
fn recipe_3_has_metadata() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_3 = recipes
        .iter()
        .find(|r| r.number == 3)
        .expect("Recipe 3 not found");

    assert!(!recipe_3.title.is_empty(), "Recipe 3 should have a title");
    assert!(
        recipe_3.title.contains("Checkbox"),
        "Recipe 3 should be about Checkbox"
    );
}

#[test]
fn recipe_2_polish_sha_exists_in_git() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let polish = recipe_2
        .commits
        .iter()
        .find(|c| c.phase == "Polish")
        .expect("Recipe 2 Polish phase not found");

    verify_commit_exists(&polish.sha);
}

#[test]
fn recipe_2_has_all_four_phases() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let phase_1_found = recipe_2.commits.iter().any(|c| c.phase.contains("Phase 1"));
    let phase_2_found = recipe_2.commits.iter().any(|c| c.phase.contains("Phase 2"));
    let phase_3_found = recipe_2.commits.iter().any(|c| c.phase.contains("Phase 3"));
    let polish_found = recipe_2.commits.iter().any(|c| c.phase == "Polish");

    assert!(phase_1_found, "Recipe 2 should have Phase 1");
    assert!(phase_2_found, "Recipe 2 should have Phase 2");
    assert!(phase_3_found, "Recipe 2 should have Phase 3");
    assert!(polish_found, "Recipe 2 should have Polish phase");
}

#[test]
fn recipe_2_claimed_line_counts_are_positive() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    for commit in &recipe_2.commits {
        assert!(
            commit.claimed_lines.unwrap_or(0) > 0,
            "Recipe 2 {} claimed line count should be positive",
            commit.phase
        );
    }
}

#[test]
fn recipe_2_claimed_line_counts_are_reasonable() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    for commit in &recipe_2.commits {
        let lines = commit.claimed_lines.unwrap_or(0);
        assert!(
            lines < 5000,
            "Recipe 2 {} claimed line count {} seems unreasonably large",
            commit.phase,
            lines
        );
    }
}

#[test]
fn recipe_2_line_counts_show_progression() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let mut prev_lines = 0;

    for commit in &recipe_2.commits {
        let lines = commit.claimed_lines.unwrap_or(0);
        assert!(
            lines >= prev_lines,
            "Recipe 2 {} line count {} should be >= previous {}",
            commit.phase,
            lines,
            prev_lines
        );
        prev_lines = lines;
    }
}

#[test]
fn recipe_2_phase_1_line_count_matches_git() {
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

    let actual_lines = get_file_line_count_at_commit(&phase_1.sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual_lines,
        phase_1.claimed_lines.unwrap_or(0),
        "Recipe 2 Phase 1 claimed {} lines but git shows {}",
        phase_1.claimed_lines.unwrap_or(0),
        actual_lines
    );
}

#[test]
fn recipe_2_phase_2_line_count_matches_git() {
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

    let actual_lines = get_file_line_count_at_commit(&phase_2.sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual_lines,
        phase_2.claimed_lines.unwrap_or(0),
        "Recipe 2 Phase 2 claimed {} lines but git shows {}",
        phase_2.claimed_lines.unwrap_or(0),
        actual_lines
    );
}

#[test]
fn recipe_2_phase_3_line_count_matches_git() {
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

    let actual_lines = get_file_line_count_at_commit(&phase_3.sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual_lines,
        phase_3.claimed_lines.unwrap_or(0),
        "Recipe 2 Phase 3 claimed {} lines but git shows {}",
        phase_3.claimed_lines.unwrap_or(0),
        actual_lines
    );
}

#[test]
fn recipe_2_polish_line_count_matches_git() {
    let recipes = parse_recipes_from_claude_md();
    let recipe_2 = recipes
        .iter()
        .find(|r| r.number == 2)
        .expect("Recipe 2 not found");

    let polish = recipe_2
        .commits
        .iter()
        .find(|c| c.phase == "Polish")
        .expect("Recipe 2 Polish phase not found");

    let actual_lines = get_file_line_count_at_commit(&polish.sha, "src/shell/platform/x11.rs");
    assert_eq!(
        actual_lines,
        polish.claimed_lines.unwrap_or(0),
        "Recipe 2 Polish claimed {} lines but git shows {}",
        polish.claimed_lines.unwrap_or(0),
        actual_lines
    );
}

/// Get the line count of a file at a specific git commit
fn get_file_line_count_at_commit(sha: &str, file_path: &str) -> usize {
    let output = Command::new("git")
        .args(["show", &format!("{}:{}", sha, file_path)])
        .output()
        .expect("Failed to run git show");

    if !output.status.success() {
        return 0;
    }

    let content = String::from_utf8_lossy(&output.stdout);
    content.lines().count()
}
