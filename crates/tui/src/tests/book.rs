//! The questions about the book (`Ctrl+K`): the register could not place two entries.

use std::fs;

use crate::app::Notice;

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
    // Both place names are scripted locations: only the scenes are asked about.
    let places = "[[ort]]\nid = \"hofhofho\"\nname = \"HOF\"\n\n[[ort]]\nid = \"kellerke\"\nname = \"KELLER\"\n";
    fs::write(scratch.dir.join("orte.toml"), places).unwrap_or_else(|e| unreachable!("{e}"));
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
        frame.contains("which scene of the script is this?"),
        "{frame}"
    );
    assert!(frame.contains("Not in the script"), "{frame}");
    assert!(frame.contains("MATCH"), "{frame}");
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
    assert!(app.question().is_none());
    assert_eq!(app.book_note, "");
}

#[test]
fn not_in_the_book_closes_the_question() {
    let (_s, mut app) = swapped();
    ctrl(&mut app, 'k');
    // Up from the first row wraps to the last: "Not in the script".
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);

    let ask = app.question().map(|q| q.ask).unwrap_or_default();
    assert!(!ask.contains("Entry 1 "), "entry 1 is answered: {ask}");
}

#[test]
fn esc_keeps_the_questions_open() {
    let (_s, mut app) = swapped();
    ctrl(&mut app, 'k');
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.asked_entry().as_deref(), Some("1"));
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
    assert!(again.question().is_none(), "{}", again.book_note);
    let _ = screen(&mut again, 100, 24);
}

#[test]
fn an_unsaved_book_is_saved_before_questions_are_answered() {
    let (scratch, mut app) = swapped();
    ctrl_end(&mut app);
    typed(&mut app, "\n\nINT. GARTEN – TAG\n\nVögel.\n");
    app.autosave();

    ctrl(&mut app, 'k');

    assert_eq!(app.mode, Mode::Normal, "no question on an unsaved book");
    let register = fs::read_to_string(scratch.dir.join("szenen.toml")).unwrap_or_default();
    assert!(!register.contains("GARTEN"), "{register}");
    let frame = screen(&mut app, 100, 24);
    assert!(frame.contains("Save first"), "{frame}");
}

#[test]
fn a_long_folder_name_does_not_cut_the_notice() {
    let name = format!("kt-{}-long-{}.klappe", std::process::id(), "x".repeat(40));
    let scratch = Scratch::named("INT. HOF – TAG\n\nEin Hund.\n", &name);
    let notices = [
        (Notice::SaveFirst, "Save first (Ctrl+S)"),
        (Notice::SaveFailed("disk full".into()), "disk full"),
        (Notice::Pdf("PDF written to out.pdf".into()), "out.pdf"),
        (Notice::Loaded("Loaded the backup".into()), "the backup"),
    ];

    for (notice, shown) in notices {
        let mut app = scratch.app();
        app.notice = Some(notice);
        let frame = screen(&mut app, 100, 24);
        assert!(frame.contains(shown), "{shown}\n{frame}");
    }
}

/// Answers the first question through the dialog.
fn answer_first(app: &mut App) {
    ctrl(app, 'k');
    press(app, KeyCode::Enter);
    press(app, KeyCode::Esc);
}

#[test]
fn ctrl_z_takes_back_an_answer_and_ctrl_y_does_it_again() {
    let (scratch, mut app) = swapped();
    answer_first(&mut app);
    assert_eq!(app.asked_entry().as_deref(), Some("2"));

    ctrl(&mut app, 'z');
    assert_eq!(app.asked_entry().as_deref(), Some("1"));
    assert!(app.book_note.contains("2"), "{}", app.book_note);
    ctrl(&mut app, 'k');
    assert_eq!(app.mode, Mode::Book, "the question is asked again");
    press(&mut app, KeyCode::Esc);

    drop(app);
    let reopened = scratch.app();
    assert_eq!(
        reopened.asked_entry().as_deref(),
        Some("1"),
        "the register on disk has it open again"
    );
    let mut app = reopened;
    answer_first(&mut app);
    ctrl(&mut app, 'z');
    ctrl(&mut app, 'y');
    assert_eq!(app.asked_entry().as_deref(), Some("2"));
    drop(app);
    let again = scratch.app();
    assert_eq!(again.asked_entry().as_deref(), Some("2"));
}

#[test]
fn ctrl_z_undoes_typing_after_an_answer_before_the_answer() {
    let (_s, mut app) = swapped();
    answer_first(&mut app);
    ctrl_end(&mut app);
    let text = app.text();
    typed(&mut app, "Ah");

    ctrl(&mut app, 'z');
    assert_eq!(app.text(), text, "only the typing is undone");
    assert_eq!(app.asked_entry().as_deref(), Some("2"));

    ctrl(&mut app, 'z');
    assert_eq!(app.asked_entry().as_deref(), Some("1"));
    assert_eq!(app.text(), text);

    ctrl(&mut app, 'y');
    assert_eq!(app.asked_entry().as_deref(), Some("2"));
    ctrl(&mut app, 'y');
    assert!(app.text().ends_with("Ah"), "then the typing again");
}

#[test]
fn typing_after_an_answer_does_not_join_the_earlier_step() {
    let (_s, mut app) = swapped();
    ctrl_end(&mut app);
    typed(&mut app, "ab");
    ctrl(&mut app, 's');
    answer_first(&mut app);
    typed(&mut app, "c");

    ctrl(&mut app, 'z');
    assert!(app.text().trim_end().ends_with("ab"), "{}", app.text());
    assert_eq!(app.asked_entry().as_deref(), Some("2"));
}

#[test]
fn ctrl_y_brings_an_answer_back_after_undoing_typing_before_it() {
    let (_s, mut app) = swapped();
    ctrl_end(&mut app);
    typed(&mut app, "ab");
    ctrl(&mut app, 's');
    answer_first(&mut app);

    ctrl(&mut app, 'z');
    ctrl(&mut app, 'z');
    ctrl(&mut app, 'y');
    assert!(app.text().ends_with("ab"), "first the typing again");
    ctrl(&mut app, 'y');

    assert_eq!(app.asked_entry().as_deref(), Some("2"));
}

#[test]
fn typing_after_taking_back_an_answer_drops_its_redo() {
    let (_s, mut app) = swapped();
    answer_first(&mut app);
    ctrl(&mut app, 'z');
    ctrl_end(&mut app);
    typed(&mut app, "x");
    ctrl(&mut app, 'z');

    ctrl(&mut app, 'y');

    assert!(app.text().ends_with('x'), "the typing comes back");
    assert_eq!(app.asked_entry().as_deref(), Some("1"));
}

#[test]
fn a_new_edit_at_the_same_depth_drops_the_answer_redo() {
    let (_s, mut app) = swapped();
    ctrl_end(&mut app);
    typed(&mut app, "ab");
    ctrl(&mut app, 's');
    answer_first(&mut app);
    ctrl(&mut app, 'z');
    ctrl(&mut app, 'z');
    typed(&mut app, "c");

    ctrl(&mut app, 'y');

    assert_eq!(app.asked_entry().as_deref(), Some("1"));
}
