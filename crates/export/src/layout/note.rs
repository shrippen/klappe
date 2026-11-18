//! A note on paper (LH-16): its card, then its lines of text from the card's top left corner.
//! Stellprobe wrapped the text, one line per `\n`; the lines are the same on screen.

use super::{item_prims, Map, BASELINE_SHIFT};
use crate::canvas::{Anchor, Prim, Weight};
use crate::error::ExportError;
use crate::palette::Palette;
use klappe_stellprobe::{Item, Shape, NOTE_TEXT_M};

/// Line pitch as a multiple of the font size, as on screen.
const LINE_SPACING: f64 = 1.25;
/// Margin between card and text, as a multiple of the font size, as on screen.
const TEXT_GAP_EM: f64 = 0.3;

/// The prims of the note `item`: its card, then its lines inside the card.
pub(super) fn prims(
    item: &Item,
    map: &Map,
    pal: &Palette,
    body_m: f64,
    out: &mut Vec<Prim>,
) -> Result<(), ExportError> {
    let Shape::Polygon { points } = &item.shape else {
        return item_prims(item, map, pal, body_m, out);
    };

    // The card alone: `item_prims` would put the whole label beside it.
    let card = Item {
        label: None,
        ..item.clone()
    };
    item_prims(&card, map, pal, body_m, out)?;

    let Some(corner) = points.first().map(|p| map.pt(*p)) else {
        return Ok(());
    };
    let text = item.label.as_deref().unwrap_or_default();

    // A turned card carries its text: each line starts where the turn puts it and turns by the
    // card's angle (the bearing of its top edge).
    let turn_deg = match points.get(1) {
        Some(second) => points[0].bearing_deg(*second),
        None => 0.0,
    };

    // The text is sized in metres like the card, so it fits at every scale.
    let size_pt = NOTE_TEXT_M * map.scale;
    let gap_pt = TEXT_GAP_EM * size_pt;

    for (i, line) in text.lines().enumerate() {
        // The middle of the line, as on screen, then down to its baseline.
        let middle = corner.1 + gap_pt + size_pt * (0.5 + LINE_SPACING * i as f64);
        let start = (corner.0 + gap_pt, middle + size_pt * BASELINE_SHIFT);
        out.push(Prim::Text {
            rotation_deg: turn_deg,
            class: "label".into(),
            anchor: Anchor::Start,
            at: turned(start, corner, turn_deg),
            size_pt,
            weight: Weight::Regular,
            rgb: pal.foreground,
            text: line.to_owned(),
        });
    }

    Ok(())
}

/// `p` turned clockwise on the page by `deg` about `centre`.
fn turned(p: (f64, f64), centre: (f64, f64), deg: f64) -> (f64, f64) {
    let (sin, cos) = deg.to_radians().sin_cos();
    let (x, y) = (p.0 - centre.0, p.1 - centre.1);

    (centre.0 + x * cos - y * sin, centre.1 + x * sin + y * cos)
}
