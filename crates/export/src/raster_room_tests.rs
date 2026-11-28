//! A prim flagged for the room is cut at the room, not at the wider plan area (Q1-07).
#![allow(clippy::unwrap_used)]

use crate::canvas::{Doc, Fill, Geom, Prim, Rect};
use crate::palette::Rgb;
use crate::raster::render;

const PAPER: Rgb = Rgb {
    r: 250,
    g: 250,
    b: 245,
};
const INK: Rgb = Rgb {
    r: 10,
    g: 20,
    b: 30,
};
/// A 96 x 48 pt page at 150 dpi is 200 x 100 pixels.
const PAGE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 96.0,
    h: 48.0,
};
const ROOM: Rect = Rect {
    x: 24.0,
    y: 12.0,
    w: 24.0,
    h: 12.0,
};

#[test]
fn a_room_prim_is_cut_at_the_room() {
    let prim = Prim::Shape {
        class: "t".into(),
        geom: Geom::Rect(PAGE),
        fill: Some(Fill {
            rgb: INK,
            opacity: 1.0,
        }),
        stroke: None,
    };
    let doc = Doc {
        width_pt: PAGE.w,
        height_pt: PAGE.h,
        paper: PAPER,
        clip: PAGE,
        room: ROOM,
        plan: vec![prim],
        at_room: vec![true],
        chrome: Vec::new(),
        pages: Vec::new(),
        sheets: Vec::new(),
    };
    let s = render(&doc).unwrap();
    let at = |x: u32, y: u32| {
        let i = (y * s.width + x) as usize * 3;

        [s.pixels[i], s.pixels[i + 1], s.pixels[i + 2]]
    };

    assert_eq!(at(75, 37), [INK.r, INK.g, INK.b]);
    assert_eq!(at(30, 10), [PAPER.r, PAPER.g, PAPER.b]);
    assert_eq!(at(150, 80), [PAPER.r, PAPER.g, PAPER.b]);
}
