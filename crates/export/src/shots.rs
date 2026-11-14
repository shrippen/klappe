//! The shot list as cells, and as CSV (LH-82, RFC 4180).
//!
//! ```text
//! FloorPlan.shots (cut order) + cameras ─► cells ─┬─► csv
//!                                                 └─► table (the PDF sheets)
//! ```

use klappe_model::{Angle, FloorPlan, Movement, Shot, ShotSize};

use crate::canvas::num;
use crate::page::{ShotColumns, ShotTexts};

pub(crate) const COLUMNS: usize = 14;

const SEPARATOR: char = ',';
const QUOTE: char = '"';
const LINE_END: &str = "\r\n";
const POINT: char = '.';

/// Column heads, in table order.
pub(crate) fn heads(columns: &ShotColumns) -> [&str; COLUMNS] {
    [
        &columns.number,
        &columns.camera,
        &columns.lens_mm,
        &columns.size,
        &columns.angle,
        &columns.movement,
        &columns.from_s,
        &columns.to_s,
        &columns.duration_s,
        &columns.description,
        &columns.props,
        &columns.gear,
        &columns.crew,
        &columns.note,
    ]
}

/// One shot's cells; numbers use `decimal` between whole and fraction.
pub(crate) fn cells(
    shot: &Shot,
    plan: &FloorPlan,
    texts: &ShotTexts,
    decimal: char,
) -> [String; COLUMNS] {
    let camera = plan.camera(&shot.camera);
    let lens_mm = shot.lens_mm.or(camera.map(|c| c.lens_mm));
    let number = |v: f64| num(v).replace(POINT, &decimal.to_string());

    [
        shot.number.clone(),
        camera.map_or_else(|| shot.camera.clone(), |c| c.label.clone()),
        lens_mm.map(number).unwrap_or_default(),
        size_name(shot.size, texts),
        angle_name(shot.angle, texts),
        movement_name(shot.movement, texts),
        number(shot.from_s),
        number(shot.to_s),
        number(shot.duration_s()),
        shot.description.clone(),
        shot.props.clone(),
        shot.gear.clone(),
        shot.crew.clone(),
        shot.note.clone(),
    ]
}

fn size_name(size: ShotSize, texts: &ShotTexts) -> String {
    texts
        .sizes
        .iter()
        .find(|(s, _)| *s == size)
        .map_or_else(|| size.name().to_owned(), |(_, name)| name.clone())
}

fn angle_name(angle: Angle, texts: &ShotTexts) -> String {
    texts
        .angles
        .iter()
        .find(|(a, _)| *a == angle)
        .map_or_else(|| angle.name().to_owned(), |(_, name)| name.clone())
}

fn movement_name(movement: Movement, texts: &ShotTexts) -> String {
    texts
        .movements
        .iter()
        .find(|(m, _)| *m == movement)
        .map_or_else(|| movement.name().to_owned(), |(_, name)| name.clone())
}

/// The plan's shots as CSV: head row, then one row per shot in cut order. Numbers use a
/// point, whatever the language.
pub(crate) fn csv(plan: &FloorPlan, texts: &ShotTexts) -> String {
    let mut out = row(&heads(&texts.columns));
    for shot in &plan.shots {
        out.push_str(&row(&cells(shot, plan, texts, POINT)));
    }

    out
}

fn row<S: AsRef<str>>(fields: &[S]) -> String {
    let fields: Vec<String> = fields.iter().map(|f| field(f.as_ref())).collect();

    fields.join(&SEPARATOR.to_string()) + LINE_END
}

/// Quoted when it holds a separator, a quote or a line break; quotes are doubled.
fn field(text: &str) -> String {
    let plain = !text.contains([SEPARATOR, QUOTE, '\r', '\n']);
    if plain {
        return text.to_owned();
    }

    let doubled = text.replace(QUOTE, &format!("{QUOTE}{QUOTE}"));

    format!("{QUOTE}{doubled}{QUOTE}")
}
