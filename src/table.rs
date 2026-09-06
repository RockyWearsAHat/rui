//! A table: an optional header row, then one row per entry.

use crate::accessibility::Role;
use crate::element::El;
use crate::input::Key;
use crate::style::{Align, Length, Radius, Tone};
use crate::theme::Metrics;
use crate::widgets::{col, divider, row as make_row};
use std::rc::Rc;

/// One column of a table: how wide it is and how its cells sit in it.
#[derive(Clone)]
pub struct Column {
    width: Length,
    align: Align,
}

/// A column of the given width. `Length::Fill(1.0)` takes what is left over.
pub fn column(key: &'static str, width: Length) -> Column {
    let _ = key;
    Column {
        width,
        align: Align::Start,
    }
}

impl Column {
    /// Where cells sit across the column. Default `Align::Start`.
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

/// One row and what it holds.
#[allow(clippy::type_complexity)]
pub struct Row<S> {
    #[allow(dead_code)]
    key: String,
    cells: Vec<El<S>>,
    on_click: Option<Box<dyn Fn(&mut S) + 'static>>,
    height: f32,
    selected: bool,
}

/// A row named `key`, with one cell per column.
pub fn table_row<S: 'static>(key: impl Into<String>, cells: Vec<El<S>>) -> Row<S> {
    Row {
        key: key.into(),
        cells,
        on_click: None,
        height: Metrics::DEFAULT.row_height,
        selected: false,
    }
}

impl<S: 'static> Row<S> {
    /// What clicking anywhere on the row does. A row with this is focusable.
    pub fn on_click(mut self, action: impl Fn(&mut S) + 'static) -> Self {
        self.on_click = Some(Box::new(action));
        self
    }

    /// Override the row's height. Default: the theme's `row_height`.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Draw it as the chosen one.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// A table: an optional header row, then one row per entry.
pub fn table<S: 'static>(
    columns: &[Column],
    header: Option<Vec<El<S>>>,
    rows: Vec<Row<S>>,
) -> El<S> {
    // Build the body rows
    let mut body_elements: Vec<El<S>> = Vec::new();
    let row_count = rows.len();

    for (idx, table_row) in rows.into_iter().enumerate() {
        // Build cells for this row with proper width and alignment
        let mut row_cells: Vec<El<S>> = Vec::new();
        for (cell_idx, cell) in table_row.cells.into_iter().enumerate() {
            if cell_idx < columns.len() {
                let col_def = &columns[cell_idx];
                let mut cell_el = cell.w(col_def.width).text_align(col_def.align).clip();

                // Fill columns get .grow()
                if matches!(col_def.width, Length::Fill(_)) {
                    cell_el = cell_el.grow();
                }

                row_cells.push(cell_el);
            } else {
                row_cells.push(cell);
            }
        }

        // Create the row element
        let mut row_el = make_row(row_cells)
            .key(format!("row-{}", idx))
            .role(Role::ListItem)
            .h(table_row.height);

        // Apply selected or interactive styling
        if table_row.selected {
            row_el = row_el.fill(Tone::Selection);
        } else if table_row.on_click.is_some() {
            row_el = row_el.hover_fill(Tone::Raised);
        }

        // Apply click handler and keyboard support if present
        if let Some(action) = table_row.on_click {
            let action = Rc::new(action);
            let action_click = action.clone();
            let action_key = action.clone();
            row_el = row_el
                .focusable()
                .on_click(move |state: &mut S| {
                    action_click(state);
                })
                .on_key(move |state: &mut S, key: Key, _| {
                    if matches!(key, Key::Enter | Key::Space) {
                        action_key(state);
                    }
                });
        }

        body_elements.push(row_el);

        // Add divider between rows (but not after the last row)
        if idx < row_count - 1 {
            body_elements.push(divider());
        }
    }

    // Build the header if present
    let mut table_contents: Vec<El<S>> = Vec::new();

    if let Some(header_cells) = header {
        let mut header_row_cells: Vec<El<S>> = Vec::new();
        for (cell_idx, cell) in header_cells.into_iter().enumerate() {
            if cell_idx < columns.len() {
                let col_def = &columns[cell_idx];

                // Extract text from El<S> - for now we'll wrap it in a container
                // The spec says header cells are "drawn as micro(label).color(Tone::Muted)"
                // This means we need to apply micro styling to the cell content
                let header_cell = cell
                    .text_size(11.0) // TYPE_MICRO
                    .color(Tone::Muted)
                    .w(col_def.width)
                    .text_align(col_def.align)
                    .clip();

                if matches!(col_def.width, Length::Fill(_)) {
                    header_row_cells.push(header_cell.grow());
                } else {
                    header_row_cells.push(header_cell);
                }
            } else {
                header_row_cells.push(cell.text_size(11.0).color(Tone::Muted).clip());
            }
        }

        let header_row_height = 38.0; // ROW_TABLE_HEAD
        let header_row = make_row(header_row_cells)
            .h(header_row_height)
            .fill(Tone::SurfaceDeep)
            .role(Role::ListItem);

        table_contents.push(header_row);

        // Add divider between header and body
        if !body_elements.is_empty() {
            table_contents.push(divider());
        }
    }

    // Add body rows
    table_contents.extend(body_elements);

    // Wrap everything in a col with table styling
    col(table_contents)
        .round(Radius::Panel)
        .border(1.0, Tone::Border)
        .clip()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Harness;
    use crate::widgets::{badge, caption, link};

    #[derive(Default)]
    struct St {
        clicked: bool,
    }

    /// A row shaped exactly like Forge's repo list: a `link()` (hover-
    /// coloured, no click handler of its own) and a `caption()` in the name
    /// cell, with the actual `on_click` on the row.
    fn view(_state: &St) -> El<St> {
        let columns = vec![
            column("name", Length::Fill(1.0)),
            column("branch", Length::Auto),
        ];
        let name_cell = col((link("forge").text_size(16.0).bold(), caption("desc")))
            .gap(4.0)
            .grow();
        let row = table_row("forge", vec![name_cell, badge("main")])
            .on_click(|state: &mut St| state.clicked = true);
        table(&columns, None, vec![row])
    }

    /// Regression test: clicking a `link()` nested in a table row's cell must
    /// still activate the row's own `on_click`.
    ///
    /// `link()` sets a hover colour (`El::hover_color`), which is enough to
    /// make `El::interactive()` true — a bare hover style still needs
    /// `interact()` to run so it knows when to draw itself. But
    /// [`resolve_hit`]'s hit-test used to treat *any* interactive element,
    /// hover-only ones included, as capturing the click for whatever point it
    /// covers — so a `link()` drawn deeper in the tree than the row that
    /// contains it stole the row's own click the moment the point landed on
    /// the link's text, which in a name column is most of the row. Nothing
    /// ever reached `table_row::on_click`.
    ///
    /// Fixed by splitting `El::interactive` (broad: gates whether `interact`
    /// bothers computing a response at all) from the new, narrower
    /// `El::captures_click` (real handlers and focus only) and having
    /// [`Hit`] track them separately — `target` for anything that can act on
    /// a click, `passive` for anything that merely watches the pointer — with
    /// `target` always winning. A `link()` with nothing behind it still gets
    /// its hover colour from `passive`; a `link()` inside a clickable row no
    /// longer takes the row's click away from it.
    #[test]
    fn clicking_on_the_name_cells_link_still_fires_the_rows_on_click() {
        let mut harness = Harness::new(St::default(), view).size(400.0, 200.0);
        harness.click_text("forge");
        assert!(
            harness.state().clicked,
            "clicking the repo name inside the row's first cell should still fire the row's on_click"
        );
    }
}
