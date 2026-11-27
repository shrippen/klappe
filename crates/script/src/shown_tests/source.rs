//! The source view (D-36): the editor hides nothing, the text is the source, and what the
//! user types is what is written.

use super::*;

/// Where the typed `x` goes in every line, every position.
const TYPED: &str = "x";

/// A book of every sort of line, as the editor would not write them: blanks, trailing blanks, a
/// no-break space, marks that touch, a lone mark.
const ODD: &str = "  INT. KÜCHE - TAG #1#  \n\n*a**b*** und __ _ \\\n[[n]] ANNA ^\n\
(V.O.)\n> mitte <\n.\n@\n~\n=\n#\nÄ🎬Ö **";

fn books() -> [&'static str; 3] {
    [EXAMPLE, CANON, ODD]
}

#[test]
fn the_source_view_shows_every_line_as_it_is() {
    for book in books() {
        let lines: Vec<&str> = book.split('\n').collect();

        for (at, shown) in shown_lines(book, &[]).iter().enumerate() {
            let mut cols: Vec<usize> = lines[at].char_indices().map(|(i, _)| i).collect();
            cols.push(lines[at].len());

            assert_eq!(shown.text, lines[at], "line {at}");
            assert_eq!(shown.cols, cols, "line {at}");
            assert!(
                shown.runs.iter().all(|r| r.marks == Marks::default()),
                "line {at}"
            );
            assert_eq!(shown.hidden, Hidden::No, "line {at}");
        }
    }
}

#[test]
fn a_typed_character_in_the_source_view_is_written_as_typed() {
    for book in books() {
        for (at, line) in book.split('\n').enumerate() {
            let shown = shown_lines(book, &[]).remove(at);

            for (pos, (byte, _)) in line.char_indices().chain([(line.len(), ' ')]).enumerate() {
                let change = LineEdit {
                    removed: pos..pos,
                    typed: TYPED.to_owned(),
                    runs: Vec::new(),
                };
                let new = edit_line(line, shown.kind, shown.shape, &[], &change);
                let want = format!("{}{TYPED}{}", &line[..byte], &line[byte..]);

                assert_eq!(new, want, "line {at} {line:?} at {pos}");
            }
        }
    }
}
