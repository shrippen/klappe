//! The labels of the sun and north arrows (LH-52).

use super::{item_prims, Map, BASELINE_SHIFT, LABEL_PT};
use crate::canvas::{Anchor, Prim, Pt, Weight};
use crate::error::ExportError;
use crate::metrics::text_width_pt;
use crate::palette::Palette;
use klappe_stellprobe::{Item, Shape};

/// The label of a sun or north arrow, centred on the arrow as the app does, so it stays off the
/// other arrow of the pair, and moved sideways into the set's clip (not up or down: on a very
/// large set "N" and the sun's label can meet, as in the app zoomed out). Right of the
/// shape, like the other polygons, it ran over the neighbour or out of the clip. A label wider
/// than the set breaks at its slash into two lines ("180° / 61°" over a 3 m wide set).
fn label(label: &str, bounds: &[Pt], map: &Map, pal: &Palette, out: &mut Vec<Prim>) {
    let (left, right) = bounds
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, r), p| (l.min(p.0), r.max(p.0)));
    let (top, bottom) = bounds
        .iter()
        .fold((f64::MAX, f64::MIN), |(t, b), p| (t.min(p.1), b.max(p.1)));
    let centre = ((left + right) / 2.0, (top + bottom) / 2.0);
    let width = |t: &str| text_width_pt(t, LABEL_PT, Weight::Regular);
    let lines: Vec<String> = match width(label) > map.frame.w {
        true => label.split(" / ").map(str::to_owned).collect(),
        false => vec![label.to_owned()],
    };
    let first_y = centre.1 + LABEL_PT * BASELINE_SHIFT - (lines.len() - 1) as f64 * LABEL_PT / 2.0;

    for (row, text) in lines.into_iter().enumerate() {
        let half = width(&text) / 2.0;
        let (low, high) = (map.frame.x + half, map.frame.x + map.frame.w - half);
        let x = match low <= high {
            true => centre.0.clamp(low, high),
            false => map.frame.x + map.frame.w / 2.0,
        };

        out.push(Prim::Text {
            class: "label".into(),
            anchor: Anchor::Middle,
            at: (x, first_y + row as f64 * LABEL_PT),
            size_pt: LABEL_PT,
            weight: Weight::Regular,
            rgb: pal.foreground,
            text,
        });
    }
}

/// The prims of the arrow `item`: the arrow without a label, then the label on it.
pub(super) fn prims(
    item: &Item,
    map: &Map,
    pal: &Palette,
    body_m: f64,
    out: &mut Vec<Prim>,
) -> Result<(), ExportError> {
    let bare = Item {
        label: None,
        ..item.clone()
    };
    item_prims(&bare, map, pal, body_m, out)?;

    let (Some(text), Shape::Polygon { points }) = (&item.label, &item.shape) else {
        return Ok(());
    };
    let bounds: Vec<Pt> = points.iter().map(|p| map.pt(*p)).collect();
    label(text, &bounds, map, pal, out);

    Ok(())
}
