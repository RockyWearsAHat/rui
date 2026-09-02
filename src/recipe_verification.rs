//! Recipe verification utilities for CLAUDE.md recipe documentation.
//!
//! This module provides parsing and validation of recipe documentation,
//! ensuring that claimed commits and line counts remain grounded in git history.

use std::fs;

/// Represents a single commit entry from a recipe's Commit list section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCommit {
    /// Phase name (e.g., "Phase 1", "Phase 2", "Polish")
    pub phase: String,
    /// Git commit SHA
    pub sha: String,
    /// Commit message
    pub message: String,
    /// Claimed line count at this commit
    pub claimed_lines: Option<usize>,
}

/// Represents a parsed recipe with its commit list and metadata
#[derive(Debug, Clone)]
pub struct ParsedRecipe {
    /// Recipe number (1, 2, 3)
    pub number: usize,
    /// Recipe title (e.g., "WASM Backend")
    pub title: String,
    /// Recipe purpose description
    pub purpose: String,
    /// Optional status (e.g., "Complete")
    pub status: Option<String>,
    /// List of commits documented in the recipe
    pub commits: Vec<RecipeCommit>,
}

/// Parse CLAUDE.md and extract all recipes with their metadata and commit lists
///
/// # Panics
/// Panics if CLAUDE.md cannot be read.
pub fn parse_recipes_from_claude_md() -> Vec<ParsedRecipe> {
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
