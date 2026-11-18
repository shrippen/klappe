//! Backup and autosave (DN-06), the help (DN-04) and undo (DB-09).

use std::fs;

use super::support::*;
use super::*;

#[test]
fn autosave_writes_the_backup_and_the_next_start_asks() {
    let scratch = Scratch::new("INT. A - TAG\n\nx");
    let mut app = scratch.app();
    typed(&mut app, "neu ");
    app.autosave();

    assert_eq!(
        scratch.script(),
        "INT. A - TAG\n\nx",
        "the project file is untouched"
    );
    let mut again = scratch.app();
    assert_eq!(again.mode, Mode::Backup);
    assert!(screen(&mut again, 100, 24).contains("Unsaved changes of an earlier session"));

    press(&mut again, KeyCode::Char('r'));
    assert_eq!(again.mode, Mode::Normal);
    assert_eq!(again.text(), "neu INT. A - TAG\n\nx");
    assert!(again.is_modified(), "restored text is not saved yet");

    ctrl(&mut again, 'z');
    assert_eq!(again.text(), "INT. A - TAG\n\nx", "a restore can be undone");
}

#[test]
fn discarding_quitting_and_saving_handle_the_backup() {
    let scratch = Scratch::new("x");
    let mut app = scratch.app();
    typed(&mut app, "a");
    app.autosave();

    // The question stays until it is answered: no editing meanwhile, so nothing can be lost.
    let mut later = scratch.app();
    press(&mut later, KeyCode::Esc);
    typed(&mut later, "b");
    assert_eq!(later.mode, Mode::Backup);
    assert_eq!(later.text(), "x", "no typing while the backup waits");
    assert!(!later.is_modified());
    assert_eq!(
        press_ctrl(&mut later, 'q'),
        Flow::Quit,
        "Ctrl+Q leaves it for later"
    );
    assert_eq!(
        scratch.app().mode,
        Mode::Backup,
        "the backup is still there"
    );

    let mut discard = scratch.app();
    press(&mut discard, KeyCode::Char('d'));
    assert_eq!(discard.text(), "x");
    assert_eq!(scratch.app().mode, Mode::Normal);

    let mut saved = scratch.app();
    typed(&mut saved, "c");
    saved.autosave();
    ctrl(&mut saved, 's');
    assert_eq!(
        scratch.app().mode,
        Mode::Normal,
        "a save removes the backup"
    );
    assert_eq!(scratch.script(), "cx");
}

fn press_ctrl(app: &mut App, c: char) -> Flow {
    ctrl(app, c)
}

#[test]
fn the_backup_question_names_its_keys() {
    let scratch = Scratch::new("x");
    let mut app = scratch.app();
    typed(&mut app, "a");
    app.autosave();

    let mut again = scratch.app();
    let frame = screen(&mut again, 100, 24);
    assert!(frame.contains("r Restore"), "{frame}");
    assert!(frame.contains("d Discard"), "{frame}");
    assert!(frame.contains("Ctrl+Q"), "{frame}");
}

#[test]
fn a_failed_restore_keeps_the_question_open() {
    let scratch = Scratch::new("x");
    let mut app = scratch.app();
    typed(&mut app, "a");
    app.autosave();

    // The backup folder is gone by the time the answer comes.
    let mut again = scratch.app();
    assert_eq!(again.mode, Mode::Backup);
    std::fs::remove_dir_all(scratch.dir.join(".sicherung")).unwrap_or_else(|e| unreachable!("{e}"));
    press(&mut again, KeyCode::Char('r'));
    assert_eq!(again.mode, Mode::Backup, "still asking");
    assert!(screen(&mut again, 100, 24).contains("Saving failed"));

    // Discarding what is not there ends the question, and then autosave writes again.
    press(&mut again, KeyCode::Char('d'));
    assert_eq!(again.mode, Mode::Normal);
    typed(&mut again, "b");
    again.autosave();
    assert_eq!(
        scratch.app().mode,
        Mode::Backup,
        "the new work is in a backup"
    );
}

#[test]
fn autosave_writes_nothing_without_a_change() {
    let scratch = Scratch::new("x");
    let mut app = scratch.app();
    app.autosave();
    assert_eq!(scratch.app().mode, Mode::Normal);

    typed(&mut app, "a");
    press(&mut app, KeyCode::Backspace);
    // Back to the saved text is still a change of the editor; the backup holds it harmlessly.
    app.autosave();
    ctrl(&mut app, 's');
    assert_eq!(scratch.app().mode, Mode::Normal);
}

