//! Toast notification component.
//!
//! Toast is a short confirmation message that floats at the bottom of the window.
//! It is not focusable and not clickable — a dismissible message is a dialog.

use crate::{
    dot, row, text, Anchor, El, Elevation, Overlay, OverlayPlacement, Radius, Status, Tone,
};

/// A short confirmation, floating at the bottom of the window.
///
/// It has no timer of its own: the caller keeps the message and clears it.
pub fn toast<S: 'static>(message: &str, status: Status) -> El<S> {
    row((dot(status, 3.0), text(message).text_size(12.0)))
        .gap(8.0)
        .pad_x(12.0)
        .pad_y(8.0)
        .fill(Tone::Raised)
        .border(1.0, Tone::Border)
        .round(Radius::Pill)
        .elevation(Elevation::Overlay)
        .key("toast")
        .label(message)
        .layer(Anchor::Over)
        .overlay(Overlay::Popover)
        .overlay_placement(OverlayPlacement::bottom_center(-24.0))
}
