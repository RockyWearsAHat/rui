//! Breadcrumb navigation component for displaying and navigating through paths.

use crate::{link, row, text, Align, El, Tone};

/// A path, every segment but the last clickable.
///
/// `choose` is called with the index of the segment clicked, so the caller
/// rebuilds the path from `segments[..=index]`.
pub fn breadcrumb<S: 'static>(
    segments: &[&str],
    choose: impl Fn(&mut S, usize) + Copy + 'static,
) -> El<S> {
    const S2: f32 = 4.0;
    const HIT_MIN: f32 = 28.0;

    let mut children: Vec<El<S>> = Vec::new();

    for (i, &segment) in segments.iter().enumerate() {
        if i > 0 {
            // Add separator between segments
            children.push(text("/").color(Tone::Muted).key(format!("sep-{}", i)));
        }

        if i < segments.len() - 1 {
            // Clickable link for all but the last segment
            let seg = segment.to_string();
            children.push(
                link(seg.clone())
                    .key(segment)
                    .min_h(HIT_MIN)
                    .on_click(move |s| choose(s, i)),
            );
        } else {
            // Non-clickable text for the last segment
            children.push(text(segment).bold().color(Tone::Text).key(segment));
        }
    }

    row(children).gap(S2).h(HIT_MIN).align(Align::Center).clip()
}
