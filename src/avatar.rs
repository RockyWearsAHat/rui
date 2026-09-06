//! A square identicon derived from a name.

use crate::accessibility::Role;
use crate::color::Color;
use crate::element::El;
use crate::geom::{Rect, Size};
use crate::style::Radius;
use crate::widgets;

/// A square identicon derived from a name — the same name always draws the
/// same picture, on every machine, with no network and no image files.
pub fn avatar<S: 'static>(name: &str, size: f32) -> El<S> {
    let color = avatar_color(name);
    let hash = fnv_1a_hash(name.trim().to_lowercase().as_bytes());

    widgets::draw(Size::new(size, size), move |painter, rect| {
        // Background
        painter.fill(rect, Radius::Units(size * 0.25), crate::style::Tone::Sunken);

        // 5x5 grid with mirroring: columns 0-2 are hashed, 3-4 mirror 1-0
        let cell_size = rect.w / 5.0;
        let inset = size * 0.12;

        for row in 0..5 {
            for col in 0..5 {
                // Determine if this cell should be on
                let bit_index = if col < 3 {
                    row * 3 + col
                } else if col == 3 {
                    row * 3 + 1
                } else {
                    // col == 4
                    row * 3
                };

                let is_on = (hash >> bit_index) & 1 != 0;

                if is_on {
                    let x = rect.x + col as f32 * cell_size + inset;
                    let y = rect.y + row as f32 * cell_size + inset;
                    let cell_rect = Rect {
                        x,
                        y,
                        w: cell_size - 2.0 * inset,
                        h: cell_size - 2.0 * inset,
                    };
                    painter.fill(cell_rect, Radius::Units(0.0), color);
                }
            }
        }
    })
    .role(Role::Image)
    .label(name)
}

/// The colour that name resolves to, for anything that wants to match it.
pub fn avatar_color(name: &str) -> Color {
    let hash = fnv_1a_hash(name.trim().to_lowercase().as_bytes());
    let hue = (hash % 360) as f32;
    hsl_to_rgb(hue, 0.55, 0.58)
}

/// FNV-1a 64-bit hash function.
fn fnv_1a_hash(bytes: &[u8]) -> u64 {
    const FNV_64_PRIME: u64 = 0x100000001b3;
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_64_PRIME);
    }
    hash
}

/// Convert HSL to RGB.
///
/// hue: 0-360
/// saturation: 0-1
/// lightness: 0-1
fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> Color {
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_prime = hue / 60.0;
    let x = c * (1.0 - (hue_prime % 2.0 - 1.0).abs());
    let m = lightness - c / 2.0;

    let (r_prime, g_prime, b_prime) = if hue_prime < 1.0 {
        (c, x, 0.0)
    } else if hue_prime < 2.0 {
        (x, c, 0.0)
    } else if hue_prime < 3.0 {
        (0.0, c, x)
    } else if hue_prime < 4.0 {
        (0.0, x, c)
    } else if hue_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r_prime + m) * 255.0).round() as u8;
    let g = ((g_prime + m) * 255.0).round() as u8;
    let b = ((b_prime + m) * 255.0).round() as u8;

    Color::rgb(r, g, b)
}
