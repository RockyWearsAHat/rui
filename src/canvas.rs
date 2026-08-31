//! The pixel buffer everything is drawn into, and the shapes it can draw.
//!
//! # One buffer, one format, every platform
//!
//! Pixels are `0xAARRGGBB` words. Read as bytes on a little-endian machine that
//! is B, G, R, A — which is exactly what Core Graphics wants from a
//! `kCGImageAlphaNoneSkipFirst | kCGBitmapByteOrder32Little` bitmap, what a
//! Win32 32-bit `BI_RGB` device-independent bitmap wants, and what an X11
//! `TrueColor` visual at depth 24 or 32 wants. One format satisfies all three
//! backends, so presenting a frame is a copy and never a conversion.
//!
//! The buffer is always opaque. Nothing composites this window against anything
//! else, so alpha is a property of the *paint*, not of the surface, and
//! [`crate::color::blend_over`] can drop the divide that a translucent
//! destination would need.
//!
//! # Logical units in, device pixels out
//!
//! Every public method takes logical coordinates and multiplies by [`scale`]
//! itself. That conversion happens here and nowhere else, which is what lets a
//! layout be written once and be correct on a HiDPI display. The exception is
//! [`Canvas::fill_mask`], which takes device pixels: a glyph is rasterised at
//! the size it will actually occupy, so rounding its position anywhere but at
//! the device grid would blur it.
//!
//! # Antialiasing
//!
//! Every shape is a rectangle with shaped corners, and every one of them is
//! drawn from its signed distance field: the distance from a pixel's centre to
//! the shape's edge, turned into coverage by `clamp(0.5 - distance, 0, 1)`. This
//! is one function for fills and strokes both, it is exact rather than sampled,
//! and it antialiases fractional coordinates without a supersampling pass. A
//! circle is a square rounded by half its side; a hairline is a stroke.
//!
//! There are two corner shapes, [`Corner::Round`] and [`Corner::Cut`], and they
//! cost the same: both are a distance field evaluated over the same band of
//! pixels, and neither is a path. That matters because the corner is the single
//! strongest signal of what an interface *is* — a rounded corner reads as a card
//! in a document, and a cut one reads as a machined panel — and a toolkit that
//! can only draw one of them has already decided.
//!
//! # Gradients and glows cost what a flat fill costs
//!
//! Two things above a flat fill are what stop an interface reading as a grid of
//! grey boxes: a surface that shades from top to bottom, and a halo around the
//! control a view is mostly for.
//!
//! Both are drawn by the same scan as everything else. A gradient here is
//! vertical only, which is the one direction that costs nothing: a row of a
//! vertical gradient is a single colour, so the bulk span that writes a panel's
//! interior still writes one word repeatedly, and the mix happens once per row
//! rather than once per pixel. A horizontal or angled gradient would have to be
//! evaluated per pixel and is deliberately not offered.
//!
//! A glow is the distance field read *outside* the shape instead of across its
//! edge, so it scans the band it occupies and never the area it surrounds — the
//! same trade that makes a one-pixel outline cheap. It stops at the shape's edge
//! because whatever casts it is drawn on top.
//!
//! [`scale`]: Canvas::scale

use crate::color::{blend_over, Color};
use crate::geom::Rect;

/// What shape a rectangle's four corners are.
///
/// Every fill, stroke, and glow takes one of these rather than a bare radius, so
/// that "what shape is a panel" is a decision made once in the theme instead of
/// separately by each widget that happens to draw one.
///
/// # Why a cut corner is offered at all
///
/// A rounded corner is the corner of a card, and a card is a piece of paper. The
/// console is not showing paper — it is an instrument reporting on a machine —
/// and a corner cut at forty-five degrees is what a panel bolted into a rack
/// looks like. The two are the same amount of work to draw and one word apart to
/// choose between, which is the point: the interface's whole character can be
/// changed from the theme without a widget being touched.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    /// A right angle.
    Square,
    /// A quarter-circle of this radius, in logical units.
    Round(f32),
    /// A forty-five degree cut running this far along each edge it meets.
    Cut(f32),
}

impl Corner {
    /// How far the corner treatment reaches along an edge.
    ///
    /// The same measurement for both shapes, which is what lets one clamp, one
    /// interior-span calculation, and one set of bounds serve them both.
    pub fn size(self) -> f32 {
        match self {
            Self::Square => 0.0,
            Self::Round(size) | Self::Cut(size) => size,
        }
    }

    /// The same corner, grown by `amount` — what a shape's outer edge needs.
    ///
    /// A glow cast from a shape spread outward, or a focus ring drawn around
    /// one, has to keep the corner *shape* while taking the larger size, or a cut
    /// panel ends up haloed by a rounded one.
    pub fn grown(self, amount: f32) -> Self {
        match self {
            Self::Square => Self::Square,
            Self::Round(size) => Self::Round(size + amount),
            Self::Cut(size) => Self::Cut(size + amount),
        }
    }

    /// The same corner at `size`, keeping which shape it is.
    pub fn resized(self, size: f32) -> Self {
        match self {
            Self::Square => Self::Square,
            Self::Round(_) => Self::Round(size),
            Self::Cut(_) => Self::Cut(size),
        }
    }
}

/// A coverage bitmap: how much of each pixel a shape covers, 0 to 255.
///
/// Produced by the glyph rasteriser and consumed by [`Canvas::fill_mask`]. It
/// carries no colour, so one rasterised glyph serves every colour it is ever
/// drawn in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mask {
    /// Width in device pixels.
    pub width: u32,
    /// Height in device pixels.
    pub height: u32,
    /// `width * height` coverage values, row-major.
    pub coverage: Vec<u8>,
}

impl Mask {
    /// An empty mask, which draws nothing.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Whether the mask covers no area.
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// A rectangle of device pixels, in whole pixels.
///
/// Clipping is integral because a partly-clipped pixel is not a thing the
/// buffer can represent; rounding outward would let drawing escape its region
/// by up to a pixel, so bounds round *inward* to the pixels wholly inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PixelBounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl PixelBounds {
    fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }

    fn is_empty(self) -> bool {
        self.right <= self.left || self.bottom <= self.top
    }
}

/// The smallest backing scale a display may claim before it is disbelieved.
///
/// A zero or negative scale collapses every layout to nothing, and a windowing
/// system reporting one is a bug to survive rather than to propagate.
const MIN_SCALE: f32 = 0.25;

/// The largest backing scale a display may claim, for the same reason.
const MAX_SCALE: f32 = 8.0;

/// An unmarked pixel: black, fully opaque.
const OPAQUE_BLACK: u32 = 0xff00_0000;

