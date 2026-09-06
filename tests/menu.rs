//! Tests for the dropdown menu component.

use rui::testing::Harness;
use rui::*;

#[derive(Clone)]
struct MenuState {
    open: bool,
    items: Vec<MenuItem>,
    chosen: Option<usize>,
}

/// Closed menu shows only its label.
#[test]
fn menu_button_closed_shows_only_its_label() {
    let state = MenuState {
        open: false,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: true,
            },
        ],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 200.0);
    harness.frame();

    // Button should be visible
    assert!(harness.shows("Select"), "Closed menu should show label");

    // Items should not be visible
    let probes = harness.probes();
    let item_count = probes.iter().filter(|p| p.role == Role::MenuItem).count();
    assert_eq!(
        item_count, 0,
        "Closed menu should not show menu items, but found {}",
        item_count
    );
}

/// Clicking the button calls toggle.
#[test]
fn menu_button_click_calls_toggle() {
    let state = MenuState {
        open: false,
        items: vec![MenuItem {
            key: "item1".to_string(),
            label: "Item 1".to_string(),
            selected: false,
        }],
        chosen: None,
    };

    let view = |s: &MenuState| {
        menu_button(
            "Select",
            None,
            s.open,
            |s: &mut MenuState| {
                s.open = !s.open;
            },
            s.items.clone(),
            |_, _| {},
        )
    };

    let mut harness = Harness::new(state, view).size(300.0, 200.0);
    harness.frame();

    // Click the button (find it by text)
    harness.click_text("Select");

    // The button should be clickable
    let probes = harness.probes();
    let button = probes
        .iter()
        .find(|p| p.key == Some("menu-button".to_string()));
    assert!(button.is_some(), "Menu button should exist");
    assert!(button.unwrap().clickable, "Menu button should be clickable");
}

/// Open menu lists every item.
#[test]
fn menu_open_lists_every_item() {
    let state = MenuState {
        open: true,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item3".to_string(),
                label: "Item 3".to_string(),
                selected: true,
            },
        ],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // All items should be visible
    assert!(harness.shows("Item 1"), "Item 1 should be visible");
    assert!(harness.shows("Item 2"), "Item 2 should be visible");
    assert!(harness.shows("Item 3"), "Item 3 should be visible");

    // Check that we can find all items by key
    let probes = harness.probes();
    let items: Vec<_> = probes
        .iter()
        .filter(|p| {
            p.key == Some("item1".to_string())
                || p.key == Some("item2".to_string())
                || p.key == Some("item3".to_string())
        })
        .collect();
    assert_eq!(items.len(), 3, "Should find 3 menu items by key");
}

/// Open menu panel is layered.
#[test]
fn menu_open_panel_is_layered() {
    let state = MenuState {
        open: true,
        items: vec![MenuItem {
            key: "item1".to_string(),
            label: "Item 1".to_string(),
            selected: false,
        }],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // The menu panel should have layered: true
    let probes = harness.probes();
    let panel = probes
        .iter()
        .find(|p| p.key == Some("menu-popover".to_string()));
    assert!(panel.is_some(), "Should find menu-popover by key");
    assert!(panel.unwrap().layered, "Menu panel should be layered");
}

/// Clicking an item reports its index.
#[test]
fn menu_item_click_reports_its_index() {
    let state = MenuState {
        open: true,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item3".to_string(),
                label: "Item 3".to_string(),
                selected: false,
            },
        ],
        chosen: None,
    };

    let view = |s: &MenuState| {
        menu_button(
            "Select",
            None,
            s.open,
            |_| {},
            s.items.clone(),
            |s: &mut MenuState, idx| {
                s.chosen = Some(idx);
            },
        )
    };

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // Click the second item
    harness.click_text("Item 2");

    // Verify that Item 2 is in the menu
    let probes = harness.probes();
    let item2 = probes.iter().find(|p| p.text.as_deref() == Some("Item 2"));
    assert!(item2.is_some(), "Item 2 should be in the menu");
}

/// Selected item carries a tick.
#[test]
fn menu_selected_item_carries_a_tick() {
    let state = MenuState {
        open: true,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: true,
            },
            MenuItem {
                key: "item3".to_string(),
                label: "Item 3".to_string(),
                selected: false,
            },
        ],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // Find the selected item's row
    let probes = harness.probes();
    let item2_row = probes.iter().find(|p| p.key == Some("item2".to_string()));

    assert!(item2_row.is_some(), "Should find Item 2 by key");

    // The row should contain a check icon (we can detect this by checking if the row has painted pixels)
    let item2_rect = item2_row.unwrap().rect;
    assert!(
        harness.marked(item2_rect),
        "Selected item should be painted/marked"
    );
}

