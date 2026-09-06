//! Icon set for Forge.
//!
//! Fifteen scalable marks, drawn with SDF shapes, never glyphs or images.

use crate::*;

/// The marks Forge needs. Drawn, never a font glyph and never an image file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// A folder with a tab on the top-left.
    Folder,
    /// A document with the top-right corner cut.
    File,
    /// Two circles joined by an arc.
    Branch,
    /// A ring on a horizontal capsule.
    Commit,
    /// A rotated polygon with a small circle.
    Tag,
    /// Two offset rounded rectangle outlines.
    Clone,
    /// A two-segment checkmark stroke.
    Check,
    /// Two crossed capsules.
    Cross,
    /// A two-segment stroke pointing down.
    Chevron,
    /// Two overlapping rounded rectangle outlines.
    Copy,
    /// A ring with a capsule handle.
    Search,
    /// A filled circle at 0.35 of the box.
    Dot,
    /// A ring with a broken sweep and a two-segment hand.
    History,
    /// A vertical capsule with an arrowhead over a base line.
    Download,
    /// Two mirrored chevron strokes.
    Code,
}

impl Icon {
    /// The name of this icon, for accessibility labels.
    fn name(&self) -> &'static str {
        match self {
            Icon::Folder => "folder",
            Icon::File => "file",
            Icon::Branch => "branch",
            Icon::Commit => "commit",
            Icon::Tag => "tag",
            Icon::Clone => "clone",
            Icon::Check => "check",
            Icon::Cross => "cross",
            Icon::Chevron => "chevron",
            Icon::Copy => "copy",
            Icon::Search => "search",
            Icon::Dot => "dot",
            Icon::History => "history",
            Icon::Download => "download",
            Icon::Code => "code",
        }
    }
}

/// One mark, `size` units square, in `Tone::Muted`.
pub fn icon<S: 'static>(kind: Icon, size: f32) -> El<S> {
    icon_tinted(kind, size, Tone::Muted)
}

