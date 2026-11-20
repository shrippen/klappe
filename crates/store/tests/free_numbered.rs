//! The first free `<stem>-<N><ext>` in a folder.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_store::free_numbered;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

#[test]
fn an_empty_or_missing_folder_gives_one() {
    let empty = scratch("free-numbered-empty");
    fs::create_dir_all(&empty).unwrap();
    let missing = scratch("free-numbered-missing");

    assert_eq!(
        free_numbered(&empty, "unbenannt", ".klappe").unwrap(),
        (1, empty.join("unbenannt-1.klappe"))
    );
    assert_eq!(
        free_numbered(&missing, "unbenannt", ".klappe").unwrap(),
        (1, missing.join("unbenannt-1.klappe"))
    );
}

#[test]
fn the_first_gap_is_taken() {
    let dir = scratch("free-numbered-gap");
    fs::create_dir_all(dir.join("unbenannt-1.klappe")).unwrap();
    fs::create_dir_all(dir.join("unbenannt-3.klappe")).unwrap();
    fs::create_dir_all(dir.join("untitled-1.klappe")).unwrap();

    let (n, path) = free_numbered(&dir, "unbenannt", ".klappe").unwrap();

    assert_eq!(n, 2);
    assert_eq!(path, dir.join("unbenannt-2.klappe"));
}
