//! A minimal PDF 1.4 writer: vector paths and Helvetica text, nothing else.
//!
//! ```text
//! 1 Catalog → 2 Pages → pages, each a Page object and its content stream (last)
//!                         3 Helvetica, 4 Helvetica-Bold, 5… ExtGState (fill opacity)
//! ```

use std::fmt::Write;

use crate::canvas::{dash_array, num, Anchor, Doc, Fill, Geom, Prim, Rect, Sheet, Stroke, Weight};
use crate::palette::Rgb;

const HEADER: &[u8] = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n";
const OBJ_CATALOG: usize = 1;
const OBJ_PAGES: usize = 2;
const OBJ_FONT_REGULAR: usize = 3;
const OBJ_FONT_BOLD: usize = 4;
const OBJ_FIRST_STATE: usize = 5;
/// Circle quarter as a cubic Bézier: 4/3 * (sqrt 2 - 1).
const BEZIER_KAPPA: f64 = 0.552_284_749_8;
/// Mean advance of Helvetica text in em; PDF has no layout, so centring estimates the width
/// instead of carrying a metrics table (a few points off on a one-word label).
const AVG_CHAR_EM: f64 = 0.5;
/// Opacity is keyed in thousandths so equal values share one state.
const OPACITY_STEPS: f64 = 1000.0;

pub(crate) fn write(doc: &Doc) -> Vec<u8> {
    let states = opacities(doc);

    // Page 1 is the plan, then the sheets; each page is two objects, page and content.
    let mut pages = vec![(doc.width_pt, doc.height_pt, content(doc, &states))];
    for sheet in &doc.sheets {
        pages.push((sheet.width_pt, sheet.height_pt, sheet_content(doc, sheet)));
    }
    let first_page = OBJ_FIRST_STATE + states.len();
    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", first_page + 2 * i))
        .collect();

    let mut resources =
        format!("<< /Font << /F1 {OBJ_FONT_REGULAR} 0 R /F2 {OBJ_FONT_BOLD} 0 R >> /ExtGState <<");
    for i in 0..states.len() {
        let _ = write!(resources, " /G{i} {} 0 R", OBJ_FIRST_STATE + i);
    }
    resources.push_str(" >> >>");

    let mut objects: Vec<Vec<u8>> = vec![
        format!("<< /Type /Catalog /Pages {OBJ_PAGES} 0 R >>").into_bytes(),
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        )
        .into_bytes(),
        font("Helvetica").into_bytes(),
        font("Helvetica-Bold").into_bytes(),
    ];
    for key in &states {
        let ca = f64::from(*key) / OPACITY_STEPS;
        objects.push(format!("<< /Type /ExtGState /ca {} >>", num3(ca)).into_bytes());
    }
    for (i, (width_pt, height_pt, content)) in pages.iter().enumerate() {
        objects.push(
            format!(
                "<< /Type /Page /Parent {OBJ_PAGES} 0 R /MediaBox [0 0 {} {}] /Resources {resources} /Contents {} 0 R >>",
                num(*width_pt),
                num(*height_pt),
                first_page + 2 * i + 1
            )
            .into_bytes(),
        );
        objects.push(stream(content));
    }
    debug_assert_eq!(objects.len(), first_page - 1 + 2 * pages.len());

    let mut out = HEADER.to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_at = out.len();
    let mut table = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in &offsets {
        let _ = writeln!(table, "{offset:010} 00000 n ");
    }
    let _ = write!(
        table,
        "trailer\n<< /Size {} /Root {OBJ_CATALOG} 0 R >>\nstartxref\n{xref_at}\n%%EOF\n",
        objects.len() + 1
    );
    out.extend_from_slice(table.as_bytes());

    out
}

fn font(name: &str) -> String {
    format!("<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>")
}

fn stream(content: &[u8]) -> Vec<u8> {
    let mut out = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
    out.extend_from_slice(content);
    out.extend_from_slice(b"\nendstream");

    out
}

