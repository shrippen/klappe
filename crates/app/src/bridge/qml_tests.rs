//! Runs the QML scenarios in `tests/controller` against the real controller, offscreen: the
//! controller cannot be built from Rust, QML makes it. A scenario ends with `Qt.exit(0)` when
//! it holds and `Qt.exit(1)` (after `console.error`) when it does not.

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::path::{Path, PathBuf};

fn scenarios() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/controller");
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "qml"))
        .collect();
    found.sort();

    found
}

#[test]
fn controller_scenarios_hold() {
    std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    let mut app = QGuiApplication::new();
    let mut failed = Vec::new();

    for file in scenarios() {
        let mut engine = QQmlApplicationEngine::new();
        let url = QUrl::from(&QString::from(format!("file://{}", file.display())));
        if let Some(mut engine) = engine.as_mut() {
            engine.as_mut().load(&url);
        }

        let code = app.as_mut().map_or(1, |a| a.exec());
        if code != 0 {
            failed.push(file);
        }
    }

    assert!(failed.is_empty(), "scenarios failed: {failed:?}");
}
