//! Loading placeholders with a sweeping shimmer effect.

use crate::accessibility::Role;
use crate::element::El;
use crate::geom::{Rect, Size};
use crate::style::{Length, Radius, Tone};
use crate::widgets;

/// One shimmering placeholder bar.
pub fn skeleton<S: 'static>(width: Length, height: f32) -> El<S> {
    let bar = widgets::draw(Size::new(100.0, height), |painter, rect| {
        // Fill the background with the sunken tone
        painter.fill(rect, Radius::Units(4.0), Tone::Sunken);

        // Add the shimmer band that sweeps left to right
        let phase = painter.phase("skeleton", 1.4);

        // The band sweeps across the rectangle: phase goes from 0.0 to 1.0
        // We'll make it sweep left to right and repeat
        let band_width = rect.w * 0.2; // 20% of the width for the shimmer band
        let band_x = rect.x + (phase * (rect.w + band_width)) - band_width;

        // Create a shimmer rectangle that moves across
        let shimmer_rect = Rect::new(band_x, rect.y, band_width, rect.h);

        // Only paint the portion that overlaps with the main rect
        let overlap = shimmer_rect.intersect(rect);
        if !overlap.is_empty() {
            painter.fill(overlap, Radius::Units(4.0), Tone::Raised);
        }
    });

    bar.w(width).h(height).role(Role::Group).label("loading")
}

/// `rows` placeholder rows of `row_height`, each a bar of a plausibly
/// varying width, so a loading list does not read as a striped rectangle.
pub fn skeleton_rows<S: 'static>(rows: usize, row_height: f32) -> El<S> {
    let widths = [0.9, 0.65, 0.8, 0.5, 0.75];

    let items: Vec<El<S>> = (0..rows)
        .map(|i| {
            let width_fraction = widths[i % widths.len()];
            skeleton::<S>(Length::Fill(width_fraction), row_height).key(format!("skeleton-{}", i))
        })
        .collect();

    crate::widgets::col(items).gap(4.0)
}
