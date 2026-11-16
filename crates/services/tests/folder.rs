//! Rename, archive and trash of a project folder from the start page, on a copy of the example
//! project. The trash is a folder of the test: `XDG_DATA_HOME` is set before the first call.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{rename_project, ServiceError, Session};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("folder")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
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

fn project(parent: &Path, name: &str) -> PathBuf {
    let dir = parent.join(name);
    copy_dir(&example(), &dir);
    dir
}

#[test]
fn a_rename_changes_the_title_and_nothing_else() {
    let parent = scratch("rename");
    let dir = project(&parent, "film.klappe");
    let script = Session::open(&dir).unwrap().script_text().to_owned();

    rename_project(&dir, "  Neuer Titel ").unwrap();

    let session = Session::open(&dir).unwrap();
    assert_eq!(session.project().title, "Neuer Titel");
    assert_eq!(session.script_text(), script);
    assert!(parent.join("film.klappe").is_dir());
}

#[test]
fn a_blank_title_is_refused_and_the_project_stays() {
    let dir = project(&scratch("rename-blank"), "film.klappe");
    let title = Session::open(&dir).unwrap().project().title.clone();

    let refused = rename_project(&dir, "  ");

    assert!(matches!(refused, Err(ServiceError::BadValue { .. })));
    assert_eq!(Session::open(&dir).unwrap().project().title, title);
}

/// Trash and archive: the trash is checked in its freedesktop folder (`XDG_DATA_HOME`), so on
/// Linux only.
#[cfg(target_os = "linux")]
mod trash {
    use std::sync::Once;

    use klappe_services::{archive_project, trash_project};

    use super::*;

    const LOCAL_SIGNATURE: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];

    /// Leads the system trash into the tests' own folder, on the same disk as the projects.
    fn own_trash() -> PathBuf {
        static ONCE: Once = Once::new();
        let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join("folder-trash");
        ONCE.call_once(|| {
            let _ = std::fs::remove_dir_all(&home);
            std::env::set_var("XDG_DATA_HOME", &home);
        });

        home.join("Trash/files")
    }

    /// The stored entries of a ZIP written by `zip_files`: name and bytes, in order.
    fn entries(zip: &[u8]) -> Vec<(String, Vec<u8>)> {
        let u16_at = |at: usize| usize::from(u16::from_le_bytes([zip[at], zip[at + 1]]));
        let u32_at = |at: usize| u32::from_le_bytes(zip[at..at + 4].try_into().unwrap()) as usize;
        let mut found = Vec::new();
        let mut at = 0;
        while zip[at..at + 4] == LOCAL_SIGNATURE {
            let size = u32_at(at + 18);
            let name_at = at + 30;
            let name_len = u16_at(at + 26);
            let data_at = name_at + name_len + u16_at(at + 28);
            let name = String::from_utf8(zip[name_at..name_at + name_len].to_vec()).unwrap();
            found.push((name, zip[data_at..data_at + size].to_vec()));
            at = data_at + size;
        }

        found
    }

    #[test]
    fn trashing_takes_the_folder_away_and_the_trash_has_it() {
        let trash = own_trash();
        let dir = project(&scratch("trash"), "wegdamit.klappe");

        trash_project(&dir).unwrap();

        assert!(!dir.exists());
        assert!(trash.join("wegdamit.klappe/projekt.toml").is_file());
    }

    #[test]
    fn an_archive_holds_every_file_of_the_folder_and_the_folder_goes() {
        let trash = own_trash();
        let parent = scratch("archive");
        let dir = project(&parent, "archiv.klappe");
        std::fs::create_dir_all(dir.join(".sicherung")).unwrap();
        std::fs::write(dir.join(".sicherung/fertig"), b"x").unwrap();
        let want = klappe_store::read_files(&dir).unwrap();

        let zip = archive_project(&dir).unwrap();

        assert_eq!(zip, parent.join("archiv.klappe.zip"));
        assert!(!dir.exists());
        assert!(trash.join("archiv.klappe").is_dir());
        let got = entries(&std::fs::read(&zip).unwrap());
        let want: Vec<_> = want
            .into_iter()
            .map(|(name, bytes)| (format!("archiv.klappe/{name}"), bytes))
            .collect();
        assert!(want.len() > 3);
        assert_eq!(got, want);
    }

    #[test]
    fn a_taken_archive_name_is_not_overwritten() {
        own_trash();
        let parent = scratch("archive-taken");
        std::fs::write(parent.join("zwei.klappe.zip"), b"old").unwrap();
        let dir = project(&parent, "zwei.klappe");

        let zip = archive_project(&dir).unwrap();

        assert_eq!(zip, parent.join("zwei.klappe (2).zip"));
        assert_eq!(
            std::fs::read(parent.join("zwei.klappe.zip")).unwrap(),
            b"old"
        );
    }
}
