//! Widths of Helvetica text, so the shot table can wrap where the printer would.

use crate::canvas::Weight;

/// First char of `ADVANCES`: space.
const FIRST_CHAR: u32 = 0x20;
/// Advance in 1/1000 em of Helvetica, space (0x20) to tilde (0x7E).
const ADVANCES: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278,
    278, // space - /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0 - 9
    278, 278, 584, 584, 584, 556, 1015, // : - @
    667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667,
    611, 722, 667, 944, 667, 667, 611, // A - Z
    278, 278, 278, 469, 556, 333, // [ - `
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500,
    278, 556, 500, 722, 500, 500, 500, // a - z
    334, 260, 334, 584, // { - ~
];
/// Accented letters and symbols: wider than most, so a line never runs over.
const OTHER_ADVANCE: u16 = 700;
/// Helvetica-Bold is a little wider than Helvetica.
const BOLD_FACTOR: f64 = 1.08;
const EM_UNITS: f64 = 1000.0;

pub(crate) fn text_width_pt(text: &str, size_pt: f64, weight: Weight) -> f64 {
    let units: f64 = text.chars().map(|c| f64::from(advance(c))).sum();
    let width = units / EM_UNITS * size_pt;

    match weight {
        Weight::Regular => width,
        Weight::Bold => width * BOLD_FACTOR,
    }
}

fn advance(c: char) -> u16 {
    let index = u32::from(c).checked_sub(FIRST_CHAR).map(|i| i as usize);

    index
        .and_then(|i| ADVANCES.get(i))
        .copied()
        .unwrap_or(OTHER_ADVANCE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_has_the_right_length() {
        // '~' is 0x7E, the last entry.
        assert_eq!(ADVANCES.len(), 0x7E - 0x20 + 1);
        assert_eq!(advance('~'), 584);
        assert_eq!(advance('A'), 667);
        assert_eq!(advance('a'), 556);
        assert_eq!(advance('z'), 500);
        assert_eq!(advance('0'), 556);
        assert_eq!(advance('ö'), OTHER_ADVANCE);
    }
}
