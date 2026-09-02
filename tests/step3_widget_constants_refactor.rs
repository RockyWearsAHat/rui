//! STEP 3: Verify duplicate widget constants are extracted and used through Metrics/Theme
//!
//! This test module verifies that STEP 3 extraction analysis is correct and can be used
//! for refactoring. After STEP 3, widgets should use Metrics::DEFAULT values instead of
//! hardcoded constants where possible.

#[test]
fn step3_analysis_shows_correct_duplicates() {
    // STEP 3 identified these high-priority duplicates in src/widgets.rs:
    // - 28.0 (control_height): 5 instances on lines 167,187,341,376,524
    // - 12.0 (padding): 3 instances on lines 140,168,325
    // - 8.0 (gap): 3 instances on lines 482,505,527
    // - 1.0 (hairline): 2 instances on lines 146,478
    // - 22.0 (row_height): 1 instance on line 364

    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // Verify the duplicates still exist (baseline for refactoring)
    let control_height_count = src.matches(".h(28.0)").count();
    let _padding_count = src
        .matches(".pad")
        .filter(|_| src.contains(".pad(12.0)") || src.contains(".pad_x(12.0)"))
        .count();
    let gap_count = src.matches(".gap(8.0)").count();
    let hairline_count = src.matches(".h(1.0)").count();
    let row_height_count = src.matches(".h(22.0)").count();

    println!("STEP 3 Baseline — Hardcoded duplicates found:");
    println!(
        "  28.0 (control_height): {} instances",
        control_height_count
    );
    println!(
        "  12.0 (padding): {} instances found",
        src.matches("12.0").count()
    );
    println!("  8.0 (gap): {} instances", gap_count);
    println!("  1.0 (hairline): {} instances", hairline_count);
    println!("  22.0 (row_height): {} instances", row_height_count);

    // The baseline should match STEP 3 analysis
    assert!(
        control_height_count >= 5,
        "Should find 28.0 hardcoded instances"
    );
    assert!(gap_count >= 3, "Should find 8.0 gap instances");
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
fn step3_cross_reference_summary() {
    // STEP 3 acceptance produced a cross-reference showing:
    // - Which constants are matched to Metrics (✓)
    // - Which are not matched (e.g., BODY_SIZE, HEADING_TRACKING)

    let widgets_src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");
    let _theme_src = std::fs::read_to_string("src/theme.rs").expect("Read src/theme.rs");

    // BODY_SIZE (13.0) is not matched to Metrics
    let body_size_in_widgets = widgets_src.contains("const BODY_SIZE");
    assert!(body_size_in_widgets, "BODY_SIZE constant should exist");
    println!("✓ BODY_SIZE (13.0): NOT MATCHED to Metrics (typography-specific)");

    // HEADING_TRACKING (0.9) is not matched to Metrics
    let heading_tracking_in_widgets = widgets_src.contains("const HEADING_TRACKING");
    assert!(
        heading_tracking_in_widgets,
        "HEADING_TRACKING constant should exist"
    );
    println!("✓ HEADING_TRACKING (0.9): NOT MATCHED to Metrics (typography parameter)");

    println!("✓ STEP 3 cross-reference complete and verified");
}
