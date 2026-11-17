//! Scene numbers (DB-33): set when saving, never when opening; the form `Ctrl+N`.

use std::fs;

use super::support::*;
use super::*;

const BOOK: &str = "INT. A - TAG\n\nEr wartet.\n\nINT. B - NACHT #7#\n\nSie geht.";
const NUMBERED: &str = "INT. A - TAG #1#\n\nEr wartet.\n\nINT. B - NACHT #2#\n\nSie geht.";

fn automatic(book: &str) -> (Scratch, App) {
    let scratch = Scratch::automatic(book);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl_end(&mut app);

    (scratch, app)
}

fn project_file(scratch: &Scratch) -> String {
    fs::read_to_string(scratch.dir.join("projekt.toml")).unwrap_or_default()
}

#[test]
fn opening_leaves_the_numbers_as_they_are() {
    let scratch = Scratch::new(BOOK);
    let app = scratch.app();

    assert_eq!(app.text(), BOOK);
    assert!(!app.is_modified());
    assert_eq!(scratch.script(), BOOK);
}

#[test]
fn saving_numbers_the_scenes_in_the_file_and_the_editor() {
    let (scratch, mut app) = automatic(BOOK);

    ctrl(&mut app, 's');

    assert_eq!(scratch.script(), NUMBERED);
    assert_eq!(app.text(), NUMBERED);
    assert!(!app.is_modified());
}

#[test]
fn the_form_locks_the_numbers_and_sets_the_first_number() {
    let (scratch, mut app) = automatic(BOOK);

    ctrl(&mut app, 'n');
    assert_eq!(app.mode, Mode::Numbers);
    assert!(screen(&mut app, 100, 24).contains("automatic"));
    press(&mut app, KeyCode::Right);
    assert!(screen(&mut app, 100, 24).contains("locked"));
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Backspace);
    typed(&mut app, "40");
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.mode, Mode::Normal);
    let file = project_file(&scratch);
    // Locked is what a project without the key is.
    assert!(!file.contains("scene_numbering"), "{file}");
    assert!(file.contains("first_scene = 40"), "{file}");
    assert_eq!(
        scratch.script(),
        BOOK,
        "the book changes with the next save"
    );

    // Locked, the 7 stays and the scene before it becomes A7. The example's scene 12 was never
    // in this book: it gets no struck heading.
    ctrl(&mut app, 's');
    assert_eq!(
        scratch.script(),
        "INT. A - TAG #A7#\n\nEr wartet.\n\nINT. B - NACHT #7#\n\nSie geht."
    );
}

#[test]
fn escape_leaves_the_numbering_as_it_was() {
    let (scratch, mut app) = automatic(BOOK);

    ctrl(&mut app, 'n');
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.mode, Mode::Normal);
    assert!(project_file(&scratch).contains("scene_numbering = \"automatic\""));
}