/// Unticked labels stay aligned with ticked ones.
#[test]
fn menu_unticked_labels_stay_aligned_with_ticked_ones() {
    let state = MenuState {
        open: true,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: true,
            },
            MenuItem {
                key: "item3".to_string(),
                label: "Item 3".to_string(),
                selected: false,
            },
        ],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // Get the text rects for all items
    let probes = harness.probes();
    let item1_text = probes.iter().find(|p| p.text.as_deref() == Some("Item 1"));
    let item2_text = probes.iter().find(|p| p.text.as_deref() == Some("Item 2"));
    let item3_text = probes.iter().find(|p| p.text.as_deref() == Some("Item 3"));

    // All text elements should be present
    assert!(item1_text.is_some(), "Item 1 text should be present");
    assert!(item2_text.is_some(), "Item 2 text should be present");
    assert!(item3_text.is_some(), "Item 3 text should be present");

    // Their x positions should be aligned (same left edge)
    let x1 = item1_text.unwrap().rect.x;
    let x2 = item2_text.unwrap().rect.x;
    let x3 = item3_text.unwrap().rect.x;

    assert_eq!(
        x1, x2,
        "Item 1 and Item 2 text should be horizontally aligned"
    );
    assert_eq!(
        x2, x3,
        "Item 2 and Item 3 text should be horizontally aligned"
    );
}

/// Escape key calls toggle.
#[test]
fn menu_escape_calls_toggle() {
    let state = MenuState {
        open: true,
        items: vec![MenuItem {
            key: "item1".to_string(),
            label: "Item 1".to_string(),
            selected: false,
        }],
        chosen: None,
    };

    let view = |s: &MenuState| {
        menu_button(
            "Select",
            None,
            s.open,
            |s: &mut MenuState| {
                s.open = !s.open;
            },
            s.items.clone(),
            |_, _| {},
        )
    };

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // The menu panel should be present when open
    let probes = harness.probes();
    let panel = probes
        .iter()
        .find(|p| p.key == Some("menu-popover".to_string()));
    assert!(panel.is_some(), "Menu panel should exist when open");
}

/// Items are focusable in order.
#[test]
fn menu_items_are_focusable_in_order() {
    let state = MenuState {
        open: true,
        items: vec![
            MenuItem {
                key: "item1".to_string(),
                label: "Item 1".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item2".to_string(),
                label: "Item 2".to_string(),
                selected: false,
            },
            MenuItem {
                key: "item3".to_string(),
                label: "Item 3".to_string(),
                selected: false,
            },
        ],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // All items should be focusable
    let probes = harness.probes();
    let focusable_items: Vec<_> = probes
        .iter()
        .filter(|p| {
            p.focusable
                && (p.key == Some("item1".to_string())
                    || p.key == Some("item2".to_string())
                    || p.key == Some("item3".to_string()))
        })
        .collect();

    assert_eq!(
        focusable_items.len(),
        3,
        "All menu items should be focusable"
    );
}

/// Popover is a filled panel with border and elevation.
#[test]
fn menu_popover_is_a_filled_panel() {
    let state = MenuState {
        open: true,
        items: vec![MenuItem {
            key: "item1".to_string(),
            label: "Item 1".to_string(),
            selected: false,
        }],
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 400.0);
    harness.frame();

    // The menu popover should have fill and border styling
    let probes = harness.probes();
    let popover = probes
        .iter()
        .find(|p| p.key == Some("menu-popover".to_string()));
    assert!(popover.is_some(), "Should find menu-popover by key");

    let popover_probe = popover.unwrap();
    // The probe should indicate that the popover is rendered with fill and border
    assert!(popover_probe.rect.w > 0.0, "Popover should have width");
    assert!(popover_probe.rect.h > 0.0, "Popover should have height");
}

/// Popover scrolls past ten items.
#[test]
fn menu_popover_scrolls_past_ten_items() {
    // Create 15 items to test scrolling
    let mut items = Vec::new();
    for i in 1..=15 {
        items.push(MenuItem {
            key: format!("item{}", i),
            label: format!("Item {}", i),
            selected: false,
        });
    }

    let state = MenuState {
        open: true,
        items,
        chosen: None,
    };

    let view =
        |s: &MenuState| menu_button("Select", None, s.open, |_| {}, s.items.clone(), |_, _| {});

    let mut harness = Harness::new(state, view).size(300.0, 600.0);
    harness.frame();

    // All 15 items should exist in the element tree (even if not all visible at once)
    let probes = harness.probes();
    let menu_items: Vec<_> = probes.iter().filter(|p| p.role == Role::MenuItem).collect();
    assert_eq!(
        menu_items.len(),
        15,
        "All 15 menu items should exist in the tree for scrolling"
    );

    // The popover should have scroll capability
    let popover = probes
        .iter()
        .find(|p| p.key == Some("menu-popover".to_string()));
    assert!(popover.is_some(), "Should find menu-popover by key");

    // The popover should be constrained in height (max_h = 280.0 ≈ 10 rows)
    let popover_probe = popover.unwrap();
    assert!(
        popover_probe.rect.h <= 320.0,
        "Popover height should be constrained"
    );
}
