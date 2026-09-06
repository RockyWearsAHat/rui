//! A label that appears above the child when open.

use crate::element::El;
use crate::overlay::{Overlay, OverlayPlacement};
use crate::style::{Anchor, Radius, Tone};
use crate::widgets::{col, micro, panel};
use crate::Elevation;

/// A label that appears above `child` while `open`.
///
/// The open flag and any delay belong to the caller: `on_hover` fires with
/// true when the pointer arrives and false when it leaves, and the caller
/// decides what that means.
pub fn tooltip<S: 'static>(
    child: El<S>,
    label: impl Into<String>,
    open: bool,
    on_hover: impl Fn(&mut S, bool) + 'static,
) -> El<S> {
    let label_str = label.into();

    // The bubble appears only when open
    let bubble = if open {
        Some(
            panel(micro(label_str.clone()))
                .key("tooltip")
                .layer(Anchor::Above)
                .overlay(Overlay::Popover)
                .overlay_placement(OverlayPlacement::bottom_center(-6.0))
                .fill(Tone::Raised)
                .border(1.0, Tone::Border)
                .round(Radius::Units(6.0)) // R_SMALL = 6.0
                .pad_x(8.0) // S3 = 8.0
                .pad_y(4.0) // S2 = 4.0
                .elevation(Elevation::Overlay),
        )
    } else {
        None
    };

    // The child always has the label for accessibility
    let labeled_child = child.on_hover(on_hover).label(label_str);

    // Build the column with child and optional bubble
    if let Some(b) = bubble {
        col((labeled_child, b))
    } else {
        col(labeled_child)
    }
}
