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

use rui::{Appearance, FontId, Height, Space, TextRole, Theme};

/// Verify TextRole enum values are correctly resolved by Theme.
#[test]
fn theme_resolves_text_roles() {
    let theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);

    // TextRole::Title — used for window titles and primary headings
    assert_eq!(theme.text_size(TextRole::Title), 15.0);

    // TextRole::Heading — used for section labels
    assert_eq!(theme.text_size(TextRole::Heading), 10.5);

    // TextRole::Body — used for ordinary text content
    assert_eq!(theme.text_size(TextRole::Body), 13.0);

    // TextRole::Caption — used for secondary text and explanations
    assert_eq!(theme.text_size(TextRole::Caption), 11.5);

    // TextRole::Micro — used for smallest readable text
    assert_eq!(theme.text_size(TextRole::Micro), 9.5);

    // TextRole::Code — used for machine output
    assert_eq!(theme.text_size(TextRole::Code), 11.5);
}

/// Verify Space enum values are correctly resolved by Theme.
#[test]
fn theme_resolves_spacing_levels() {
    let theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);

    // Space::Small — gap between closely-related items (4.0 in Metrics::DEFAULT)
    assert_eq!(theme.spacing(Space::Small), 4.0);

    // Space::Normal — standard gap between items (8.0 in Metrics::DEFAULT)
    assert_eq!(theme.spacing(Space::Normal), 8.0);

    // Space::Large — gap between sections (16.0 in Metrics::DEFAULT)
    assert_eq!(theme.spacing(Space::Large), 16.0);
}

/// Verify Height enum values are correctly resolved by Theme.
#[test]
fn theme_resolves_control_heights() {
    let theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);

    // Height::Control — button and text field height (28.0 in Metrics::DEFAULT)
    assert_eq!(theme.control_height(Height::Control), 28.0);

    // Height::Row — single row in a list or table (22.0 in Metrics::DEFAULT)
    assert_eq!(theme.control_height(Height::Row), 22.0);
}

/// Verify that Theme provides all necessary methods for R1 widget refactoring.
#[test]
fn theme_r1_api_is_complete() {
    let theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);

    // All TextRole variants must resolve to a valid size
    for role in &[
        TextRole::Title,
        TextRole::Heading,
        TextRole::Body,
        TextRole::Caption,
        TextRole::Micro,
        TextRole::Code,
    ] {
        let size = theme.text_size(*role);
        assert!(
            size > 0.0 && size < 100.0,
            "TextRole {:?} resolved to invalid size: {}",
            role,
            size
        );
    }

    // All Space variants must resolve to a valid gap
    for space in &[Space::Small, Space::Normal, Space::Large] {
        let gap = theme.spacing(*space);
        assert!(
            gap > 0.0 && gap < 50.0,
            "Space {:?} resolved to invalid gap: {}",
            space,
            gap
        );
    }

    // All Height variants must resolve to a valid height
    for height in &[Height::Control, Height::Row] {
        let h = theme.control_height(*height);
        assert!(
            h > 0.0 && h < 100.0,
            "Height {:?} resolved to invalid height: {}",
            height,
            h
        );
    }
}

/// Verify that Space and Height values follow expected hierarchy.
#[test]
fn spacing_hierarchy_is_consistent() {
    let theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);

    // Space hierarchy: Small < Normal < Large
    let small = theme.spacing(Space::Small);
    let normal = theme.spacing(Space::Normal);
    let large = theme.spacing(Space::Large);

    assert!(
        small < normal && normal < large,
        "Space hierarchy violated: Small={} Normal={} Large={}",
        small,
        normal,
        large
    );
}

/// Verify that theme resolves correctly in both light and dark appearances.
/// All layout metrics (text sizes, spacing, heights) must be appearance-independent.
#[test]
fn theme_roles_work_in_both_appearances() {
    let light_theme = Theme::new(Appearance::Light, FontId::FIRST, FontId::FIRST);
    let dark_theme = Theme::new(Appearance::Dark, FontId::FIRST, FontId::FIRST);

    // All TextRole variants should resolve to same sizes regardless of appearance
    for role in [
        TextRole::Title,
        TextRole::Heading,
        TextRole::Body,
        TextRole::Caption,
        TextRole::Micro,
        TextRole::Code,
    ] {
        assert_eq!(
            light_theme.text_size(role),
            dark_theme.text_size(role),
            "text_size({:?}) must be appearance-independent",
            role
        );
    }

    // All Space variants should resolve to same values regardless of appearance
    for space in [Space::Small, Space::Normal, Space::Large] {
        assert_eq!(
            light_theme.spacing(space),
            dark_theme.spacing(space),
            "spacing({:?}) must be appearance-independent",
            space
        );
    }

    // All Height variants should resolve to same values regardless of appearance
    for height in [Height::Control, Height::Row] {
        assert_eq!(
            light_theme.control_height(height),
            dark_theme.control_height(height),
            "control_height({:?}) must be appearance-independent",
            height
        );
    }
}

/// Verify that TextRole, Space, and Height are publicly exported from rui crate.
#[test]
fn r1_enums_are_public_api() {
    // This test ensures TextRole, Space, and Height can be imported directly
    // from the rui crate root, making them available for widget authors.
    //
    // If this test fails to compile, it means the enums are not properly exported
    // from lib.rs, and widgets cannot access them.

    let _role: TextRole = TextRole::Body;
    let _space: Space = Space::Normal;
    let _height: Height = Height::Control;

    // If we got here, all enums are properly exported
}

/// Verify that all TextRole variants are accessible.
#[test]
fn all_text_roles_accessible() {
    let _title = TextRole::Title;
    let _heading = TextRole::Heading;
    let _body = TextRole::Body;
    let _caption = TextRole::Caption;
    let _micro = TextRole::Micro;
    let _code = TextRole::Code;
}

/// Verify that all Space variants are accessible.
#[test]
fn all_space_variants_accessible() {
    let _small = Space::Small;
    let _normal = Space::Normal;
    let _large = Space::Large;
}

/// Verify that all Height variants are accessible.
#[test]
fn all_height_variants_accessible() {
    let _control = Height::Control;
    let _row = Height::Row;
}
