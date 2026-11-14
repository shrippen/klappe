//! Liberation Sans (SIL OFL 1.1, `fonts/LICENSE`) read as TrueType, for the PNG raster.
//! Metrically the same as Arial and Helvetica, so the layout's line breaks hold.
//!
//! ```text
//! char ─► cmap ─► glyph id ─┬─► hmtx   (advance)
//!                           └─► loca ─► glyf ─► contours ─► line segments
//! ```
//!
//! Only what the raster needs: no kerning, no hinting, no shaping.

use std::sync::OnceLock;

use crate::canvas::{Pt, Weight};

const REGULAR_TTF: &[u8] = include_bytes!("../fonts/LiberationSans-Regular.ttf");
const BOLD_TTF: &[u8] = include_bytes!("../fonts/LiberationSans-Bold.ttf");

/// Largest distance in points between a curve and the segments that replace it.
const FLATNESS_TOLERANCE_PT: f64 = 0.02;
/// Composite glyphs nest at most this deep; more means a cyclic or broken font.
const MAX_COMPOSITE_DEPTH: u32 = 6;
const NOTDEF_GLYPH: u16 = 0;

const TAG_HEAD: &[u8; 4] = b"head";
const TAG_MAXP: &[u8; 4] = b"maxp";
const TAG_CMAP: &[u8; 4] = b"cmap";
const TAG_HHEA: &[u8; 4] = b"hhea";
const TAG_HMTX: &[u8; 4] = b"hmtx";
const TAG_LOCA: &[u8; 4] = b"loca";
const TAG_GLYF: &[u8; 4] = b"glyf";

/// Simple glyph flags.
const ON_CURVE: u8 = 0x01;
const X_SHORT: u8 = 0x02;
const Y_SHORT: u8 = 0x04;
const REPEAT: u8 = 0x08;
const X_SAME_OR_POSITIVE: u8 = 0x10;
const Y_SAME_OR_POSITIVE: u8 = 0x20;

/// Composite glyph flags.
const ARGS_ARE_WORDS: u16 = 0x0001;
const ARGS_ARE_XY: u16 = 0x0002;
const HAS_SCALE: u16 = 0x0008;
const MORE_COMPONENTS: u16 = 0x0020;
const HAS_XY_SCALE: u16 = 0x0040;
const HAS_2X2: u16 = 0x0080;
/// 2.14 fixed point.
const F2DOT14_ONE: f64 = 16384.0;

