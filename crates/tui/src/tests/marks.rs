//! What the editor shows distinctly (DB-05).

use ratatui::style::Modifier;

use super::support::*;
use super::*;

const SHOWN: &str = "INT. A - TAG\n\nSie *flüstert*, **ruft**, ***schreit***, _leise_ [[prüfen]].\n\nANNA\nHallo.\n\nBEN ^\nWas?\n\n> ENDE <\n\n~Sie singen\n\n[[Eine Notiz]]\n\nvorher /* weg\nnoch weg\nende */ danach\n";

fn shown() -> (Scratch, Buffer) {
    let (s, mut app) = app_with(SHOWN);
    press(&mut app, KeyCode::Home);
    let buffer = render(&mut app, 100, 40);

    (s, buffer)
}

fn cell<'a>(buffer: &'a Buffer, needle: &str, offset: u16) -> &'a ratatui::buffer::Cell {
    let (x, y) = find(buffer, 100, 40, needle);

    &buffer[(x + offset, y)]
}

#[test]
fn emphasis_shows_as_bold_italic_and_underline() {
    let (_s, b) = shown();

    assert!(cell(&b, "*flüstert*", 2)
        .modifier
        .contains(Modifier::ITALIC));
    assert!(cell(&b, "**ruft**", 3).modifier.contains(Modifier::BOLD));
    let both = cell(&b, "***schreit***", 4).modifier;
    assert!(both.contains(Modifier::BOLD) && both.contains(Modifier::ITALIC));
    assert!(cell(&b, "_leise_", 2)
        .modifier
        .contains(Modifier::UNDERLINED));
    assert!(!cell(&b, "Sie *flüstert*", 0)
        .modifier
        .contains(Modifier::ITALIC));
}

#[test]
fn notes_show_in_the_note_colour_inline_and_on_their_own_line() {
    let (_s, b) = shown();

    assert_eq!(cell(&b, "[[prüfen]]", 3).fg, theme::NOTE);
    assert_eq!(cell(&b, "[[Eine Notiz]]", 3).fg, theme::NOTE);
    assert_ne!(cell(&b, "ruft", 0).fg, theme::NOTE);
}

#[test]
fn a_boneyard_is_dimmed_over_its_lines() {
    let (_s, b) = shown();

    for needle in ["/* weg", "noch weg", "ende */"] {
        let c = cell(&b, needle, 1);
        assert_eq!(c.fg, theme::MUTED, "{needle}");
        assert!(c.modifier.contains(Modifier::DIM), "{needle}");
    }
    assert_ne!(cell(&b, "danach", 0).fg, theme::MUTED);
    assert_ne!(cell(&b, "vorher", 0).fg, theme::MUTED);
}

#[test]
fn a_dual_cue_has_its_own_colour_and_a_plain_one_has_not() {
    let (_s, b) = shown();

    assert_eq!(cell(&b, "BEN ^", 0).fg, theme::DUAL);
    assert_eq!(cell(&b, "BEN ^", 4).fg, theme::DUAL);
    assert_eq!(cell(&b, "ANNA", 0).fg, theme::STRONG);
}

#[test]
fn the_dialogue_of_a_dual_cue_has_its_colour_too() {
    let (_s, b) = shown();

    assert_eq!(cell(&b, "Was?", 0).fg, theme::DUAL);
    assert_ne!(cell(&b, "Hallo.", 0).fg, theme::DUAL);
}

#[test]
fn lyrics_show_in_their_colour_and_italic() {
    let (_s, b) = shown();
    let c = cell(&b, "~Sie singen", 3);

    assert_eq!(c.fg, theme::LYRICS);
    assert!(c.modifier.contains(Modifier::ITALIC));
}

#[test]
fn a_centered_line_is_drawn_in_the_middle() {
    let (_s, b) = shown();
    let (x, _) = find(&b, 100, 40, "> ENDE <");

    // The list takes a quarter of the 100 columns; the text area is left of the middle.
    assert!(x > 40, "{x}");
}
