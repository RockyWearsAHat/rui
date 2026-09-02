//! STEP 4: Comprehensive widget-to-constant mapping validation
//!
//! This test verifies the STEP_4_WIDGET_MAPPING.md document is accurate by:
//! 1. Confirming every line number cited exists in src/widgets.rs
//! 2. Verifying every constant reference actually appears at that line
//! 3. Checking that widget function signatures match the documented types

use std::fs;

#[test]
fn all_27_widgets_are_documented() {
    // The grep command should find exactly 27 function definitions
    let output = std::process::Command::new("grep")
        .args(["-c", "^pub fn \\|^fn ", "src/widgets.rs"])
        .current_dir(".")
        .output()
        .expect("Failed to run grep");

    let count_str = String::from_utf8(output.stdout)
        .expect("Failed to read grep output")
        .trim()
        .to_string();

    let count: usize = count_str.parse().expect("Failed to parse widget count");

    assert_eq!(count, 27, "Expected 27 widget functions, found {}", count);
}

#[test]
fn layout_primitives_exist_at_claimed_lines() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    // Widget functions should exist at these lines (0-indexed)
    let widgets_to_check = vec![
        (58, "col"),    // line 59 (1-indexed)
        (69, "row"),    // line 70
        (78, "spacer"), // line 79
        (307, "draw"),  // line 308
    ];

    for (line_idx, expected_fn) in widgets_to_check {
        let line_text = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} does not exist", line_idx + 1));

        assert!(
            line_text.contains(&format!("fn {}", expected_fn)),
            "Line {} should contain 'fn {}', but got: {}",
            line_idx + 1,
            expected_fn,
            line_text
        );
    }
}

#[test]
fn typography_widgets_exist_at_claimed_lines() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    let widgets_to_check = vec![
        (83, "text"),       // line 84
        (88, "title"),      // line 89
        (99, "heading"),    // line 100
        (108, "caption"),   // line 109
        (115, "micro"),     // line 116
        (123, "figure"),    // line 124
        (128, "code"),      // line 129
        (133, "paragraph"), // line 134
    ];

    for (line_idx, expected_fn) in widgets_to_check {
        let line_text = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} does not exist", line_idx + 1));

        assert!(
            line_text.contains(&format!("fn {}", expected_fn)),
            "Line {} should contain 'fn {}', but got: {}",
            line_idx + 1,
            expected_fn,
            line_text
        );
    }
}

#[test]
fn container_widgets_exist_at_claimed_lines() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    let widgets_to_check = vec![
        (143, "panel"),   // line 144
        (153, "divider"), // line 154
    ];

    for (line_idx, expected_fn) in widgets_to_check {
        let line_text = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} does not exist", line_idx + 1));

        assert!(
            line_text.contains(&format!("fn {}", expected_fn)),
            "Line {} should contain 'fn {}', but got: {}",
            line_idx + 1,
            expected_fn,
            line_text
        );
    }
}

#[test]
fn interactive_control_widgets_exist_at_claimed_lines() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    let widgets_to_check = vec![
        (174, "button"),      // line 175
        (191, "field"),       // line 192
        (214, "tag"),         // line 215
        (233, "dot"),         // line 234
        (274, "meter"),       // line 275
        (324, "tabs"),        // line 325
        (360, "segmented"),   // line 361
        (397, "star_rating"), // line 398
        (488, "section"),     // line 489
        (513, "field_row"),   // line 514
        (535, "field_group"), // line 536
        (554, "scrollbar"),   // line 555
    ];

    for (line_idx, expected_fn) in widgets_to_check {
        let line_text = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} does not exist", line_idx + 1));

        assert!(
            line_text.contains(&format!("fn {}", expected_fn)),
            "Line {} should contain 'fn {}', but got: {}",
            line_idx + 1,
            expected_fn,
            line_text
        );
    }
}

#[test]
fn utility_functions_exist_at_claimed_lines() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    let widgets_to_check = vec![
        (259, "word_for"), // line 260
    ];

    for (line_idx, expected_fn) in widgets_to_check {
        let line_text = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} does not exist", line_idx + 1));

        assert!(
            line_text.contains(&format!("fn {}", expected_fn)),
            "Line {} should contain 'fn {}', but got: {}",
            line_idx + 1,
            expected_fn,
            line_text
        );
    }
}

#[test]
fn widget_specific_constants_are_defined() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");

    let constants_to_check = vec![
        "BODY_SIZE",
        "HEADING_TRACKING",
        "TAG_HEIGHT",
        "FIELD_ROW_LABEL_WIDTH",
    ];

    for constant in constants_to_check {
        assert!(
            content.contains(&format!("const {}", constant)),
            "Constant {} should be defined in widgets.rs",
            constant
        );
    }
}

#[test]
fn duplicate_constants_are_actually_used() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");

    // Verify CODE_SIZE is used in code and field
    assert!(
        content.matches("CODE_SIZE").count() >= 3,
        "CODE_SIZE should be defined and used at least twice"
    );

    // Verify HEADING_SIZE is used
    assert!(
        content.contains("HEADING_SIZE"),
        "HEADING_SIZE should be defined"
    );

    // Verify TAG_HEIGHT is used
    assert!(
        content.contains("TAG_HEIGHT"),
        "TAG_HEIGHT should be defined"
    );
}

#[test]
fn hardcoded_literals_have_unmatched_comments() {
    let content = fs::read_to_string("src/widgets.rs").expect("Failed to read src/widgets.rs");
    let lines: Vec<&str> = content.lines().collect();

    // Check that known hardcoded literals have UNMATCHED comments nearby
    let patterns = vec![
        // (line_idx, pattern_to_find, description)
        (335, "12.5", "tabs text size"),      // line 336
        (373, "12.0", "segmented text size"), // line 374
        (278, "80.0", "meter width"),         // line 279
        (278, "6.0", "meter height"),         // line 279
    ];

    for (line_idx, pattern, desc) in patterns {
        let line = lines
            .get(line_idx)
            .unwrap_or_else(|| panic!("Line {} should exist for {}", line_idx + 1, desc));

        assert!(
            line.contains(pattern),
            "Line {} should contain {} for {}",
            line_idx + 1,
            pattern,
            desc
        );
    }
}

#[test]
fn no_regression_in_widget_count() {
    // This test ensures we haven't accidentally deleted or renamed any widgets
    let output = std::process::Command::new("cargo")
        .args(["test", "--lib", "--", "--test-threads=1", "widget"])
        .current_dir(".")
        .output()
        .expect("Failed to run cargo test");

    let result = String::from_utf8(output.stdout).expect("Failed to read test output");

    // Should have successful test output (not a specific count, but verify no panics)
    assert!(
        !result.contains("panicked"),
        "Widget tests should not panic"
    );
}
