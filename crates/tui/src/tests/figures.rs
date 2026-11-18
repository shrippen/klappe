//! The questions about the names of the book (`Ctrl+K`, DB-50): a cue no figure answers to.

use std::fs;

use super::support::*;
use super::*;

const BOOK: &str = "INT. SÄNGERSAAL – TAG #12#\n\nElisabeth tritt an den Tisch.\n\n\
ELISABETH\nIhr kommt spät.\n\nSTIMME (V.O.)\nKomm.\n\nSTIMME\nKomm doch.\n\n";

fn figures_file(scratch: &Scratch) -> String {
    fs::read_to_string(scratch.dir.join("figuren.toml")).unwrap_or_default()
}

#[test]
fn ctrl_k_asks_about_a_name_no_figure_has() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    assert!(app.book_note.contains('1'), "{}", app.book_note);

    ctrl(&mut app, 'k');

    assert_eq!(app.mode, Mode::Book);
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("STIMME, 2 cue(s)"), "{frame}");
    assert!(frame.contains("New figure STIMME"), "{frame}");
    assert!(frame.contains("Is Elisabeth"), "{frame}");
    assert!(frame.contains("Not a figure"), "{frame}");
    assert!(frame.contains("Name of the book"), "{frame}");
}

#[test]
fn not_a_figure_is_written_at_once_and_ends_the_question() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');

    // New figure, Is Elisabeth, Not a figure.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.mode, Mode::Normal, "no question left");
    assert_eq!(app.book_note, "");
    assert!(figures_file(&scratch).contains("STIMME"));
    let mut again = scratch.app();
    assert!(again.figure_question().is_none());
    let _ = screen(&mut again, 100, 24);
}

#[test]
fn a_new_figure_is_written_and_undo_asks_again() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');

    press(&mut app, KeyCode::Enter);

    assert!(figures_file(&scratch).contains("name = \"Stimme\""));
    assert!(app.figure_question().is_none());

    ctrl(&mut app, 'z');

    assert_eq!(
        app.figure_question().map(|q| q.name.as_str()),
        Some("STIMME")
    );
    assert!(!figures_file(&scratch).contains("Stimme"));
}

#[test]
fn up_from_the_first_line_stays_on_it() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');

    press(&mut app, KeyCode::Up);

    assert_eq!(app.book_sel, 0);
}