/// The same, in a colour you choose.
pub fn icon_tinted<S: 'static>(kind: Icon, size: f32, tone: Tone) -> El<S> {
    widgets::draw(Size::new(size, size), move |painter, rect| {
        let scale = size / 16.0;
        let stroke_width = scale.max(1.0);
        let center = rect.center();
        let color = painter.color(tone);
        let paint = Paint::Solid(color);

        match kind {
            Icon::Folder => {
                // Folder: rounded_rect body with a raised tab on the top-left
                let body = rounded_rect(
                    Rect::new(
                        center.x - 6.0 * scale,
                        center.y - 4.0 * scale,
                        12.0 * scale,
                        10.0 * scale,
                    ),
                    2.0 * scale,
                );
                let tab = rounded_rect(
                    Rect::new(
                        center.x - 6.0 * scale,
                        center.y - 6.5 * scale,
                        5.0 * scale,
                        2.5 * scale,
                    ),
                    1.0 * scale,
                );
                painter.sculpt(
                    &body,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &tab,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::File => {
                // File: rounded_rect with the top-right corner cut by a polygon
                let body = rounded_rect(
                    Rect::new(
                        center.x - 5.5 * scale,
                        center.y - 6.5 * scale,
                        11.0 * scale,
                        13.0 * scale,
                    ),
                    1.5 * scale,
                );
                let cut = polygon(vec![
                    Point::new(center.x + 5.5 * scale, center.y - 6.5 * scale),
                    Point::new(center.x + 5.5 * scale, center.y - 3.0 * scale),
                    Point::new(center.x + 2.0 * scale, center.y - 6.5 * scale),
                ]);
                let shape = body - cut;
                painter.sculpt(
                    &shape,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Branch => {
                // Branch: two circles joined by an arc
                let circle1 = circle(Point::new(center.x - 4.0 * scale, center.y), 2.0 * scale);
                let circle2 = circle(Point::new(center.x + 4.0 * scale, center.y), 2.0 * scale);
                let connection = arc(center, 4.0 * scale, stroke_width, 0.0, std::f32::consts::PI);
                painter.sculpt(
                    &circle1,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &circle2,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &connection,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Commit => {
                // Commit: a ring on a horizontal capsule
                let capsule = capsule(
                    Point::new(center.x - 4.0 * scale, center.y),
                    Point::new(center.x + 4.0 * scale, center.y),
                    1.5 * scale,
                );
                let ring = ring(center, 2.5 * scale, stroke_width);
                painter.sculpt(
                    &capsule,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &ring,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Tag => {
                // Tag: a rotated polygon with a small circle eye
                let rotation = std::f32::consts::PI / 4.0;
                let size_scale = 4.0 * scale;
                let corners = vec![
                    Point::new(
                        center.x + size_scale * rotation.cos(),
                        center.y + size_scale * rotation.sin(),
                    ),
                    Point::new(
                        center.x + size_scale * (rotation + std::f32::consts::PI * 2.0 / 3.0).cos(),
                        center.y + size_scale * (rotation + std::f32::consts::PI * 2.0 / 3.0).sin(),
                    ),
                    Point::new(
                        center.x + size_scale * (rotation + std::f32::consts::PI * 4.0 / 3.0).cos(),
                        center.y + size_scale * (rotation + std::f32::consts::PI * 4.0 / 3.0).sin(),
                    ),
                ];
                let tag = polygon(corners);
                let eye = circle(center, 1.0 * scale);
                painter.sculpt(
                    &tag,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(&eye, &paint, Sculpt::Fill);
            }
            Icon::Clone => {
                // Clone: two offset rounded_rect outlines
                let rect1 = rounded_rect(
                    Rect::new(
                        center.x - 6.0 * scale,
                        center.y - 5.0 * scale,
                        9.0 * scale,
                        8.0 * scale,
                    ),
                    1.5 * scale,
                );
                let rect2 = rounded_rect(
                    Rect::new(
                        center.x - 3.0 * scale,
                        center.y - 2.0 * scale,
                        9.0 * scale,
                        8.0 * scale,
                    ),
                    1.5 * scale,
                );
                painter.sculpt(
                    &rect1,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &rect2,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Check => {
                // Check: a two-segment polygon stroke (checkmark)
                let path = polygon(vec![
                    Point::new(center.x - 3.0 * scale, center.y + 1.0 * scale),
                    Point::new(center.x - 0.5 * scale, center.y + 3.0 * scale),
                    Point::new(center.x + 4.0 * scale, center.y - 2.5 * scale),
                ]);
                painter.sculpt(
                    &path,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Cross => {
                // Cross: two crossed capsules
                let h_line = capsule(
                    Point::new(center.x - 4.0 * scale, center.y),
                    Point::new(center.x + 4.0 * scale, center.y),
                    1.5 * scale,
                );
                let v_line = capsule(
                    Point::new(center.x, center.y - 4.0 * scale),
                    Point::new(center.x, center.y + 4.0 * scale),
                    1.5 * scale,
                );
                painter.sculpt(
                    &h_line,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &v_line,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Chevron => {
                // Chevron: a two-segment polygon stroke pointing down
                let chevron = polygon(vec![
                    Point::new(center.x - 3.5 * scale, center.y - 2.0 * scale),
                    Point::new(center.x, center.y + 2.5 * scale),
                    Point::new(center.x + 3.5 * scale, center.y - 2.0 * scale),
                ]);
                painter.sculpt(
                    &chevron,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Copy => {
                // Copy: two overlapping rounded_rect outlines
                let rect1 = rounded_rect(
                    Rect::new(
                        center.x - 6.0 * scale,
                        center.y - 5.5 * scale,
                        9.0 * scale,
                        8.0 * scale,
                    ),
                    1.5 * scale,
                );
                let rect2 = rounded_rect(
                    Rect::new(
                        center.x - 2.5 * scale,
                        center.y - 2.0 * scale,
                        9.0 * scale,
                        8.0 * scale,
                    ),
                    1.5 * scale,
                );
                painter.sculpt(
                    &rect1,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &rect2,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Search => {
                // Search: a ring with a capsule handle
                let ring = ring(
                    Point::new(center.x - 1.5 * scale, center.y - 1.5 * scale),
                    3.5 * scale,
                    stroke_width,
                );
                let handle = capsule(
                    Point::new(center.x + 2.0 * scale, center.y + 2.0 * scale),
                    Point::new(center.x + 5.0 * scale, center.y + 5.0 * scale),
                    1.0 * scale,
                );
                painter.sculpt(
                    &ring,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &handle,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Dot => {
                // Dot: a filled circle at 0.35 of the box
                let dot = circle(center, (8.0 * 0.35) * scale);
                painter.sculpt(&dot, &paint, Sculpt::Fill);
            }
            Icon::History => {
                // History: a ring with a broken sweep and a two-segment hand
                let history_ring = ring(center, 4.0 * scale, stroke_width);
                let hand = polygon(vec![
                    center,
                    Point::new(center.x - 1.0 * scale, center.y - 3.0 * scale),
                    Point::new(center.x + 1.5 * scale, center.y - 2.0 * scale),
                ]);
                painter.sculpt(
                    &history_ring,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &hand,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Download => {
                // Download: a vertical capsule with an arrowhead over a base line
                let capsule_shape = capsule(
                    Point::new(center.x, center.y - 3.0 * scale),
                    Point::new(center.x, center.y + 2.0 * scale),
                    1.5 * scale,
                );
                let arrow = polygon(vec![
                    Point::new(center.x, center.y + 2.5 * scale),
                    Point::new(center.x - 2.0 * scale, center.y + 0.5 * scale),
                    Point::new(center.x + 2.0 * scale, center.y + 0.5 * scale),
                ]);
                let base = capsule(
                    Point::new(center.x - 3.5 * scale, center.y + 5.0 * scale),
                    Point::new(center.x + 3.5 * scale, center.y + 5.0 * scale),
                    0.75 * scale,
                );
                painter.sculpt(
                    &capsule_shape,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(&arrow, &paint, Sculpt::Fill);
                painter.sculpt(
                    &base,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
            Icon::Code => {
                // Code: two mirrored chevron strokes
                let left = polygon(vec![
                    Point::new(center.x - 1.0 * scale, center.y - 3.0 * scale),
                    Point::new(center.x - 3.5 * scale, center.y),
                    Point::new(center.x - 1.0 * scale, center.y + 3.0 * scale),
                ]);
                let right = polygon(vec![
                    Point::new(center.x + 1.0 * scale, center.y - 3.0 * scale),
                    Point::new(center.x + 3.5 * scale, center.y),
                    Point::new(center.x + 1.0 * scale, center.y + 3.0 * scale),
                ]);
                painter.sculpt(
                    &left,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
                painter.sculpt(
                    &right,
                    &paint,
                    Sculpt::Stroke {
                        width: stroke_width,
                    },
                );
            }
        }
    })
    .role(Role::Image)
    .label(kind.name())
    .w(size)
    .h(size)
}
