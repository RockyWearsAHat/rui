//! STEP 3 REFACTOR: Replace hardcoded widget constants with Metrics::DEFAULT values
//!
//! This test verifies that the high-priority widget duplicates identified in STEP 3
//! have been refactored to use Metrics::DEFAULT instead of hardcoded literals.

#[test]
fn step3_refactor_button_uses_metrics_control_height() {
    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // button() should use Metrics::DEFAULT.control_height instead of hardcoded 28.0
    // After refactoring, the hardcoded .h(28.0) on line 167 should be gone
    // and replaced with .h(Metrics::DEFAULT.control_height)

    let button_fn = src
        .lines()
        .skip(164) // line 165 is "pub fn button<S>(label: impl Into<String>) -> El<S> {"
        .take(15)
        .collect::<String>();

    // After refactoring, button should reference Metrics::DEFAULT.control_height
    if button_fn.contains("Metrics::DEFAULT.control_height") {
        println!("✓ button() uses Metrics::DEFAULT.control_height");
        return;
    }

    // For now, this will fail (RED phase) - we'll refactor to make it pass
    // The test documents what we're working toward
    panic!("button() should use Metrics::DEFAULT.control_height instead of hardcoded 28.0");
}

#[test]
fn step3_refactor_field_uses_metrics_control_height() {
    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    let field_fn = src
        .lines()
        .skip(181) // line 182 is "pub fn field<S>(value: impl Into<String>) -> El<S> {"
        .take(15)
        .collect::<String>();

    if field_fn.contains("Metrics::DEFAULT.control_height") {
        println!("✓ field() uses Metrics::DEFAULT.control_height");
        return;
    }

    panic!("field() should use Metrics::DEFAULT.control_height instead of hardcoded 28.0");
}

#[test]
fn step3_refactor_panel_uses_metrics_padding() {
    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    let panel_fn = src
        .lines()
        .skip(133) // line 134 is "pub fn panel<S>(children: impl Children<S>) -> El<S> {"
        .take(8)
        .collect::<String>();

    if panel_fn.contains("Metrics::DEFAULT.padding") {
        println!("✓ panel() uses Metrics::DEFAULT.padding");
        return;
    }

    panic!("panel() should use Metrics::DEFAULT.padding instead of hardcoded 12.0");
}

#[test]
fn step3_refactor_divider_uses_metrics_hairline() {
    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    let divider_fn = src
        .lines()
        .skip(143) // line 144 is "pub fn divider<S>() -> El<S> {"
        .take(6)
        .collect::<String>();

    if divider_fn.contains("Metrics::DEFAULT.hairline") {
        println!("✓ divider() uses Metrics::DEFAULT.hairline");
        return;
    }

    panic!("divider() should use Metrics::DEFAULT.hairline instead of hardcoded 1.0");
}

#[test]
fn step3_refactor_metrics_imported() {
    let src = std::fs::read_to_string("src/widgets.rs").expect("Read src/widgets.rs");

    // After refactoring, widgets.rs should import Metrics
    if src.contains("use crate::theme::{") && src.contains("Metrics") {
        println!("✓ Metrics imported in widgets.rs");
        return;
    }

    panic!("widgets.rs should import Metrics for refactored constants");
}
