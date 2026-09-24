// rui's recorded frame, drawn as instanced quads. Each instance is one shape
// from src/gpu.rs, evaluated with the same signed distance fields and the same
// `clamp(0.5 - d)` coverage rule as the software canvas (src/canvas.rs), in the
// same sRGB byte space, so the two draw the same picture.
//
// Output is premultiplied, for a canvas configured with alphaMode
// "premultiplied": what the interface leaves transparent shows the page under
// it.

struct Uniforms { size: vec2f, pad: vec2f }

struct Instance {
  quad: vec4f,  // left, top, right, bottom — device pixels, whole, clipped
  c0: vec4f,    // straight-alpha sRGB 0..1
  c1: vec4f,
  a: vec4f,     // the field; per kind, below
  b: vec4f,
  g: vec4f,     // the paint; per paint mode
  info: vec4f,  // kind, style, paint, extra (stroke half-width or glow reach)
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read> shapes: array<Instance>;
@group(0) @binding(2) var atlas: texture_2d<f32>;
@group(0) @binding(3) var picture: texture_2d<f32>;

struct Out {
  @builtin(position) pos: vec4f,
  @location(0) @interpolate(flat) index: u32,
}

@vertex
fn vs(@builtin(vertex_index) v: u32, @builtin(instance_index) i: u32) -> Out {
  let q = shapes[i].quad;
  let corner = vec2f(f32(v & 1u), f32((v >> 1u) & 1u));
  let p = mix(q.xy, q.zw, corner);
  var o: Out;
  o.pos = vec4f(p.x / u.size.x * 2.0 - 1.0, 1.0 - p.y / u.size.y * 2.0, 0.0, 1.0);
  o.index = i;
  return o;
}

const TAU: f32 = 6.28318530718;

// src/canvas.rs DistanceField::at. a = centre x, centre y, half w, half h;
// b = corner size, cut (0/1).
fn rect_sd(p: vec2f, a: vec4f, b: vec4f) -> f32 {
  let rel = abs(p - a.xy);
  let size = b.x;
  if (b.y > 0.5) {
    let off = rel - a.zw;
    var to_rect: f32;
    if (off.x > 0.0 && off.y > 0.0) { to_rect = length(off); } else { to_rect = max(off.x, off.y); }
    let to_diag = (rel.x + rel.y - (a.z + a.w - size)) * 0.70710678;
    return max(to_rect, to_diag);
  }
  let off = rel - (a.zw - vec2f(size));
  if (off.x > 0.0 && off.y > 0.0) { return length(off) - size; }
  return max(off.x, off.y) - size;
}

// Band::at. a = centre x, centre y, radius, half; b = start, sweep, has ends.
fn band_sd(p: vec2f, a: vec4f, b: vec4f) -> f32 {
  let rel = p - a.xy;
  let radial = abs(length(rel) - a.z);
  if (b.z < 0.5) { return radial - a.w; }
  let ang = atan2(rel.y, rel.x) - b.x;
  if (ang - TAU * floor(ang / TAU) <= b.y) { return radial - a.w; }
  let e0 = a.xy + a.z * vec2f(cos(b.x), sin(b.x));
  let e1 = a.xy + a.z * vec2f(cos(b.x + b.y), sin(b.x + b.y));
  return min(length(p - e0), length(p - e1)) - a.w;
}

// Segment::at. a = from, to; b.x = half width.
fn segment_sd(p: vec2f, a: vec4f, b: vec4f) -> f32 {
  let pa = p - a.xy;
  let ba = a.zw - a.xy;
  let along = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-12), 0.0, 1.0);
  return length(pa - ba * along) - b.x;
}

@fragment
fn fs(in: Out) -> @location(0) vec4f {
  let s = shapes[in.index];
  let p = in.pos.xy; // pixel centre, as the software scan samples
  let kind = u32(s.info.x + 0.5);
  let style = u32(s.info.y + 0.5);
  let mode = u32(s.info.z + 0.5);
  let extra = s.info.w;

  if (kind == 4u) {
    // A picture: premultiplied texels, one per device pixel, anchored at a.xy.
    let texel = vec2i(floor(p - s.a.xy));
    return textureLoad(picture, texel, 0) * s.c0.a;
  }

  var cov: f32;
  if (kind == 3u) {
    // A glyph: coverage from the atlas; a.xy is its device origin, a.zw its
    // atlas origin.
    let texel = vec2i(floor(p - s.a.xy) + s.a.zw);
    cov = textureLoad(atlas, texel, 0).r;
  } else {
    var d: f32;
    if (kind == 0u) { d = rect_sd(p, s.a, s.b); }
    else if (kind == 1u) { d = band_sd(p, s.a, s.b); }
    else { d = segment_sd(p, s.a, s.b); }

    if (style == 0u) {
      cov = clamp(0.5 - d, 0.0, 1.0);
    } else if (style == 1u) {
      cov = clamp(0.5 - (abs(d) - extra), 0.0, 1.0);
    } else if (style == 2u) {
      if (d <= 0.0 || d >= extra) { discard; }
      let r = 1.0 - d / extra;
      cov = r * r;
    } else {
      let core = clamp(0.5 - d, 0.0, 1.0);
      var halo = 0.0;
      if (extra > 0.0 && d > 0.0 && d < extra) { let r = 1.0 - d / extra; halo = r * r; }
      cov = max(core, halo);
    }
  }
  if (cov <= 0.0) { discard; }

  var t = 0.0;
  if (mode == 1u) {
    t = clamp((p.y - s.g.x) * s.g.y, 0.0, 1.0);
  } else if (mode == 2u) {
    t = clamp((p.x - s.g.x) * s.g.z + (p.y - s.g.y) * s.g.w, 0.0, 1.0);
  } else if (mode == 3u) {
    t = clamp(length(p - s.g.xy) * s.g.z, 0.0, 1.0);
  }
  let color = mix(s.c0, s.c1, t);
  let alpha = color.a * cov;
  return vec4f(color.rgb * alpha, alpha);
}
