//! The backup folder when a write of it crashed (LH-05): what a load takes and what it leaves.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::Project;
use klappe_store::{drop_backup, load, load_backup, save, save_backup};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// The checked-in example (`store.rs` checks that it is the one the tests build).
fn example() -> Project {
    load(Path::new(EXAMPLE)).unwrap().project
}

fn changed(title: &str) -> Project {
    let mut project = example();
    project.title = title.into();
    project
}

#[test]
fn a_crashed_backup_write_is_ignored() {
    let dir = test_dir("backup-crash-unfinished");
    save(&dir, &example(), None).unwrap();
    save_backup(&dir, &changed("Alt"), None).unwrap();
    // A write that died before the marker.
    save(&dir.join(".sicherung.neu"), &changed("Halb"), None).unwrap();

    let backup = load_backup(&dir).unwrap().unwrap();

    assert_eq!(backup.project.title, "Alt");
}

#[test]
fn an_unfinished_write_alone_is_no_backup() {
    let dir = test_dir("backup-crash-alone");
    save(&dir, &example(), None).unwrap();
    save(&dir.join(".sicherung.neu"), &changed("Halb"), None).unwrap();

    assert_eq!(load_backup(&dir).unwrap(), None);
}

#[test]
fn a_finished_write_without_old_backup_is_loaded() {
    let dir = test_dir("backup-crash-finished");
    save(&dir, &example(), None).unwrap();
    // Crash after the old backup was removed, before the rename.
    let new = dir.join(".sicherung.neu");
    save(&new, &changed("Neu"), None).unwrap();
    fs::write(new.join("fertig"), "").unwrap();

    let backup = load_backup(&dir).unwrap().unwrap();

    assert_eq!(backup.project.title, "Neu");
}

#[test]
fn a_finished_write_beats_the_old_backup() {
    let dir = test_dir("backup-crash-both");
    save(&dir, &example(), None).unwrap();
    save_backup(&dir, &changed("Alt"), None).unwrap();
    let new = dir.join(".sicherung.neu");
    save(&new, &changed("Neu"), None).unwrap();
    fs::write(new.join("fertig"), "").unwrap();

    assert_eq!(load_backup(&dir).unwrap().unwrap().project.title, "Neu");
}

#[test]
fn save_backup_clears_leftovers_and_drop_removes_both() {
    let dir = test_dir("backup-leftover");
    save(&dir, &example(), None).unwrap();
    let new = dir.join(".sicherung.neu");
    save(&new, &changed("Halb"), None).unwrap();

    save_backup(&dir, &changed("Ganz"), None).unwrap();

    assert!(!new.exists());
    assert_eq!(load_backup(&dir).unwrap().unwrap().project.title, "Ganz");
    save(&new, &changed("Halb"), None).unwrap();
    drop_backup(&dir).unwrap();
    assert!(!new.exists() && !dir.join(".sicherung").exists());
}

#[test]
fn a_backup_folder_without_marker_is_not_read() {
    let dir = test_dir("backup-no-marker");
    save(&dir, &example(), None).unwrap();
    save_backup(&dir, &changed("Alt"), None).unwrap();
    // What a crash in the middle of a delete leaves: no marker, parts gone.
    fs::remove_file(dir.join(".sicherung/fertig")).unwrap();
    fs::remove_dir_all(dir.join(".sicherung/szenen")).unwrap();

    assert_eq!(load_backup(&dir).unwrap(), None);
}

/// A delete that fails midway must leave no readable backup.
#[cfg(unix)]
#[test]
fn a_failed_drop_leaves_no_valid_backup() {
    use std::os::unix::fs::PermissionsExt;

    let dir = test_dir("backup-drop-fails");
    save(&dir, &example(), None).unwrap();
    save_backup(&dir, &changed("Alt"), None).unwrap();
    let scenes = dir.join(".sicherung/szenen");
    fs::set_permissions(&scenes, fs::Permissions::from_mode(0o500)).unwrap();
    // Root may delete in a read-only folder; then no delete can be made to fail here.
    if fs::write(scenes.join("probe"), "").is_ok() {
        fs::set_permissions(&scenes, fs::Permissions::from_mode(0o700)).unwrap();
        return;
    }

    let dropped = drop_backup(&dir);
    fs::set_permissions(&scenes, fs::Permissions::from_mode(0o700)).unwrap();

    assert!(dropped.is_err());
    assert_eq!(load_backup(&dir).unwrap(), None);
}

/// Crash between removing `.sicherung` and the rename leaves only a finished `.neu`; the
/// next write must not delete it before its own is finished.
#[test]
fn a_finished_leftover_survives_a_failing_write() {
    let dir = test_dir("backup-double-crash");
    save(&dir, &example(), None).unwrap();
    let new = dir.join(".sicherung.neu");
    save(&new, &changed("Gerettet"), None).unwrap();
    fs::write(new.join("fertig"), "").unwrap();
    let mut bad = example();
    bad.locations[0].id = "..".into();

    assert!(save_backup(&dir, &bad, None).is_err());

    assert_eq!(
        load_backup(&dir).unwrap().unwrap().project.title,
        "Gerettet"
    );
}
