//! What a frame *is* when a GPU draws it: a list of shapes, not pixels.
//!
//! # Why a recording, and not a second rasteriser
//!
//! Every visible mark the [`Canvas`](crate::Canvas) makes funnels through a
//! dozen device-space helpers — fill a shaped rectangle, stroke it, glow round
//! it, mark a band, a capsule, a coverage mask, a picture. Each of those is
//! already described by a signed distance field and a paint, which is exactly
//! what a fragment shader evaluates. So a recording canvas does not rasterise
//! at all: each helper appends one [`Instance`] — the field's parameters, the
//! paint's, the device rectangle it can touch — and a backend draws the whole
//! list as instanced quads in a handful of draw calls. The shader evaluates the
//! *same* distance functions with the *same* `clamp(0.5 - d)` coverage rule, in
//! the same sRGB byte space, so a recorded frame and a rasterised one are the
//! same picture.
//!
//! A frame comparison is then a comparison of two lists of a few hundred
//! instances instead of eight million bytes, and presenting one uploads a few
//! kilobytes instead of converting and copying a full-window bitmap — the
//! difference between a browser tab that keeps up with a trackpad and one that
//! freezes on the first scroll.
//!
//! This module is platform-agnostic on purpose: recording is plain Rust and is
//! tested natively. Only presenting it (the `webgpu` feature's wasm backend)
//! needs a GPU.
//!
//! # Textures
//!
//! Glyph masks go into one shared coverage atlas and pictures into textures of
//! their own, both keyed by content so a mask or a picture drawn every frame is
//! uploaded once. Both caches are per-thread and outlive any one canvas, since
//! a surface swaps two canvases every frame and both must refer to the same
//! texture ids.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::color::Color;

/// One shape, as the shader reads it: seven `vec4<f32>`s.
///
/// `quad` is the device rectangle (left, top, right, bottom, whole pixels,
/// already clipped) the shape can touch; nothing outside it is shaded, which is
/// how the clip is honoured without a per-pixel test. `a` and `b` are the
/// shape's field, `g` its paint, and `meta` says how to read the rest:
/// `(kind, style, paint, extra) — `info` in the shader`. See `shader.wgsl` for each field's layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Instance {
    /// Left, top, right, bottom: the device pixels shaded.
    pub quad: [f32; 4],
    /// The first colour, straight-alpha sRGB 0..1.
    pub c0: [f32; 4],
    /// The second colour, for a gradient.
    pub c1: [f32; 4],
    /// The field's first parameters.
    pub a: [f32; 4],
    /// The field's second parameters.
    pub b: [f32; 4],
    /// The paint's parameters.
    pub g: [f32; 4],
    /// Kind, style, paint mode, and one extra scalar.
    pub meta: [f32; 4],
}

/// Floats per instance, for a backend handing the list to a GPU buffer.
pub const INSTANCE_FLOATS: usize = 28;

/// Which distance field an instance evaluates.
pub(crate) mod kind {
    pub const RECT: f32 = 0.0;
    pub const BAND: f32 = 1.0;
    pub const SEGMENT: f32 = 2.0;
    pub const GLYPH: f32 = 3.0;
    pub const IMAGE: f32 = 4.0;
}

/// How the distance becomes coverage.
pub(crate) mod style {
    /// `clamp(0.5 - d)`.
    pub const FILL: f32 = 0.0;
    /// The same, with `d` folded about the edge by `extra` (half the thickness).
    pub const STROKE: f32 = 1.0;
    /// A quadratic halo `extra` wide outside the edge, nothing inside.
    pub const GLOW: f32 = 2.0;
    /// A lit core and its halo together: `glow_coverage`.
    pub const BEAM: f32 = 3.0;
}

/// How the colour varies across the shape.
pub(crate) mod paint {
    pub const SOLID: f32 = 0.0;
    /// `g = (top y, 1 / height, _, _)`.
    pub const VERTICAL: f32 = 1.0;
    /// `g = (origin x, origin y, axis x / |axis|², axis y / |axis|²)`.
    pub const LINEAR: f32 = 2.0;
    /// `g = (centre x, centre y, 1 / radius, _)`.
    pub const RADIAL: f32 = 3.0;
}

