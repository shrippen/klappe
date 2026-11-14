// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use klappe_store::{is_empty_dir, project_mtime, remove_prefixed, write_atomic, write_tree};

fn fresh(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn set_mtime(path: &Path, age_s: u64) {
    let time = SystemTime::now() - Duration::from_secs(age_s);
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(time)
        .unwrap();
}

#[test]
fn write_atomic_makes_folders_and_leaves_no_temp_file() {
    let dir = fresh("tree-atomic");
    let path = dir.join("a/b.pdf");

    write_atomic(&path, b"one").unwrap();
    write_atomic(&path, b"two").unwrap();

    assert_eq!(fs::read(&path).unwrap(), b"two");
    assert_eq!(fs::read_dir(dir.join("a")).unwrap().count(), 1);
}

#[test]
fn a_missing_or_empty_folder_is_empty() {
    let dir = fresh("tree-empty");
    assert!(is_empty_dir(&dir).unwrap());

    fs::create_dir_all(&dir).unwrap();
    assert!(is_empty_dir(&dir).unwrap());

    fs::write(dir.join("x"), "x").unwrap();
    assert!(!is_empty_dir(&dir).unwrap());
}

#[test]
fn a_file_is_no_folder_to_list() {
    let dir = fresh("tree-notdir");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f");
    fs::write(&file, "x").unwrap();

    assert!(is_empty_dir(&file).is_err());
}

#[test]
fn a_tree_is_written_whole() {
    let dir = fresh("tree-write");

    write_tree(&dir, &[("a.txt", b"a"), ("sub/b.txt", b"b")]).unwrap();

    assert_eq!(fs::read(dir.join("a.txt")).unwrap(), b"a");
    assert_eq!(fs::read(dir.join("sub/b.txt")).unwrap(), b"b");
}

#[test]
fn a_tree_replaces_an_empty_folder_and_a_cut_off_copy() {
    let dir = fresh("tree-cut");
    let beside = dir.with_file_name("tree-cut.part");
    let _ = fs::remove_dir_all(&beside);
    fs::create_dir_all(beside.join("szenen")).unwrap();
    fs::write(beside.join("old.txt"), "halb").unwrap();
    fs::create_dir_all(&dir).unwrap();

    write_tree(&dir, &[("a.txt", b"a")]).unwrap();

    assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    assert!(!beside.exists());
}

#[test]
fn remove_prefixed_keeps_other_files() {
    let dir = fresh("tree-prefix");
    fs::create_dir_all(&dir).unwrap();
    for name in ["ab-1.png", "ab-2.png", "cd-1.png"] {
        fs::write(dir.join(name), "x").unwrap();
    }

    remove_prefixed(&dir, "ab-");
    remove_prefixed(&dir.join("missing"), "ab-");

    assert!(!dir.join("ab-1.png").exists());
    assert!(!dir.join("ab-2.png").exists());
    assert!(dir.join("cd-1.png").exists());
}

#[test]
fn the_mtime_is_the_newest_file_and_skips_the_backup() {
    let dir = fresh("tree-mtime");
    fs::create_dir_all(dir.join("szenen")).unwrap();
    fs::create_dir_all(dir.join(".sicherung")).unwrap();
    fs::write(dir.join("projekt.toml"), "x").unwrap();
    fs::write(dir.join("szenen/s.toml"), "x").unwrap();
    fs::write(dir.join(".sicherung/projekt.toml"), "x").unwrap();
    set_mtime(&dir.join("projekt.toml"), 3000);
    set_mtime(&dir.join("szenen/s.toml"), 2000);
    set_mtime(&dir.join(".sicherung/projekt.toml"), 10);

    let newest = project_mtime(&dir).unwrap();

    let expected = fs::metadata(dir.join("szenen/s.toml"))
        .unwrap()
        .modified()
        .unwrap();
    assert_eq!(newest, expected);
    assert!(project_mtime(&dir.join("missing")).is_none());
}
