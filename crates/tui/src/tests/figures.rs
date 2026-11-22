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
    assert!(frame.contains("New character STIMME"), "{frame}");
    assert!(frame.contains("Is Elisabeth"), "{frame}");
    assert!(frame.contains("Not a character"), "{frame}");
    assert!(frame.contains("Name in the script"), "{frame}");
}

#[test]
fn not_a_figure_is_written_at_once_and_ends_the_question() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');

    // New figure, Is Elisabeth, Not a character.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.mode, Mode::Normal, "no question left");
    assert_eq!(app.book_note, "");
    assert!(figures_file(&scratch).contains("STIMME"));
    let mut again = scratch.app();
    assert!(again.asks_no_figure());
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
    assert!(app.asks_no_figure());

    ctrl(&mut app, 'z');

    assert_eq!(app.asked_figure().as_deref(), Some("STIMME"));
    assert!(!figures_file(&scratch).contains("Stimme"));
}

/// Review of PR 313: an answer to a cue another program had answered made no session step but
/// was counted as one; Ctrl+Z then took back the earlier answer instead of the typing.
#[test]
fn a_gone_cue_is_no_step_of_ctrl_z() {
    let scratch = Scratch::new(&format!("{BOOK}CHOR\nHm.\n\n"));
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Esc);
    ctrl_end(&mut app);
    let text = app.text();
    typed(&mut app, "Ah");
    ctrl(&mut app, 's');
    let mut other = klappe_services::Session::open(&scratch.dir).unwrap();
    other
        .resolve_figure("CHOR", klappe_services::FigureChoice::NotFigure)
        .unwrap();

    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Esc);
    ctrl(&mut app, 'z');

    assert_eq!(app.text(), text, "the typing is taken back first");
    assert!(figures_file(&scratch).contains("Stimme"));
}