/// How an instance meets what is already drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    /// Source-over: [`crate::color::blend_over`].
    Over,
    /// Adding light: [`crate::color::blend_add`].
    Add,
}

/// A run of consecutive instances one draw call can cover: same blend, same
/// picture bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Batch {
    /// The first instance in the run.
    pub first: u32,
    /// How many instances it covers.
    pub count: u32,
    /// How they meet what is drawn.
    pub blend: Blend,
    /// The picture this run samples, if any; 0 is "none".
    pub image: u32,
}

/// A recorded frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Recording {
    /// What the frame starts from, straight-alpha sRGB 0..1; `None` is fully
    /// transparent.
    pub clear: Option<[f32; 4]>,
    /// Every shape, in painting order.
    pub instances: Vec<Instance>,
    /// Runs of those one draw call each.
    pub batches: Vec<Batch>,
    /// The atlas generation every glyph in this frame was placed under. A
    /// frame from an older generation points at masks that are gone.
    pub atlas_generation: u32,
}

impl Recording {
    pub(crate) fn reset(&mut self) {
        self.clear = None;
        self.instances.clear();
        self.batches.clear();
        self.atlas_generation = with_textures(|t| t.atlas.generation);
    }

    /// Appends one instance, extending the last batch when it can.
    pub(crate) fn push(&mut self, instance: Instance, blend: Blend, image: u32) {
        let index = self.instances.len() as u32;
        self.instances.push(instance);
        if let Some(last) = self.batches.last_mut() {
            // A run that binds no picture can adopt the next one's, and one
            // that binds a picture carries it for any shape after it.
            let compatible = last.blend == blend
                && (image == 0 || last.image == 0 || last.image == image);
            if compatible && last.first + last.count == index {
                last.count += 1;
                if image != 0 {
                    last.image = image;
                }
                return;
            }
        }
        self.batches.push(Batch {
            first: index,
            count: 1,
            blend,
            image,
        });
    }

    /// The instances as one flat float list, for a GPU buffer, written into
    /// `out` so a backend can keep one allocation across frames.
    pub fn floats_into(&self, out: &mut Vec<f32>) {
        out.clear();
        out.reserve(self.instances.len() * INSTANCE_FLOATS);
        for i in &self.instances {
            for v in [i.quad, i.c0, i.c1, i.a, i.b, i.g, i.meta] {
                out.extend_from_slice(&v);
            }
        }
    }
}

/// A straight-alpha colour as the shader wants it.
pub(crate) fn rgba(color: Color) -> [f32; 4] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    ]
}

