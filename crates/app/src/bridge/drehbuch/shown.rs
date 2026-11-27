//! The screenplay as the editor shows it (D-36), for `scriptShownJson` and `scriptEditJson`.
//! Positions are UTF-16 units, as the text area counts them.

use klappe_services::{
    edit_shown, replace_source, shown_lines, Hide, Marks, Run, ShownChange, ShownEdit, ShownLine,
    ShownSpot,
};
use serde_json::{json, Value};

/// What the editor leaves out of its text. Step by step it hides more (`Hide`).
const HIDE: [Hide; 1] = [Hide::Emphasis];

/// A shown line as `{ text, runs: [{ from, to, bold, italic, underline }], cols }`. `runs`
/// are the stretches that have emphasis. `cols` has per unit of the text the unit where it
/// stands in the source line, and the one where the text ends; null where each unit stands
/// at itself.
fn line_json(line: &str, shown: &ShownLine) -> Value {
    // Where each character of the text starts, in units.
    let mut starts = Vec::with_capacity(shown.text.len() + 1);
    let mut unit = 0;
    for ch in shown.text.chars() {
        starts.push(unit);
        unit += ch.len_utf16();
    }
    starts.push(unit);

    let runs: Vec<Value> = shown
        .runs
        .iter()
        .filter(|r| r.marks.bold || r.marks.italic || r.marks.underline)
        .map(|r| {
            json!({
                "from": starts[r.start],
                "to": starts[r.end],
                "bold": r.marks.bold,
                "italic": r.marks.italic,
                "underline": r.marks.underline,
            })
        })
        .collect();

    json!({ "text": shown.text, "runs": runs, "cols": cols_json(line, shown) })
}

fn cols_json(line: &str, shown: &ShownLine) -> Value {
    if line.is_ascii() && shown.cols.iter().enumerate().all(|(i, col)| *col == i) {
        return Value::Null;
    }

    // The unit of each byte where a character of the source line starts.
    let mut units = vec![0; line.len() + 1];
    let mut unit = 0;
    for (at, ch) in line.char_indices() {
        units[at] = unit;
        unit += ch.len_utf16();
    }
    units[line.len()] = unit;

    let mut cols = Vec::with_capacity(shown.cols.len());
    for (ch, at) in shown.text.chars().zip(&shown.cols) {
        cols.push(units[*at]);
        if ch.len_utf16() == 2 {
            cols.push(units[*at] + 1);
        }
    }
    cols.push(units[shown.cols.last().copied().unwrap_or(0)]);

    json!(cols)
}

/// The lines of `source` as the editor shows them.
pub(in crate::bridge) fn shown_json(source: &str) -> String {
    let lines = source
        .split('\n')
        .zip(shown_lines(source, &HIDE))
        .map(|(line, shown)| line_json(line, &shown))
        .collect();

    Value::Array(lines).to_string()
}

fn number(change: &Value, key: &str) -> usize {
    change[key]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or_default()
}

fn flag(run: &Value, key: &str) -> bool {
    run[key].as_bool().unwrap_or_default()
}

/// The change `{ line, from, endLine, to, typed, runs }`; `endLine` is `line` when missing.
fn read_change(change: &str) -> Option<ShownChange> {
    let change: Value = serde_json::from_str(change).ok()?;
    let line = number(&change, "line");
    let end = change["endLine"]
        .as_u64()
        .map_or(line, |_| number(&change, "endLine"));
    let runs = change["runs"]
        .as_array()
        .map(|runs| {
            runs.iter()
                .map(|run| Run {
                    start: number(run, "from"),
                    end: number(run, "to"),
                    marks: Marks {
                        bold: flag(run, "bold"),
                        italic: flag(run, "italic"),
                        underline: flag(run, "underline"),
                        note: false,
                        boneyard: false,
                    },
                })
                .collect()
        })
        .unwrap_or_default();

    Some(ShownChange {
        from: ShownSpot {
            line,
            col: number(&change, "from"),
        },
        to: ShownSpot {
            line: end,
            col: number(&change, "to"),
        },
        typed: change["typed"].as_str()?.to_owned(),
        runs,
    })
}

/// An edit as `{ source, from, to, shown }`: the old lines `from..to` now the lines `shown`.
fn edited_json(edited: &ShownEdit) -> String {
    let lines: Vec<&str> = edited.source.split('\n').collect();
    let shown: Vec<Value> = edited
        .shown
        .iter()
        .enumerate()
        .map(|(n, shown)| line_json(lines[edited.from + n], shown))
        .collect();

    json!({
        "source": edited.source,
        "from": edited.from,
        "to": edited.to,
        "shown": shown,
    })
    .to_string()
}

/// `source` after the change, as `edited_json`. Empty for a change that cannot be read.
pub(in crate::bridge) fn edit_json(source: &str, change: &str) -> String {
    let Some(change) = read_change(change) else {
        return String::new();
    };

    edited_json(&edit_shown(source, &HIDE, &change))
}

/// `source` with the text from line `line`, column `from` to line `end`, column `to`
/// replaced by `text`, as `edited_json`. The columns are UTF-16 units of the source lines.
pub(in crate::bridge) fn replace_json(
    source: &str,
    (line, from): (usize, usize),
    (end, to): (usize, usize),
    text: &str,
) -> String {
    let at = |line, col| ShownSpot { line, col };

    edited_json(&replace_source(
        source,
        &HIDE,
        at(line, from),
        at(end, to),
        text,
    ))
}