/// A buffer of pixels and the operations that mark it.
pub struct Canvas {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
    scale: f32,
    clip: PixelBounds,
}

impl Canvas {
    /// A canvas of `width` by `height` *device* pixels at `scale`, filled black.
    ///
    /// `scale` is the display's backing scale factor: 1.0 for an ordinary
    /// screen, 2.0 for a Retina one. It is clamped to a sane range because a
    /// zero or negative scale would collapse every layout to nothing, and a
    /// windowing system reporting one is a bug we should survive rather than
    /// propagate.
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        let mut canvas = Self {
            pixels: Vec::new(),
            width: 0,
            height: 0,
            scale: 1.0,
            clip: PixelBounds {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
        };
        canvas.resize(width, height, scale);
        canvas
    }

    /// Resizes the buffer, discarding its contents.
    ///
    /// Contents are discarded rather than rescaled: the caller redraws every
    /// frame, so preserving them would cost a copy that is immediately painted
    /// over. Resizing also resets the clip, since the old one may no longer be
    /// inside the surface.
    ///
    /// The allocation is kept and re-lengthened rather than replaced. A window
    /// being dragged by its corner resizes on every frame of the gesture, and a
    /// surface is megabytes; asking the allocator for a fresh one sixty times a
    /// second is a page fault per four kilobytes of window, which is felt as the
    /// resize stuttering.
    pub fn resize(&mut self, width: u32, height: u32, scale: f32) {
        self.width = width;
        self.height = height;
        self.scale = if scale.is_finite() {
            scale.clamp(MIN_SCALE, MAX_SCALE)
        } else {
            1.0
        };
        self.clip = PixelBounds {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        self.pixels.clear();
        self.pixels
            .resize((width as usize) * (height as usize), OPAQUE_BLACK);
    }

    /// Width in device pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in device pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Device pixels per logical unit.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// The whole surface, in logical units.
    pub fn bounds(&self) -> Rect {
        Rect::new(
            0.0,
            0.0,
            self.width as f32 / self.scale,
            self.height as f32 / self.scale,
        )
    }

    /// The pixels, for a backend to present.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// The current clip, in logical units.
    pub fn clip(&self) -> Rect {
        Rect::new(
            self.clip.left as f32 / self.scale,
            self.clip.top as f32 / self.scale,
            (self.clip.right - self.clip.left) as f32 / self.scale,
            (self.clip.bottom - self.clip.top) as f32 / self.scale,
        )
    }

    /// Narrows the clip to `rect` and answers the previous one, for restoring.
    ///
    /// The new clip is always the *intersection* with the old one, so a nested
    /// region can never draw outside its parent however it was called.
    #[must_use = "the previous clip must be restored, or later drawing stays clipped"]
    pub fn push_clip(&mut self, rect: Rect) -> Rect {
        let previous = self.clip();
        self.clip = self.clip.intersect(self.device_bounds(rect));
        previous
    }

    /// Restores a clip taken from [`Canvas::push_clip`].
    pub fn pop_clip(&mut self, previous: Rect) {
        self.clip = PixelBounds {
            left: 0,
            top: 0,
            right: self.width as i32,
            bottom: self.height as i32,
        }
        .intersect(self.device_bounds(previous));
    }

    /// Whether anything drawn inside `rect` could be visible.
    ///
    /// Widgets ask this to skip work — a list scrolled past its thousandth row
    /// should cost nothing to not draw.
    pub fn is_visible(&self, rect: Rect) -> bool {
        !self.clip.intersect(self.device_bounds(rect)).is_empty()
    }

    /// Paints the entire surface, ignoring the clip.
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color.to_argb() | 0xff00_0000);
    }

    /// The same, shading from `top` at the first row to `bottom` at the last.
    ///
    /// A window filled with one flat value has no depth for anything to sit in
    /// front of; shading it very slightly downward gives every panel something
    /// to be lit against. This replaces the flat clear rather than being drawn
    /// over it, so it costs a single pass exactly as the flat one does.
    pub fn clear_vertical(&mut self, top: Color, bottom: Color) {
        if self.height == 0 {
            return;
        }
        let last = (self.height - 1).max(1) as f32;
        for y in 0..self.height {
            let word = top.mix(bottom, y as f32 / last).to_argb() | 0xff00_0000;
            let row = (y as usize) * self.width as usize;
            self.pixels[row..row + self.width as usize].fill(word);
        }
    }

    /// Fills a rectangle with square corners.
    ///
    /// # Coordinate System Contract
    ///
    /// The `rect` parameter is in **window-logical units**, not device pixels.
    /// All coordinates are automatically scaled by the display's scale factor.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.fill(rect, Corner::Square, color);
    }

    /// Fills a rectangle whose corners are shaped by `corner`.
    ///
    /// The corner size is clamped to half the shorter side, so asking for more
    /// than the shape can hold yields a capsule — or, cut, a diamond — rather
    /// than an inverted corner.
    ///
    /// # Coordinate System Contract
    ///
    /// The `rect` parameter is in **window-logical units**, not device pixels.
    /// All coordinates are automatically scaled by the display's scale factor.
    pub fn fill(&mut self, rect: Rect, corner: Corner, color: Color) {
        if !color.is_visible() || rect.is_empty() {
            return;
        }
        let device = self.device_rect(rect);
        let corner = self.device_corner(corner, device);

        if corner.size() <= 0.0 && color.a == 255 && is_pixel_aligned(device) {
            self.fill_aligned_opaque(device, color);
            return;
        }
        self.fill_shape(device, corner, Paint::solid(color));
    }

    /// Fills a shaped rectangle shading from `top` at its top edge to `bottom`.
    ///
    /// Vertical because that is the direction a row of pixels is constant in,
    /// which keeps this exactly as cheap as a flat fill; see the module's own
    /// notes. A gradient between two equal colours is a flat fill and is drawn
    /// as one.
    ///
    /// # Coordinate System Contract
    ///
    /// The `rect` parameter is in **window-logical units**, not device pixels.
    /// All coordinates are automatically scaled by the display's scale factor.
    pub fn fill_vertical(&mut self, rect: Rect, corner: Corner, top: Color, bottom: Color) {
        if rect.is_empty() || (!top.is_visible() && !bottom.is_visible()) {
            return;
        }
        let device = self.device_rect(rect);
        let corner = self.device_corner(corner, device);
        self.fill_shape(device, corner, Paint::vertical(device, top, bottom));
    }

    /// Draws a soft halo outside a shaped rectangle's edge.
    ///
    /// `blur` is how far the halo reaches beyond the shape, in logical units,
    /// fading from `color` at the edge to nothing at that distance. Nothing
    /// inside the shape is touched: a glow exists to be seen *around* whatever
    /// casts it, and the caster is drawn on top.
    ///
    /// `spread` grows the shape the halo is cast from before it is drawn, which
    /// is how a shadow is offset outward from a small control without the halo
    /// hugging its outline so tightly that it reads as a blurred border. The
    /// corner grows with it, so a cut panel is haloed by a cut shape.
    ///
    /// # Coordinate System Contract
    ///
    /// The `rect` and `blur`/`spread` parameters are in **window-logical units**.
    /// All coordinates are automatically scaled by the display's scale factor.
    pub fn shadow(&mut self, rect: Rect, corner: Corner, blur: f32, spread: f32, color: Color) {
        if !color.is_visible() || rect.is_empty() || blur <= 0.0 {
            return;
        }
        let cast = rect.expand(crate::geom::Insets::uniform(spread));
        let device = self.device_rect(cast);
        let corner = self.device_corner(corner.grown(spread), device);
        self.glow_shape(device, corner, blur * self.scale, color);
    }

    /// Strokes the outline of a shaped rectangle, centred on its edge.
    ///
    /// Centred rather than inside or outside because that is what makes a
    /// one-pixel border land on one row of pixels instead of straddling two and
    /// rendering as two grey rows.
    ///
    /// # Coordinate System Contract
    ///
    /// The `rect` and `thickness` parameters are in **window-logical units**.
    /// All coordinates are automatically scaled by the display's scale factor.
    pub fn stroke(&mut self, rect: Rect, corner: Corner, thickness: f32, color: Color) {
        if !color.is_visible() || rect.is_empty() || thickness <= 0.0 {
            return;
        }
        let device = self.device_rect(rect);
        let corner = self.device_corner(corner, device);
        self.stroke_shape(device, corner, thickness * self.scale, color);
    }

    /// A logical corner in device pixels, clamped to what the shape can hold.
    ///
    /// Both the scaling and the clamp happen here rather than in each of the four
    /// entry points, so a corner cannot be scaled twice or clamped against the
    /// wrong rectangle.
    fn device_corner(&self, corner: Corner, device: Rect) -> Corner {
        let limit = device.w.min(device.h) / 2.0;
        corner.resized((corner.size() * self.scale).clamp(0.0, limit.max(0.0)))
    }

    /// Draws a coverage mask in `color`, with its top-left at a *device* pixel.
    ///
    /// Device coordinates because masks come from the glyph rasteriser, which
    /// renders at the exact size and subpixel offset the text will occupy.
    pub fn fill_mask(&mut self, left: i32, top: i32, mask: &Mask, color: Color) {
        if !color.is_visible() || mask.is_empty() {
            return;
        }
        let bounds = self.clip.intersect(PixelBounds {
            left,
            top,
            right: left + mask.width as i32,
            bottom: top + mask.height as i32,
        });
        if bounds.is_empty() {
            return;
        }

        for y in bounds.top..bounds.bottom {
            let mask_row = ((y - top) as usize) * mask.width as usize;
            let pixel_row = (y as usize) * self.width as usize;
            for x in bounds.left..bounds.right {
                let coverage = mask.coverage[mask_row + (x - left) as usize];
                if coverage == 0 {
                    continue;
                }
                let index = pixel_row + x as usize;
                self.pixels[index] = blend_over(self.pixels[index], color, coverage);
            }
        }
    }

    /// A logical rectangle in device pixels, unrounded.
    fn device_rect(&self, rect: Rect) -> Rect {
        Rect {
            x: rect.x * self.scale,
            y: rect.y * self.scale,
            w: rect.w * self.scale,
            h: rect.h * self.scale,
        }
    }

    /// The whole pixels a logical rectangle covers, rounded outward.
    ///
    /// Outward for clipping, so that a region's own antialiased edge is not
    /// clipped away by the region it belongs to.
    fn device_bounds(&self, rect: Rect) -> PixelBounds {
        let device = self.device_rect(rect);
        PixelBounds {
            left: device.min_x().floor().max(0.0) as i32,
            top: device.min_y().floor().max(0.0) as i32,
            right: (device.max_x().ceil().max(0.0) as i32).min(self.width as i32),
            bottom: (device.max_y().ceil().max(0.0) as i32).min(self.height as i32),
        }
    }

    /// The fast path: whole pixels, square corners, no transparency.
    ///
    /// This is the common case — panel backgrounds, table rows, separators —
    /// and it is a row-wise fill with no arithmetic per pixel.
    fn fill_aligned_opaque(&mut self, device: Rect, color: Color) {
        let bounds = self.clip.intersect(PixelBounds {
            left: device.min_x() as i32,
            top: device.min_y() as i32,
            right: device.max_x() as i32,
            bottom: device.max_y() as i32,
        });
        if bounds.is_empty() {
            return;
        }
        let word = color.to_argb() | 0xff00_0000;
        for y in bounds.top..bounds.bottom {
            let row = (y as usize) * self.width as usize;
            let start = row + bounds.left as usize;
            let end = row + bounds.right as usize;
            self.pixels[start..end].fill(word);
        }
    }

    /// Fills a rounded rectangle from its signed distance field.
    ///
    /// # Why the interior is not evaluated
    ///
    /// Coverage only varies within about a pixel of the edge; everywhere else it
    /// is exactly one. Evaluating the distance field across a whole panel means
    /// millions of square roots to discover that the middle is, in fact, filled.
    /// So each row is split: the span that is unambiguously inside is written
    /// straight out, and the distance field is evaluated only on the margins.
    ///
    /// For the panels this console is made of, that is the difference between a
    /// frame costing tens of milliseconds and costing one.
    fn fill_shape(&mut self, device: Rect, corner: Corner, paint: Paint) {
        let bounds = self.clip.intersect(PixelBounds {
            left: (device.min_x() - 1.0).floor() as i32,
            top: (device.min_y() - 1.0).floor() as i32,
            right: (device.max_x() + 1.0).ceil() as i32,
            bottom: (device.max_y() + 1.0).ceil() as i32,
        });
        if bounds.is_empty() {
            return;
        }

        let size = corner.size();
        let field = DistanceField::new(device, corner);
        // A pixel is wholly inside once it is a pixel clear of the straight
        // edges and of the corner treatment, which is `size` in from each side —
        // true of an arc and of a cut alike, since both reach exactly that far.
        let solid_left = (device.min_x() + size + 1.0).ceil() as i32;
        let solid_right = (device.max_x() - size - 1.0).floor() as i32;
        let solid_top = device.min_y() + size + 1.0;
        let solid_bottom = device.max_y() - size - 1.0;

        for y in bounds.top..bounds.bottom {
            let sample_y = y as f32 + 0.5;
            let row = (y as usize) * self.width as usize;
            // One colour for the whole row, which is what makes a vertical
            // gradient cost no more than a flat fill.
            let color = paint.at(sample_y);

            let interior = (sample_y >= solid_top && sample_y <= solid_bottom)
                .then(|| (solid_left.max(bounds.left), solid_right.min(bounds.right)))
                .filter(|(start, end)| end > start);

            let (left_end, right_start) = match interior {
                Some((start, end)) => (start, end),
                None => (bounds.right, bounds.right),
            };

            self.blend_span(row, bounds.left, left_end, sample_y, &field, color);
            if let Some((start, end)) = interior {
                if color.a == 255 {
                    let word = color.to_argb() | 0xff00_0000;
                    self.pixels[row + start as usize..row + end as usize].fill(word);
                } else {
                    for x in start..end {
                        let index = row + x as usize;
                        self.pixels[index] = blend_over(self.pixels[index], color, 255);
                    }
                }
            }
            self.blend_span(row, right_start, bounds.right, sample_y, &field, color);
        }
    }

    /// Draws the halo outside a shape's edge.
    ///
    /// Banded exactly as [`Canvas::stroke_shape`] is, and for the same reason:
    /// between the corners the only marked pixels are the two vertical strips
    /// beside the shape, and everything between them is inside it and untouched.
    fn glow_shape(&mut self, device: Rect, corner: Corner, blur: f32, color: Color) {
        let bounds = self.clip.intersect(PixelBounds {
            left: (device.min_x() - blur).floor() as i32,
            top: (device.min_y() - blur).floor() as i32,
            right: (device.max_x() + blur).ceil() as i32,
            bottom: (device.max_y() + blur).ceil() as i32,
        });
        if bounds.is_empty() {
            return;
        }

        let field = DistanceField::new(device, corner);
        let straight_top = device.min_y() + corner.size();
        let straight_bottom = device.max_y() - corner.size();
        let left_edge = device.min_x().floor() as i32;
        let right_edge = device.max_x().ceil() as i32;

        for y in bounds.top..bounds.bottom {
            let sample_y = y as f32 + 0.5;
            let row = (y as usize) * self.width as usize;

            if sample_y > straight_top && sample_y < straight_bottom && right_edge > left_edge {
                self.glow_span(
                    row,
                    bounds.left,
                    left_edge.min(bounds.right),
                    sample_y,
                    &field,
                    blur,
                    color,
                );
                self.glow_span(
                    row,
                    right_edge.max(bounds.left),
                    bounds.right,
                    sample_y,
                    &field,
                    blur,
                    color,
                );
            } else {
                self.glow_span(
                    row,
                    bounds.left,
                    bounds.right,
                    sample_y,
                    &field,
                    blur,
                    color,
                );
            }
        }
    }

    /// Strokes a rounded rectangle's outline, centred on its edge.
    ///
    /// Only the band the line actually occupies is evaluated. Scanning the whole
    /// rectangle to draw a one-pixel border around it costs the area of the
    /// panel to mark its perimeter, which for the outlines here was most of the
    /// work in a frame.
    fn stroke_shape(&mut self, device: Rect, corner: Corner, thickness: f32, color: Color) {
        // Half a line width to each side of the edge, plus a pixel of
        // antialiasing, is everything the stroke can touch.
        let reach = thickness / 2.0 + 1.0;
        let bounds = self.clip.intersect(PixelBounds {
            left: (device.min_x() - reach).floor() as i32,
            top: (device.min_y() - reach).floor() as i32,
            right: (device.max_x() + reach).ceil() as i32,
            bottom: (device.max_y() + reach).ceil() as i32,
        });
        if bounds.is_empty() {
            return;
        }

        let field = DistanceField::new(device, corner);
        // Between the corners, the only marked pixels are the two vertical
        // edges; everything between them is inside the shape and untouched.
        let straight_top = device.min_y() + corner.size() + reach;
        let straight_bottom = device.max_y() - corner.size() - reach;
        let left_edge = (device.min_x() + reach).ceil() as i32;
        let right_edge = (device.max_x() - reach).floor() as i32;

        for y in bounds.top..bounds.bottom {
            let sample_y = y as f32 + 0.5;
            let row = (y as usize) * self.width as usize;

            if sample_y > straight_top && sample_y < straight_bottom && right_edge > left_edge {
                self.stroke_span(
                    row,
                    bounds.left,
                    left_edge.min(bounds.right),
                    sample_y,
                    &field,
                    thickness,
                    color,
                );
                self.stroke_span(
                    row,
                    right_edge.max(bounds.left),
                    bounds.right,
                    sample_y,
                    &field,
                    thickness,
                    color,
                );
            } else {
                self.stroke_span(
                    row,
                    bounds.left,
                    bounds.right,
                    sample_y,
                    &field,
                    thickness,
                    color,
                );
            }
        }
    }

    /// Blends one run of pixels according to how much of the shape covers them.
    fn blend_span(
        &mut self,
        row: usize,
        from: i32,
        to: i32,
        sample_y: f32,
        field: &DistanceField,
        color: Color,
    ) {
        for x in from..to {
            let distance = field.at(x as f32 + 0.5, sample_y);
            let coverage = 0.5 - distance;
            if coverage <= 0.0 {
                continue;
            }
            let index = row + x as usize;
            self.pixels[index] =
                blend_over(self.pixels[index], color, (coverage.min(1.0) * 255.0) as u8);
        }
    }

    /// The same, for a halo: coverage falls away with distance outside the edge.
    ///
    /// The falloff is quadratic rather than linear because a linear ramp has a
    /// visible hard end where it reaches zero, and a glow with an outline around
    /// it is not a glow.
    #[allow(clippy::too_many_arguments)]
    fn glow_span(
        &mut self,
        row: usize,
        from: i32,
        to: i32,
        sample_y: f32,
        field: &DistanceField,
        blur: f32,
        color: Color,
    ) {
        for x in from..to {
            let distance = field.at(x as f32 + 0.5, sample_y);
            // Inside the shape is left alone: whatever casts the glow covers it.
            if distance <= 0.0 || distance >= blur {
                continue;
            }
            let remaining = 1.0 - distance / blur;
            let index = row + x as usize;
            self.pixels[index] = blend_over(
                self.pixels[index],
                color,
                (remaining * remaining * 255.0) as u8,
            );
        }
    }

    /// The same, for an outline: the distance is folded about the shape's edge.
    #[allow(clippy::too_many_arguments)]
    fn stroke_span(
        &mut self,
        row: usize,
        from: i32,
        to: i32,
        sample_y: f32,
        field: &DistanceField,
        thickness: f32,
        color: Color,
    ) {
        let half = thickness / 2.0;
        for x in from..to {
            let distance = field.at(x as f32 + 0.5, sample_y).abs() - half;
            let coverage = 0.5 - distance;
            if coverage <= 0.0 {
                continue;
            }
            let index = row + x as usize;
            self.pixels[index] =
                blend_over(self.pixels[index], color, (coverage.min(1.0) * 255.0) as u8);
        }
    }
}