/// A 64-bit content hash, fast enough to run over every glyph every frame.
pub(crate) fn hash_bytes(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed ^ 0x9e37_79b9_7f4a_7c15 ^ (bytes.len() as u64).wrapping_mul(0x100_0000_01b3);
    let mut chunks = bytes.chunks_exact(8);
    for chunk in &mut chunks {
        let word = u64::from_le_bytes(chunk.try_into().unwrap_or([0; 8]));
        h = (h ^ word).wrapping_mul(0x0000_0100_0000_01b3).rotate_left(29);
    }
    for &byte in chunks.remainder() {
        h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    h ^ (h >> 31)
}

/// Side of the square glyph atlas, in texels.
pub const ATLAS_SIZE: u32 = 2048;

/// A shelf packer over the coverage atlas.
///
/// Glyphs are packed left to right along shelves as tall as the tallest glyph
/// on them. When the atlas fills, it is emptied and its generation bumped:
/// every glyph is placed again as it is next drawn, and a frame recorded under
/// the old generation is drawn again rather than presented (see
/// [`Recording::atlas_generation`]).
#[derive(Default)]
pub(crate) struct Atlas {
    slots: HashMap<u64, (u32, u32)>,
    shelf_x: u32,
    shelf_y: u32,
    shelf_h: u32,
    pub generation: u32,
    /// Masks placed since a backend last took them: (x, y, w, h, coverage).
    pub pending: Vec<(u32, u32, u32, u32, Vec<u8>)>,
}

impl Atlas {
    /// Where the mask lives in the atlas, placing it if it is new.
    fn place(&mut self, width: u32, height: u32, coverage: &[u8]) -> Option<(u32, u32)> {
        if width > ATLAS_SIZE || height > ATLAS_SIZE {
            return None;
        }
        let key = hash_bytes(((width as u64) << 32) | height as u64, coverage);
        if let Some(&slot) = self.slots.get(&key) {
            return Some(slot);
        }
        // One texel of padding keeps a neighbour out of reach of any filter.
        let (w, h) = (width + 1, height + 1);
        if self.shelf_x + w > ATLAS_SIZE {
            self.shelf_y += self.shelf_h;
            self.shelf_x = 0;
            self.shelf_h = 0;
        }
        if self.shelf_y + h > ATLAS_SIZE {
            self.slots.clear();
            self.pending.clear();
            self.shelf_x = 0;
            self.shelf_y = 0;
            self.shelf_h = 0;
            self.generation = self.generation.wrapping_add(1);
        }
        let slot = (self.shelf_x, self.shelf_y);
        self.shelf_x += w;
        self.shelf_h = self.shelf_h.max(h);
        self.slots.insert(key, slot);
        self.pending
            .push((slot.0, slot.1, width, height, coverage.to_vec()));
        Some(slot)
    }
}

/// Pictures, each in a texture of its own, keyed by content.
#[derive(Default)]
pub(crate) struct Images {
    by_key: HashMap<u64, u32>,
    /// id → (key, frame last drawn).
    live: HashMap<u32, (u64, u64)>,
    next_id: u32,
    pub frame: u64,
    /// Pictures a backend has yet to create: (id, width, height, premultiplied RGBA).
    pub pending: Vec<(u32, u32, u32, Vec<u8>)>,
    /// Pictures a backend should release.
    pub dropped: Vec<u32>,
}

impl Images {
    /// The texture id for a picture, creating it from `make` the first time.
    pub(crate) fn id(&mut self, key: u64, width: u32, height: u32, make: impl FnOnce() -> Vec<u8>) -> u32 {
        if let Some(&id) = self.by_key.get(&key) {
            if let Some(entry) = self.live.get_mut(&id) {
                entry.1 = self.frame;
            }
            return id;
        }
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let id = self.next_id;
        self.by_key.insert(key, id);
        self.live.insert(id, (key, self.frame));
        self.pending.push((id, width, height, make()));
        id
    }

    /// Starts a frame, releasing any picture not drawn for a while.
    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;
        let frame = self.frame;
        let stale: Vec<u32> = self
            .live
            .iter()
            .filter(|(_, &(_, last))| frame.saturating_sub(last) > 120)
            .map(|(&id, _)| id)
            .collect();
        for id in stale {
            if let Some((key, _)) = self.live.remove(&id) {
                self.by_key.remove(&key);
            }
            self.dropped.push(id);
        }
    }
}

/// Every texture a recording can refer to, shared by all canvases on a thread.
#[derive(Default)]
pub struct Textures {
    pub(crate) atlas: Atlas,
    pub(crate) images: Images,
}

impl Textures {
    /// Glyph masks placed since the last call: (x, y, w, h, coverage).
    pub fn take_atlas_uploads(&mut self) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        std::mem::take(&mut self.atlas.pending)
    }

    /// Pictures to create since the last call: (id, w, h, premultiplied RGBA).
    pub fn take_image_uploads(&mut self) -> Vec<(u32, u32, u32, Vec<u8>)> {
        std::mem::take(&mut self.images.pending)
    }

    /// Starts a presented frame: pictures not drawn for a couple of seconds
    /// are released (see [`Self::take_image_drops`]).
    pub fn begin_frame(&mut self) {
        self.images.begin_frame();
    }

    /// Picture ids no longer drawn.
    pub fn take_image_drops(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.images.dropped)
    }

    /// The current atlas generation; see [`Recording::atlas_generation`].
    pub fn atlas_generation(&self) -> u32 {
        self.atlas.generation
    }
}

thread_local! {
    static TEXTURES: RefCell<Textures> = RefCell::new(Textures::default());
}

/// Runs `f` against this thread's textures.
pub fn with_textures<T>(f: impl FnOnce(&mut Textures) -> T) -> T {
    TEXTURES.with(|cell| f(&mut cell.borrow_mut()))
}

