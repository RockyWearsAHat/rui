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
