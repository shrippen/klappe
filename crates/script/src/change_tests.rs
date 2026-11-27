//! A change of the shown text across lines: joining, splitting, pasting, and what stays.

use crate::{edit_shown, replace_source, Change, Hide, Marks, Run, Spot};

const EMPHASIS: [Hide; 1] = [Hide::Emphasis];
const ALL: [Hide; 5] = [
    Hide::Emphasis,
    Hide::Forcing,
    Hide::Number,
    Hide::Notes,
    Hide::TitlePage,
];

const BOLD: Marks = Marks {
    bold: true,
    italic: false,
    underline: false,
    note: false,
    boneyard: false,
};

fn spot(line: usize, col: usize) -> Spot {
    Spot { line, col }
}

fn change(from: Spot, to: Spot, typed: &str) -> Change {
    Change {
        from,
        to,
        typed: typed.to_owned(),
        runs: Vec::new(),
    }
}

#[test]
fn typing_in_a_bold_word_stays_bold() {
    let mut typed = change(spot(0, 10), spot(0, 10), "xx");
    typed.runs = vec![Run {
        start: 0,
        end: 2,
        marks: BOLD,
    }];

    let edited = edit_shown("Das ist **fett** und", &EMPHASIS, &typed);

    assert_eq!(edited.source, "Das ist **fexxtt** und");
    assert_eq!(edited.shown.len(), 1);
    assert_eq!(edited.shown[0].text, "Das ist fexxtt und");
}

#[test]
fn a_typed_mark_is_shown_as_emphasis() {
    let edited = edit_shown("a *x b", &EMPHASIS, &change(spot(0, 4), spot(0, 4), "*"));
    assert_eq!(edited.source, "a *x* b");
    assert_eq!(edited.shown[0].text, "a x b");
    assert!(edited.shown[0].runs[1].marks.italic);
}

#[test]
fn lines_are_joined_and_keep_their_emphasis() {
    let source = "*a*\nx\n*b*\n";
    let join = change(spot(0, 1), spot(2, 0), "");

    let edited = edit_shown(source, &EMPHASIS, &join);

    assert_eq!(edited.source, "*ab*\n");
    assert_eq!((edited.from, edited.to), (0, 3));
    assert_eq!(edited.shown.len(), 1);
}

#[test]
fn a_bold_word_is_split_by_enter() {
    let split = change(spot(0, 7), spot(0, 7), "\n");

    let edited = edit_shown("Ein **langes** Wort", &EMPHASIS, &split);

    assert_eq!(edited.source, "Ein **lan**\n**ges** Wort");
    assert_eq!(edited.shown.len(), 2);
    assert_eq!(edited.shown[1].text, "ges Wort");
}

#[test]
fn several_lines_are_put_in() {
    let paste = change(spot(1, 2), spot(1, 2), "x\n*y*\nz");

    let edited = edit_shown("a\nbcde\nf", &EMPHASIS, &paste);

    assert_eq!(edited.source, "a\nbcx\n*y*\nzde\nf");
    assert_eq!((edited.from, edited.to), (1, 2));
    assert_eq!(edited.shown.len(), 3);
    assert_eq!(edited.shown[1].text, "y");
}

#[test]
fn a_backslash_and_a_note_in_the_rest_stay() {
    let source = "Anna\nEin \\*Stern\\* [[n]] Ende";
    let join = change(spot(0, 4), spot(1, 4), "");

    let edited = edit_shown(source, &ALL, &join);

    assert_eq!(edited.source, "Anna\\*Stern\\* [[n]] Ende");
}

#[test]
fn a_change_does_not_touch_the_other_lines() {
    let source = "Ein \\*a\\*\n\nZwei _b_ [[n]]\n\nDrei";
    let edited = edit_shown(source, &ALL, &change(spot(4, 4), spot(4, 4), "!"));

    assert_eq!(edited.source, "Ein \\*a\\*\n\nZwei _b_ [[n]]\n\nDrei!");
    assert_eq!((edited.from, edited.to), (4, 5));
}

#[test]
fn a_line_a_boneyard_covers_is_shown_again() {
    let source = "a\n*b*\nc";

    let edited = edit_shown(source, &EMPHASIS, &change(spot(0, 0), spot(0, 0), "/*"));

    assert_eq!(edited.source, "/*a\n*b*\nc");
    assert_eq!((edited.from, edited.to), (0, 3));
    assert_eq!(edited.shown[1].text, "*b*");
}

/// `typed` put in at the end of the first line one character at a time, as the editor
/// is given it; the source after the last one.
fn typed_by_char(start: &str, typed: &str) -> String {
    let mut source = start.to_owned();

    for ch in typed.chars() {
        let line = crate::shown_lines(&source, &EMPHASIS).swap_remove(0);
        let at = spot(0, line.text.chars().count());
        source = edit_shown(&source, &EMPHASIS, &change(at, at, &ch.to_string())).source;
    }

    source
}

