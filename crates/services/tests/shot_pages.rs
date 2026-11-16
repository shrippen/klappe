//! Output with one page per shot or per camera (LH-87).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{ExportFormat, ExportOptions, Language, Pages, PaperSetup, Session};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open(name: &str) -> (Session, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    (Session::open(&dir).unwrap(), dir)
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

fn texts() -> klappe_services::Texts {
    Language::English.print_texts()
}

/// The number of pages of a PDF: its page objects.
fn pages_in(pdf: &[u8]) -> usize {
    String::from_utf8_lossy(pdf).matches("/Type /Page ").count()
}

#[test]
fn one_pdf_page_per_shot() {
    let (mut s, dir) = open("pages-shot");
    let out = dir.join("shots.pdf");

    s.export_pages(&out, PaperSetup::default(), &texts(), Pages::PerShot)
        .unwrap();

    let shots = s.plan().unwrap().shots.len();
    assert_eq!(shots, 5);
    assert_eq!(pages_in(&std::fs::read(&out).unwrap()), shots);
}

#[test]
fn one_pdf_page_per_camera() {
    let (mut s, dir) = open("pages-camera");
    let out = dir.join("cameras.pdf");

    s.export_pages(&out, PaperSetup::default(), &texts(), Pages::PerCamera)
        .unwrap();

    assert_eq!(pages_in(&std::fs::read(&out).unwrap()), 2);
}

#[test]
fn a_png_folder_has_one_picture_per_shot_named_by_it() {
    let (mut s, dir) = open("pages-files");
    let out = dir.join("pictures");
    s.set_time(3.0);

    let files = s
        .export_page_files(&out, PaperSetup::default(), &texts(), Pages::PerShot)
        .unwrap();

    assert_eq!(files.len(), 5);
    assert_eq!(
        files[0].file_name().unwrap().to_string_lossy(),
        "12-01-12_1.png"
    );
    for file in &files {
        assert!(std::fs::read(file).unwrap().starts_with(b"\x89PNG"));
    }
    // The playhead stays where it was.
    assert_eq!(s.time_s(), 3.0);
}

/// The camera of the page is strong, the rest is paler: the picture differs from the plain
/// plan, and the page of another camera differs from it.
#[test]
fn a_page_shows_its_camera_strong_and_the_rest_faded() {
    let (mut s, dir) = open("pages-fade");
    let plain = dir.join("plain.png");
    let a = dir.join("a");
    let b = dir.join("b");
    s.set_time(0.0);
    s.export_png(&plain, PaperSetup::default(), texts())
        .unwrap();

    let by_camera = s
        .export_page_files(&a, PaperSetup::default(), &texts(), Pages::PerCamera)
        .unwrap();
    let by_shot = s
        .export_page_files(&b, PaperSetup::default(), &texts(), Pages::PerShot)
        .unwrap();

    let plain = std::fs::read(&plain).unwrap();
    let camera_a = std::fs::read(&by_camera[0]).unwrap();
    let camera_b = std::fs::read(&by_camera[1]).unwrap();
    assert_ne!(camera_a, plain);
    assert_ne!(camera_a, camera_b);
    assert_eq!(by_shot.len(), 5);
}

#[test]
fn nothing_to_write_pages_for_is_an_error() {
    let (mut s, dir) = open("pages-none");
    let id = s.add_plan("Leer").unwrap();
    assert!(!id.is_empty());
    s.plan().unwrap();
    // No script match, no shots, no cameras in the new plan.
    let out = dir.join("none.pdf");

    let result = s.export_pages(&out, PaperSetup::default(), &texts(), Pages::PerShot);

    assert!(matches!(
        result,
        Err(klappe_services::ServiceError::NoPages)
    ));
    assert!(!out.exists());
}

#[test]
fn the_formats_go_through_export() {
    let (mut s, dir) = open("pages-format");
    let options = ExportOptions {
        paper: PaperSetup::default(),
        order: klappe_services::ShotOrder::Cut,
        texts: texts(),
        shown: (0.0, 0.0),
        camera: None,
    };

    s.export(ExportFormat::ShotPages, &dir.join("a.pdf"), &options)
        .unwrap();
    s.export(ExportFormat::CameraFiles, &dir.join("files"), &options)
        .unwrap();

    assert!(dir.join("a.pdf").exists());
    assert_eq!(std::fs::read_dir(dir.join("files")).unwrap().count(), 2);
}
