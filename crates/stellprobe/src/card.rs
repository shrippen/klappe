//! The card a note is drawn as (LH-16): a rectangle with its text wrapped, decided once; the
//! GUI and the export draw the lines as they are.

use klappe_model::Point;

/// The card's width.
const CARD_WIDTH_M: f64 = 2.4;
/// Inner margin on every side.
const CARD_PAD_M: f64 = 0.1;
/// Width of one character, estimated: the same lines on screen and on paper.
const CHAR_WIDTH_M: f64 = 0.1;
/// Characters in one line of text, rounded: 21.999… must not give 21.
const CHARS_PER_LINE: usize = ((CARD_WIDTH_M - 2.0 * CARD_PAD_M) / CHAR_WIDTH_M + 0.5) as usize;
/// Height of one line of text.
const LINE_HEIGHT_M: f64 = 0.2;

/// The corners of the card with top left `at`, and its text, lines joined by `\n`.
pub(crate) fn note_card(at: Point, text: &str) -> (Vec<Point>, String) {
    let lines = wrap(text, CHARS_PER_LINE);
    let height_m = lines.len() as f64 * LINE_HEIGHT_M + 2.0 * CARD_PAD_M;
    let corners = vec![
        at,
        Point::new(at.x_m + CARD_WIDTH_M, at.y_m),
        Point::new(at.x_m + CARD_WIDTH_M, at.y_m + height_m),
        Point::new(at.x_m, at.y_m + height_m),
    ];

    (corners, lines.join("\n"))
}

/// Greedy wrap at `per_line` characters; a longer word is cut. Always at least one line.
fn wrap(text: &str, per_line: usize) -> Vec<String> {
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
        let (_, text) = note_card(Point::new(0.0, 0.0), &"x".repeat(50));
        let lengths: Vec<usize> = text.lines().map(|l| l.chars().count()).collect();

        assert_eq!(lengths, [22, 22, 6]);
    }

    #[test]
    fn the_card_grows_with_its_lines() {
        let at = Point::new(1.0, 2.0);
        let (one, _) = note_card(at, "kurz");
        let (two, text) = note_card(at, &"wort ".repeat(10));

        assert_eq!(one[0], at);
        assert_eq!(one[1].x_m, 1.0 + CARD_WIDTH_M);
        assert!(two[2].y_m > one[2].y_m);
        assert!(text.contains('\n'));
    }

    #[test]
    fn a_click_anywhere_on_the_card_hits_it() {
        let at = Point::new(3.0, 4.0);
        let (points, label) = note_card(at, "Der Vorhang wird erst nach dem dritten Gong zu");
        let card = Item {
            role: Role::Note,
            shape: Shape::Polygon { points },
            label: Some(label),
            source: None,
        };

        assert!(hit(&card, Point::new(3.5, 4.3), 0.0));
        assert!(hit(&card, Point::new(5.3, 4.5), 0.0));
        assert!(!hit(&card, Point::new(2.0, 4.3), 0.0));
    }
}
