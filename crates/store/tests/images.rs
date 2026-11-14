// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_store::{add_image, image_size, StoreError};

/// A PNG header of `w` x `h` pixels followed by `tag`, so that files differ in content.
fn png(w: u32, h: u32, tag: &[u8]) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(&13_u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(tag);
    b
}

fn jpeg(w: u16, h: u16) -> Vec<u8> {
    let mut b = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08];
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&[0x01, 0x01, 0x11, 0x00]);
    b
}

/// A fresh folder for one test: the project and a place to take images from.
fn test_dirs(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&root);
    let (project, source) = (root.join("projekt.klappe"), root.join("quelle"));
    fs::create_dir_all(&source).unwrap();

    (project, source)
}

fn file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, bytes).unwrap();

    path
}

#[test]
fn copies_into_bilder_and_reads_the_size() {
    let (project, source) = test_dirs("img-copy");
    let saal = file(&source, "saal.png", &png(1000, 640, b""));
    let hof = file(&source, "hof.JPG", &jpeg(800, 600));

    assert_eq!(add_image(&project, &saal).unwrap(), "bilder/saal.png");
    assert_eq!(add_image(&project, &hof).unwrap(), "bilder/hof.JPG");

    assert!(project.join("bilder/saal.png").is_file());
    assert_eq!(
        image_size(&project, "bilder/saal.png").unwrap(),
        (1000, 640)
    );
    assert_eq!(image_size(&project, "bilder/hof.JPG").unwrap(), (800, 600));
}

#[test]
fn same_content_is_not_copied_twice() {
    let (project, source) = test_dirs("img-same");
    let saal = file(&source, "saal.png", &png(10, 10, b""));

    let first = add_image(&project, &saal).unwrap();
    let second = add_image(&project, &saal).unwrap();

    assert_eq!(first, second);
    assert_eq!(fs::read_dir(project.join("bilder")).unwrap().count(), 1);
}

#[test]
fn other_content_under_the_same_name_is_numbered() {
    let (project, source) = test_dirs("img-clash");
    let other = source.join("andere");
    fs::create_dir_all(&other).unwrap();
    let a = file(&source, "saal.png", &png(10, 10, b"a"));
    let b = file(&other, "saal.png", &png(10, 10, b"b"));
    let c = file(&source.join("andere"), "saal.jpg", &jpeg(5, 5));

    assert_eq!(add_image(&project, &a).unwrap(), "bilder/saal.png");
    assert_eq!(add_image(&project, &b).unwrap(), "bilder/saal-2.png");
    // Again: the second one is already there, no third copy.
    assert_eq!(add_image(&project, &b).unwrap(), "bilder/saal-2.png");
    assert_eq!(add_image(&project, &c).unwrap(), "bilder/saal.jpg");

    let d = file(&other, "saal.png", &png(10, 10, b"d"));
    assert_eq!(add_image(&project, &d).unwrap(), "bilder/saal-3.png");
}

#[test]
fn other_files_are_refused() {
    let (project, source) = test_dirs("img-refused");
    let gif = file(&source, "saal.gif", &png(10, 10, b""));
    let fake = file(&source, "saal.png", b"not an image");

    assert!(matches!(
        add_image(&project, &gif),
        Err(StoreError::BadImage { .. })
    ));
    assert!(matches!(
        add_image(&project, &fake),
        Err(StoreError::BadImage { .. })
    ));
    assert!(!project.join("bilder").exists());
}

#[test]
fn size_of_a_missing_or_outside_file_is_an_error() {
    let (project, _) = test_dirs("img-size");

    assert!(matches!(
        image_size(&project, "bilder/fehlt.png"),
        Err(StoreError::Io { .. })
    ));
    assert!(matches!(
        image_size(&project, "../quelle/x.png"),
        Err(StoreError::BadId { .. })
    ));
}

#[test]
fn a_jpeg_cut_off_in_its_frame_header_is_refused() {
    let (project, source) = test_dirs("img-cut-jpeg");
    // Ends after the width: the export could not read its colour model.
    let head = jpeg(800, 600);
    let cut = file(&source, "saal.jpg", &head[..head.len() - 4]);

    assert!(matches!(
        add_image(&project, &cut),
        Err(StoreError::BadImage { .. })
    ));
}
