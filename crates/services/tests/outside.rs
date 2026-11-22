//! Changes made outside the session, in the project folder (two sessions on one folder).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::Session;

const SCRIPT_FILE: &str = "drehbuch.fountain";
const ADDED_LINE: &str = "Ein Satz, den das Fenster nicht kennt.\n";

/// A copy of the example to write to.
fn copy(name: &str) -> PathBuf {
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
                continue;
            }
            std::fs::copy(entry.path(), target).unwrap();
        }
    }

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example, &dir);
    dir
}

#[test]
fn a_save_keeps_the_script_another_session_saved() {
    let dir = copy("outside-script-kept");
    let mut terminal = Session::open(&dir).unwrap();
    let mut window = Session::open(&dir).unwrap();

    let text = format!("{}{ADDED_LINE}", terminal.script_text());
    terminal.set_script_text(&text);
    terminal.save_script().unwrap();

    window.add_plan("Plan B").unwrap();
    window.save().unwrap();

    let on_disk = std::fs::read_to_string(dir.join(SCRIPT_FILE)).unwrap();
    assert!(
        on_disk.contains(ADDED_LINE),
        "the window's save lost the text"
    );
}
