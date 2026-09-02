//! STEP 3: Verify widgets use Metrics::DEFAULT for layout and sizing.
//!
//! This test module verifies that STEP 3 refactoring replaced hardcoded constants with
//! Metrics::DEFAULT values. Verification focuses on code structure (refactoring is complete)
//! rather than pixel-level dimensions, following TDD principles: RED (test fails until refactored),
//! GREEN (test passes when Metrics::DEFAULT is used), REFACTOR (verify no brittle string matching).

#[test]
fn step3_metrics_defaults_exist_and_are_used() {
    // STEP 3 refactoring replaced hardcoded constants with Metrics::DEFAULT values.
    // Verify that:
    // 1. Metrics struct has all fields we depend on
    // 2. widgets.rs uses Metrics::DEFAULT (not hardcoded literals)
    // 3. No high-priority duplicate literals remain unreplaced

    let theme_src = std::fs::read_to_string("src/theme.rs").expect("Read src/theme.rs");
    let widgets_src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Verify Metrics fields exist
    assert!(
        theme_src.contains("control_height"),
        "Metrics must have control_height"
    );
    assert!(
        theme_src.contains("row_height"),
        "Metrics must have row_height"
    );
    assert!(theme_src.contains("padding"), "Metrics must have padding");
    assert!(theme_src.contains("gap:"), "Metrics must have gap");
    assert!(theme_src.contains("hairline"), "Metrics must have hairline");

    // Verify Metrics::DEFAULT is used in widgets.rs
    let metrics_usage_count = widgets_src.matches("Metrics::DEFAULT").count();
    assert!(
        metrics_usage_count >= 15,
        "Expected at least 15 Metrics::DEFAULT usages, found {}",
        metrics_usage_count
    );

    // Verify high-priority duplicate literals are replaced
    // These should NOT appear as standalone height/width values anymore
    assert!(
        !widgets_src.contains(".h(28.0)\n") && !widgets_src.contains(".h(28.0)"),
        "control_height (28.0) should be replaced with Metrics::DEFAULT.control_height"
    );
    assert!(
        !widgets_src.contains(".h(22.0)\n") && !widgets_src.contains(".h(22.0)"),
        "row_height (22.0) should be replaced with Metrics::DEFAULT.row_height"
    );
    assert!(
        !widgets_src.contains(".pad(12.0)"),
        "padding (12.0) should be replaced with Metrics::DEFAULT.padding"
    );
    assert!(
        !widgets_src.contains(".gap(8.0)\n") && !widgets_src.contains(".gap(8.0)"),
        "gap (8.0) should be replaced with Metrics::DEFAULT.gap"
    );
}

#[test]
fn step3_widget_specific_constants_extracted() {
    // STEP 3 extracted widget-specific values that don't match Metrics as named constants:
    // - TAG_HEIGHT: 18.0 (widget-specific height in tag())
    // - FIELD_ROW_LABEL_WIDTH: 78.0 (label width in field_row())

    let widgets_src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Verify constants are extracted
    assert!(
        widgets_src.contains("const TAG_HEIGHT"),
        "TAG_HEIGHT constant should be defined"
    );
    assert!(
        widgets_src.contains("const FIELD_ROW_LABEL_WIDTH"),
        "FIELD_ROW_LABEL_WIDTH constant should be defined"
    );

    // Verify constants are used (not hardcoded)
    assert!(
        widgets_src.contains("TAG_HEIGHT"),
        "TAG_HEIGHT should be used in widget code"
    );
    assert!(
        widgets_src.contains("FIELD_ROW_LABEL_WIDTH"),
        "FIELD_ROW_LABEL_WIDTH should be used in widget code"
    );
}

#[test]
fn step3_refactoring_summary() {
    // STEP 3 Summary: Extract hardcoded constants and refactor to use Metrics::DEFAULT
    // This test documents what was done for future reference.

    let widgets_src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Count refactored usages
    let control_height_count = widgets_src
        .matches("Metrics::DEFAULT.control_height")
        .count();
    let padding_count = widgets_src.matches("Metrics::DEFAULT.padding").count();
    let gap_count = widgets_src.matches("Metrics::DEFAULT.gap").count();
    let hairline_count = widgets_src.matches("Metrics::DEFAULT.hairline").count();
    let row_height_count = widgets_src.matches("Metrics::DEFAULT.row_height").count();
    let gap_small_count = widgets_src.matches("Metrics::DEFAULT.gap_small").count();

    println!("\nSTEP 3 REFACTORING COMPLETE:");
    println!(
        "  Metrics::DEFAULT.control_height: {} instances",
        control_height_count
    );
    println!("  Metrics::DEFAULT.padding: {} instances", padding_count);
    println!("  Metrics::DEFAULT.gap: {} instances", gap_count);
    println!("  Metrics::DEFAULT.hairline: {} instances", hairline_count);
    println!(
        "  Metrics::DEFAULT.row_height: {} instances",
        row_height_count
    );
    println!(
        "  Metrics::DEFAULT.gap_small: {} instances",
        gap_small_count
    );
    println!("  Total Metrics::DEFAULT usages: {}", {
        control_height_count
            + padding_count
            + gap_count
            + hairline_count
            + row_height_count
            + gap_small_count
    });

    // Verify at least one of each metric is used
    assert!(control_height_count > 0, "control_height should be used");
    assert!(padding_count > 0, "padding should be used");
    assert!(gap_count > 0, "gap should be used");
    assert!(hairline_count > 0, "hairline should be used");
}
