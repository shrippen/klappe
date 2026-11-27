//! The limits of Fountain the fuzz of changes names, one test each. A typed note and a typed `^`
//! at the end of a cue are limits too, but the fuzz does not read back what is typed.

use super::*;

#[test]
fn an_untouched_backslash_keeps_a_note_text() {
    let hide = [Hide::Emphasis];
    let new = edited("\\[[n]]a", &hide, &replace_text(5..6, ""));

    assert_eq!(new, "\\[\\[n]]");
    assert_eq!(kind_in(&new, &hide), TextKind::Action);
}

#[test]
fn a_typed_note_is_a_note() {
    let new = splice("Hallo [[a]]", 5..5, "[[b]]");
    let shown = shown_one(&new);

    assert_eq!(new, "Hallo[[b]] [[a]]");
    assert_eq!(shown.text, "Hallo ");
    assert_eq!(shown.notes.len(), 2);
}

#[test]
fn a_note_line_is_one_note() {
    let source = "[[n]]^";
    let new = splice(source, 0..1, "[[n]]");
    let shown = shown_one(&new);

    assert_eq!(new, "\\[\\[n]][[n]]");
    assert_eq!(
        (shown.kind, shown.text.as_str()),
        (TextKind::Action, "[[n]]")
    );
    assert_eq!(shown.notes.len(), 1);
}

#[test]
fn a_note_before_a_hash_is_no_section() {
    let new = splice("[[n]]a", 0..1, "#");

    assert_eq!(new, "#[[n]]");
    assert_eq!(shown_one(&new).kind, TextKind::Section);
}

#[test]
fn a_heading_cannot_end_with_its_number() {
    let new = splice("INT.", 4..4, "#1#");

    assert_eq!(new, "INT.#1#");
    assert_eq!(shown_one(&new).text, "INT.");
    assert_eq!(shown_one(&new).number.as_deref(), Some("1"));
}

#[test]
fn a_heading_cannot_start_with_a_dot() {
    // `.` alone is the forcing character, but it shows until a character follows.
    let new = splice("INT.", 0..3, "");

    assert_eq!(new, ".");
    assert_eq!(shown_one(&new).kind, TextKind::Heading);
    assert_eq!(shown_one(&new).text, ".");
}

#[test]
fn a_transition_cannot_end_with_a_chevron() {
    let new = splice(">Text", 4..4, "<");

    assert_eq!(new, ">Text<");
    assert_eq!(shown_one(&new).kind, TextKind::Centered);
}

#[test]
fn a_cue_cannot_end_with_a_caret() {
    let new = splice("@Anna", 4..4, "^");

    assert_eq!(new, "@Anna^");
    assert_eq!(read_as(TextKind::Character, &new, &ALL).text, "Anna");
}

#[test]
fn a_note_behind_a_parenthetical_makes_it_speech() {
    let source = "Hallo[[n]]";
    let shown = read_as(TextKind::Dialogue, source, &ALL);
    let new = edit_line(
        source,
        shown.kind,
        shown.shape,
        &ALL,
        &replace_text(0..5, "(leise)"),
    );
    let read = read_as(TextKind::Dialogue, &new, &ALL);

    assert_eq!(new, "(leise)[[n]]");
    assert_eq!(
        (read.kind, read.text.as_str()),
        (TextKind::Dialogue, "(leise)")
    );
}

#[test]
fn a_blank_with_a_note_is_a_note() {
    let new = splice("Hallo[[n]]", 0..5, " ");

    assert_eq!(new, " [[n]]");
    assert_eq!(shown_one(&new).kind, TextKind::Note);
}

#[test]
fn a_typed_boneyard_opener_is_protected() {
    let new = splice("a\\*", 0..1, "/");

    assert_eq!(new, "/\\*");
    assert_eq!(shown_one(&new).text, "/*");
    assert_eq!(shown_one(&new).kind, TextKind::Action);
}

#[test]
fn a_run_that_ends_behind_a_slash_stays_plain() {
    let source = edited(
        "a/",
        &ALL,
        &format_only(&[run(0, 2, mark(false, true, false))]),
    );

    assert_eq!(source, "a/");
}

#[test]
fn a_run_that_would_touch_the_stars_before_it_stays_plain() {
    let change = format_only(&[
        run(0, 1, mark(false, true, false)),
        run(1, 2, mark(true, true, false)),
    ]);
    let source = edited("ab", &ALL, &change);

    assert_eq!(
        shown_one(&source).runs,
        [
            run(0, 1, mark(false, true, false)),
            run(1, 2, mark(false, false, false)),
        ]
    );
}
