//! What ODS and XLSX share: telling numbers from text, and XML escaping.

use crate::shots::{Kind, Table};

/// One worksheet: its name and what it holds.
pub(crate) struct Sheet<'a> {
    pub(crate) name: &'a str,
    pub(crate) table: Table,
}

/// A cell of the table; numbers keep their text, which is already a valid XML number.
pub(crate) enum Cell<'a> {
    Text(&'a str),
    Number(&'a str),
}

/// Rows with the head first; empty cells are left out (`None`).
pub(crate) fn grid(table: &Table) -> Vec<Vec<Option<Cell<'_>>>> {
    let head = table.head.iter().map(|h| Some(Cell::Text(h))).collect();
    let mut grid = vec![head];

    for row in &table.rows {
        grid.push(
            row.iter()
                .enumerate()
                .map(|(col, text)| cell(text, table.kinds[col]))
                .collect(),
        );
    }

    grid
}

fn cell(text: &str, kind: Kind) -> Option<Cell<'_>> {
    if text.is_empty() {
        return None;
    }
    let is_number = kind == Kind::Numbers && text.parse::<f64>().is_ok_and(f64::is_finite);
    if is_number {
        return Some(Cell::Number(text));
    }

    Some(Cell::Text(text))
}

/// Text for element content or an attribute value: the five XML escapes, and control
/// characters XML 1.0 forbids dropped.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }

    out
}
