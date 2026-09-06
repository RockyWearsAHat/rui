//! A row of tabs, each with a count and a status lamp.

use crate::accessibility::Role;
use crate::element::El;
use crate::input::Key;
use crate::style::Tone;
use crate::theme::{Metrics, Status, MICRO_SIZE};
use crate::widgets::{col, divider, dot, row, text};

/// A tab: its word, the number beside it, and a status lamp if it has one.
pub struct TabItem<'a> {
    /// The word on the tab.
    pub label: &'a str,
    /// A count drawn as a badge after the label, or `None`.
    pub count: Option<usize>,
    /// A status dot after the label — how the Checks tab reports CI.
    pub status: Option<Status>,
}

/// A row of tabs, one of them chosen.
///
/// Each tab is as wide as its own word, with a 2px accent bar under the chosen one.
/// Counts appear as badges and status as a dot. The row supports keyboard navigation
/// with left/right arrow keys wrapping at both ends.
pub fn tab_bar<S: 'static>(
    items: &[TabItem<'_>],
    selected: usize,
    choose: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    use crate::element::Node;

    let item_count = items.len();

    let tabs: Vec<El<S>> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let chosen = index == selected;

            // Build the content: label + optional count + optional status
            let mut content_elements: Vec<El<S>> = vec![];
            content_elements.push(text(item.label).text_size(12.5));

            if let Some(count) = item.count {
                content_elements.push(
                    text(count.to_string())
                        .text_size(MICRO_SIZE)
                        .color(if chosen { Tone::Accent } else { Tone::Muted }),
                );
            }

            if let Some(status) = item.status {
                content_elements.push(dot(status, 3.0));
            }

            col((
                row(content_elements)
                    .h(26.0)
                    .pad_x(Metrics::DEFAULT.padding),
                El::of(Node::Stack)
                    .h(2.0)
                    .fill(if chosen { Tone::Accent } else { Tone::Clear }),
            ))
            .key(item.label)
            .color(if chosen { Tone::Accent } else { Tone::Muted })
            .hover_color(Tone::Text)
            .role(Role::Tab)
            .selected(chosen)
            .on_click(move |state: &mut S| choose(state, index))
            .on_key(move |state: &mut S, key: Key, _| match key {
                Key::Right => {
                    let next = (selected + 1) % item_count;
                    choose(state, next);
                }
                Key::Left => {
                    let next = if selected == 0 {
                        item_count - 1
                    } else {
                        selected - 1
                    };
                    choose(state, next);
                }
                _ => {}
            })
        })
        .collect();

    col((row(tabs).role(Role::TabList), divider())).h(Metrics::DEFAULT.control_height)
}
