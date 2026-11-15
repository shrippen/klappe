//! The shot list as cells, and as CSV (LH-82, RFC 4180).
//!
//! ```text
//! FloorPlan.shots (cut order) + cameras ─► cells ─┬─► csv
//!                                                 └─► table (the PDF sheets)
//! ```

use klappe_model::{Angle, FloorPlan, Movement, Shot, ShotSize};
use klappe_stellprobe::{ordered_shots, shot_lens, ShotLens, ShotOrder};

use crate::canvas::num;
use crate::page::{ShotColumns, ShotTexts};

pub(crate) const COLUMNS: usize = 14;

const SEPARATOR: char = ',';
const QUOTE: char = '"';
const LINE_END: &str = "\r\n";
const POINT: char = '.';
const SPAN: char = '\u{2013}';

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
    let number = |v: f64| num(v).replace(POINT, &decimal.to_string());
    let lens = camera.map(|c| shot_lens(c, shot));

    [
        shot.number.clone(),
        camera.map_or_else(|| shot.camera.clone(), |c| c.label.clone()),
        lens_text(lens, shot, &number),
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

/// The lens column: one value, or "35–85" for a zoom. A camera that is gone leaves the shot's
/// own lens, if any.
fn lens_text(lens: Option<ShotLens>, shot: &Shot, number: &dyn Fn(f64) -> String) -> String {
    let Some(lens) = lens else {
        return shot.lens_mm.map(number).unwrap_or_default();
    };
    let Some((min_mm, max_mm)) = lens.span_mm() else {
        return number(lens.from_mm);
    };

    format!("{}{SPAN}{}", number(min_mm), number(max_mm))
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

/// The shot list as text: head row and one row per shot. In shooting order a first column
/// holds the setup number. Numbers use a point, whatever the language.
pub(crate) struct Table {
    pub(crate) head: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
    /// Per column, so a spreadsheet stores numbers as such.
    pub(crate) kinds: Vec<Kind>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Numbers,
    Text,
}

/// Columns of `COLUMNS` that hold numbers: lens, from, to, duration.
const NUMERIC_COLUMNS: [usize; 4] = [2, 6, 7, 8];

pub(crate) fn table(plan: &FloorPlan, texts: &ShotTexts, order: ShotOrder) -> Table {
    let with_setup = order == ShotOrder::Shooting;
    let mut head: Vec<String> = Vec::new();
    let mut kinds: Vec<Kind> = Vec::new();
    if with_setup {
        head.push(texts.columns.setup.clone());
        kinds.push(Kind::Numbers);
    }
    for (i, name) in heads(&texts.columns).iter().enumerate() {
        head.push((*name).to_owned());
        kinds.push(if NUMERIC_COLUMNS.contains(&i) {
            Kind::Numbers
        } else {
            Kind::Text
        });
    }

    let mut rows = Vec::new();
    for (setup, shot) in ordered_shots(plan, order) {
        let mut cells = cells(shot, plan, texts, POINT).to_vec();
        if with_setup {
            cells.insert(0, setup.to_string());
        }
        rows.push(cells);
    }

    Table { head, rows, kinds }
}

/// The plan's shots as CSV: head row, then one row per shot in `order`.
pub(crate) fn csv(plan: &FloorPlan, texts: &ShotTexts, order: ShotOrder) -> String {
    let table = table(plan, texts, order);

    let mut out = row(&table.head);
    for cells in &table.rows {
        out.push_str(&row(cells));
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
