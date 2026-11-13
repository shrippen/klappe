//! What both writers draw: shapes and text in page points, origin top left, y down.
//! One list, two serialisers, so SVG and PDF cannot drift apart.

use crate::palette::Rgb;

pub(crate) type Pt = (f64, f64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Geom {
    Open(Vec<Pt>),
    Closed(Vec<Pt>),
    Circle { center: Pt, radius: f64 },
    Rect(Rect),
}

/// Information is not only colour: the dash tells paths and the axis apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dash {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Fill {
    pub rgb: Rgb,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Stroke {
    pub rgb: Rgb,
    pub width_pt: f64,
    pub dash: Dash,
}

/// Where `at.0` sits on the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Anchor {
    Start,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Weight {
    Regular,
    Bold,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Prim {
    Shape {
        class: String,
        geom: Geom,
        fill: Option<Fill>,
        stroke: Option<Stroke>,
    },
    Text {
        class: String,
        at: Pt,
        anchor: Anchor,
        size_pt: f64,
        weight: Weight,
        rgb: Rgb,
        text: String,
    },
}

#[derive(Debug)]
pub(crate) struct Doc {
    pub width_pt: f64,
    pub height_pt: f64,
    pub paper: Rgb,
    /// The plan area; `plan` is clipped to it (the axis runs past the set).
    pub clip: Rect,
    pub plan: Vec<Prim>,
    /// Title, scale bar, legend.
    pub chrome: Vec<Prim>,
}

/// Dash pattern in points for a stroke of `width_pt`; empty for solid.
pub(crate) fn dash_array(dash: Dash, width_pt: f64) -> Vec<f64> {
    match dash {
        Dash::Solid => Vec::new(),
        Dash::Dashed => vec![6.0 * width_pt.max(1.0), 3.0 * width_pt.max(1.0)],
        Dash::Dotted => vec![width_pt.max(1.0), 2.5 * width_pt.max(1.0)],
    }
}

/// A number as short as possible: two decimals, no trailing zeros.
pub(crate) fn num(v: f64) -> String {
    if !v.is_finite() {
        return "0".to_string();
    }

    let text = format!("{v:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    match text {
        "" | "-0" => "0".to_string(),
        t => t.to_string(),
    }
}
