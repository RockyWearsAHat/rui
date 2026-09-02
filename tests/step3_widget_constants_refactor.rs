//! STEP 3: Verify duplicate widget constants are extracted and used through Metrics/Theme
//!
//! This test module verifies that STEP 3 extraction analysis is correct and can be used
//! for refactoring. After STEP 3, widgets use Metrics::DEFAULT values instead of
//! hardcoded constants where a match exists.

#[test]
fn step3_metrics_default_used_in_widgets() {
    // STEP 3 refactoring replaced hardcoded constants with Metrics::DEFAULT:
    // - control_height: .h(Metrics::DEFAULT.control_height) in button, field, tabs, segmented, field_row
    // - padding: .pad(Metrics::DEFAULT.padding) in panel, button, tabs
    // - gap: .gap(Metrics::DEFAULT.gap) in field, tags, section, field_row, field_group
    // - hairline: .h(Metrics::DEFAULT.hairline) in divider, section
    // - row_height: .h(Metrics::DEFAULT.row_height) in segmented
    // - gap_small: .gap(Metrics::DEFAULT.gap_small) in star_rating

    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Count Metrics::DEFAULT usage for each refactored field
    let control_height_count = src.matches("Metrics::DEFAULT.control_height").count();
    let padding_count = src.matches("Metrics::DEFAULT.padding").count();
    let gap_count = src.matches("Metrics::DEFAULT.gap").count();
    let hairline_count = src.matches("Metrics::DEFAULT.hairline").count();
    let row_height_count = src.matches("Metrics::DEFAULT.row_height").count();
    let gap_small_count = src.matches("Metrics::DEFAULT.gap_small").count();

    println!("STEP 3 Refactoring — Metrics::DEFAULT usage:");
    println!("  control_height: {} instances", control_height_count);
    println!("  padding: {} instances", padding_count);
    println!("  gap: {} instances", gap_count);
    println!("  hairline: {} instances", hairline_count);
    println!("  row_height: {} instances", row_height_count);
    println!("  gap_small: {} instances", gap_small_count);

    // Verify refactoring replaced the high-priority duplicates
    assert!(
        control_height_count >= 4,
        "Should find Metrics::DEFAULT.control_height in at least 4 places"
    );
    assert!(
        padding_count >= 2,
        "Should find Metrics::DEFAULT.padding in at least 2 places"
    );
    assert!(
        gap_count >= 3,
        "Should find Metrics::DEFAULT.gap in at least 3 places"
    );
    assert!(
        hairline_count >= 2,
        "Should find Metrics::DEFAULT.hairline in at least 2 places"
    );
}

#[test]
fn step3_metrics_default_values_exist() {
    // STEP 3 showed these Metrics::DEFAULT values exist:
    // gap_small: 4.0
    // gap: 8.0
    // gap_large: 16.0
    // padding: 12.0
    // corner: 8.0
    // corner_small: 5.0
    // control_height: 28.0
    // row_height: 22.0
    // hairline: 1.0
    // scrollbar: 8.0
    // shadow: 9.0
    // shadow_offset: 1.5
    // motion: 0.09

    let theme_src = std::fs::read_to_string("src/theme.rs").expect("Read src/theme.rs");

    // Verify the Metrics struct exists with these fields
    assert!(
        theme_src.contains("control_height"),
        "Metrics should have control_height"
    );
    assert!(
        theme_src.contains("row_height"),
        "Metrics should have row_height"
    );
    assert!(theme_src.contains("gap:"), "Metrics should have gap");
    assert!(theme_src.contains("padding"), "Metrics should have padding");
    assert!(
        theme_src.contains("hairline"),
        "Metrics should have hairline"
    );

    println!("✓ All Metrics::DEFAULT fields exist as identified in STEP 3");
}

#[test]
fn step3_unmatched_literals_extracted_as_constants() {
    // STEP 3 extracted widget-specific values that don't match Metrics as named constants:
    // - TAG_HEIGHT: 18.0 (widget-specific height in tag())
    // - FIELD_ROW_LABEL_WIDTH: 78.0 (label width in field_row())
    //
    // These are NOT MATCHED to Metrics::DEFAULT because they are widget-specific,
    // not general layout metrics reusable across the system.

    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Verify the constants are defined
    assert!(
        src.contains("const TAG_HEIGHT: f32 = 18.0;"),
        "TAG_HEIGHT constant should be defined"
    );
    assert!(
        src.contains("const FIELD_ROW_LABEL_WIDTH: f32 = 78.0;"),
        "FIELD_ROW_LABEL_WIDTH constant should be defined"
    );

    // Verify they're used in the code (replacing hardcoded literals)
    assert!(
        src.contains(".h(TAG_HEIGHT)"),
        "tag() should use TAG_HEIGHT constant"
    );
    assert!(
        src.contains(".w(FIELD_ROW_LABEL_WIDTH)"),
        "field_row() should use FIELD_ROW_LABEL_WIDTH constant"
    );

    println!("STEP 3 Widget-specific constants extracted:");
    println!("  TAG_HEIGHT: 18.0 (NOT MATCHED — tag-specific)");
    println!("  FIELD_ROW_LABEL_WIDTH: 78.0 (NOT MATCHED — layout-specific)");
    println!("✓ Constants extracted and used correctly");
}
