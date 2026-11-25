//! `klappe-cli report` on the example project (`testdata/beispiel.klappe`). The files go to
//! cargo's per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn out_file(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_file(&path);

    path
}

fn report(kind: &str, out: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["report", "--project", EXAMPLE, "--report", kind])
        .args(["--language", "de", "--out"])
        .arg(out)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn the_scene_report_is_a_csv() {
    let out = out_file("cli-report-szenen.csv");

    let output = report("szenen", &out);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Nummer,Kopf,Länge (Seiten),Figuren\r\n"));
    assert!(text.lines().count() > 1);
}

#[test]
fn the_contact_list_is_a_csv() {
    let out = out_file("cli-report-kontakte.csv");

    let output = report("kontakte", &out);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Name,Department,Funktion,Telefon,Mail,Adresse\r\n"));
}

#[test]
fn every_report_is_a_pdf() {
    for kind in ["szenen", "spielorte", "figuren", "auszug", "kontakte"] {
        let out = out_file(&format!("cli-report-{kind}.pdf"));

        let output = report(kind, &out);

        assert!(output.status.success(), "{kind}: {}", stderr(&output));
        assert!(std::fs::read(&out).unwrap().starts_with(b"%PDF"), "{kind}");
    }
}

#[test]
fn a_workbook_is_written_by_its_extension() {
    for extension in ["ods", "xlsx"] {
        let out = out_file(&format!("cli-report.{extension}"));

        let output = report("spielorte", &out);

        assert!(output.status.success(), "{}", stderr(&output));
        assert!(std::fs::read(&out).unwrap().starts_with(b"PK"));
    }
}

#[test]
fn an_unknown_report_is_refused() {
    let out = out_file("cli-report-none.pdf");

    let output = report("kosten", &out);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("szenen"));
    assert!(!out.exists());
}

#[test]
fn an_unknown_extension_is_refused_in_german() {
    let out = out_file("cli-report.png");

    let output = report("szenen", &out);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: Das Format von"));
    assert!(!out.exists());
}
