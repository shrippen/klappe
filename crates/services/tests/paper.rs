//! The export sheet: written to the file, kept in the project, not an undo step.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    ExportFormat, ExportOptions, Language, Orientation, Paper, PaperSetup, Session, ShotOrder,
};

const A4_PORTRAIT_MEDIA_BOX: &str = "/MediaBox [0 0 595.28 841.89]";
const LETTER_PORTRAIT: PaperSetup = PaperSetup {
    size: Paper::Letter,
    orientation: Orientation::Portrait,
};

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
fn a_portrait_pdf_has_the_upright_media_box() {
    let s = Session::open(&copy("paper-pdf")).unwrap();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("paper-portrait.pdf");
    let setup = PaperSetup {
        size: Paper::A4,
        orientation: Orientation::Portrait,
    };

    s.export_pdf(
        &path,
        setup,
        Language::English.print_texts(),
        ShotOrder::Cut,
    )
    .unwrap();

    let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
    assert!(text.contains(A4_PORTRAIT_MEDIA_BOX), "{text}");
}

#[test]
fn the_paper_is_kept_in_the_project() {
    let dir = copy("paper-kept");
    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.project().paper, PaperSetup::default());
    assert!(!s.is_dirty());

    s.set_paper(LETTER_PORTRAIT);
    assert!(s.is_dirty());
    assert!(!s.can_undo());
    s.save().unwrap();

    assert_eq!(
        Session::open(&dir).unwrap().project().paper,
        LETTER_PORTRAIT
    );
}

#[test]
fn the_same_paper_changes_nothing() {
    let mut s = Session::open(&copy("paper-same")).unwrap();

    s.set_paper(PaperSetup::default());

    assert!(!s.is_dirty());
}

/// Undo takes back the edit, not the sheet chosen after it.
#[test]
fn undo_and_redo_leave_the_paper() {
    let mut s = Session::open(&copy("paper-undo")).unwrap();
    s.add_shot(None).unwrap();

    s.set_paper(LETTER_PORTRAIT);
    s.undo();
    assert_eq!(s.project().paper, LETTER_PORTRAIT);
    s.redo();
    assert_eq!(s.project().paper, LETTER_PORTRAIT);
}

#[test]
fn the_overview_lists_the_sizes_and_the_sheet() {
    let mut s = Session::open(&copy("paper-overview")).unwrap();
    s.set_paper(LETTER_PORTRAIT);

    let overview = serde_json::to_value(s.overview()).unwrap();

    assert_eq!(overview["paper_size"], "letter");
    assert_eq!(overview["paper_orientation"], "portrait");
    let ids: Vec<&str> = overview["papers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["a4", "a3", "letter"]);
    assert_eq!(
        overview["orientations"],
        serde_json::json!(["portrait", "landscape"])
    );
}

fn options(paper: PaperSetup) -> ExportOptions {
    ExportOptions {
        paper,
        order: ShotOrder::Cut,
        texts: Language::German.print_texts(),
        shown: (800.0, 450.0),
        camera: Some("a".to_owned()),
    }
}

#[test]
fn the_overview_lists_what_each_export_needs() {
    let s = Session::open(&copy("paper-formats")).unwrap();

    let overview = serde_json::to_value(s.overview()).unwrap();

    let formats = overview["formats"].as_array().unwrap();
    let ids: Vec<&str> = formats.iter().map(|f| f["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        [
            "pdf",
            "png",
            "svg",
            "csv",
            "ods",
            "xlsx",
            "frames",
            "view3d",
            "camera",
            "shotpages",
            "camerapages",
            "shotfiles",
            "camerafiles",
            "mp4"
        ]
    );
    let on_paper: Vec<&str> = formats
        .iter()
        .filter(|f| f["paper"] == true)
        .map(|f| f["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        on_paper,
        [
            "pdf",
            "png",
            "svg",
            "frames",
            "shotpages",
            "camerapages",
            "shotfiles",
            "camerafiles"
        ]
    );
    let extensions: Vec<_> = formats.iter().map(|f| f["extension"].clone()).collect();
    assert_eq!(extensions[6], serde_json::Value::Null);
    assert_eq!(extensions[7], "png");
    assert_eq!(extensions[8], "png");
    assert_eq!(extensions[3], "csv");
    // One PDF per shot or camera is a file, one PNG per shot or camera a folder (LH-87).
    assert_eq!(extensions[9], "pdf");
    assert_eq!(extensions[10], "pdf");
    assert_eq!(extensions[11], serde_json::Value::Null);
    assert_eq!(extensions[12], serde_json::Value::Null);
    assert_eq!(extensions[13], "mp4");
}

/// The plan printed on a sheet makes it the project's; lists and views of it do not.
#[test]
fn an_export_keeps_the_paper_it_printed_on() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("paper-export-out");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let mut s = Session::open(&copy("paper-export")).unwrap();
    s.set_time(6.0);
    s.toggle_view();

    // The video draws 500 frames and needs ffmpeg: `video.rs` tests it.
    for format in ExportFormat::ALL {
        if format == ExportFormat::Video {
            continue;
        }
        let target = out.join(format.name());
        s.set_paper(PaperSetup::default());

        s.export(format, &target, &options(LETTER_PORTRAIT))
            .unwrap();

        let expected = if format.needs_paper() {
            LETTER_PORTRAIT
        } else {
            PaperSetup::default()
        };
        assert_eq!(s.project().paper, expected, "{}", format.name());
    }
}

#[test]
fn a_failed_export_keeps_the_paper() {
    let dir = copy("paper-failed");
    let mut s = Session::open(&dir).unwrap();
    // A file is no folder: nothing can be written below it.
    let target = dir.join("projekt.toml").join("plan.pdf");

    let result = s.export(ExportFormat::Pdf, &target, &options(LETTER_PORTRAIT));

    assert!(result.is_err());
    assert_eq!(s.project().paper, PaperSetup::default());
}