/// What colour a fill is at a given device row.
///
/// A shape is filled row by row, and both kinds of fill this canvas offers —
/// flat, and shading top to bottom — answer one colour for a whole row. Keeping
/// that as the interface is what lets one scan serve both, and is why an angled
/// gradient is not offered: it would make the colour vary along the row and cost
/// a mix per pixel.
#[derive(Debug, Clone, Copy)]
struct Paint {
    top: Color,
    bottom: Color,
    /// Device y of the top edge.
    y: f32,
    /// One over the height, or zero when the paint is flat.
    ///
    /// Held as a reciprocal so the per-row lookup is a multiply, and zero is
    /// what marks a flat paint — which then needs no branch of its own.
    inverse_height: f32,
}

impl Paint {
    /// One colour everywhere.
    fn solid(color: Color) -> Self {
        Self {
            top: color,
            bottom: color,
            y: 0.0,
            inverse_height: 0.0,
        }
    }

    /// Shading from `top` at the rectangle's top edge to `bottom` at its bottom.
    fn vertical(device: Rect, top: Color, bottom: Color) -> Self {
        Self {
            top,
            bottom,
            y: device.y,
            inverse_height: if device.h > 0.0 { 1.0 / device.h } else { 0.0 },
        }
    }

    /// The colour at a device row.
    fn at(&self, y: f32) -> Color {
        if self.inverse_height == 0.0 {
            return self.top;
        }
        self.top
            .mix(self.bottom, (y - self.y) * self.inverse_height)
    }
}

