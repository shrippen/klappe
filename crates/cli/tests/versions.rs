//! `klappe-cli lock-version`, `versions`, `compare` and `script-pdf --revisions` (DB-40 to DB-42).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use klappe_services::{Language, Session};

fn scene(head: &str, tag: &str) -> String {
    let mut s = format!("INT. {head} – TAG\n\n");
    for i in 1..=3 {
        s.push_str(&format!("{tag} geht durch den Raum und tut Ding {i}.\n\n"));
    }
    s
}

/// A saved project of two scenes.
fn project(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    s.set_script_text(&format!(
        "{}{}",
        scene("SAAL", "Alpha"),
        scene("KÜCHE", "Beta")
    ));
    s.save(klappe_services::OnOutside::Ask).unwrap();

    dir
}

fn cli(args: &[&str], dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(args)
        .arg("--project")
        .arg(dir)
        .args(["--language", "en"])
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn a_version_is_locked_listed_and_compared() {
    let dir = project("cli-versions");

    // Nothing locked: nothing to list, nothing to compare with.
    assert!(stdout(&cli(&["versions"], &dir)).contains("No locked versions"));
    let early = cli(&["compare"], &dir);
    assert!(!early.status.success());
    assert!(String::from_utf8_lossy(&early.stderr).contains("no locked version"));

    let locked = stdout(&cli(&["lock-version", "--name", "Drehfassung"], &dir));
    assert!(
        locked.contains("Locked version \"Drehfassung\" (White,"),
        "{locked}"
    );

    let listed = stdout(&cli(&["versions"], &dir));
    let fields: Vec<&str> = listed.trim().split('\t').collect();
    assert_eq!(fields[1], "Drehfassung");
    assert_eq!(fields[3], "White");
    let id = fields[0].to_owned();

    // The book is the version: every scene unchanged.
    let same = stdout(&cli(&["compare"], &dir));
    assert_eq!(same.lines().count(), 2, "{same}");
    assert!(same.lines().all(|l| l.ends_with("unchanged")), "{same}");

    // Change the kitchen, add a scene: the revision lists both.
    let mut s = Session::open(&dir).unwrap();
    let text = s
        .script_text()
        .replace("Beta geht durch den Raum und tut Ding 2.", "Beta schläft.");
    s.set_script_text(&format!("{text}{}", scene("HOF", "Gamma")));
    s.save(klappe_services::OnOutside::Ask).unwrap();

    let diff = stdout(&cli(&["compare", "--from", &id], &dir));
    let lines: Vec<&str> = diff.lines().collect();
    assert!(lines[0].ends_with("unchanged"), "{diff}");
    assert!(lines[1].ends_with("changed"), "{diff}");
    assert_eq!(lines[2], "  - Beta geht durch den Raum und tut Ding 2.");
    assert_eq!(lines[3], "  + Beta schläft.");
    assert!(
        lines[4].contains("INT. HOF – TAG") && lines[4].ends_with("new"),
        "{diff}"
    );

    let bad = cli(&["compare", "--from", "nope"], &dir);
    assert!(!bad.status.success());
}

#[test]
fn the_name_must_be_new_and_not_empty() {
    let dir = project("cli-version-name");
    stdout(&cli(&["lock-version", "--name", "Drehfassung"], &dir));

    let used = cli(&["lock-version", "--name", "Drehfassung"], &dir);
    let empty = cli(&["lock-version", "--name", " "], &dir);

    assert!(!used.status.success());
    assert!(String::from_utf8_lossy(&used.stderr).contains("already a version"));
    assert!(!empty.status.success());
}

#[test]
fn the_pdf_marks_revisions_when_asked() {
    let dir = project("cli-version-pdf");
    stdout(&cli(&["lock-version", "--name", "Drehfassung"], &dir));
    let mut s = Session::open(&dir).unwrap();
    let text = s
        .script_text()
        .replace("Alpha geht durch den Raum und tut Ding 1.", "Alpha ruht.");
    s.set_script_text(&text);
    s.save(klappe_services::OnOutside::Ask).unwrap();

    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-version-pdf.pdf");
    for (flag, marked) in [("--revisions", true), ("--header", false)] {
        let output = cli(&["script-pdf", flag, "--out", out.to_str().unwrap()], &dir);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let pdf = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
        assert_eq!(pdf.contains("(*) Tj"), marked);
        assert_eq!(pdf.contains("(Blue Revision"), marked);
    }
}

#[test]
fn a_version_is_removed_and_the_next_colour_follows_the_last() {
    let dir = project("cli-version-remove");
    for name in ["Eins", "Zwei"] {
        stdout(&cli(&["lock-version", "--name", name], &dir));
    }
    let listed = stdout(&cli(&["versions"], &dir));
    let first = listed
        .lines()
        .next()
        .unwrap()
        .split('\t')
        .next()
        .unwrap()
        .to_owned();

    let removed = stdout(&cli(&["remove-version", "--id", &first], &dir));
    assert!(removed.contains(&first), "{removed}");
    assert!(!cli(&["remove-version", "--id", &first], &dir)
        .status
        .success());

    stdout(&cli(&["lock-version", "--name", "Drei"], &dir));
    let colours: Vec<String> = stdout(&cli(&["versions"], &dir))
        .lines()
        .map(|l| l.split('\t').nth(3).unwrap().to_owned())
        .collect();
    assert_eq!(colours, ["Blue", "Pink"]);
}
