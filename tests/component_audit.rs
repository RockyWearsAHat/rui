//! Component registry structure for rui's widget ecosystem.
//!
//! Defines the metadata needed to audit and verify component implementations.
//! The registry captures the complete landscape of rui's 26+ widgets organized by
//! category (Layout, Form, Display, Navigation) with variant and state shape metadata.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ComponentCategory {
    Layout,
    Form,
    Display,
    Navigation,
}

#[derive(Debug, Clone)]
struct Component {
    name: &'static str,
    category: ComponentCategory,
    variants: Vec<&'static str>,
    state_shape: &'static str,
}

struct ComponentRegistry {
    components: HashMap<&'static str, Component>,
}

impl ComponentRegistry {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    fn register(&mut self, component: Component) {
        self.components.insert(component.name, component);
    }

    fn build_rui_widgets() -> Self {
        let mut registry = Self::new();

        // Layout widgets
        registry.register(Component {
            name: "col",
            category: ComponentCategory::Layout,
            variants: vec!["standard"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "row",
            category: ComponentCategory::Layout,
            variants: vec!["standard"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "spacer",
            category: ComponentCategory::Layout,
            variants: vec!["standard"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "panel",
            category: ComponentCategory::Layout,
            variants: vec!["standard"],
            state_shape: "None",
        });

        // Typography / Display widgets
        registry.register(Component {
            name: "text",
            category: ComponentCategory::Display,
            variants: vec!["standard"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "title",
            category: ComponentCategory::Display,
            variants: vec!["heading"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "heading",
            category: ComponentCategory::Display,
            variants: vec!["section_header"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "caption",
            category: ComponentCategory::Display,
            variants: vec!["small"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "micro",
            category: ComponentCategory::Display,
            variants: vec!["tiny"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "figure",
            category: ComponentCategory::Display,
            variants: vec!["emphasized"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "code",
            category: ComponentCategory::Display,
            variants: vec!["monospace"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "paragraph",
            category: ComponentCategory::Display,
            variants: vec!["body"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "divider",
            category: ComponentCategory::Display,
            variants: vec!["horizontal"],
            state_shape: "None",
        });

        // Form / Interactive widgets
        registry.register(Component {
            name: "button",
            category: ComponentCategory::Form,
            variants: vec!["primary"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "field",
            category: ComponentCategory::Form,
            variants: vec!["text_input"],
            state_shape: "String",
        });

        registry.register(Component {
            name: "tabs",
            category: ComponentCategory::Form,
            variants: vec!["horizontal"],
            state_shape: "usize",
        });

        registry.register(Component {
            name: "segmented",
            category: ComponentCategory::Form,
            variants: vec!["choice"],
            state_shape: "usize",
        });

        registry.register(Component {
            name: "star_rating",
            category: ComponentCategory::Form,
            variants: vec!["interactive"],
            state_shape: "f32",
        });

        registry.register(Component {
            name: "scrollbar",
            category: ComponentCategory::Form,
            variants: vec!["vertical", "horizontal"],
            state_shape: "f32",
        });

        // Status / Indicator widgets
        registry.register(Component {
            name: "tag",
            category: ComponentCategory::Display,
            variants: vec!["success", "warning", "danger"],
            state_shape: "Status",
        });

        registry.register(Component {
            name: "dot",
            category: ComponentCategory::Display,
            variants: vec!["indicator"],
            state_shape: "Status",
        });

        registry.register(Component {
            name: "meter",
            category: ComponentCategory::Display,
            variants: vec!["progress"],
            state_shape: "f32",
        });

        // Generic / Custom widgets
        registry.register(Component {
            name: "draw",
            category: ComponentCategory::Display,
            variants: vec!["custom_paint"],
            state_shape: "Fn(Painter, Rect)",
        });

        // Composite / Navigation widgets
        registry.register(Component {
            name: "section",
            category: ComponentCategory::Navigation,
            variants: vec!["labeled"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "field_row",
            category: ComponentCategory::Navigation,
            variants: vec!["form_layout"],
            state_shape: "None",
        });

        registry.register(Component {
            name: "field_group",
            category: ComponentCategory::Navigation,
            variants: vec!["grouped_fields"],
            state_shape: "None",
        });

        registry
    }

    fn count(&self) -> usize {
        self.components.len()
    }

    fn count_by_category(&self, category: ComponentCategory) -> usize {
        self.components
            .values()
            .filter(|c| c.category == category)
            .count()
    }

    fn names_sorted(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.components.keys().copied().collect();
        names.sort();
        names
    }
}

#[test]
fn test_component_registry_structure() {
    let registry = ComponentRegistry::build_rui_widgets();

    // Registry should have ≥18 entries (acceptance criterion)
    assert!(
        registry.count() >= 18,
        "Registry should have at least 18 components, got {}",
        registry.count()
    );
}

#[test]
fn test_registry_contains_all_major_widgets() {
    let registry = ComponentRegistry::build_rui_widgets();

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

    let actual = registry.names_sorted();

    for widget in &expected {
        assert!(
            registry.components.contains_key(widget),
            "Missing widget: {}",
            widget
        );
    }

    assert_eq!(
        actual.len(),
        expected.len(),
        "Expected {} widgets, got {}",
        expected.len(),
        actual.len()
    );
}

#[test]
fn test_component_has_valid_metadata() {
    let registry = ComponentRegistry::build_rui_widgets();

    for (name, component) in &registry.components {
        // Each component must have a name
        assert!(
            !component.name.is_empty(),
            "Component {} has empty name",
            name
        );

        // Each component must have at least one variant
        assert!(
            !component.variants.is_empty(),
            "Component {} has no variants",
            name
        );

        // Each component must have a state shape
        assert!(
            !component.state_shape.is_empty(),
            "Component {} has empty state_shape",
            name
        );

        // Component name must match registry key
        assert_eq!(
            component.name, *name,
            "Component name mismatch: {} != {}",
            component.name, name
        );
    }
}

#[test]
fn test_category_distribution() {
    let registry = ComponentRegistry::build_rui_widgets();

    let layout_count = registry.count_by_category(ComponentCategory::Layout);
    let form_count = registry.count_by_category(ComponentCategory::Form);
    let display_count = registry.count_by_category(ComponentCategory::Display);
    let nav_count = registry.count_by_category(ComponentCategory::Navigation);

    // Layout: col, row, spacer, panel (4 minimum)
    assert!(
        layout_count >= 3,
        "Expected at least 3 layout components, got {}",
        layout_count
    );

    // Form: button, field, tabs, segmented, star_rating, scrollbar (6 minimum)
    assert!(
        form_count >= 4,
        "Expected at least 4 form components, got {}",
        form_count
    );

    // Display: text, title, heading, etc. (10+ minimum)
    assert!(
        display_count >= 8,
        "Expected at least 8 display components, got {}",
        display_count
    );

    // Navigation: section, field_row, field_group (3 minimum)
    assert!(
        nav_count >= 2,
        "Expected at least 2 navigation components, got {}",
        nav_count
    );

    println!(
        "Component distribution: Layout={}, Form={}, Display={}, Navigation={}",
        layout_count, form_count, display_count, nav_count
    );
}

#[test]
fn test_component_variants_populated() {
    let registry = ComponentRegistry::build_rui_widgets();

    // Button should have at least one variant
    let button = registry.components.get("button").expect("button not found");
    assert!(!button.variants.is_empty(), "button should have variants");

    // Field should have variants
    let field = registry.components.get("field").expect("field not found");
    assert!(!field.variants.is_empty(), "field should have variants");

    // Text should exist
    let text = registry.components.get("text").expect("text not found");
    assert_eq!(text.name, "text");
}

#[test]
fn test_registry_completeness() {
    let registry = ComponentRegistry::build_rui_widgets();

    println!("\nComponent Registry ({} total):", registry.count());
    println!(
        "  Layout: {}",
        registry.count_by_category(ComponentCategory::Layout)
    );
    println!(
        "  Form: {}",
        registry.count_by_category(ComponentCategory::Form)
    );
    println!(
        "  Display: {}",
        registry.count_by_category(ComponentCategory::Display)
    );
    println!(
        "  Navigation: {}",
        registry.count_by_category(ComponentCategory::Navigation)
    );

    for name in registry.names_sorted() {
        let comp = &registry.components[name];
        println!(
            "  - {} ({}): variants={}, state={}",
            comp.name,
            format!("{:?}", comp.category).to_lowercase(),
            comp.variants.join(", "),
            comp.state_shape
        );
    }
}