/// The signed distance from a point to a shaped rectangle's edge.
///
/// Negative inside, positive outside, and zero exactly on the edge. Precomputed
/// from the shape so that a scan does not recompute its centre and half-extents
/// for every pixel.
///
/// # How one field serves both corner shapes
///
/// A rounded rectangle is the set of points within `radius` of an inner
/// rectangle shrunk by that much — the classic form, and the reason the square
/// root is only ever taken in the corners.
///
/// A cut rectangle is the *intersection* of the full rectangle with the diagonal
/// half-planes that slice its corners off, and the distance to an intersection
/// of convex regions is the greater of the distances to each. The diagonal's
/// distance is `(|dx| + |dy| - k) / √2` — a plane whose normal is the diagonal —
/// so it costs an add and a multiply and never a square root at all.
///
/// Which arm runs is a branch on a `bool` that is constant for the whole scan,
/// so the predictor gets it right every time after the first pixel.
struct DistanceField {
    center_x: f32,
    center_y: f32,
    /// Half-width less the corner size: the straight section's half-extent.
    straight_x: f32,
    /// Half-height less the corner size.
    straight_y: f32,
    /// How far the corner treatment reaches along each edge.
    size: f32,
    /// Half-width and half-height, for the cut form's own rectangle test.
    half_x: f32,
    half_y: f32,
    /// Where the diagonal that cuts the corner sits: `|dx| + |dy| = corner_line`.
    corner_line: f32,
    /// Whether the corners are cut rather than rounded.
    cut: bool,
}

