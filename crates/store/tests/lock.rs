//! The Sperrstand of the scene numbers: `nummern.toml` (DB-33).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::LockedNumber;
use klappe_store::{load_lock, save_lock, StoreError};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

fn number(number: &str, id: &str) -> LockedNumber {
    LockedNumber {
        number: number.into(),
        id: id.into(),
    }
}

#[test]
fn a_missing_file_is_no_lock() {
    let dir = test_dir("lock-missing");

    assert_eq!(load_lock(&dir).unwrap(), None);
}

#[test]
fn the_lock_keeps_its_order_and_a_struck_heading_has_no_id() {
    let dir = test_dir("lock-roundtrip");
    let numbers = vec![
        number("12", "aaaa"),
        number("12A", ""),
        number("13", "bbbb"),
    ];

    save_lock(&dir, &numbers).unwrap();

    let text = fs::read_to_string(dir.join("nummern.toml")).unwrap();
    assert_eq!(text.matches("id =").count(), 2, "{text}");
    assert_eq!(load_lock(&dir).unwrap(), Some(numbers));
}

#[test]
fn an_empty_lock_is_a_lock() {
    let dir = test_dir("lock-empty");

    save_lock(&dir, &[]).unwrap();

    assert_eq!(load_lock(&dir).unwrap(), Some(Vec::new()));
}

#[test]
fn conflict_marks_are_refused_with_their_line() {
    let dir = test_dir("lock-conflict");
    fs::write(
        dir.join("nummern.toml"),
        "[[nummer]]\nnumber = \"1\"\n<<<<<<< HEAD\n",
    )
    .unwrap();

    let error = load_lock(&dir).unwrap_err();

    assert!(
        matches!(error, StoreError::Conflict { line: 3, .. }),
        "{error}"
    );
}
