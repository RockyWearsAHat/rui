//! STEP 1: Verify all public types in accessibility.rs have doc comments.
//!
//! This test ensures that all 8 public types (Role, AccessState, AccessActions,
//! AccessNode, AccessTree, AccessUpdate, Violation, Fault) in the accessibility
//! module have documentation comments explaining their purpose, role, and usage.

use std::fs;

#[test]
fn accessibility_types_have_doc_comments() {
    let content =
        fs::read_to_string("src/accessibility.rs").expect("Could not read src/accessibility.rs");

    // The 8 public types that need documentation
    let types_to_check = vec![
        "Role",
        "AccessState",
        "AccessActions",
        "AccessNode",
        "AccessTree",
        "AccessUpdate",
        "Violation",
        "Fault",
    ];

    for type_name in types_to_check {
        // Find the line with the type definition
        let lines: Vec<&str> = content.lines().collect();
        let mut found_type = false;
        let mut has_doc = false;

        for i in 0..lines.len() {
            if lines[i].contains(&format!("pub enum {}", type_name))
                || lines[i].contains(&format!("pub struct {}", type_name))
            {
                found_type = true;
                // Check if there's a doc comment before it
                // Look backwards from current line to find /// comment
                for j in (0..i).rev() {
                    if lines[j].trim().starts_with("///") {
                        has_doc = true;
                        break;
                    }
                    // Stop if we hit another declaration
                    if lines[j].starts_with("pub ") && j != i {
                        break;
                    }
                }
                break;
            }
        }

        assert!(
            found_type,
            "Type {} not found in accessibility.rs",
            type_name
        );
        assert!(
            has_doc,
            "Type {} in accessibility.rs does not have doc comments",
            type_name
        );
    }
}

#[test]
fn accessibility_doc_comments_not_empty() {
    let content =
        fs::read_to_string("src/accessibility.rs").expect("Could not read src/accessibility.rs");

    let types = vec![
        "Role",
        "AccessState",
        "AccessActions",
        "AccessNode",
        "AccessTree",
        "AccessUpdate",
        "Violation",
        "Fault",
    ];

    for type_name in types {
        let lines: Vec<&str> = content.lines().collect();

        for i in 0..lines.len() {
            if lines[i].contains(&format!("pub enum {}", type_name))
                || lines[i].contains(&format!("pub struct {}", type_name))
            {
                // Look backwards for doc comments with substantial content
                let mut doc_lines = Vec::new();
                for j in (0..i).rev() {
                    let line = lines[j].trim();
                    if line.starts_with("///") {
                        doc_lines.push(line);
                    } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with("#[")
                    {
                        break;
                    }
                }

                // Check if we found any doc comments with actual content
                let has_substantial_doc = doc_lines.iter().any(|line| {
                    let content = line.trim_start_matches("///").trim();
                    !content.is_empty()
                });

                assert!(
                    has_substantial_doc,
                    "Type {} must have doc comments with actual content",
                    type_name
                );
                break;
            }
        }
    }
}
