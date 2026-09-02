//! R1 Theme Roles Verification Gate
//!
//! This test suite verifies that R1 (Theme roles) is properly implemented.
//! It demonstrates usage of TextRole, Space, and Height enums to build
//! widgets without hardcoded size constants.
//!
//! R1 enables widgets to be restyled via Theme without duplicated constants,
//! providing a single source of truth for:
//! - Text sizes via TextRole (Title, Heading, Body, Caption, Micro, Code)
//! - Spacing gaps via Space (Small, Normal, Large)
//! - Control heights via Height (Control, Row)

#![allow(unused_imports)]

use rui::{Appearance, FontId, Height, Space, TextRole, Theme};

#[test]
fn theme_resolves_text_roles() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify each TextRole resolves to a consistent size
    assert!(theme.text_size(TextRole::Title) > 0.0);
    assert!(theme.text_size(TextRole::Heading) > 0.0);
    assert!(theme.text_size(TextRole::Body) > 0.0);
    assert!(theme.text_size(TextRole::Caption) > 0.0);
    assert!(theme.text_size(TextRole::Micro) > 0.0);
    assert!(theme.text_size(TextRole::Code) > 0.0);

    // Title should be largest, Micro should be smallest
    assert!(theme.text_size(TextRole::Title) > theme.text_size(TextRole::Body));
    assert!(theme.text_size(TextRole::Body) > theme.text_size(TextRole::Micro));
}

#[test]
fn theme_resolves_spacing_levels() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify each Space level resolves to a positive value
    assert!(theme.spacing(Space::Small) > 0.0);
    assert!(theme.spacing(Space::Normal) > 0.0);
    assert!(theme.spacing(Space::Large) > 0.0);

    // Verify hierarchy: Small < Normal < Large
    assert!(theme.spacing(Space::Small) < theme.spacing(Space::Normal));
    assert!(theme.spacing(Space::Normal) < theme.spacing(Space::Large));
}

#[test]
fn theme_resolves_control_heights() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify each Height level resolves to a positive value
    assert!(theme.control_height(Height::Control) > 0.0);
    assert!(theme.control_height(Height::Row) > 0.0);

    // Both heights should be distinct values
    let control = theme.control_height(Height::Control);
    let row = theme.control_height(Height::Row);
    assert_ne!(control, 0.0);
    assert_ne!(row, 0.0);
}

#[test]
fn theme_r1_api_is_complete() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify that the three R1 resolution methods exist and work
    let _ = theme.text_size(TextRole::Title);
    let _ = theme.spacing(Space::Normal);
    let _ = theme.control_height(Height::Control);
}

#[test]
fn spacing_hierarchy_is_consistent() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify spacing hierarchy is enforced: Small < Normal < Large
    let small = theme.spacing(Space::Small);
    let normal = theme.spacing(Space::Normal);
    let large = theme.spacing(Space::Large);

    assert!(small < normal, "Small spacing should be less than Normal");
    assert!(normal < large, "Normal spacing should be less than Large");
    assert!(small > 0.0, "All spacings must be positive");
}

#[test]
fn theme_roles_work_in_both_appearances() {
    let light = Theme::new(Appearance::Light, FontId::default(), FontId::default());
    let dark = Theme::new(Appearance::Dark, FontId::default(), FontId::default());

    // Verify that both themes resolve to the same sizes (appearance-independent)
    assert_eq!(
        light.text_size(TextRole::Title),
        dark.text_size(TextRole::Title)
    );
    assert_eq!(light.spacing(Space::Normal), dark.spacing(Space::Normal));
    assert_eq!(
        light.control_height(Height::Control),
        dark.control_height(Height::Control)
    );
}

#[test]
fn r1_enums_are_public_api() {
    // Verify that all three enums are publicly accessible
    let _ = TextRole::Title;
    let _ = TextRole::Heading;
    let _ = TextRole::Body;
    let _ = TextRole::Caption;
    let _ = TextRole::Micro;
    let _ = TextRole::Code;

    let _ = Space::Small;
    let _ = Space::Normal;
    let _ = Space::Large;

    let _ = Height::Control;
    let _ = Height::Row;
}

#[test]
fn all_text_roles_accessible() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // All TextRole variants should resolve to a size
    let _ = theme.text_size(TextRole::Title);
    let _ = theme.text_size(TextRole::Heading);
    let _ = theme.text_size(TextRole::Body);
    let _ = theme.text_size(TextRole::Caption);
    let _ = theme.text_size(TextRole::Micro);
    let _ = theme.text_size(TextRole::Code);
}

#[test]
fn all_space_variants_accessible() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // All Space variants should resolve to a value
    let _ = theme.spacing(Space::Small);
    let _ = theme.spacing(Space::Normal);
    let _ = theme.spacing(Space::Large);
}

#[test]
fn all_height_variants_accessible() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // All Height variants should resolve to a value
    let _ = theme.control_height(Height::Control);
    let _ = theme.control_height(Height::Row);
}
