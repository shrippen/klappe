use std::fmt::Write;

use crate::canvas::{dash_array, num, Anchor, Doc, Fill, Geom, Prim, Pt, Rect, Stroke, Weight};
use crate::image::base64;

const FONT_FAMILY: &str = "Helvetica, Arial, sans-serif";
const CLIP_ID: &str = "plan";

pub(crate) fn write(doc: &Doc) -> String {
    let (w, h) = (num(doc.width_pt), num(doc.height_pt));
    let mut out = String::new();

    // Writing to a String cannot fail.
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}pt\" height=\"{h}pt\" viewBox=\"0 0 {w} {h}\">"
    );
    let _ = writeln!(
        out,
        "<rect class=\"paper\" width=\"{w}\" height=\"{h}\" fill=\"{}\"/>",
        doc.paper.hex()
    );
    let _ = writeln!(
        out,
        "<clipPath id=\"{CLIP_ID}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/></clipPath>",
        num(doc.clip.x),
        num(doc.clip.y),
        num(doc.clip.w),
        num(doc.clip.h)
    );

    let _ = writeln!(out, "<g clip-path=\"url(#{CLIP_ID})\">");
    for prim in &doc.plan {
        out.push_str(&element(prim));
    }
    out.push_str("</g>\n");

    for prim in &doc.chrome {
        out.push_str(&element(prim));
    }
    out.push_str("</svg>\n");

    out
}

fn pts(points: &[Pt]) -> String {
    let parts: Vec<String> = points
        .iter()
        .map(|(x, y)| format!("{},{}", num(*x), num(*y)))
        .collect();

    parts.join(" ")
}

fn paint(fill: Option<Fill>, stroke: Option<Stroke>) -> String {
    let mut out = String::new();

    match fill {
        Some(f) if f.opacity < 1.0 => {
            let _ = write!(
                out,
                " fill=\"{}\" fill-opacity=\"{}\"",
                f.rgb.hex(),
                num(f.opacity)
            );
        }
        Some(f) => {
            let _ = write!(out, " fill=\"{}\"", f.rgb.hex());
        }
        None => out.push_str(" fill=\"none\""),
    }

    let Some(s) = stroke else {
        return out;
    };
    let _ = write!(
        out,
        " stroke=\"{}\" stroke-width=\"{}\"",
        s.rgb.hex(),
        num(s.width_pt)
    );

    let dash = dash_array(s.dash, s.width_pt);
    if !dash.is_empty() {
        let parts: Vec<String> = dash.iter().map(|d| num(*d)).collect();
        let _ = write!(out, " stroke-dasharray=\"{}\"", parts.join(" "));
    }
    out.push_str(" stroke-linejoin=\"round\"");

    out
}

fn element(prim: &Prim) -> String {
    match prim {
        Prim::Shape {
            class,
            geom,
            fill,
            stroke,
        } => {
            let look = paint(*fill, *stroke);
            match geom {
                Geom::Open(p) => {
                    format!("<polyline class=\"{class}\" points=\"{}\"{look}/>\n", pts(p))
                }
                Geom::Closed(p) => {
                    format!("<polygon class=\"{class}\" points=\"{}\"{look}/>\n", pts(p))
                }
                Geom::Circle { center, radius } => format!(
                    "<circle class=\"{class}\" cx=\"{}\" cy=\"{}\" r=\"{}\"{look}/>\n",
                    num(center.0),
                    num(center.1),
                    num(*radius)
                ),
                Geom::Rect(Rect { x, y, w, h }) => format!(
                    "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{look}/>\n",
                    num(*x),
                    num(*y),
                    num(*w),
                    num(*h)
                ),
            }
        }
        Prim::Image { .. } => image(prim),
        Prim::Text {
            class,
            at,
            anchor,
            size_pt,
            weight,
            rgb,
            text,
        } => {
            let bold = match weight {
                Weight::Bold => " font-weight=\"bold\"",
                Weight::Regular => "",
            };
            let anchor = match anchor {
                Anchor::Middle => " text-anchor=\"middle\"",
                Anchor::Start => "",
            };
            format!(
                "<text class=\"{class}\" x=\"{}\" y=\"{}\" font-family=\"{FONT_FAMILY}\" font-size=\"{}\"{bold}{anchor} fill=\"{}\">{}</text>\n",
                num(at.0),
                num(at.1),
                num(*size_pt),
                rgb.hex(),
                escape(text)
            )
        }
    }
}

/// Embedded as a `data:` URI, so the file can be sent or opened anywhere: a relative path
/// would point into the project folder and break once the SVG is moved out of it.
fn image(prim: &Prim) -> String {
    let Prim::Image {
        class,
        at,
        width_pt,
        height_pt,
        rotation_deg,
        opacity,
        picture,
    } = prim
    else {
        return String::new();
    };

    // SVG turns clockwise for a positive angle too (y points down).
    let turn = match *rotation_deg == 0.0 {
        true => String::new(),
        false => format!(
            " transform=\"rotate({} {} {})\"",
            num(*rotation_deg),
            num(at.0),
            num(at.1)
        ),
    };

    format!(
        "<image class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" opacity=\"{}\"{turn} href=\"data:{};base64,{}\"/>\n",
        num(at.0),
        num(at.1),
        num(*width_pt),
        num(*height_pt),
        num(*opacity),
        picture.mime(),
        base64(&picture.bytes)
    )
}

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }

    out
}
