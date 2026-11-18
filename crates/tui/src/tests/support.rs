//! A project folder of its own for a test, with the screenplay it needs.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A copy of the example project in the temporary folder, removed when dropped.
pub(super) struct Scratch {
    pub dir: PathBuf,
}

fn copy_all(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| unreachable!("{e}"));
    for entry in fs::read_dir(from).unwrap_or_else(|e| unreachable!("{e}")) {
        let entry = entry.unwrap_or_else(|e| unreachable!("{e}"));
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_all(&entry.path(), &target);
            continue;
        }
        fs::copy(entry.path(), target).unwrap_or_else(|e| unreachable!("{e}"));
    }
}

impl Scratch {
    pub(super) fn new(script: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::named(script, &format!("kt-{}-{n}.klappe", std::process::id()))
    }

    /// Like `new`, in a folder of the given name.
    pub(super) fn named(script: &str, name: &str) -> Self {
        // A folder per process, so two test runs in one temp dir never remove each other's.
        let dir = std::env::temp_dir()
            .join(format!("kt-{}", std::process::id()))
            .join(name);
        copy_all(&example(), &dir);
        fs::write(dir.join("drehbuch.fountain"), script).unwrap_or_else(|e| unreachable!("{e}"));

        Self { dir }
    }

    pub(super) fn app(&self) -> App {
        App::open(&self.dir, Language::English).unwrap_or_else(|e| unreachable!("{e}"))
    }

    pub(super) fn script(&self) -> String {
        fs::read_to_string(self.dir.join("drehbuch.fountain")).unwrap_or_default()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// An app on a screenplay of the given text, the cursor at its end.
pub(super) fn app_with(script: &str) -> (Scratch, App) {
    let scratch = Scratch::new(script);
    let mut app = scratch.app();
    let _ = screen(&mut app, 100, 24);
    ctrl_end(&mut app);

    (scratch, app)
}

/// Cursor to the end of the text.
pub(super) fn ctrl_end(app: &mut App) {
    for _ in 0..app.text().lines().count() + 2 {
        press(app, KeyCode::Down);
    }
    press(app, KeyCode::End);
}

pub(super) fn keys(app: &mut App, codes: &[KeyCode]) {
    for code in codes {
        press(app, *code);
    }
}

pub(super) fn shift(app: &mut App, code: KeyCode) -> Flow {
    let flow = app.handle_key(KeyEvent::new(code, KeyModifiers::SHIFT));
    assert_backup_answered(app);

    flow
}
