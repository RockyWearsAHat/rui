//! STEP 1: Phase 1 — Red phase TDD scaffolding for theme roles.
//!
//! This test module imports non-existent enums and demonstrates how Theme
//! roles *should* resolve text sizes, gaps, and control heights without
//! hardcoded constants. Write this first to fail cleanly, then implement
//! the minimum code to make it pass.

use rui::text::FontId;
use rui::theme::{Appearance, Height, Space, TextRole, Theme};

fn test_theme() -> Theme {
    Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST)
}

/// Theme roles should resolve text sizes without hardcoded constants.
#[test]
fn text_role_resolves_sizes_from_theme() {
    let theme = test_theme();

    // TextRole enum should resolve sizes through Theme
    let title_size = theme.text_size(TextRole::Title);
    let heading_size = theme.text_size(TextRole::Heading);
    let caption_size = theme.text_size(TextRole::Caption);
    let code_size = theme.text_size(TextRole::Code);

    // Sizes should be positive and distinct
    assert!(title_size > 0.0, "Title size should be positive");
    assert!(heading_size > 0.0, "Heading size should be positive");
    assert!(caption_size > 0.0, "Caption size should be positive");
    assert!(code_size > 0.0, "Code size should be positive");

    // Title should be larger than heading
    assert!(
        title_size > heading_size,
        "Title should be larger than heading"
    );
}

/// Space roles should resolve gaps from Theme metrics.
#[test]
fn space_role_resolves_gaps_from_theme() {
    let theme = test_theme();

    // Space enum should resolve gaps through Theme metrics
    let small_gap = theme.spacing(Space::Small);
    let normal_gap = theme.spacing(Space::Normal);
    let large_gap = theme.spacing(Space::Large);

    // Gaps should be positive and ordered
    assert!(small_gap > 0.0, "Small gap should be positive");
    assert!(normal_gap > 0.0, "Normal gap should be positive");
    assert!(large_gap > 0.0, "Large gap should be positive");

    // Gaps should increase with size
    assert!(
        small_gap < normal_gap,
        "Small should be less than normal gap"
    );
    assert!(
        normal_gap < large_gap,
        "Normal should be less than large gap"
    );
}

/// Height roles should resolve control heights from Theme metrics.
#[test]
fn height_role_resolves_control_heights_from_theme() {
    let theme = test_theme();

    // Height enum should resolve control heights through Theme
    let control_h = theme.control_height(Height::Control);
    let row_h = theme.control_height(Height::Row);

    // Heights should be positive
    assert!(control_h > 0.0, "Control height should be positive");
    assert!(row_h > 0.0, "Row height should be positive");

    // Both should be reasonable dimensions
    assert!(control_h >= 20.0, "Control height should be at least 20");
    assert!(row_h >= 20.0, "Row height should be at least 20");
}

/// Widgets should parameterize sizing through roles, not hardcoded constants.
///
/// This test shows the pattern: a button's height comes from Height::Control
/// resolved through the theme, not from a hardcoded 28.0.
#[test]
fn widgets_use_theme_roles_not_constants() {
    let theme = test_theme();

    // Example: a button's size should come from Height::Control
    let button_height = theme.control_height(Height::Control);

    // And text inside it should use TextRole::Body (or similar)
    let text_height = theme.text_size(TextRole::Body);

    // The button should reserve space based on control height, not 28.0
    assert!(
        button_height > text_height,
        "Button height should accommodate text"
    );

    // Gaps inside the button should come from Space roles
    let inner_gap = theme.spacing(Space::Small);
    assert!(
        inner_gap < button_height,
        "Inner gap should fit inside button"
    );
}

/// Text role should have standard sizes for all semantic levels.
#[test]
fn text_role_has_all_semantic_sizes() {
    let theme = test_theme();

    // All semantic text roles should resolve
    let _title = theme.text_size(TextRole::Title);
    let _heading = theme.text_size(TextRole::Heading);
    let _body = theme.text_size(TextRole::Body);
    let _caption = theme.text_size(TextRole::Caption);
    let _micro = theme.text_size(TextRole::Micro);
    let _code = theme.text_size(TextRole::Code);

    // All should be positive
    assert!(theme.text_size(TextRole::Title) > 0.0);
    assert!(theme.text_size(TextRole::Heading) > 0.0);
    assert!(theme.text_size(TextRole::Body) > 0.0);
    assert!(theme.text_size(TextRole::Caption) > 0.0);
    assert!(theme.text_size(TextRole::Micro) > 0.0);
    assert!(theme.text_size(TextRole::Code) > 0.0);
}