#[derive(Debug)]
struct FontError(&'static str);

type Res<T> = Result<T, FontError>;

/// One point of a glyph in font units, y up.
#[derive(Debug, Clone, Copy)]
struct GlyphPoint {
    x: f64,
    y: f64,
    on_curve: bool,
}

type Contour = Vec<GlyphPoint>;

/// Two by two matrix plus offset, applied to a composite's component.
#[derive(Debug, Clone, Copy)]
struct Affine {
    xx: f64,
    xy: f64,
    yx: f64,
    yy: f64,
    dx: f64,
    dy: f64,
}

#[derive(Debug)]
struct Font {
    data: &'static [u8],
    units_per_em: f64,
    long_loca: bool,
    glyph_count: u16,
    metric_count: u16,
    cmap: usize,
    hmtx: usize,
    loca: usize,
    glyf: usize,
}

fn rd_u16(data: &[u8], at: usize) -> Res<u16> {
    let bytes = data.get(at..at + 2).ok_or(FontError("read past end"))?;

    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn rd_i16(data: &[u8], at: usize) -> Res<i16> {
    Ok(rd_u16(data, at)? as i16)
}

fn rd_u32(data: &[u8], at: usize) -> Res<u32> {
    let bytes = data.get(at..at + 4).ok_or(FontError("read past end"))?;

    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn rd_u8(data: &[u8], at: usize) -> Res<u8> {
    data.get(at).copied().ok_or(FontError("read past end"))
}

fn table(data: &[u8], tag: &[u8; 4]) -> Res<usize> {
    let count = usize::from(rd_u16(data, 4)?);

    for i in 0..count {
        let entry = 12 + i * 16;
        let name = data
            .get(entry..entry + 4)
            .ok_or(FontError("bad directory"))?;
        if name == tag {
            return Ok(rd_u32(data, entry + 8)? as usize);
        }
    }

    Err(FontError("table missing"))
}

/// The cmap subtable to use: full Unicode (format 12) first, else the BMP one.
fn pick_cmap(data: &[u8], cmap: usize) -> Res<usize> {
    let count = usize::from(rd_u16(data, cmap + 2)?);
    let mut bmp = None;

    for i in 0..count {
        let rec = cmap + 4 + i * 8;
        let platform = rd_u16(data, rec)?;
        let encoding = rd_u16(data, rec + 2)?;
        let offset = cmap + rd_u32(data, rec + 4)? as usize;
        let unicode = platform == 0 || (platform == 3 && (encoding == 1 || encoding == 10));
        if !unicode {
            continue;
        }

        match rd_u16(data, offset)? {
            12 => return Ok(offset),
            4 if bmp.is_none() => bmp = Some(offset),
            _ => {}
        }
    }

    bmp.ok_or(FontError("no unicode cmap"))
}

impl Font {
    fn parse(data: &'static [u8]) -> Res<Font> {
        let head = table(data, TAG_HEAD)?;
        let maxp = table(data, TAG_MAXP)?;
        let hhea = table(data, TAG_HHEA)?;
        let units = rd_u16(data, head + 18)?;
        if units == 0 {
            return Err(FontError("unitsPerEm is zero"));
        }

        Ok(Font {
            data,
            units_per_em: f64::from(units),
            long_loca: rd_i16(data, head + 50)? != 0,
            glyph_count: rd_u16(data, maxp + 4)?,
            metric_count: rd_u16(data, hhea + 34)?,
            cmap: pick_cmap(data, table(data, TAG_CMAP)?)?,
            hmtx: table(data, TAG_HMTX)?,
            loca: table(data, TAG_LOCA)?,
            glyf: table(data, TAG_GLYF)?,
        })
    }

    fn glyph_id(&self, c: char) -> u16 {
        self.lookup(u32::from(c)).unwrap_or(NOTDEF_GLYPH)
    }

    fn lookup(&self, code: u32) -> Res<u16> {
        match rd_u16(self.data, self.cmap)? {
            12 => self.lookup_12(code),
            _ => self.lookup_4(code),
        }
    }

    fn lookup_4(&self, code: u32) -> Res<u16> {
        let d = self.data;
        let base = self.cmap;
        let Ok(code) = u16::try_from(code) else {
            return Ok(NOTDEF_GLYPH);
        };
        let seg_count = usize::from(rd_u16(d, base + 6)?) / 2;
        let ends = base + 14;
        let starts = ends + seg_count * 2 + 2;
        let deltas = starts + seg_count * 2;
        let ranges = deltas + seg_count * 2;

        for i in 0..seg_count {
            if code > rd_u16(d, ends + i * 2)? {
                continue;
            }

            let start = rd_u16(d, starts + i * 2)?;
            if code < start {
                return Ok(NOTDEF_GLYPH);
            }

            let delta = rd_u16(d, deltas + i * 2)?;
            let range = rd_u16(d, ranges + i * 2)?;
            if range == 0 {
                return Ok(code.wrapping_add(delta));
            }

            let at = ranges + i * 2 + usize::from(range) + usize::from(code - start) * 2;
            let glyph = rd_u16(d, at)?;
            if glyph == 0 {
                return Ok(NOTDEF_GLYPH);
            }

            return Ok(glyph.wrapping_add(delta));
        }

        Ok(NOTDEF_GLYPH)
    }

    fn lookup_12(&self, code: u32) -> Res<u16> {
        let d = self.data;
        let groups = rd_u32(d, self.cmap + 12)? as usize;

        for i in 0..groups {
            let at = self.cmap + 16 + i * 12;
            let (start, end) = (rd_u32(d, at)?, rd_u32(d, at + 4)?);
            if code < start || code > end {
                continue;
            }

            let glyph = rd_u32(d, at + 8)? + (code - start);

            return Ok(u16::try_from(glyph).unwrap_or(NOTDEF_GLYPH));
        }

        Ok(NOTDEF_GLYPH)
    }

    /// Advance in font units.
    fn advance(&self, glyph: u16) -> Res<f64> {
        if self.metric_count == 0 {
            return Err(FontError("no metrics"));
        }

        let index = glyph.min(self.metric_count - 1);

        Ok(f64::from(rd_u16(
            self.data,
            self.hmtx + usize::from(index) * 4,
        )?))
    }

    /// The glyph's bytes in `glyf`; empty for a blank glyph such as space.
    fn glyph_data(&self, glyph: u16) -> Res<&'static [u8]> {
        if glyph >= self.glyph_count {
            return Err(FontError("glyph out of range"));
        }

        let i = usize::from(glyph);
        let (from, to) = if self.long_loca {
            (
                rd_u32(self.data, self.loca + i * 4)? as usize,
                rd_u32(self.data, self.loca + i * 4 + 4)? as usize,
            )
        } else {
            (
                usize::from(rd_u16(self.data, self.loca + i * 2)?) * 2,
                usize::from(rd_u16(self.data, self.loca + i * 2 + 2)?) * 2,
            )
        };
        if to < from {
            return Err(FontError("loca not ascending"));
        }

        self.data
            .get(self.glyf + from..self.glyf + to)
            .ok_or(FontError("glyph outside glyf"))
    }

    fn contours(&self, glyph: u16, depth: u32) -> Res<Vec<Contour>> {
        if depth > MAX_COMPOSITE_DEPTH {
            return Err(FontError("composite nesting too deep"));
        }

        let data = self.glyph_data(glyph)?;
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let count = rd_i16(data, 0)?;
        if count >= 0 {
            return simple_glyph(data, count as usize);
        }

        self.composite_glyph(data, depth)
    }

    fn composite_glyph(&self, data: &[u8], depth: u32) -> Res<Vec<Contour>> {
        let mut out = Vec::new();
        let mut at = 10;

        loop {
            let flags = rd_u16(data, at)?;
            let component = rd_u16(data, at + 2)?;
            at += 4;

            let (arg1, arg2);
            if flags & ARGS_ARE_WORDS != 0 {
                (arg1, arg2) = (
                    f64::from(rd_i16(data, at)?),
                    f64::from(rd_i16(data, at + 2)?),
                );
                at += 4;
            } else {
                (arg1, arg2) = (
                    f64::from(rd_u8(data, at)? as i8),
                    f64::from(rd_u8(data, at + 1)? as i8),
                );
                at += 2;
            }

            let mut m = Affine {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                dx: 0.0,
                dy: 0.0,
            };
            // Point matching (no XY flag) is not used by the fonts we ship; offset stays 0.
            if flags & ARGS_ARE_XY != 0 {
                (m.dx, m.dy) = (arg1, arg2);
            }

            let f = |off: usize| rd_i16(data, off).map(|v| f64::from(v) / F2DOT14_ONE);
            if flags & HAS_SCALE != 0 {
                m.xx = f(at)?;
                m.yy = m.xx;
                at += 2;
            } else if flags & HAS_XY_SCALE != 0 {
                (m.xx, m.yy) = (f(at)?, f(at + 2)?);
                at += 4;
            } else if flags & HAS_2X2 != 0 {
                (m.xx, m.xy, m.yx, m.yy) = (f(at)?, f(at + 2)?, f(at + 4)?, f(at + 6)?);
                at += 8;
            }

            for mut contour in self.contours(component, depth + 1)? {
                for p in &mut contour {
                    (p.x, p.y) = (
                        m.xx * p.x + m.yx * p.y + m.dx,
                        m.xy * p.x + m.yy * p.y + m.dy,
                    );
                }
                out.push(contour);
            }

            if flags & MORE_COMPONENTS == 0 {
                return Ok(out);
            }
        }
    }
}

fn simple_glyph(data: &[u8], contour_count: usize) -> Res<Vec<Contour>> {
    let mut ends = Vec::with_capacity(contour_count);
    for i in 0..contour_count {
        ends.push(usize::from(rd_u16(data, 10 + i * 2)?));
    }

    let Some(&last) = ends.last() else {
        return Ok(Vec::new());
    };
    let point_count = last + 1;
    let mut at = 10 + contour_count * 2;
    at += 2 + usize::from(rd_u16(data, at)?);

    let mut flags = Vec::with_capacity(point_count);
    while flags.len() < point_count {
        let flag = rd_u8(data, at)?;
        at += 1;
        flags.push(flag);
        if flag & REPEAT == 0 {
            continue;
        }

        let repeats = rd_u8(data, at)?;
        at += 1;
        for _ in 0..repeats {
            flags.push(flag);
        }
    }
    flags.truncate(point_count);

    let xs = coords(data, &mut at, &flags, X_SHORT, X_SAME_OR_POSITIVE)?;
    let ys = coords(data, &mut at, &flags, Y_SHORT, Y_SAME_OR_POSITIVE)?;

    let mut out = Vec::with_capacity(contour_count);
    let mut start = 0;
    for end in ends {
        if end < start || end >= point_count {
            return Err(FontError("bad contour end"));
        }

        out.push(
            (start..=end)
                .map(|i| GlyphPoint {
                    x: xs[i],
                    y: ys[i],
                    on_curve: flags[i] & ON_CURVE != 0,
                })
                .collect(),
        );
        start = end + 1;
    }

    Ok(out)
}

/// One axis of a simple glyph: deltas, short or long, summed up.
fn coords(data: &[u8], at: &mut usize, flags: &[u8], short: u8, same: u8) -> Res<Vec<f64>> {
    let mut value = 0i32;
    let mut out = Vec::with_capacity(flags.len());

    for &flag in flags {
        if flag & short != 0 {
            let d = i32::from(rd_u8(data, *at)?);
            *at += 1;
            value += if flag & same != 0 { d } else { -d };
        } else if flag & same == 0 {
            value += i32::from(rd_i16(data, *at)?);
            *at += 2;
        }
        out.push(f64::from(value));
    }

    Ok(out)
}

fn mid(a: Pt, b: Pt) -> Pt {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// Appends the segments of the quadratic `from`-`ctrl`-`to`, ending at `to`.
fn flatten_quad(out: &mut Vec<Pt>, from: Pt, ctrl: Pt, to: Pt) {
    let (dx, dy) = (from.0 - 2.0 * ctrl.0 + to.0, from.1 - 2.0 * ctrl.1 + to.1);
    // Chord error of n equal steps is |dx,dy| / (4 n^2).
    let dev = dx.hypot(dy) / 4.0;
    let steps = (dev / FLATNESS_TOLERANCE_PT).sqrt().ceil().max(1.0) as u32;

    for i in 1..=steps {
        let t = f64::from(i) / f64::from(steps);
        let u = 1.0 - t;
        out.push((
            u * u * from.0 + 2.0 * u * t * ctrl.0 + t * t * to.0,
            u * u * from.1 + 2.0 * u * t * ctrl.1 + t * t * to.1,
        ));
    }
}

/// A TrueType contour (on and off curve points, implied on points between two off points) as a
/// closed polygon, already placed: `pen_x` plus `scale`, y flipped to point down.
fn flatten(contour: &[GlyphPoint], pen_x: f64, scale: f64) -> Vec<Pt> {
    let place = |p: &GlyphPoint| (pen_x + p.x * scale, -p.y * scale);
    let pts: Vec<(Pt, bool)> = contour.iter().map(|p| (place(p), p.on_curve)).collect();
    let (Some(&first), Some(&last)) = (pts.first(), pts.last()) else {
        return Vec::new();
    };

    // The start must be on the curve: the first point, the last, or the midpoint of both.
    let (start, skip) = if first.1 {
        (first.0, 1)
    } else if last.1 {
        (last.0, 0)
    } else {
        (mid(first.0, last.0), 0)
    };
    let rest = if first.1 || !last.1 {
        &pts[skip..]
    } else {
        &pts[..pts.len() - 1]
    };

    let mut out = vec![start];
    let mut cur = start;
    let mut ctrl: Option<Pt> = None;

    for &(p, on) in rest {
        match (ctrl, on) {
            (None, true) => {
                out.push(p);
                cur = p;
            }
            (None, false) => ctrl = Some(p),
            (Some(c), true) => {
                flatten_quad(&mut out, cur, c, p);
                cur = p;
                ctrl = None;
            }
            (Some(c), false) => {
                let m = mid(c, p);
                flatten_quad(&mut out, cur, c, m);
                cur = m;
                ctrl = Some(p);
            }
        }
    }
    if let Some(c) = ctrl {
        flatten_quad(&mut out, cur, c, start);
    }

    // Closed implicitly: drop the repeat of the start.
    if out.len() > 1 && out.last() == Some(&start) {
        out.pop();
    }

    out
}

fn font(weight: Weight) -> Res<&'static Font> {
    static REGULAR: OnceLock<Res<Font>> = OnceLock::new();
    static BOLD: OnceLock<Res<Font>> = OnceLock::new();

    let (cell, data) = match weight {
        Weight::Regular => (&REGULAR, REGULAR_TTF),
        Weight::Bold => (&BOLD, BOLD_TTF),
    };

    cell.get_or_init(|| Font::parse(data))
        .as_ref()
        .map_err(|e| FontError(e.0))
}

fn outline(text: &str, weight: Weight, size_pt: f64) -> Res<Vec<Vec<Pt>>> {
    let font = font(weight)?;
    let scale = size_pt / font.units_per_em;
    let mut pen = 0.0;
    let mut out = Vec::new();

    for c in text.chars() {
        let glyph = font.glyph_id(c);
        for contour in font.contours(glyph, 0)? {
            let poly = flatten(&contour, pen, scale);
            if poly.len() >= 3 {
                out.push(poly);
            }
        }
        pen += font.advance(glyph)? * scale;
    }

    Ok(out)
}

/// Closed outlines of `text`, origin on the baseline at the start, y down, fill nonzero.
/// Empty if the font cannot be read.
pub(crate) fn text_outline(text: &str, weight: Weight, size_pt: f64) -> Vec<Vec<Pt>> {
    outline(text, weight, size_pt).unwrap_or_default()
}

/// Advance width of `text` without kerning; 0 if the font cannot be read.
pub(crate) fn text_width_pt(text: &str, weight: Weight, size_pt: f64) -> f64 {
    let width = |font: &Font| -> Res<f64> {
        let mut units = 0.0;
        for c in text.chars() {
            units += font.advance(font.glyph_id(c))?;
        }

        Ok(units / font.units_per_em * size_pt)
    };

    font(weight).and_then(width).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics;

    /// Arial's advances are Helvetica's, rounded to 1/2048 em (0.0005 em).
    const ADVANCE_TOLERANCE_EM: f64 = 0.001;

    fn area(poly: &[Pt]) -> f64 {
        let mut sum = 0.0;
        for (i, a) in poly.iter().enumerate() {
            let b = poly[(i + 1) % poly.len()];
            sum += a.0 * b.1 - b.0 * a.1;
        }

        sum / 2.0
    }

    #[test]
    fn regular_advances_match_helvetica() {
        for code in 32u8..=126 {
            let text = char::from(code).to_string();
            let ours = text_width_pt(&text, Weight::Regular, 1.0);
            let helv = metrics::text_width_pt(&text, 1.0, Weight::Regular);

            assert!(
                (ours - helv).abs() <= ADVANCE_TOLERANCE_EM,
                "{text:?}: {ours} vs {helv}"
            );
        }
    }

    #[test]
    fn bold_font_loads_and_is_wider() {
        let regular = text_width_pt("Hamburg", Weight::Regular, 10.0);
        let bold = text_width_pt("Hamburg", Weight::Bold, 10.0);

        assert!(bold > regular, "{bold} vs {regular}");
        assert!(!text_outline("Hamburg", Weight::Bold, 10.0).is_empty());
    }

    #[test]
    fn umlauts_and_sharp_s_have_outlines() {
        for c in ["ä", "ö", "ü", "Ä", "Ö", "Ü", "ß"] {
            assert!(!text_outline(c, Weight::Regular, 10.0).is_empty(), "{c}");
        }
    }

    #[test]
    fn o_has_two_opposite_contours() {
        let o = text_outline("O", Weight::Regular, 100.0);

        assert_eq!(o.len(), 2);
        assert!(
            area(&o[0]) * area(&o[1]) < 0.0,
            "{} {}",
            area(&o[0]),
            area(&o[1])
        );
    }

    #[test]
    fn composite_glyph_has_its_parts() {
        let a = text_outline("A", Weight::Regular, 100.0).len();
        let umlaut = text_outline("Ä", Weight::Regular, 100.0).len();

        // Base letter plus two dots.
        assert_eq!(umlaut, a + 2);
    }

    #[test]
    fn outline_sits_on_the_baseline_y_down() {
        let h = text_outline("H", Weight::Regular, 100.0);
        let (min_y, max_y) = h
            .iter()
            .flatten()
            .fold((f64::MAX, f64::MIN), |(lo, hi), p| {
                (lo.min(p.1), hi.max(p.1))
            });

        assert!(max_y.abs() < 0.5, "baseline {max_y}");
        assert!((-75.0..-65.0).contains(&min_y), "cap height {min_y}");
    }

    #[test]
    fn space_has_width_but_no_outline() {
        assert!(text_outline(" ", Weight::Regular, 10.0).is_empty());
        assert!(text_width_pt(" ", Weight::Regular, 10.0) > 0.0);
    }

    #[test]
    fn missing_char_falls_back_to_notdef() {
        // U+4E2D is not in Liberation Sans.
        assert!(!text_outline("\u{4e2d}", Weight::Regular, 10.0).is_empty());
    }

    #[test]
    fn size_scales_linearly() {
        let a = text_width_pt("Klappe", Weight::Regular, 10.0);
        let b = text_width_pt("Klappe", Weight::Regular, 20.0);

        assert!((b - 2.0 * a).abs() < 1e-9);
    }
}
