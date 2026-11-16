//! `klappe-cli export` on the example project (`testdata/beispiel.klappe`). The files go to
//! cargo's per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn out_file(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    path
}

/// `klappe-cli export` on the example, scene 12, with `extra` arguments.
fn export(out: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["export", "--project", EXAMPLE, "--scene", "12", "--out"])
        .arg(out)
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn exports_a_png_by_extension() {
    let out = out_file("cli-plan.png");

    let output = export(&out, &["--at", "6", "--language", "de"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(PNG_SIGNATURE));
}

#[test]
fn the_format_option_beats_the_extension() {
    let out = out_file("cli-plan.dat");

    let output = export(&out, &["--format", "svg", "--language", "en"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("<svg"));
    assert!(text.contains("Axis"));
}

#[test]
fn exports_a_pdf_on_letter_paper() {
    let out = out_file("cli-plan.pdf");

    let output = export(&out, &["--paper", "letter"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
    assert!(text.starts_with("%PDF-"));
    assert!(text.contains("/MediaBox [0 0 792 612]"));
}

#[test]
fn exports_a_portrait_pdf() {
    let out = out_file("cli-portrait.pdf");

    let output = export(&out, &["--paper", "a4", "--orientation", "portrait"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
    assert!(text.contains("/MediaBox [0 0 595.28 841.89]"));
    assert!(
        !text.contains("/MediaBox [0 0 841.89 595.28]"),
        "the shot list is upright too"
    );
}

#[test]
fn an_unknown_paper_is_refused() {
    let out = out_file("cli-tabloid.pdf");

    let output = export(&out, &["--paper", "tabloid"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("use one of a4, a3, letter"));
    assert!(!out.exists());
}

#[test]
fn an_unknown_scene_fails_without_a_file() {
    let out = out_file("cli-none.png");
    let output = Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["export", "--project", EXAMPLE, "--scene", "99", "--out"])
        .arg(&out)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: no scene 99"));
    assert!(!out.exists());
}

#[test]
fn an_unknown_plan_fails() {
    let out = out_file("cli-noplan.png");

    let output = export(&out, &["--plan", "nope", "--language", "en"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: Plan not found: nope"));
}

/// Audit 3, finding 1: the CLI says what the app says, in its `--language`.
#[test]
fn a_service_error_uses_the_catalogue_text() {
    let out = out_file("cli-noplan-de.png");

    let output = export(&out, &["--plan", "nope", "--language", "de"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: Plan nicht gefunden: nope"));
}

#[test]
fn an_unknown_extension_asks_for_a_format() {
    let out = out_file("cli-plan.jpg");

    let output = export(&out, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("use --format"));
}

#[test]
fn a_negative_time_is_refused() {
    let out = out_file("cli-neg.png");

    let output = export(&out, &["--at=-1"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("time must be"));
}

#[test]
fn exports_the_shot_list_as_ods_and_xlsx_by_extension() {
    for name in ["cli-list.ods", "cli-list.xlsx"] {
        let out = out_file(name);

        let output = export(&out, &["--language", "de"]);

        assert!(output.status.success(), "{}", stderr(&output));
        let bytes = std::fs::read(&out).unwrap();
        assert_eq!(&bytes[..2], b"PK", "{name}");
        assert!(
            String::from_utf8_lossy(&bytes).contains("Shotliste"),
            "{name}"
        );
    }
}

#[test]
fn exports_one_picture_per_mark_into_a_folder() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-frames");
    let _ = std::fs::remove_dir_all(&dir);

    let output = export(&dir, &["--format", "frames"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names.len(), 5, "{names:?}");
    assert_eq!(names[0], "12-01-00-00.png");
    assert_eq!(names[4], "12-05-00-20.png");
    let first = std::fs::read(dir.join(&names[0])).unwrap();
    assert!(first.starts_with(PNG_SIGNATURE));
}
