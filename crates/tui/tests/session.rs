//! Saving through the session, on a copy of the example project.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::Session;
use klappe_tui::{App, Flow, Language};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    dir
}

fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) -> Flow {
    app.handle_key(KeyEvent::new(code, modifiers))
}

#[test]
fn save_writes_the_edit_and_a_new_session_reads_it() {
    let dir = copy("tui-save");
    let mut app = App::open(&dir, Language::En).unwrap();

    for c in "Äö – ".chars() {
        key(&mut app, KeyCode::Char(c), KeyModifiers::NONE);
    }
    assert!(app.is_modified());
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert!(!app.is_modified());

    let text = Session::open(&dir).unwrap().script_text().to_owned();
    assert!(
        text.starts_with("Äö – INT. SÄNGERSAAL – TAG #12#\n"),
        "{text}"
    );
    assert_eq!(text, app.text());
}

#[test]
fn unchanged_save_keeps_the_file_text() {
    let dir = copy("tui-same");
    let before = std::fs::read_to_string(dir.join("drehbuch.fountain")).unwrap();
    let mut app = App::open(&dir, Language::En).unwrap();
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert_eq!(
        std::fs::read_to_string(dir.join("drehbuch.fountain")).unwrap(),
        before
    );
}

#[test]
fn saved_project_quits_without_asking() {
    let dir = copy("tui-quit");
    let mut app = App::open(&dir, Language::En).unwrap();
    key(&mut app, KeyCode::Char('x'), KeyModifiers::NONE);
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert_eq!(
        key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL),
        Flow::Quit
    );
}
