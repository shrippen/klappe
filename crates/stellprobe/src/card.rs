//! The card a note is drawn as (LH-16): a rectangle with its text wrapped, decided once; the
//! GUI and the export draw the lines as they are.

use klappe_model::{Point, NOTE_MIN_WIDTH_M};

/// Inner margin on every side.
const CARD_PAD_M: f64 = 0.1;
/// Width of one character, estimated: the same lines on screen and on paper.
const CHAR_WIDTH_M: f64 = 0.1;
/// Height of one line of text.
const LINE_HEIGHT_M: f64 = 0.2;
/// Line pitch as a multiple of the font size: the canvas and the export both use it.
const LINE_SPACING: f64 = 1.25;
/// Font size of a note's text, in metres: the text scales with the plan, like its card.
pub const NOTE_TEXT_M: f64 = LINE_HEIGHT_M / LINE_SPACING;

/// Characters in one line of a card `width_m` wide, rounded: 21.999… must not give 21. At
/// least one, so a narrow card still takes text.
fn chars_per_line(width_m: f64) -> usize {
    (((width_m - 2.0 * CARD_PAD_M) / CHAR_WIDTH_M + 0.5) as usize).max(1)
}

/// The corners of the card with top left `at` and width `width_m` (not below the minimum), turned
/// clockwise by `rotation_deg` about `at`, and its text, lines joined by `\n`.
pub(crate) fn note_card(
    at: Point,
    width_m: f64,
    rotation_deg: f64,
    text: &str,
) -> (Vec<Point>, String) {
    let width_m = width_m.max(NOTE_MIN_WIDTH_M);
    let lines = wrap(text, chars_per_line(width_m));
    let height_m = lines.len() as f64 * LINE_HEIGHT_M + 2.0 * CARD_PAD_M;
    let corners = vec![
        at,
        Point::new(at.x_m + width_m, at.y_m).turned(at, rotation_deg),
        Point::new(at.x_m + width_m, at.y_m + height_m).turned(at, rotation_deg),
        Point::new(at.x_m, at.y_m + height_m).turned(at, rotation_deg),
    ];

    (corners, lines.join("\n"))
}

/// Wrap at `per_line` characters, keeping the text's own line breaks.
fn wrap(text: &str, per_line: usize) -> Vec<String> {
    text.split('\n')
        .flat_map(|paragraph| wrap_paragraph(paragraph, per_line))
        .collect()
}

/// Greedy wrap at `per_line` characters; a longer word is cut. Always at least one line.
fn wrap_paragraph(text: &str, per_line: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();

        // A word that never fits a line is cut into pieces of a line.
        while word.len() > per_line {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            lines.push(word.drain(..per_line).collect());
        }

        let fits = current.chars().count() + 1 + word.len() <= per_line;
        if !current.is_empty() && !fits {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.extend(word);
    }

    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{Item, Role, Shape};
    use crate::hit::hit;
    use klappe_model::NOTE_WIDTH_M;

    #[test]
    fn short_text_is_one_line() {
        assert_eq!(wrap("Vorhang zu", 22), ["Vorhang zu"]);
        assert_eq!(wrap("", 22), [""]);
    }

    #[test]
    fn long_text_wraps_at_words() {
        assert_eq!(wrap("aaa bbb ccc", 7), ["aaa bbb", "ccc"]);
    }

    #[test]
    fn a_long_word_is_cut() {
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    /// 2.4 m wide, 0.1 m margins, 0.1 m a character: 22 characters, not the 21 that
    /// truncating 21.999… gave.
    #[test]
    fn a_line_has_the_characters_the_card_holds() {
        let (_, text) = note_card(Point::new(0.0, 0.0), NOTE_WIDTH_M, 0.0, &"x".repeat(50));
        let lengths: Vec<usize> = text.lines().map(|l| l.chars().count()).collect();

        assert_eq!(lengths, [22, 22, 6]);
    }

    #[test]
    fn a_wider_card_wraps_into_fewer_lines() {
        let at = Point::new(0.0, 0.0);
        let text = "wort ".repeat(20);
        let lines = |width_m| note_card(at, width_m, 0.0, &text).1.lines().count();

        assert!(lines(4.8) < lines(NOTE_WIDTH_M));
        assert!(lines(NOTE_WIDTH_M) < lines(1.2));
    }

    #[test]
    fn a_card_is_not_narrower_than_the_minimum() {
        let at = Point::new(1.0, 1.0);
        let (corners, text) = note_card(at, 0.0, 0.0, "abc def");

        assert_eq!(corners[1].x_m, 1.0 + NOTE_MIN_WIDTH_M);
        assert_eq!(text, note_card(at, NOTE_MIN_WIDTH_M, 0.0, "abc def").1);
    }

    #[test]
    fn an_own_line_break_stays() {
        let (_, text) = note_card(
            Point::new(0.0, 0.0),
            NOTE_WIDTH_M,
            0.0,
            "Akt 1\nVorhang\n\nEnde",
        );

        assert_eq!(text, "Akt 1\nVorhang\n\nEnde");
    }

    #[test]
    fn the_card_grows_with_its_lines() {
        let at = Point::new(1.0, 2.0);
        let (one, _) = note_card(at, NOTE_WIDTH_M, 0.0, "kurz");
        let (two, text) = note_card(at, NOTE_WIDTH_M, 0.0, &"wort ".repeat(10));

        assert_eq!(one[0], at);
        assert_eq!(one[1].x_m, 1.0 + NOTE_WIDTH_M);
        assert!(two[2].y_m > one[2].y_m);
        assert!(text.contains('\n'));
    }

    #[test]
    fn a_click_anywhere_on_the_card_hits_it() {
        let at = Point::new(3.0, 4.0);
        let (points, label) = note_card(
            at,
            NOTE_WIDTH_M,
            0.0,
            "Der Vorhang wird erst nach dem dritten Gong zu",
        );
        let card = Item {
            role: Role::Note,
            shape: Shape::Polygon { points },
            label: Some(label),
            source: None,
            opacity: crate::display::FULL_OPACITY,
        };

        assert!(hit(&card, Point::new(3.5, 4.3), 0.0));
        assert!(hit(&card, Point::new(5.3, 4.5), 0.0));
        assert!(!hit(&card, Point::new(2.0, 4.3), 0.0));
    }
}
