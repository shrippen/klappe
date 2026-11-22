//! The screenplay changed outside (the window beside the terminal): the editor follows while
//! untouched, a save asks when it changed both here and there.

use klappe_services::{OnOutside, Session};

use super::support::*;
use super::*;

const BOOK: &str = "INT. A - TAG\n\nx";
const OTHER_BOOK: &str = "INT. A - TAG\n\nx\n\nVon draussen.\n";

/// What another program does: saves its own text of the screenplay.
fn change_outside(scratch: &Scratch) {
    let mut other = Session::open(&scratch.dir).unwrap_or_else(|e| unreachable!("{e}"));
    other.set_script_text(OTHER_BOOK);
    other
        .save_script(OnOutside::Overwrite)
        .unwrap_or_else(|e| unreachable!("{e}"));
}

/// An app with typed text, and the screenplay changed outside meanwhile, then Ctrl+S.
fn asked() -> (Scratch, App) {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    typed(&mut app, "neu ");
    change_outside(&scratch);
    ctrl(&mut app, 's');

    (scratch, app)
}

#[test]
fn a_save_asks_when_both_changed_the_screenplay() {
    let (scratch, mut app) = asked();

    assert_eq!(app.mode, Mode::Outside);
    assert_eq!(scratch.script(), OTHER_BOOK, "nothing was written");
    assert!(screen(&mut app, 100, 24).contains("Changed outside and here: Screenplay."));
}

#[test]
fn reload_takes_the_file_and_drops_the_typing_and_its_history() {
    let (_scratch, mut app) = asked();

    press(&mut app, KeyCode::Char('r'));

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.text(), OTHER_BOOK);
    assert!(!app.is_modified());
    assert!(screen(&mut app, 100, 24).contains("Reloaded: Screenplay"));
    ctrl(&mut app, 'z');
    assert_eq!(app.text(), OTHER_BOOK, "the editor's history starts over");
}

#[test]
fn overwrite_writes_the_editor_text() {
    let (scratch, mut app) = asked();

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(scratch.script(), format!("neu {BOOK}"));
    assert!(!app.is_modified());
}

#[test]
fn escape_cancels_and_changes_nothing() {
    let (scratch, mut app) = asked();

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(scratch.script(), OTHER_BOOK);
    assert_eq!(app.text(), format!("neu {BOOK}"));
    assert!(app.is_modified());
}

#[test]
fn an_untouched_editor_follows_a_change_outside() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    change_outside(&scratch);

    ctrl(&mut app, 's');

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.text(), OTHER_BOOK);
    assert!(!app.is_modified());
    assert!(screen(&mut app, 100, 24).contains("Reloaded: Screenplay"));
    ctrl(&mut app, 'z');
    assert_eq!(app.text(), OTHER_BOOK);
}

/// Opens the numbering form and confirms it unchanged (Ctrl+N, Enter).
fn confirm_numbers(app: &mut App) {
    ctrl(app, 'n');
    press(app, KeyCode::Enter);
}

#[test]
fn a_writer_of_the_project_keeps_the_editor_in_step_with_a_change_outside() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    change_outside(&scratch);

    confirm_numbers(&mut app);
    assert_eq!(app.text(), OTHER_BOOK);
    assert!(screen(&mut app, 100, 24).contains("Reloaded: Screenplay"));

    ctrl(&mut app, 's');
    assert_eq!(scratch.script(), OTHER_BOOK);
}

#[test]
fn a_writer_of_the_project_leaves_typing_to_the_question_of_the_save() {
    let scratch = Scratch::new(BOOK);
    let mut app = scratch.app();
    typed(&mut app, "neu ");
    change_outside(&scratch);

    confirm_numbers(&mut app);
    ctrl(&mut app, 's');

    assert_eq!(app.mode, Mode::Outside);
    assert_eq!(scratch.script(), OTHER_BOOK, "nothing was written");
    assert_eq!(app.text(), format!("neu {BOOK}"));
}
