//! STEP 2: Verify all public re-exports in lib.rs have documentation.
//!
//! This test ensures that every `pub use` statement in lib.rs is preceded by
//! a doc comment (///) that provides context for the re-exported item.

use std::fs;

#[test]
fn step_2_lib_rs_public_uses_have_doc_comments() {
    let content = fs::read_to_string("src/lib.rs").expect("Could not read src/lib.rs");
    let lines: Vec<&str> = content.lines().collect();

    let mut undocumented = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Match pub use statements (not pub mod)
        if trimmed.starts_with("pub use ") {
            // Check if the line above has a doc comment
            let has_doc = if i > 0 {
                // Look backwards for doc comment (can be multiple lines of ///)
                let mut j = i - 1;
                let mut found_doc = false;
                loop {
                    let prev_line = lines[j].trim();
                    if prev_line.starts_with("///") {
                        // Found a doc comment
                        found_doc = true;
                        break;
                    } else if prev_line.is_empty() {
                        // Empty line, keep looking
                        if j > 0 {
                            j -= 1;
                        } else {
                            break;
                        }
                    } else {
                        // Found something else (another statement, regular comment, etc)
                        break;
                    }
                }
                found_doc
            } else {
                false
            };

            if !has_doc {
                undocumented.push((i + 1, line.to_string()));
            }
        }
    }

    if !undocumented.is_empty() {
        let mut message = String::from("Undocumented pub use statements in lib.rs:\n");
        for (line_no, line) in undocumented {
            message.push_str(&format!("  Line {}: {}\n", line_no, line.trim()));
        }
        panic!("{}", message);
    }
}
