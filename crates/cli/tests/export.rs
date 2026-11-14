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
    assert!(std::fs::read(&out).unwrap().starts_with(b"%PDF-"));
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

    let output = export(&out, &["--plan", "nope"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: no plan nope"));
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