#[test]
fn a_mark_typed_by_hand_pairs_with_the_one_before() {
    assert_eq!(typed_by_char("Das ist ", "*Wort*"), "Das ist *Wort*");
    assert_eq!(typed_by_char("Das ist ", "**Wort**"), "Das ist **Wort**");
    assert_eq!(typed_by_char("Das ist ", "_Wort_"), "Das ist _Wort_");
    assert_eq!(
        typed_by_char("Das ist ", "_**Wort**_ da"),
        "Das ist _**Wort**_ da"
    );

    let shown = crate::shown_lines(&typed_by_char("", "*Wort*"), &EMPHASIS).swap_remove(0);
    assert_eq!(shown.text, "Wort");
    assert!(shown.runs[0].marks.italic);
}

#[test]
fn a_lone_mark_stays_text() {
    assert_eq!(typed_by_char("", "2 * 3"), "2 * 3");
    let source = typed_by_char("a \\*b", "*");
    assert_eq!(crate::shown_lines(&source, &EMPHASIS)[0].text, "a *b*");
}

#[test]
fn columns_and_runs_count_utf16_units() {
    let mut typed = change(spot(0, 6), spot(0, 6), "🎬");
    typed.runs = vec![Run {
        start: 0,
        end: 2,
        marks: BOLD,
    }];

    let edited = edit_shown("Ä🎬 **fett**", &EMPHASIS, &typed);

    assert_eq!(edited.source, "Ä🎬 **fe🎬tt**");
}

#[test]
fn a_replaced_range_is_source_as_it_stands() {
    // Two hits in different bold words, one replacement from the first to the last.
    let source = "**a** foo **b**\nZwei";
    let edited = replace_source(source, &EMPHASIS, spot(0, 2), spot(0, 13), "x** foo **y");

    assert_eq!(edited.source, "**x** foo **y**\nZwei");
    assert_eq!((edited.from, edited.to), (0, 1));
    assert_eq!(edited.shown[0].text, "x foo y");

    // Over lines, behind an emoji: `to` counts UTF-16 units of the source line.
    let source = "🎬 a\n*b*\nc";
    let edited = replace_source(source, &EMPHASIS, spot(0, 3), spot(1, 1), "X");
    assert_eq!(edited.source, "🎬 Xb*\nc");
    assert_eq!(edited.shown.len(), 1);

    let edited = replace_source("a\nb", &EMPHASIS, spot(0, 1), spot(0, 1), "\n\n");
    assert_eq!(edited.source, "a\n\n\nb");
}

/// The shown text of the first line of `source`, and its source.
fn shown_text(source: &str, hide: &[Hide]) -> String {
    crate::shown_lines(source, hide).swap_remove(0).text
}

#[test]
fn a_join_keeps_the_backslashes_of_the_first_line() {
    let join = change(spot(0, 13), spot(1, 0), "");
    let edited = edit_shown("Ein \\*Stern\\* x\nB", &EMPHASIS, &join);
    assert_eq!(edited.source, "Ein \\*Stern\\* xB");
    assert_eq!(edited.shown[0].text, "Ein *Stern* xB");

    let join = change(spot(0, 5), spot(1, 0), "");
    let edited = edit_shown("a \\_b\\_\nc", &EMPHASIS, &join);
    assert_eq!(edited.source, "a \\_b\\_c");

    // A selection over the break, written over with a character.
    let over = change(spot(0, 5), spot(1, 1), "Z");
    let edited = edit_shown("a \\_b\\_\nc", &EMPHASIS, &over);
    assert_eq!(edited.source, "a \\_b\\_Z");
}

#[test]
fn a_mark_does_not_pair_over_an_escaped_one() {
    let source = typed_by_char("Preis *5 \\* 3 Euro", "*");
    assert_eq!(source, "Preis *5 \\* 3 Euro*");
    assert_eq!(shown_text(&source, &EMPHASIS), "Preis 5 * 3 Euro");

    let source = typed_by_char("*a \\* b", "*");
    assert_eq!(shown_text(&source, &EMPHASIS), "a * b");
}

#[test]
fn a_join_and_a_split_keep_the_notes() {
    let join = change(spot(0, 10), spot(1, 0), "");
    let edited = edit_shown("Anna [[n]] geht\nB", &ALL, &join);
    assert_eq!(edited.source, "Anna [[n]] gehtB");

    let split = change(spot(0, 4), spot(0, 4), "\n");
    let edited = edit_shown("Anna [[n]] geht", &ALL, &split);
    assert_eq!(
        edited.source.matches("[[n]]").count(),
        1,
        "{:?}",
        edited.source
    );
    assert_eq!(edited.source, "Anna\n [[n]] geht");
}
