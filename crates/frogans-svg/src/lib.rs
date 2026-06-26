//! `frogans-svg` — render an fprt [`Representation`] into a standalone,
//! interactive SVG document.
//!
//! Pure: [`representation_to_svg`] takes `&Representation` and returns a `String`
//! — no engine, no I/O. Feed it either slide of a
//! [`sitehandler::UpdateVisual`](fprt::sitehandler::UpdateVisual) (`lead` or
//! `vignette`); the session that produced the representation is a separate concern.
//!
//! The slide's `image` is already the flattened resting view (pills, labels,
//! arrows, curve — everything), so it becomes the backdrop `<image>` verbatim.
//! Each [`Rollover`] then layers an interactive zone on top, built from its
//! pieces. A piece's true (non-rectangular) shape lives in its `raw` 1-bpp plane,
//! not its `encoded` rectangle — drawing the rectangle is what bled opaque
//! backgrounds over the curve — so every shape comes from greedy-meshing that
//! plane into vector geometry.
//!
//! Piece `kind` (= `0x838 - rawmode`) selects the role:
//! - [`KIND_HIT`] (`2001`, no color) — the clickable silhouette. Meshed into a
//!   transparent `<path>` with `pointer-events="fill"`, so hover/click fire only
//!   on the real shape, not the bounding box.
//! - `2002` — the base background layer (a blank pill, no text), drawn under.
//! - `2003` — the overlay layer (the text / inverted-arrow content), drawn on top.
//!
//! `2002` then `2003` — their order in the piece array is the paint order —
//! together form a zone's rollover appearance: blue menu text, white badge,
//! inverted arrows. They are painted on `:hover` as `<image>`s clipped to the zone
//! silhouette (the `2001` mesh). The resting appearance (darker menu text, gold
//! badge, filled arrows) is the flattened backdrop `image`, not re-drawn here.
//!
//! (Whether `2002`/`2003` are "always-on vs selected-only" or focusable is
//! interaction state the pieces don't encode; this renders them as one overlay.)
//!
//! Assumes the session selected [`ImageFormat::Png`](fprt::ImageFormat::Png).

use core::fmt::Write;

use fprt::PooledImage;
use fprt::visual::{Representation, Rollover};

/// Mask-only piece: the clickable silhouette (`enc=None`, never painted).
const KIND_HIT: i32 = 2001;

/// Render one slide ([`Representation`]) as a complete, self-contained SVG document.
pub fn representation_to_svg(rep: &Representation) -> String {
    // The backdrop's real pixel size; fall back to the geometry size.
    let (width, height) = match &rep.image {
        Some(img) => (img.width() as i32, img.height() as i32),
        None => (rep.geom[2], rep.geom[3]),
    };

    let mut defs = String::new();
    let mut body = String::new();

    if let Some(img) = &rep.image {
        push_image(&mut body, "  ", 0, 0, width, height, img, None);
    }
    for (i, rollover) in rep.rollovers.iter().enumerate() {
        push_zone(&mut body, &mut defs, i, rollover);
    }

    let mut s = String::new();
    let _ = write!(
        s,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" \
         width=\"{width}\" height=\"{height}\">\n"
    );
    s.push_str(
        "  <style>\n\
        \x20   .hot { visibility: hidden; }\n\
        \x20   .zone:hover .hot { visibility: visible; }\n\
        \x20   .hit { fill: #000; fill-opacity: 0; pointer-events: fill; cursor: pointer; }\n\
        \x20 </style>\n",
    );
    if !defs.is_empty() {
        s.push_str("  <defs>\n");
        s.push_str(&defs);
        s.push_str("  </defs>\n");
    }
    s.push_str(&body);
    s.push_str("</svg>\n");
    s
}

