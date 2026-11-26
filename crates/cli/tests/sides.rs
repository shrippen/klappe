//! `klappe-cli sides` (DB-62).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use klappe_services::{Language, OnOutside, Session};

const BOOK: &str = "INT. SAAL – TAG\n\nALPHA\nHallo.\n\nINT. KÜCHE – TAG\n\nBETA\nJa.\n\n";

fn project(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    s.set_script_text(BOOK);
    s.save(OnOutside::Ask).unwrap();

    dir
}

fn sides(dir: &Path, figure: &str, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["sides", "--language", "en", "--figure", figure])
        .arg("--project")
        .arg(dir)
        .arg("--out")
        .arg(out)
        .output()
        .unwrap()
}

#[test]
fn the_sides_hold_the_scenes_of_the_figure() {
    let dir = project("cli-sides");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-sides.pdf");

    let output = sides(&dir, "beta", &out);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pdf = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
    assert!(pdf.contains("(Ja.) Tj") && pdf.contains("(2) Tj"), "{pdf}");
    assert!(!pdf.contains("(Hallo.) Tj"), "{pdf}");
}

#[test]
fn an_unknown_figure_is_refused_in_words() {
    let dir = project("cli-sides-unknown");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-sides-unknown.pdf");
    let _ = std::fs::remove_file(&out);

    let output = sides(&dir, "gamma", &out);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Character not found: gamma"));
    assert!(!out.exists());
}