/// Places a glyph mask in the atlas; `None` if it can never fit.
pub(crate) fn place_glyph(width: u32, height: u32, coverage: &[u8]) -> Option<(u32, u32)> {
    with_textures(|t| t.atlas.place(width, height, coverage))
}

/// The WGSL every backend draws a [`Recording`] with.
pub const SHADER: &str = include_str!("gpu/shader.wgsl");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Canvas, Corner};
    use crate::geom::{Point, Rect};

    fn recording(w: u32, h: u32) -> Canvas {
        let mut canvas = Canvas::new(w, h, 2.0);
        canvas.set_recording(true);
        canvas
    }

    #[test]
    fn a_recording_canvas_keeps_no_pixels() {
        let canvas = recording(64, 64);
        assert!(canvas.pixels().is_empty());
        assert!(canvas.recording().is_some());
    }

    #[test]
    fn each_primitive_records_one_clipped_instance() {
        let mut canvas = recording(100, 100);
        canvas.fill(Rect::new(10.0, 10.0, 20.0, 20.0), Corner::Round(4.0), Color::rgb(255, 0, 0));
        canvas.stroke(Rect::new(10.0, 10.0, 20.0, 20.0), Corner::Cut(4.0), 1.0, Color::rgb(0, 255, 0));
        canvas.line(Point::new(0.0, 0.0), Point::new(40.0, 40.0), 2.0, Color::rgb(0, 0, 255));
        canvas.ring(Point::new(25.0, 25.0), 10.0, 2.0, Color::rgb(9, 9, 9));
        let rec = canvas.recording().unwrap();
        assert_eq!(rec.instances.len(), 4);
        // Everything is over-blended and binds no picture: one draw call.
        assert_eq!(rec.batches.len(), 1);
        for i in &rec.instances {
            assert!(i.quad[0] >= 0.0 && i.quad[1] >= 0.0 && i.quad[2] <= 100.0 && i.quad[3] <= 100.0);
        }
    }

    #[test]
    fn a_shape_outside_the_clip_records_nothing() {
        let mut canvas = recording(100, 100);
        let previous = canvas.push_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        canvas.fill(Rect::new(30.0, 30.0, 5.0, 5.0), Corner::Square, Color::rgb(1, 2, 3));
        canvas.pop_clip(previous);
        assert!(canvas.recording().unwrap().instances.is_empty());
    }

    #[test]
    fn an_additive_beam_starts_a_batch_of_its_own() {
        let mut canvas = recording(100, 100);
        canvas.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::rgb(1, 2, 3));
        canvas.beam(Point::new(0.0, 0.0), Point::new(30.0, 0.0), 1.0, 4.0, Color::rgb(0, 200, 255));
        canvas.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::rgb(1, 2, 3));
        let batches = &canvas.recording().unwrap().batches;
        assert_eq!(batches.iter().map(|b| b.blend).collect::<Vec<_>>(), [Blend::Over, Blend::Add, Blend::Over]);
    }

    #[test]
    fn the_same_drawing_twice_is_the_same_picture() {
        let draw = |canvas: &mut Canvas| {
            canvas.clear(Color::rgb(10, 10, 10));
            canvas.fill_vertical(Rect::new(1.0, 2.0, 30.0, 20.0), Corner::Round(3.0), Color::rgb(0, 0, 0), Color::rgb(255, 255, 255));
        };
        let (mut a, mut b) = (recording(80, 80), recording(80, 80));
        draw(&mut a);
        draw(&mut b);
        assert!(a.same_picture(&b));
        b.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::rgb(255, 0, 0));
        assert!(!a.same_picture(&b));
    }

    #[test]
    fn a_glyph_mask_is_placed_once() {
        let mask = crate::canvas::Mask { width: 3, height: 2, coverage: vec![0, 128, 255, 255, 128, 0] };
        let mut canvas = recording(50, 50);
        canvas.fill_mask(4, 4, &mask, Color::rgb(255, 255, 255));
        canvas.fill_mask(20, 4, &mask, Color::rgb(255, 255, 255));
        let rec = canvas.recording().unwrap();
        assert_eq!(rec.instances[0].a[2..], rec.instances[1].a[2..]);
    }
}
