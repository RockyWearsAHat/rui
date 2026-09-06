//! Dropdown menu component.

use crate::theme::Metrics;
use crate::*;

/// One line of a menu.
#[derive(Clone)]
pub struct MenuItem {
    /// Its identity, for `El::key` and for tests.
    pub key: String,
    /// What it says.
    pub label: String,
    /// Whether it is the current choice — drawn with a tick.
    pub selected: bool,
}

/// A button that opens a list under itself.
///
/// The open flag and the item list belong to the caller: this holds nothing
/// between frames. `toggle` opens and closes; `choose` is called with the
/// index chosen and must also close the menu.
pub fn menu_button<S: 'static>(
    label: impl Into<String>,
    leading: Option<Icon>,
    open: bool,
    toggle: impl Fn(&mut S) + Copy + 'static,
    items: Vec<MenuItem>,
    choose: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    let label = label.into();
    let button_content = {
        let mut row_items: Vec<El<S>> = Vec::new();

        // Add leading icon if present
        if let Some(icon_kind) = leading {
            row_items.push(icon(icon_kind, 12.0));
        }

        // Add label text
        row_items.push(text(label.clone()));

        // Add chevron icon
        row_items.push(icon_tinted(Icon::Chevron, 12.0, Tone::Muted));

        row(row_items).gap(Metrics::DEFAULT.gap_small)
    };

    let button = button_content
        .h(Metrics::DEFAULT.control_height)
        .pad_x(Metrics::DEFAULT.padding)
        .round(Radius::Control)
        .border(1.0, Tone::Border)
        .hover_fill(Tone::Raised)
        .focusable()
        .on_click(move |s| toggle(s))
        .key("menu-button")
        .role(Role::Button)
        .on_key(move |s, key, _| {
            if key == Key::Escape {
                toggle(s);
            }
        });

    if !open {
        // Closed menu: just show the button
        button
    } else {
        // Open menu: button + dropdown panel with items
        let panel_items: Vec<El<S>> = items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let item_key = item.key.clone();
                let item_label = item.label.clone();
                let is_selected = item.selected;

                let tick = if is_selected {
                    icon_tinted(Icon::Check, 12.0, Tone::Accent)
                } else {
                    spacer().w(12.0)
                };

                row((tick, text(item_label)))
                    .gap(Metrics::DEFAULT.gap_small)
                    .h(Metrics::DEFAULT.control_height)
                    .pad_x(Metrics::DEFAULT.padding)
                    .hover_fill(Tone::Raised)
                    .focusable()
                    .on_click(move |s| choose(s, i))
                    .key(item_key)
                    .role(Role::MenuItem)
                    .on_key(move |s, key, _| {
                        if key == Key::Enter || key == Key::Space {
                            choose(s, i);
                        } else if key == Key::Escape {
                            toggle(s);
                        }
                    })
            })
            .collect();

        let dropdown_panel = col(panel_items)
            .layer(Anchor::Below)
            .overlay(Overlay::Dropdown)
            .overlay_placement(OverlayPlacement::top_start(0.0, 4.0))
            .fill(Tone::Raised)
            .border(1.0, Tone::Border)
            .round(Radius::Panel)
            .elevation(Elevation::Overlay)
            .min_w(220.0)
            .max_h(280.0)
            .scroll()
            .key("menu-popover")
            .on_key(move |s, key, _| {
                if key == Key::Escape {
                    toggle(s);
                }
            });

        col((button, dropdown_panel))
    }
}
