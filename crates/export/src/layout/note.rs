//! A note on paper (LH-16): its card, then its lines of text from the card's top left corner.
//! Stellprobe wrapped the text, one line per `\n`; the lines are the same on screen.

use super::{item_prims, Map, LABEL_GAP_PT, LABEL_PT};
use crate::canvas::{Anchor, Prim, Weight};
use crate::error::ExportError;
use crate::palette::Palette;
use klappe_stellprobe::{Item, Role, Shape};

/// Line pitch as a multiple of the font size, as on screen.
const LINE_SPACING: f64 = 1.25;

/// The prims of `item`; a note also gets its lines inside the card, others as `item_prims`.
pub(super) fn prims(
    item: &Item,
    map: &Map,
    pal: &Palette,
    body_m: f64,
    out: &mut Vec<Prim>,
) -> Result<(), ExportError> {
    let (Role::Note, Shape::Polygon { points }) = (item.role, &item.shape) else {
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

    for (i, line) in text.lines().enumerate() {
        out.push(Prim::Text {
            class: "label".into(),
            anchor: Anchor::Start,
            at: (
                corner.0 + LABEL_GAP_PT,
                corner.1 + LABEL_GAP_PT + LABEL_PT * (1.0 + LINE_SPACING * i as f64),
            ),
            size_pt: LABEL_PT,
            weight: Weight::Regular,
            rgb: pal.foreground,
            text: line.to_owned(),
        });
    }

    Ok(())
}
