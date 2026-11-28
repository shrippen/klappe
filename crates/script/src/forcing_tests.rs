//! Typing by hand while the forcing characters are hidden (D-36): the text typed one character
//! at a time ends up as the text pasted at once, and nothing typed is lost.

use crate::{edit_shown, shown_lines, Change, Hide, Spot};

const EDITOR: [Hide; 2] = [Hide::Emphasis, Hide::Forcing];

/// What the lines above the typed one are.
const CONTEXTS: [&str; 3] = ["", "Er geht.\n\n", "ANNA\nHallo.\n\n"];

fn change_at(line: usize, col: usize, typed: &str) -> Change {
    let spot = Spot { line, col };

    Change {
        from: spot,
        to: spot,
        typed: typed.to_owned(),
        runs: Vec::new(),
    }
}

/// `typed` put one character at a time at the end of the last line of `above`, as the editor
/// hands it over; the source and the shown text of that line.
fn by_char(above: &str, typed: &str) -> (String, String) {
    let mut source = above.to_owned();

    for ch in typed.chars() {
        let line = source.split('\n').count() - 1;
        let shown = shown_lines(&source, &EDITOR).swap_remove(line);
        let col = shown.text.encode_utf16().count();
        source = edit_shown(&source, &EDITOR, &change_at(line, col, &ch.to_string())).source;
    }

    let shown = shown_lines(&source, &EDITOR).pop().map(|s| s.text);
    (source, shown.unwrap_or_default())
}

/// `typed` put at once at the end of the last line of `above`.
fn at_once(above: &str, typed: &str) -> (String, String) {
    let line = above.split('\n').count() - 1;
    let source = edit_shown(above, &EDITOR, &change_at(line, 0, typed)).source;
    let shown = shown_lines(&source, &EDITOR).pop().map(|s| s.text);

    (source, shown.unwrap_or_default())
}

/// What a writer types by hand at the start of a line, with the shown text it ends in.
const TYPED: [(&str, &str); 18] = [
    ("...und dann", "...und dann"),
    ("!", "!"),
    ("> ", "> "),
    ("~", "~"),
    (".5 Liter", "5 Liter"),
    ("@x", "x"),
    ("@", "@"),
    (".", "."),
    ("x > y", "x > y"),
    ("Er sagt ~la", "Er sagt ~la"),
    (">ENDE<", "ENDE"),
    ("~la", "la"),
    ("@mcClane", "mcClane"),
    ("BOB ^", "BOB ^"),
    ("> ENDE <", " ENDE"),
    ("> SCHNITT:", " SCHNITT:"),
    ("~ la", " la"),
    ("@ BOB", " BOB"),
];

#[test]
fn typing_by_hand_is_pasting() {
    for above in CONTEXTS {
        for (typed, shown) in TYPED {
            let by_char = by_char(above, typed);

            assert_eq!(by_char, at_once(above, typed), "{above:?} {typed:?}");
            assert_eq!(by_char.1, shown, "{above:?} {typed:?}");
        }
    }
}

#[test]
fn a_typed_forcing_character_stays_in_the_source() {
    for (typed, _) in TYPED {
        let (source, _) = by_char("", typed);

        assert_eq!(source, typed);
    }
}

#[test]
fn a_forcing_character_alone_shows() {
    for mark in ["!", "@", ">", "~", "."] {
        let shown = shown_lines(mark, &EDITOR).swap_remove(0);

        assert_eq!(shown.text, mark);
    }
}
