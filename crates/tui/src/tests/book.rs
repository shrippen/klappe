//! The questions about the book (`Ctrl+K`): the register could not place two entries.

use std::fs;

use super::support::*;
use super::*;

const HOF: &str = "INT. HOF – NACHT";
const KELLER: &str = "INT. KELLER – NACHT";

/// Entries "1 Keller" and "2 Hof" against a book with the headings swapped: both asked.
fn swapped() -> (Scratch, App) {
    let book = format!("{HOF} #1#\n\nEin Hund bellt.\n\n{KELLER} #2#\n\nWasser tropft.\n\n");
    let scratch = Scratch::new(&book);
    let entry = |id: &str, number: &str, heading: &str| {
        format!("[[szene]]\nid = \"{id}\"\nart = \"scene\"\nnumber = \"{number}\"\nheading = \"{heading}\"\n\n")
    };
    let register = entry("aaaa", "1", KELLER) + &entry("bbbb", "2", HOF);
    fs::write(scratch.dir.join("szenen.toml"), register).unwrap_or_else(|e| unreachable!("{e}"));
    let _ = fs::remove_dir_all(scratch.dir.join("szenen"));
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);

    (scratch, app)
}

#[test]
fn ctrl_k_asks_the_first_question_with_its_candidates() {
    let (_s, mut app) = swapped();
    assert!(app.book_note.contains("2"), "{}", app.book_note);

    ctrl(&mut app, 'k');
    assert_eq!(app.mode, Mode::Book);
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Entry 1 INT. KELLER"), "{frame}");
    assert!(
        frame.contains("which scene of the book is this?"),
        "{frame}"
    );
    assert!(frame.contains("Not in the book"), "{frame}");
    assert!(frame.contains("BOOK"), "{frame}");
}

#[test]
fn choosing_a_candidate_goes_on_to_the_next_question() {
    let (_s, mut app) = swapped();
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, Mode::Book, "the second question follows");
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Entry 2 INT. HOF"), "{frame}");

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.mode, Mode::Normal, "no question left");
    assert!(app.book_question().is_none());
    assert_eq!(app.book_note, "");
}

#[test]
fn not_in_the_book_closes_the_question() {
    let (_s, mut app) = swapped();
    ctrl(&mut app, 'k');
    // Up from the first row wraps to the last: "Not in the book".
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);

    assert_ne!(
        app.book_question().map(|q| q.number.as_str()),
        Some("1"),
        "entry 1 is answered"
    );
}

#[test]
fn esc_keeps_the_questions_open() {
    let (_s, mut app) = swapped();
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.book_question().map(|q| q.number.as_str()), Some("1"));
    assert!(app.book_note.contains("2"), "{}", app.book_note);
    ctrl(&mut app, 'k');
    assert_eq!(app.book_sel, 0, "the dialog starts at the top again");
}

#[test]
fn ctrl_k_without_questions_does_nothing() {
    let mut app = app();
    ctrl(&mut app, 'k');

    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn an_answer_is_kept_without_saving_the_book() {
    let (scratch, mut app) = swapped();
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);

    let quit = ctrl(&mut app, 'q');
    drop(app);

    assert!(matches!(quit, Flow::Quit));
    let mut again = scratch.app();
    assert!(again.book_question().is_none(), "{}", again.book_note);
    let _ = screen(&mut again, 100, 24);
}