fn num3(v: f64) -> String {
    let text = format!("{v:.3}");

    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn key(opacity: f64) -> u32 {
    (opacity.clamp(0.0, 1.0) * OPACITY_STEPS).round() as u32
}

/// The distinct translucent fills of the page, in first-use order.
fn opacities(doc: &Doc) -> Vec<u32> {
    let mut keys = Vec::new();
    for prim in doc.plan.iter().chain(&doc.chrome) {
        let Prim::Shape {
            fill: Some(Fill { opacity, .. }),
            ..
        } = prim
        else {
            continue;
        };
        let k = key(*opacity);
        if k < OPACITY_STEPS as u32 && !keys.contains(&k) {
            keys.push(k);
        }
    }

    keys
}

/// The sheet's paper colour over the whole page.
fn paper(rgb: Rgb, width_pt: f64, height_pt: f64) -> String {
    let [r, g, b] = rgb.unit();

    format!(
        "{} {} {} rg 0 0 {} {} re f\n",
        num3(r),
        num3(g),
        num3(b),
        num(width_pt),
        num(height_pt)
    )
}

fn sheet_content(doc: &Doc, sheet: &Sheet) -> Vec<u8> {
    let mut s = paper(doc.paper, sheet.width_pt, sheet.height_pt);
    for prim in &sheet.prims {
        s.push_str(&operators(prim, sheet.height_pt, &[]));
    }

    s.into_bytes()
}

fn content(doc: &Doc, states: &[u32]) -> Vec<u8> {
    let h = doc.height_pt;
    let mut s = paper(doc.paper, doc.width_pt, h);

    // The plan is clipped to the set; the axis runs past it.
    let c = doc.clip;
    let _ = writeln!(
        s,
        "q {} {} {} {} re W n",
        num(c.x),
        num(h - c.y - c.h),
        num(c.w),
        num(c.h)
    );
    for prim in &doc.plan {
        s.push_str(&operators(prim, h, states));
    }
    s.push_str("Q\n");

    for prim in &doc.chrome {
        s.push_str(&operators(prim, h, states));
    }

    s.into_bytes()
}

fn text_width(text: &str, size_pt: f64) -> f64 {
    text.chars().count() as f64 * size_pt * AVG_CHAR_EM
}

fn colour(rgb: [f64; 3]) -> String {
    format!("{} {} {}", num3(rgb[0]), num3(rgb[1]), num3(rgb[2]))
}

fn operators(prim: &Prim, h: f64, states: &[u32]) -> String {
    match prim {
        Prim::Shape {
            geom, fill, stroke, ..
        } => shape_ops(geom, *fill, *stroke, h, states),
        Prim::Text {
            at,
            anchor,
            size_pt,
            weight,
            rgb,
            text,
            ..
        } => {
            let font = match weight {
                Weight::Regular => "F1",
                Weight::Bold => "F2",
            };
            let x = match anchor {
                Anchor::Start => at.0,
                Anchor::Middle => at.0 - text_width(text, *size_pt) / 2.0,
            };
            format!(
                "BT /{font} {} Tf {} rg {} {} Td ({}) Tj ET\n",
                num(*size_pt),
                colour(rgb.unit()),
                num(x),
                num(h - at.1),
                pdf_string(text)
            )
        }
    }
}

fn shape_ops(
    geom: &Geom,
    fill: Option<Fill>,
    stroke: Option<Stroke>,
    h: f64,
    states: &[u32],
) -> String {
    let mut s = String::from("q ");

    if let Some(f) = fill {
        let _ = write!(s, "{} rg ", colour(f.rgb.unit()));
        let state = states.iter().position(|k| *k == key(f.opacity));
        if let (Some(i), true) = (state, f.opacity < 1.0) {
            let _ = write!(s, "/G{i} gs ");
        }
    }
    if let Some(st) = stroke {
        let _ = write!(s, "{} RG {} w ", colour(st.rgb.unit()), num(st.width_pt));
        let dash = dash_array(st.dash, st.width_pt);
        if !dash.is_empty() {
            let parts: Vec<String> = dash.iter().map(|d| num(*d)).collect();
            let _ = write!(s, "[{}] 0 d ", parts.join(" "));
        }
        s.push_str("1 j ");
    }

    s.push_str(&path(geom, h));
    s.push_str(match (fill.is_some(), stroke.is_some()) {
        (true, true) => " B",
        (true, false) => " f",
        (false, true) => " S",
        (false, false) => " n",
    });
    s.push_str(" Q\n");

    s
}

/// Path construction in PDF space (origin bottom left).
fn path(geom: &Geom, h: f64) -> String {
    let at = |(x, y): (f64, f64)| format!("{} {}", num(x), num(h - y));

    match geom {
        Geom::Open(pts) | Geom::Closed(pts) => {
            let mut s = String::new();
            for (i, p) in pts.iter().enumerate() {
                let op = if i == 0 { "m" } else { "l" };
                let _ = write!(s, "{} {op} ", at(*p));
            }
            if matches!(geom, Geom::Closed(_)) {
                s.push('h');
            }
            s.trim_end().to_string()
        }
        Geom::Circle { center, radius } => circle(*center, *radius, h),
        Geom::Rect(Rect { x, y, w, h: rh }) => {
            format!(
                "{} {} {} {} re",
                num(*x),
                num(h - y - rh),
                num(*w),
                num(*rh)
            )
        }
    }
}

/// Four Bézier quarters.
fn circle(center: (f64, f64), r: f64, h: f64) -> String {
    let (cx, cy) = (center.0, h - center.1);
    let k = r * BEZIER_KAPPA;
    let p = |x: f64, y: f64| format!("{} {}", num(x), num(y));

    format!(
        "{} m {} {} {} c {} {} {} c {} {} {} c {} {} {} c h",
        p(cx + r, cy),
        p(cx + r, cy + k),
        p(cx + k, cy + r),
        p(cx, cy + r),
        p(cx - k, cy + r),
        p(cx - r, cy + k),
        p(cx - r, cy),
        p(cx - r, cy - k),
        p(cx - k, cy - r),
        p(cx, cy - r),
        p(cx + k, cy - r),
        p(cx + r, cy - k),
        p(cx + r, cy),
    )
}

/// WinAnsi (Windows-1252) byte of a char; `?` when it has none.
fn winansi(c: char) -> u8 {
    const FALLBACK: u8 = b'?';
    // The 0x80-0x9F block, where WinAnsi differs from Latin-1.
    const SPECIAL: [(char, u8); 12] = [
        ('€', 0x80),
        ('…', 0x85),
        ('‘', 0x91),
        ('’', 0x92),
        ('“', 0x93),
        ('”', 0x94),
        ('•', 0x95),
        ('–', 0x96),
        ('—', 0x97),
        ('„', 0x84),
        ('‚', 0x82),
        ('™', 0x99),
    ];

    if let Some((_, b)) = SPECIAL.iter().find(|(ch, _)| *ch == c) {
        return *b;
    }

    match u32::from(c) {
        n @ (0x20..=0x7E | 0xA0..=0xFF) => n as u8,
        _ => FALLBACK,
    }
}

/// A literal string: ASCII stays, everything else is an octal escape, so the stream stays text.
fn pdf_string(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let b = winansi(c);
        match b {
            b'(' | b')' | b'\\' => {
                out.push('\\');
                out.push(char::from(b));
            }
            0x20..=0x7E => out.push(char::from(b)),
            _ => {
                let _ = write!(out, "\\{b:03o}");
            }
        }
    }

    out
}
