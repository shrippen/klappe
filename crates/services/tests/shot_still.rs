//! A still of the camera view as storyboard picture (LH-75), on the example project: shot
//! `s12-2` is camera `a` from 6 s to 14 s.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, Lock, Session};

const SHOT: &str = "s12-2";
const SHOT_START_S: f64 = 6.0;
const INSIDE_S: f64 = 10.0;
const OUTSIDE_S: f64 = 18.0;
const STILL_FILE: &str = "bilder/standbild-s12-2.png";
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// Where the width sits in a PNG file.
const PNG_WIDTH_AT: usize = 16;
const EXPORT_WIDTH_PX: u32 = 1920;

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

fn open(name: &str) -> (Session, PathBuf) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe"),
        &dir,
    );

    (Session::open(&dir).unwrap(), dir)
}

fn image_of(s: &Session) -> String {
    s.shots().into_iter().find(|r| r.id == SHOT).unwrap().image
}

#[test]
fn the_still_is_written_set_and_undone_in_one_step() {
    let (mut s, dir) = open("still-basic");
    s.set_time(INSIDE_S);

    s.shot_still(SHOT).unwrap();

    assert_eq!(image_of(&s), STILL_FILE);
    let bytes = std::fs::read(dir.join(STILL_FILE)).unwrap();
    assert!(bytes.starts_with(&PNG_SIGNATURE));
    let width = u32::from_be_bytes(bytes[PNG_WIDTH_AT..PNG_WIDTH_AT + 4].try_into().unwrap());
    assert_eq!(width, EXPORT_WIDTH_PX);

    s.undo();
    assert_eq!(image_of(&s), "");
    assert!(dir.join(STILL_FILE).exists());

    s.redo();
    assert_eq!(image_of(&s), STILL_FILE);
}

#[test]
fn the_same_view_gives_the_same_bytes_and_one_file() {
    let (mut s, dir) = open("still-same");
    s.set_time(INSIDE_S);
    s.shot_still(SHOT).unwrap();
    let first = std::fs::read(dir.join(STILL_FILE)).unwrap();

    s.shot_still(SHOT).unwrap();

    assert_eq!(std::fs::read(dir.join(STILL_FILE)).unwrap(), first);
    assert_eq!(image_of(&s), STILL_FILE);
    assert!(!dir.join("bilder/standbild-s12-2-2.png").exists());
}

#[test]
fn another_view_keeps_the_old_file_and_takes_a_free_name() {
    let (mut s, dir) = open("still-free");
    s.set_time(INSIDE_S);
    s.shot_still(SHOT).unwrap();
    let first = std::fs::read(dir.join(STILL_FILE)).unwrap();

    s.set_time(SHOT_START_S);
    s.shot_still(SHOT).unwrap();

    assert_eq!(std::fs::read(dir.join(STILL_FILE)).unwrap(), first);
    assert_eq!(image_of(&s), "bilder/standbild-s12-2-2.png");
}

#[test]
fn outside_the_shot_the_still_is_the_shots_start() {
    let (mut s, dir) = open("still-outside");
    s.set_time(SHOT_START_S);
    s.shot_still(SHOT).unwrap();
    let at_start = std::fs::read(dir.join(STILL_FILE)).unwrap();

    s.set_time(OUTSIDE_S);
    s.shot_still(SHOT).unwrap();

    assert_eq!(image_of(&s), STILL_FILE);
    assert_eq!(std::fs::read(dir.join(STILL_FILE)).unwrap(), at_start);
}

#[test]
fn a_locked_camera_layer_or_an_unknown_shot_refuses() {
    let (mut s, _) = open("still-refused");
    assert!(s.shot_still("nope").is_err());

    s.set_lock(Layer::Cameras, Lock::Locked);

    assert!(s.shot_still(SHOT).is_err());
    assert_eq!(image_of(&s), "");
}
