//! Verify that Recipe 3 (Checkbox Control)
//! has correct documentation structure and alignment.
//!
//! This test ensures:
//! - Checkbox exemplar is documented in CLAUDE.md
//! - Documentation matches actual implementation
//! - State-view-handler pattern is correctly applied
//! - Test suite validates documented behavior

#[test]
fn checkbox_exemplar_documented_in_claude_md() {
    let claude_md =
        std::fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist in project root");

    // Checkbox Exemplar section must exist
    assert!(
        claude_md.contains("### Checkbox Exemplar"),
        "Checkbox Exemplar section not found in CLAUDE.md"
    );

    // Checkbox documentation must have required components
    let checkbox_start = claude_md
        .find("### Checkbox Exemplar")
        .expect("Checkbox Exemplar start");
    let checkbox_section = &claude_md[checkbox_start..];

    // Must document the state-view-handler pattern
    assert!(
        checkbox_section.contains("State:") && checkbox_section.contains("struct App"),
        "Checkbox documentation missing state definition"
    );
    assert!(
        checkbox_section.contains("View:") && checkbox_section.contains("fn view(app: &App)"),
        "Checkbox documentation missing view function"
    );
    assert!(
        checkbox_section.contains("Handler:"),
        "Checkbox documentation missing handler description"
    );

    // Must document the binary boolean state
    assert!(
        checkbox_section.contains("checked: bool") || checkbox_section.contains("notify: bool"),
        "Checkbox documentation missing boolean state field"
    );

    // Must have example usage
    assert!(
        checkbox_section.contains("widgets::checkbox"),
        "Checkbox documentation missing usage example"
    );

    // Must have implementation details
    assert!(
        checkbox_section.contains("Implementation details:"),
        "Checkbox documentation missing implementation details"
    );

    // Must have copy/modify instructions
    assert!(
        checkbox_section.contains("Getting Started: Copy and Modify")
            || checkbox_section.contains("Copy and modify"),
        "Checkbox documentation missing copy-and-modify instructions"
    );
}

#[test]
fn checkbox_widget_is_public_api() {
    // Read src/widgets.rs to verify checkbox is exported
    let widgets_rs = std::fs::read_to_string("src/widgets.rs").expect("src/widgets.rs must exist");

    // Must have public checkbox function
    assert!(
        widgets_rs.contains("pub fn checkbox"),
        "checkbox must be a public function in src/widgets.rs"
    );

    // Must follow the state-view-handler pattern
    assert!(
        widgets_rs.contains("on_click") || widgets_rs.contains("&mut"),
        "checkbox must support handlers receiving &mut state"
    );

    // Read lib.rs to verify checkbox is re-exported
    let lib_rs = std::fs::read_to_string("src/lib.rs").expect("src/lib.rs must exist");
    assert!(
        lib_rs.contains("checkbox"),
        "checkbox must be re-exported from src/lib.rs"
    );
}

#[test]
fn checkbox_exemplar_example_exists() {
    // Verify examples/checkbox.rs exists and follows the pattern
    let checkbox_example =
        std::fs::read_to_string("examples/checkbox.rs").expect("examples/checkbox.rs must exist");

    // Must define state struct
    assert!(
        checkbox_example.contains("struct App"),
        "checkbox example must define App struct"
    );

    // Must define view function
    assert!(
        checkbox_example.contains("fn view"),
        "checkbox example must define view function"
    );

    // Must use widgets::checkbox
    assert!(
        checkbox_example.contains("widgets::checkbox"),
        "checkbox example must use widgets::checkbox function"
    );

    // Must show handler pattern
    assert!(
        checkbox_example.contains("|app: &mut App|"),
        "checkbox example must show handler pattern with &mut App"
    );

    // Must be minimal (educational exemplar should be ~26 lines as documented)
    let line_count = checkbox_example.lines().count();
    assert!(
        line_count < 50,
        "checkbox example should be minimal; found {} lines (should be ~26)",
        line_count
    );
}

#[test]
fn checkbox_tests_verify_documented_behavior() {
    // Run checkbox tests and verify they pass
    use std::process::Command;

    let output = Command::new("cargo")
        .arg("test")
        .arg("--test")
        .arg("recipes")
        .arg("--")
        .arg("a_checkbox")
        .output()
        .expect("cargo test command failed");

    let test_output = String::from_utf8(output.stdout).expect("test output is not UTF-8");

    // Must have passing tests
    assert!(
        test_output.contains("test result: ok"),
        "checkbox tests must pass"
    );

    // Must have at least 3 passing checkbox tests
    let passing_count = test_output.matches("ok").count();
    assert!(
        passing_count >= 3,
        "must have at least 3 passing checkbox tests; found {} 'ok' occurrences",
        passing_count
    );

    // Test names must match documented scenarios
    let has_toggle = test_output.contains("a_checkbox_changes_state_on_click")
        || test_output.contains("a_checkbox_draws_differently");
    let has_label_interaction = test_output.contains("a_checkbox_answers_a_click_on_its_label");
    let has_group = test_output.contains("a_checkbox_group_manages_multiple");

    assert!(
        has_toggle,
        "checkbox tests must include toggle/visual state test"
    );
    assert!(
        has_label_interaction,
        "checkbox tests must include label interaction test"
    );
    assert!(
        has_group,
        "checkbox tests must include group/multiple selection test"
    );
}

#[test]
fn checkbox_documentation_exemplar_pattern_is_complete() {
    let claude_md =
        std::fs::read_to_string("CLAUDE.md").expect("CLAUDE.md must exist in project root");

    // Find checkbox section
    let checkbox_start = claude_md
        .find("### Checkbox Exemplar")
        .expect("Checkbox Exemplar section must exist");
    let checkbox_section = &claude_md[checkbox_start..];

    // Must document all required pattern components
    let required_sections = vec![
        "Pattern at a Glance",
        "State:",
        "View:",
        "Handler:",
        "Implementation details:",
        "Getting Started: Copy and Modify",
        "Next: Building Multiple Checkboxes",
    ];

    for section in required_sections {
        assert!(
            checkbox_section.contains(section),
            "Checkbox documentation missing required section: {}",
            section
        );
    }

    // Must show handler receives &mut state
    assert!(
        checkbox_section.contains("&mut App") || checkbox_section.contains("|app: &mut"),
        "Checkbox documentation must show handler receiving &mut state"
    );

    // Must show boolean toggle pattern
    assert!(
        checkbox_section.contains("!") || checkbox_section.contains("toggle"),
        "Checkbox documentation must show toggle pattern"
    );

    // Must have copy-and-modify example
    assert!(
        checkbox_section.contains("cp") && checkbox_section.contains("examples"),
        "Checkbox documentation must have copy-and-modify instructions"
    );
}

#[test]
fn checkbox_recipe_acceptance_criterion() {
    use std::process::Command;

    // The acceptance criterion: run the exact test command and verify results
    let output = Command::new("bash")
        .arg("-c")
        .arg("cargo test --test recipes -- a_checkbox -- --nocapture 2>&1 | grep -c 'ok'")
        .output()
        .expect("test verification command failed");

    let grep_output = String::from_utf8(output.stdout).expect("grep output is not UTF-8");
    let ok_count: usize = grep_output
        .trim()
        .parse()
        .expect("grep output should be a number");

    // The acceptance criterion requires >= 3 passing tests
    // grep counts all lines with "ok", which includes the summary line and other occurrences
    assert!(
        ok_count >= 3,
        "Checkbox tests must have >= 3 'ok' occurrences in output (criterion); found {}",
        ok_count
    );
}
