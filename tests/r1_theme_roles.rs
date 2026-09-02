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

    // Verify each TextRole resolves to the expected size
    assert_eq!(theme.text_size(TextRole::Title), 15.0);
    assert_eq!(theme.text_size(TextRole::Heading), 10.5);
    assert_eq!(theme.text_size(TextRole::Body), 13.0);
    assert_eq!(theme.text_size(TextRole::Caption), 11.5);
    assert_eq!(theme.text_size(TextRole::Micro), 9.5);
    assert_eq!(theme.text_size(TextRole::Code), 11.5);
}

#[test]
fn theme_resolves_spacing_levels() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify exact spacing values from Metrics::DEFAULT
    assert_eq!(theme.spacing(Space::Small), 4.0);
    assert_eq!(theme.spacing(Space::Normal), 8.0);
    assert_eq!(theme.spacing(Space::Large), 16.0);
}

#[test]
fn theme_resolves_control_heights() {
    let theme = Theme::new(Appearance::Light, FontId::default(), FontId::default());

    // Verify exact control height values
    assert_eq!(theme.control_height(Height::Control), 28.0);
    assert_eq!(theme.control_height(Height::Row), 22.0);
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