/// One over the square root of two: what turns a diagonal's `|dx| + |dy|`
/// difference into a true perpendicular distance.
const DIAGONAL_SCALE: f32 = std::f32::consts::FRAC_1_SQRT_2;

impl DistanceField {
    fn new(device: Rect, corner: Corner) -> Self {
        let size = corner.size();
        let half_x = device.w / 2.0;
        let half_y = device.h / 2.0;
        Self {
            center_x: device.x + half_x,
            center_y: device.y + half_y,
            straight_x: half_x - size,
            straight_y: half_y - size,
            size,
            half_x,
            half_y,
            corner_line: half_x + half_y - size,
            cut: matches!(corner, Corner::Cut(_)),
        }
    }

    /// The distance from `(x, y)` to the edge.
    ///
    /// For a rounded shape the square root is taken only in the corners.
    /// Everywhere else the nearest edge is a straight one and the distance is a
    /// subtraction — which matters, because the corners are a few hundred pixels
    /// of a panel and the straight edges are the rest of it.
    fn at(&self, x: f32, y: f32) -> f32 {
        let from_x = (x - self.center_x).abs();
        let from_y = (y - self.center_y).abs();

        if self.cut {
            let offset_x = from_x - self.half_x;
            let offset_y = from_y - self.half_y;
            let to_rectangle = if offset_x > 0.0 && offset_y > 0.0 {
                (offset_x * offset_x + offset_y * offset_y).sqrt()
            } else {
                offset_x.max(offset_y)
            };
            let to_diagonal = (from_x + from_y - self.corner_line) * DIAGONAL_SCALE;
            return to_rectangle.max(to_diagonal);
        }

        let offset_x = from_x - self.straight_x;
        let offset_y = from_y - self.straight_y;
        if offset_x > 0.0 && offset_y > 0.0 {
            (offset_x * offset_x + offset_y * offset_y).sqrt() - self.size
        } else {
            offset_x.max(offset_y) - self.size
        }
    }
}

