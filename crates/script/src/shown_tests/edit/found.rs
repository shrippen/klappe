//! Cases the fuzz of changes found outside every limit, one test each.

use super::*;

/// `source` with `removed` of its text deleted, the emphasis of what is left as it was.
fn delete_keeping_marks(source: &str, hide: &[Hide], removed: Range<usize>) -> String {
    let shown = shown_lines(source, hide).remove(0);
    let mut marks = marks_of(&shown.runs, shown.text.chars().count());
    marks.drain(removed.clone());
    let change = LineEdit {
        removed,
        typed: String::new(),
        runs: runs(&marks),
    };

    edit_line(source, shown.kind, shown.shape, hide, &change)
}

#[test]
fn a_note_stays_a_note_when_an_emphasized_blank_follows() {
    let hide = [Hide::Emphasis];
    let source = "[[n]]*  *ÄÖÜ🎬";
    let new = delete_keeping_marks(source, &hide, 6..11);

    assert_eq!(new, "[[n]]* *");
}

#[test]
fn a_cue_stays_a_cue_when_a_word_is_typed_into_its_guard() {
    let source = "(V.O.)__^";
    let shown = read_as(TextKind::Character, source, &ALL);
    let new = edit_line(
        source,
        shown.kind,
        shown.shape,
        &ALL,
        &replace_text(1..4, "Anna"),
    );

    assert_eq!(
        read_as(TextKind::Character, &new, &ALL).kind,
        TextKind::Character,
        "{new:?}"
    );
}