#[test]
fn every_key_is_in_the_help() {
    let mut app = app();
    press(&mut app, KeyCode::F(1));
    // The help scrolls on a low screen; a tall one shows it whole.
    let frame = screen(&mut app, 100, 40);

    for key in [
        "Ctrl+S",
        "Ctrl+Q",
        "Ctrl+Z",
        "Ctrl+Y",
        "Enter",
        "Tab",
        "Ctrl+F",
        "F3",
        "Ctrl+T",
        "Alt+H",
        "Alt+T",
        "F6",
        "Ctrl+P",
        "Ctrl+B",
        "Ctrl+G",
        "F1",
        "Home",
        "PageUp",
        "Ctrl+Left",
        "Mouse wheel",
    ] {
        assert!(frame.contains(key), "{key} missing:\n{frame}");
    }
}

#[test]
fn the_help_scrolls_on_a_low_screen() {
    let mut app = app();
    press(&mut app, KeyCode::F(1));
    assert!(!screen(&mut app, 100, 12).contains("This help"));

    for _ in 0..30 {
        press(&mut app, KeyCode::Down);
    }
    assert_eq!(app.mode, Mode::Help);
    assert!(screen(&mut app, 100, 12).contains("This help"));
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.mode, Mode::Normal);
}

#[test]
fn the_german_help_names_the_new_keys() {
    let mut app = App::open(&example(), Language::German).unwrap_or_else(|e| unreachable!("{e}"));
    press(&mut app, KeyCode::F(1));
    let frame = screen(&mut app, 100, 40);

    assert!(frame.contains("Suchen und Ersetzen"), "{frame}");
    assert!(frame.contains("Titelseite"), "{frame}");
    assert!(frame.contains("F6"), "{frame}");
}

#[test]
fn undo_has_no_limit_in_the_app() {
    let (_s, mut app) = app_with("x");
    for _ in 0..300 {
        press(&mut app, KeyCode::Enter);
        typed(&mut app, "ab ");
    }
    let full = app.text();

    for _ in 0..1000 {
        ctrl(&mut app, 'z');
    }
    assert_eq!(app.text(), "x");
    for _ in 0..1000 {
        ctrl(&mut app, 'y');
    }
    assert_eq!(app.text(), full);
}

#[test]
fn the_help_scroll_stays_in_range() {
    let mut app = app();
    press(&mut app, KeyCode::F(1));
    for _ in 0..30 {
        press(&mut app, KeyCode::Down);
    }
    let _ = screen(&mut app, 100, 12);

    // 29 lines, 10 rows of room: the last first line is 19, and one Up goes back one.
    assert_eq!(app.help_scroll, 19);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.help_scroll, 18);
}

#[test]
fn the_status_line_counts_scenes_struck_from_the_book() {
    // The example's scene 12 is not in this book.
    let scratch = Scratch::new("INT. A - TAG\n\nx");
    let mut app = scratch.app();

    assert!(screen(&mut app, 120, 24).contains("Scenes not in the book: 1."));

    let same = Scratch::new("INT. SÄNGERSAAL – TAG #12#\n\nx");
    let mut fine = same.app();
    assert!(!screen(&mut fine, 120, 24).contains("not in the book"));
}

#[test]
fn an_orphan_scene_file_is_told_on_the_status_line() {
    let scratch = Scratch::new("INT. A - TAG\n\nx");
    fs::write(scratch.dir.join("szenen.toml"), "").unwrap_or_else(|e| unreachable!("{e}"));
    let mut app = scratch.app();

    let text = screen(&mut app, 120, 24);

    assert!(text.contains("without a register entry"), "{text}");
}

#[test]
fn saving_writes_the_register_too() {
    let scratch = Scratch::new("INT. A - TAG\n\nx");
    let register = scratch.dir.join("szenen.toml");
    let mut app = scratch.app();
    // The file on disk falls behind the register the session holds.
    fs::write(&register, "").unwrap_or_else(|e| unreachable!("{e}"));
    typed(&mut app, "neu ");

    ctrl(&mut app, 's');

    let text = fs::read_to_string(&register).unwrap_or_default();
    assert!(text.contains("number = \"12\""), "{text}");
    assert!(text.contains("INT. SÄNGERSAAL – TAG"), "{text}");
}
