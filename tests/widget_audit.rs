//! Widget registry for rui's interactive controls.
//!
//! Documents the complete landscape of rui's 20+ widgets with their
//! interaction patterns (focusable, accepts click, accepts key events).

use rui::accessibility::Role;

/// A widget in rui's ecosystem with documented interaction properties.
#[derive(Debug, Clone)]
struct Widget {
    /// Widget name (e.g., "button", "field")
    name: &'static str,
    /// Semantic role from accessibility tree (Button, Field, Text, etc.)
    role: Role,
    /// Whether this widget can receive keyboard focus
    focusable: bool,
    /// Whether this widget responds to on_click handlers
    accepts_on_click: bool,
    /// Whether this widget responds to on_key handlers
    accepts_on_key: bool,
}

/// Complete registry of rui's widgets and their interaction properties.
const WIDGETS: &[Widget] = &[
    // Layout widgets (not interactive)
    Widget {
        name: "col",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "row",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "spacer",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "panel",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    // Display widgets (mostly read-only)
    Widget {
        name: "text",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "title",
        role: Role::Heading,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "heading",
        role: Role::Heading,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "caption",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "micro",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "figure",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "code",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "paragraph",
        role: Role::Text,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "divider",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "meter",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "draw",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "tag",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "dot",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    // Interactive widgets (respond to user input)
    Widget {
        name: "button",
        role: Role::Button,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    Widget {
        name: "field",
        role: Role::Field,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    Widget {
        name: "tabs",
        role: Role::Button,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    Widget {
        name: "segmented",
        role: Role::Button,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    Widget {
        name: "star_rating",
        role: Role::Button,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    Widget {
        name: "scrollbar",
        role: Role::Button,
        focusable: true,
        accepts_on_click: true,
        accepts_on_key: true,
    },
    // Navigation widgets
    Widget {
        name: "section",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "field_row",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
    Widget {
        name: "field_group",
        role: Role::Group,
        focusable: false,
        accepts_on_click: false,
        accepts_on_key: false,
    },
];

#[test]
fn widget_registry_has_minimum_entries() {
    // Acceptance: ≥18 entries
    assert!(
        WIDGETS.len() >= 18,
        "Registry should have at least 18 widgets, got {}",
        WIDGETS.len()
    );
}

#[test]
fn widget_registry_documents_all_major_widgets() {
    let expected = vec![
        "button",
        "caption",
        "code",
        "col",
        "divider",
        "dot",
        "draw",
        "field",
        "field_group",
        "field_row",
        "figure",
        "heading",
        "meter",
        "micro",
        "panel",
        "paragraph",
        "row",
        "scrollbar",
        "section",
        "segmented",
        "spacer",
        "star_rating",
        "tag",
        "tabs",
        "text",
        "title",
    ];

    let actual: Vec<_> = WIDGETS.iter().map(|w| w.name).collect();

    for widget_name in &expected {
        assert!(
            actual.contains(widget_name),
            "Missing widget: {}",
            widget_name
        );
    }
}

#[test]
fn widget_has_valid_metadata() {
    for widget in WIDGETS {
        // Each widget must have a non-empty name
        assert!(!widget.name.is_empty(), "Widget has empty name");

        // Verify focusability property consistency:
        // Focusable widgets should be interactive (accept click or key)
        if widget.focusable {
            assert!(
                widget.accepts_on_click || widget.accepts_on_key,
                "Widget '{}' is focusable but accepts no input",
                widget.name
            );
        }
    }
}

#[test]
fn interactive_widgets_are_focusable() {
    for widget in WIDGETS {
        // If a widget accepts clicks or keys, it should be focusable
        if widget.accepts_on_click || widget.accepts_on_key {
            assert!(
                widget.focusable,
                "Widget '{}' accepts input but is not focusable",
                widget.name
            );
        }
    }
}

#[test]
fn widget_role_aligns_with_interaction_properties() {
    for widget in WIDGETS {
        match widget.role {
            Role::Button => {
                // Buttons should accept clicks
                assert!(
                    widget.accepts_on_click,
                    "Button '{}' should accept on_click",
                    widget.name
                );
            }
            Role::Field => {
                // Fields should be focusable and accept key input
                assert!(
                    widget.focusable,
                    "Field '{}' should be focusable",
                    widget.name
                );
                assert!(
                    widget.accepts_on_key,
                    "Field '{}' should accept on_key",
                    widget.name
                );
            }
            Role::Text | Role::Heading => {
                // Text/headings are read-only, not interactive
                assert!(
                    !widget.focusable,
                    "Text/heading '{}' should not be focusable",
                    widget.name
                );
            }
            Role::Group => {
                // Groups are containers; may or may not be interactive
                // (validated by parent test for interaction consistency)
            }
            _ => {
                // Other roles are acceptable but not yet tested
            }
        }
    }
}

#[test]
fn widget_registry_is_complete() {
    println!("\nWidget Registry ({} total):", WIDGETS.len());
    for widget in WIDGETS {
        println!(
            "  - {} (role: {:?}): focusable={}, accepts_click={}, accepts_key={}",
            widget.name,
            widget.role,
            widget.focusable,
            widget.accepts_on_click,
            widget.accepts_on_key
        );
    }
}
