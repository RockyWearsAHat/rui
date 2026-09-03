#![allow(missing_docs)]

/// Audit test: verify all examples/ files are documented in CLAUDE.md
#[test]
fn all_examples_are_documented_in_claude_md() {
    use std::fs;

    let examples_dir = "examples";
    let claude_md = "CLAUDE.md";

    // List all .rs files in examples/
    let mut example_files: Vec<String> = fs::read_dir(examples_dir)
        .expect("Failed to read examples/ directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    example_files.sort();

    // Read CLAUDE.md and extract documented example names from the Examples Directory table
    let claude_content = fs::read_to_string(claude_md).expect("Failed to read CLAUDE.md");

    // Find the Examples Directory section
    let examples_section_start = claude_content
        .find("## Examples Directory")
        .expect("Examples Directory section not found");

    // Find the next section after Examples Directory (## or end of file)
    let rest_after_examples = &claude_content[examples_section_start + 20..];
    let examples_section_end = rest_after_examples
        .find("\n## ")
        .map(|idx| examples_section_start + 20 + idx)
        .unwrap_or(claude_content.len());

    let examples_section = &claude_content[examples_section_start..examples_section_end];

    // Extract example names from the table in this section (between the backticks in | `example` | ...)
    let mut documented_examples: Vec<String> = Vec::new();
    for line in examples_section.lines() {
        // Only process lines that look like table rows (start with |)
        if line.starts_with("| `") && line.contains("` |") {
            // Extract the name between the first pair of backticks
            if let Some(start) = line.find("| `") {
                if let Some(end) = line[start + 3..].find('`') {
                    let name = line[start + 3..start + 3 + end].to_string();
                    documented_examples.push(name);
                }
            }
        }
    }
    documented_examples.sort();

    // Find examples not in documentation
    let undocumented: Vec<_> = example_files
        .iter()
        .filter(|file| !documented_examples.contains(file))
        .collect();

    // Find documented examples that don't exist as files
    let missing_files: Vec<_> = documented_examples
        .iter()
        .filter(|doc| !example_files.contains(doc))
        .collect();

    // Report findings
    let mut failures = Vec::new();

    if !undocumented.is_empty() {
        failures.push(format!(
            "Undocumented examples (missing from CLAUDE.md table): {:?}",
            undocumented
        ));
    }

    if !missing_files.is_empty() {
        failures.push(format!(
            "Documented examples that don't exist: {:?}",
            missing_files
        ));
    }

    if !failures.is_empty() {
        panic!("Documentation audit failed:\n{}", failures.join("\n"));
    }
}
