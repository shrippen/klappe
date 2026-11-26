//! `klappe-cli rename-figure` (DB-04).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use klappe_services::Session;

/// A copy of the example project: Elisabeth cues once, Burgvogt once.
fn project(name: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&from, &dir);

    dir
}

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

fn rename(dir: &Path, from: &str, to: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["rename-figure", "--project"])
        .arg(dir)
        .args(["--from", from, "--to", to, "--language", "en"])
        .output()
        .unwrap()
}

#[test]
fn the_figure_is_renamed_and_saved() {
    let dir = project("cli-rename");

    let output = rename(&dir, "Elisabeth", "Elise");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("cues changed: 1"));
    let s = Session::open(&dir).unwrap();
    assert!(s.script_text().contains("ELISE\nIhr kommt spät."));
    assert_eq!(s.project().characters[0].name, "Elise");
}

#[test]
fn a_refused_rename_says_why_and_writes_nothing() {
    let dir = project("cli-rename-refused");

    let output = rename(&dir, "Elisabeth", "Elisabeth");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("already"));
    let s = Session::open(&dir).unwrap();
    assert!(s.script_text().contains("ELISABETH\n"));
}
