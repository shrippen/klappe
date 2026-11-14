//! Placeholder until the glyph outlines are merged in.

use crate::canvas::{Pt, Weight};

pub(crate) fn text_outline(_text: &str, _weight: Weight, _size_pt: f64) -> Vec<Vec<Pt>> {
    Vec::new()
}

pub(crate) fn text_width_pt(_text: &str, _weight: Weight, _size_pt: f64) -> f64 {
    0.0
}