/// Whether a device-space rectangle lands exactly on the pixel grid.
fn is_pixel_aligned(rect: Rect) -> bool {
    let whole = |value: f32| (value - value.round()).abs() < 1.0 / 512.0;
    whole(rect.x) && whole(rect.y) && whole(rect.w) && whole(rect.h)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The colour of one device pixel, for assertions.
    fn pixel_at(canvas: &Canvas, x: u32, y: u32) -> Color {
        Color::from_argb(canvas.pixels()[(y * canvas.width() + x) as usize])
    }

    fn blank(width: u32, height: u32, scale: f32) -> Canvas {
        let mut canvas = Canvas::new(width, height, scale);
        canvas.clear(Color::BLACK);
        canvas
    }

    #[test]
    fn a_new_canvas_is_opaque_everywhere() {
        let canvas = Canvas::new(4, 4, 1.0);
        assert!(canvas.pixels().iter().all(|word| word >> 24 == 0xff));
    }

    #[test]
    fn an_absurd_scale_is_clamped_rather_than_collapsing_the_layout() {
        assert_eq!(Canvas::new(4, 4, 0.0).scale(), 0.25);
        assert_eq!(Canvas::new(4, 4, f32::NAN).scale(), 1.0);
    }

    #[test]
    fn logical_bounds_account_for_the_scale() {
        let canvas = Canvas::new(200, 100, 2.0);
        assert_eq!(canvas.bounds(), Rect::new(0.0, 0.0, 100.0, 50.0));
    }

    #[test]
    fn a_filled_rectangle_covers_exactly_its_pixels() {
        let mut canvas = blank(10, 10, 1.0);
        canvas.fill_rect(Rect::new(2.0, 2.0, 3.0, 3.0), Color::WHITE);

        assert_eq!(pixel_at(&canvas, 2, 2), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 4, 4), Color::WHITE);
        assert_eq!(
            pixel_at(&canvas, 5, 5),
            Color::BLACK,
            "right/bottom edge is exclusive"
        );
        assert_eq!(pixel_at(&canvas, 1, 1), Color::BLACK);
    }

    #[test]
    fn a_cut_corner_removes_the_corner_and_keeps_the_edges() {
        // What separates a cut rectangle from a rounded one and from a plain
        // one: the corner pixel is gone, the pixel just inside the diagonal is
        // painted, and the middle of every edge is untouched by the cut.
        let mut canvas = blank(40, 40, 1.0);
        canvas.fill(
            Rect::new(0.0, 0.0, 40.0, 40.0),
            Corner::Cut(10.0),
            Color::WHITE,
        );

        assert_eq!(
            pixel_at(&canvas, 0, 0),
            Color::BLACK,
            "the corner should be cut away"
        );
        assert_eq!(
            pixel_at(&canvas, 2, 2),
            Color::BLACK,
            "still outside the diagonal"
        );
        assert_eq!(pixel_at(&canvas, 8, 8), Color::WHITE, "inside the diagonal");
        assert_eq!(
            pixel_at(&canvas, 20, 0),
            Color::WHITE,
            "the top edge is not cut"
        );
        assert_eq!(
            pixel_at(&canvas, 0, 20),
            Color::WHITE,
            "the left edge is not cut"
        );
        assert_eq!(
            pixel_at(&canvas, 39, 39),
            Color::BLACK,
            "every corner is cut, not just one"
        );
    }

    #[test]
    fn a_cut_corner_is_a_straight_diagonal_rather_than_an_arc() {
        // The whole difference between the two shapes. A rounded corner bulges
        // outward from the chord between its ends; a cut one *is* that chord, so
        // every pixel along it sits at the same coverage.
        let mut canvas = blank(60, 60, 1.0);
        canvas.fill(
            Rect::new(0.0, 0.0, 60.0, 60.0),
            Corner::Cut(20.0),
            Color::WHITE,
        );

        // Points on the diagonal x + y = 20: all just inside, none filled solid.
        for (x, y) in [(4u32, 15u32), (10, 9), (15, 4)] {
            let on_edge = pixel_at(&canvas, x, y);
            assert!(
                on_edge.r > 40 && on_edge.r < 215,
                "({x}, {y}) should be a part-covered edge pixel, got {on_edge:?}"
            );
        }
    }

    #[test]
    fn a_cut_and_a_rounded_corner_of_the_same_size_differ_where_it_matters() {
        // A regression guard for the corner shape being silently ignored: an arc
        // covers more of its corner than the chord across it does, so the two
        // must disagree between the diagonal and the corner itself.
        let sample = |corner: Corner| {
            let mut canvas = blank(40, 40, 1.0);
            canvas.fill(Rect::new(0.0, 0.0, 40.0, 40.0), corner, Color::WHITE);
            pixel_at(&canvas, 4, 4).r
        };
        assert!(
            sample(Corner::Round(14.0)) > sample(Corner::Cut(14.0)),
            "an arc should cover more of its corner than a straight cut does"
        );
    }

    #[test]
    fn a_corner_larger_than_the_shape_is_clamped_rather_than_inverted() {
        // Asking for more than the shape can hold yields a diamond, not a
        // rectangle turned inside out by a negative straight section.
        let mut canvas = blank(20, 20, 1.0);
        canvas.fill(
            Rect::new(0.0, 0.0, 20.0, 20.0),
            Corner::Cut(999.0),
            Color::WHITE,
        );

        assert_eq!(
            pixel_at(&canvas, 10, 10),
            Color::WHITE,
            "the centre is still filled"
        );
        assert_eq!(
            pixel_at(&canvas, 1, 1),
            Color::BLACK,
            "the corners are gone entirely"
        );
        // The diamond's points sit exactly at the middle of each edge, so the
        // pixel astride one is half covered and the pixel inside it is solid.
        assert_eq!(
            pixel_at(&canvas, 10, 2),
            Color::WHITE,
            "the points of the diamond survive"
        );
    }

    #[test]
    fn growing_a_corner_keeps_the_shape_it_already_was() {
        // What a glow and a focus ring depend on: a cut panel must be haloed by
        // a cut shape, or the halo shows through its own corners.
        assert_eq!(Corner::Cut(4.0).grown(2.0), Corner::Cut(6.0));
        assert_eq!(Corner::Round(4.0).grown(2.0), Corner::Round(6.0));
        // A square corner has no size to grow: growing it would round a shape
        // that was deliberately not rounded.
        assert_eq!(Corner::Square.grown(2.0), Corner::Square);
        assert_eq!(Corner::Square.size(), 0.0);
    }

    #[test]
    fn logical_coordinates_are_multiplied_by_the_scale() {
        let mut canvas = blank(20, 20, 2.0);
        canvas.fill_rect(Rect::new(1.0, 1.0, 2.0, 2.0), Color::WHITE);

        // Logical (1,1)-(3,3) is device (2,2)-(6,6).
        assert_eq!(pixel_at(&canvas, 2, 2), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 5, 5), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 6, 6), Color::BLACK);
    }

    #[test]
    fn a_fractional_edge_is_antialiased_rather_than_snapped() {
        let mut canvas = blank(10, 10, 1.0);
        canvas.fill_rect(Rect::new(2.5, 0.0, 5.0, 10.0), Color::WHITE);

        let edge = pixel_at(&canvas, 2, 5);
        assert!(
            edge.r > 0 && edge.r < 255,
            "expected a partly covered pixel, got {edge:?}"
        );
    }

    #[test]
    fn drawing_outside_the_surface_is_clipped_not_wrapped() {
        let mut canvas = blank(8, 8, 1.0);
        canvas.fill_rect(Rect::new(-100.0, -100.0, 1000.0, 1000.0), Color::WHITE);
        assert!(canvas
            .pixels()
            .iter()
            .all(|&word| Color::from_argb(word) == Color::WHITE));
    }

    #[test]
    fn a_clip_confines_drawing() {
        let mut canvas = blank(10, 10, 1.0);
        let previous = canvas.push_clip(Rect::new(0.0, 0.0, 5.0, 10.0));
        canvas.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE);
        canvas.pop_clip(previous);

        assert_eq!(pixel_at(&canvas, 4, 5), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 5, 5), Color::BLACK);
    }

    #[test]
    fn a_nested_clip_cannot_widen_its_parent() {
        let mut canvas = blank(10, 10, 1.0);
        let outer = canvas.push_clip(Rect::new(0.0, 0.0, 4.0, 10.0));
        let inner = canvas.push_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        canvas.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE);
        canvas.pop_clip(inner);
        canvas.pop_clip(outer);

        assert_eq!(pixel_at(&canvas, 3, 5), Color::WHITE);
        assert_eq!(
            pixel_at(&canvas, 5, 5),
            Color::BLACK,
            "the inner clip widened the outer one"
        );
    }

    #[test]
    fn popping_a_clip_restores_the_earlier_one() {
        let mut canvas = blank(10, 10, 1.0);
        let previous = canvas.push_clip(Rect::new(0.0, 0.0, 2.0, 2.0));
        canvas.pop_clip(previous);
        canvas.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 9, 9), Color::WHITE);
    }

    #[test]
    fn visibility_follows_the_clip() {
        let mut canvas = blank(10, 10, 1.0);
        let previous = canvas.push_clip(Rect::new(0.0, 0.0, 5.0, 5.0));
        assert!(canvas.is_visible(Rect::new(4.0, 4.0, 2.0, 2.0)));
        assert!(!canvas.is_visible(Rect::new(6.0, 6.0, 2.0, 2.0)));
        canvas.pop_clip(previous);
    }

    #[test]
    fn a_rounded_corner_is_lighter_than_the_middle() {
        let mut canvas = blank(20, 20, 1.0);
        canvas.fill(
            Rect::new(0.0, 0.0, 20.0, 20.0),
            Corner::Round(8.0),
            Color::WHITE,
        );

        assert_eq!(pixel_at(&canvas, 10, 10), Color::WHITE);
        assert_eq!(
            pixel_at(&canvas, 0, 0),
            Color::BLACK,
            "the corner should be cut away"
        );
    }

    #[test]
    fn a_large_rounded_rectangle_has_no_seam_where_the_fast_path_begins() {
        // The interior is written in bulk and the margins are evaluated pixel by
        // pixel. If those two disagree by even one step, a panel gets a visible
        // vertical line down each side.
        let mut canvas = blank(200, 120, 1.0);
        canvas.fill(
            Rect::new(10.0, 10.0, 180.0, 100.0),
            Corner::Round(12.0),
            Color::WHITE,
        );

        for x in 24..176 {
            assert_eq!(
                pixel_at(&canvas, x, 60),
                Color::WHITE,
                "a seam appeared at x = {x} on the middle row"
            );
        }
        for y in 24..96 {
            assert_eq!(
                pixel_at(&canvas, 100, y),
                Color::WHITE,
                "a seam appeared at y = {y}"
            );
        }
    }

    #[test]
    fn a_translucent_fill_covers_its_interior_evenly() {
        // The bulk path and the per-pixel path must blend identically, or the
        // middle of a translucent panel comes out a different shade from its
        // edges.
        let mut canvas = blank(120, 80, 1.0);
        let half = Color::rgba(255, 255, 255, 128);
        canvas.fill(Rect::new(5.0, 5.0, 110.0, 70.0), Corner::Round(10.0), half);

        let middle = pixel_at(&canvas, 60, 40);
        let near_edge = pixel_at(&canvas, 20, 40);
        assert_eq!(middle, near_edge, "the interior is not one even shade");
        assert!(
            middle.r > 100 && middle.r < 160,
            "expected about half, got {middle:?}"
        );
    }

    #[test]
    fn a_stroke_on_a_tall_shape_marks_both_edges_and_nothing_between() {
        // Tall enough that the straight section is scanned by the banded path
        // rather than the whole-row one.
        let mut canvas = blank(60, 200, 1.0);
        canvas.stroke(
            Rect::new(10.0, 10.0, 40.0, 180.0),
            Corner::Round(8.0),
            1.0,
            Color::WHITE,
        );

        assert!(
            pixel_at(&canvas, 10, 100).r > 100,
            "the left edge should be drawn"
        );
        assert!(
            pixel_at(&canvas, 49, 100).r > 100,
            "the right edge should be drawn"
        );
        for x in 13..47 {
            assert_eq!(
                pixel_at(&canvas, x, 100),
                Color::BLACK,
                "x = {x} should be untouched"
            );
        }
        assert!(
            pixel_at(&canvas, 30, 10).r > 100,
            "the top edge should be drawn"
        );
        assert!(
            pixel_at(&canvas, 30, 189).r > 100,
            "the bottom edge should be drawn"
        );
    }

    #[test]
    fn a_stroked_corner_joins_the_edges_that_meet_there() {
        let mut canvas = blank(80, 80, 1.0);
        canvas.stroke(
            Rect::new(10.0, 10.0, 60.0, 60.0),
            Corner::Round(20.0),
            2.0,
            Color::WHITE,
        );

        // On the arc of the top-left corner, at forty-five degrees.
        let offset = 20.0 - (20.0f32 / std::f32::consts::SQRT_2);
        let x = (10.0 + offset).round() as u32;
        let y = (10.0 + offset).round() as u32;
        assert!(
            pixel_at(&canvas, x, y).r > 80,
            "the corner arc should be drawn"
        );
        assert_eq!(
            pixel_at(&canvas, 40, 40),
            Color::BLACK,
            "the middle stays empty"
        );
    }

    #[test]
    fn an_oversized_radius_becomes_a_capsule_not_an_inverted_corner() {
        let mut canvas = blank(40, 20, 1.0);
        canvas.fill(
            Rect::new(0.0, 0.0, 40.0, 20.0),
            Corner::Round(999.0),
            Color::WHITE,
        );

        assert_eq!(pixel_at(&canvas, 20, 10), Color::WHITE);
        assert_eq!(
            pixel_at(&canvas, 20, 0),
            Color::WHITE,
            "the flat top should survive"
        );
        assert_eq!(pixel_at(&canvas, 0, 0), Color::BLACK);
    }

    #[test]
    fn a_stroke_marks_the_edge_and_leaves_the_middle_alone() {
        let mut canvas = blank(20, 20, 1.0);
        canvas.stroke(
            Rect::new(2.0, 2.0, 16.0, 16.0),
            Corner::Square,
            1.0,
            Color::WHITE,
        );

        assert!(
            pixel_at(&canvas, 10, 2).r > 100,
            "the top edge should be drawn"
        );
        assert_eq!(
            pixel_at(&canvas, 10, 10),
            Color::BLACK,
            "the middle should be untouched"
        );
    }

    #[test]
    fn a_vertical_gradient_runs_from_its_top_colour_to_its_bottom_one() {
        let mut canvas = blank(20, 100, 1.0);
        canvas.fill_vertical(
            Rect::new(0.0, 0.0, 20.0, 100.0),
            Corner::Square,
            Color::BLACK,
            Color::WHITE,
        );

        let top = pixel_at(&canvas, 10, 0);
        let middle = pixel_at(&canvas, 10, 50);
        let bottom = pixel_at(&canvas, 10, 99);
        assert!(
            top.r < 8,
            "the top should be near the top colour, got {top:?}"
        );
        assert!(
            (120..=136).contains(&middle.r),
            "the middle should be halfway, got {middle:?}"
        );
        assert!(
            bottom.r > 247,
            "the bottom should be near the bottom colour, got {bottom:?}"
        );
    }

    #[test]
    fn a_gradient_row_is_one_even_colour_across_its_width() {
        // The interior is written in bulk and the margins pixel by pixel. If the
        // gradient were evaluated differently by the two, every panel would show
        // a vertical seam a few pixels in from each side.
        let mut canvas = blank(120, 60, 1.0);
        canvas.fill_vertical(
            Rect::new(0.0, 0.0, 120.0, 60.0),
            Corner::Square,
            Color::BLACK,
            Color::WHITE,
        );

        let expected = pixel_at(&canvas, 60, 30);
        for x in 0..120 {
            assert_eq!(
                pixel_at(&canvas, x, 30),
                expected,
                "the row is not even at x = {x}"
            );
        }
    }

    #[test]
    fn a_gradient_between_two_equal_colours_is_a_flat_fill() {
        let mut canvas = blank(20, 40, 1.0);
        canvas.fill_vertical(
            Rect::new(0.0, 0.0, 20.0, 40.0),
            Corner::Square,
            Color::WHITE,
            Color::WHITE,
        );
        assert!(canvas
            .pixels()
            .iter()
            .all(|&word| Color::from_argb(word) == Color::WHITE));
    }

    #[test]
    fn a_glow_surrounds_a_shape_without_touching_its_inside() {
        let mut canvas = blank(80, 80, 1.0);
        canvas.shadow(
            Rect::new(30.0, 30.0, 20.0, 20.0),
            Corner::Round(4.0),
            10.0,
            0.0,
            Color::WHITE,
        );

        assert_eq!(
            pixel_at(&canvas, 40, 40),
            Color::BLACK,
            "the inside must be left for the caster"
        );
        assert!(
            pixel_at(&canvas, 40, 28).r > 0,
            "the halo should reach just outside the edge"
        );
        assert_eq!(
            pixel_at(&canvas, 40, 5),
            Color::BLACK,
            "the halo should not reach past its blur"
        );
    }

    #[test]
    fn a_glow_fades_with_distance_from_the_edge() {
        let mut canvas = blank(80, 80, 1.0);
        canvas.shadow(
            Rect::new(30.0, 30.0, 20.0, 20.0),
            Corner::Round(0.0),
            12.0,
            0.0,
            Color::WHITE,
        );

        let near = pixel_at(&canvas, 40, 28).r;
        let far = pixel_at(&canvas, 40, 22).r;
        assert!(
            near > far,
            "expected the halo to fade outward, got {near} then {far}"
        );
        assert!(far > 0, "the halo should still be present further out");
    }

    #[test]
    fn a_glow_with_spread_starts_further_out_than_one_without() {
        // Spread moves the band the halo occupies outward rather than widening
        // it, so what changes is where the halo *starts*, not how much of it
        // there is.
        let cast = Rect::new(30.0, 30.0, 20.0, 20.0);
        let topmost = |spread: f32| {
            let mut canvas = blank(80, 80, 1.0);
            canvas.shadow(cast, Corner::Round(0.0), 8.0, spread, Color::WHITE);
            (0..30)
                .find(|&y| pixel_at(&canvas, 40, y).r > 0)
                .expect("the halo should be drawn")
        };
        assert!(
            topmost(6.0) < topmost(0.0),
            "spread should push the halo further from the shape"
        );
    }

    #[test]
    fn an_invisible_colour_draws_nothing() {
        let mut canvas = blank(8, 8, 1.0);
        canvas.fill_rect(Rect::new(0.0, 0.0, 8.0, 8.0), Color::TRANSPARENT);
        assert_eq!(pixel_at(&canvas, 4, 4), Color::BLACK);
    }

    #[test]
    fn an_empty_rectangle_draws_nothing() {
        let mut canvas = blank(8, 8, 1.0);
        canvas.fill_rect(Rect::new(2.0, 2.0, 0.0, 5.0), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 2, 4), Color::BLACK);
    }

    #[test]
    fn a_mask_paints_its_coverage() {
        let mut canvas = blank(8, 8, 1.0);
        let mask = Mask {
            width: 2,
            height: 1,
            coverage: vec![255, 0],
        };
        canvas.fill_mask(3, 3, &mask, Color::WHITE);

        assert_eq!(pixel_at(&canvas, 3, 3), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 4, 3), Color::BLACK);
    }

    #[test]
    fn a_mask_hanging_off_the_edge_is_clipped_not_wrapped() {
        let mut canvas = blank(4, 4, 1.0);
        let mask = Mask {
            width: 4,
            height: 4,
            coverage: vec![255; 16],
        };
        canvas.fill_mask(-2, -2, &mask, Color::WHITE);

        assert_eq!(pixel_at(&canvas, 0, 0), Color::WHITE);
        assert_eq!(pixel_at(&canvas, 2, 2), Color::BLACK);
        assert_eq!(
            pixel_at(&canvas, 3, 0),
            Color::BLACK,
            "the mask wrapped onto the next row"
        );
    }

    #[test]
    fn resizing_keeps_the_canvas_consistent() {
        let mut canvas = blank(4, 4, 1.0);
        canvas.resize(10, 6, 2.0);
        assert_eq!(canvas.width(), 10);
        assert_eq!(canvas.pixels().len(), 60);
        assert_eq!(canvas.scale(), 2.0);
        assert_eq!(canvas.clip(), Rect::new(0.0, 0.0, 5.0, 3.0));
    }
}