/// One interactive zone: a hover overlay clipped to the zone silhouette, under a
/// transparent hit path of that same silhouette, grouped so `:hover` reveals the
/// overlay.
///
/// The silhouette is the union of the `2001` mask pieces (the pill / badge / arrow
/// outline). The overlay is the zone's colored pieces — the `2002` base layer then
/// the `2003` content layer, in piece-array (= paint) order — each clipped to
/// *the silhouette*, not to its own glyph plane. A colored piece is a full
/// re-render of the zone, so clipping to the silhouette replaces the resting
/// pixels underneath wholesale; clipping to the glyphs would instead let the
/// resting text's anti-aliasing bleed around them as a grey halo.
fn push_zone(body: &mut String, defs: &mut String, idx: usize, rollover: &Rollover) {
    // Zone silhouette = union of the 2001 mask pieces. Drives both the hit target
    // and the overlay's clip.
    let mut sil_d = String::new();
    for piece in &rollover.pieces {
        if piece.kind != KIND_HIT {
            continue;
        }
        if let Some(mask) = &piece.raw {
            let rects = greedy_mesh(mask.bytes(), mask.width(), mask.height());
            let g = piece.geom;
            sil_d.push_str(&rects_to_path_d(&rects, g.x, g.y));
        }
    }

    // Hover overlay: the zone's colored pieces (2002 base, then 2003 content — in
    // array order = paint order), each clipped to the silhouette. The 2001 hit
    // pieces carry no color, so they fall out here.
    let mut hot = String::new();
    let clip_id = (!sil_d.is_empty()).then(|| format!("zone{idx}"));
    for piece in &rollover.pieces {
        let Some(color) = &piece.encoded else { continue };
        let g = piece.geom;
        push_image(&mut hot, "      ", g.x, g.y, g.width, g.height, color, clip_id.as_deref());
    }

    if let Some(id) = &clip_id
        && !hot.is_empty()
    {
        let _ = write!(
            defs,
            "    <clipPath id=\"{id}\" clipPathUnits=\"userSpaceOnUse\"><path d=\"{sil_d}\"/></clipPath>\n"
        );
    }

    let _ = write!(body, "  <g class=\"zone\" data-zone=\"{idx}\">\n");
    if !hot.is_empty() {
        body.push_str("    <g class=\"hot\">\n");
        body.push_str(&hot);
        body.push_str("    </g>\n");
    }
    if !sil_d.is_empty() {
        let _ = write!(body, "    <path class=\"hit\" d=\"{sil_d}\"/>\n");
    }
    body.push_str("  </g>\n");
}

/// A maximal rectangle of set bits found by greedy meshing.
struct MeshRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

/// Greedy-mesh a 1-bpp plane (MSB-first, `ceil(w·h/8)` bytes, no row padding)
/// into a minimal set of axis-aligned rectangles covering its set bits.
///
/// For each uncovered set cell: grow the run right as far as it stays set, then
/// grow that strip down while every row of the span matches, emit the rectangle,
/// mark it covered, and continue. Pixel-exact (corners staircase); the merge just
/// minimizes the rectangle count.
fn greedy_mesh(bytes: &[u8], width: u32, height: u32) -> Vec<MeshRect> {
    let (w, h) = (width as usize, height as usize);
    let bit = |x: usize, y: usize| -> bool {
        let i = y * w + x;
        let byte = i >> 3;
        byte < bytes.len() && (bytes[byte] >> (7 - (i & 7))) & 1 == 1
    };

    let mut used = vec![false; w * h];
    let mut rects = Vec::new();
    for y0 in 0..h {
        for x0 in 0..w {
            if used[y0 * w + x0] || !bit(x0, y0) {
                continue;
            }
            // Grow right.
            let mut x1 = x0 + 1;
            while x1 < w && !used[y0 * w + x1] && bit(x1, y0) {
                x1 += 1;
            }
            // Grow down while the whole [x0, x1) span is set and free.
            let mut y1 = y0 + 1;
            'down: while y1 < h {
                for x in x0..x1 {
                    if used[y1 * w + x] || !bit(x, y1) {
                        break 'down;
                    }
                }
                y1 += 1;
            }
            for y in y0..y1 {
                for x in x0..x1 {
                    used[y * w + x] = true;
                }
            }
            rects.push(MeshRect {
                x: x0 as i32,
                y: y0 as i32,
                w: (x1 - x0) as i32,
                h: (y1 - y0) as i32,
            });
        }
    }
    rects
}

/// Render meshed rectangles as SVG path data, offset to `(ox, oy)`.
fn rects_to_path_d(rects: &[MeshRect], ox: i32, oy: i32) -> String {
    let mut d = String::with_capacity(rects.len() * 24);
    for r in rects {
        let _ = write!(d, "M{} {}h{}v{}h-{}z", ox + r.x, oy + r.y, r.w, r.h, r.w);
    }
    d
}

fn push_image(
    s: &mut String,
    indent: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    img: &PooledImage,
    clip: Option<&str>,
) {
    s.push_str(indent);
    s.push_str("<image ");
    if let Some(id) = clip {
        let _ = write!(s, "clip-path=\"url(#{id})\" ");
    }
    let _ = write!(s, "x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" href=\"data:image/png;base64,");
    base64_into(s, img.bytes());
    s.push_str("\"/>\n");
}

/// Standard base64 (RFC 4648), appended directly to `out`.
fn base64_into(out: &mut String, data: &[u8]) {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    out.reserve(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(n >> 18 & 0x3f) as usize] as char);
        out.push(TABLE[(n >> 12 & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6 & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
}
