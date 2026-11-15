//! Runs the QML scenarios in `tests/controller` against the real controller, offscreen: the
//! controller cannot be built from Rust, QML makes it. A scenario ends with `Qt.exit(0)` when
//! it holds and `Qt.exit(1)` (after `console.error`) when it does not; `lib/Watchdog.qml`
//! ends one that hangs.

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
    std::env::set_var("QML_XHR_ALLOW_FILE_READ", "1");
    let mut app = QGuiApplication::new();
    let mut failed = Vec::new();

    for file in scenarios() {
        let mut engine = QQmlApplicationEngine::new();
        let url = QUrl::from_local_file(&QString::from(file.to_string_lossy().as_ref()));
        if let Some(mut engine) = engine.as_mut() {
            // Kante, as the app finds it in development (main.rs `qml_dirs`).
            let kante = Path::new(env!("CARGO_MANIFEST_DIR")).join("qml");
            engine
                .as_mut()
                .add_import_path(&QString::from(kante.to_string_lossy().as_ref()));

            // A scenario that does not load would leave `exec` waiting for ever.
            engine
                .as_mut()
                .on_object_creation_failed(|_, url| {
                    eprintln!("could not load {}", url.to_qstring());
                    std::process::exit(1);
                })
                .release();
            engine.as_mut().load(&url);
        }

        let code = app.as_mut().map_or(1, |a| a.exec());
        if code != 0 {
            failed.push(file);
        }
    }

    // `tests/controller/export_to.qml` writes here.
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("klappe-export-test"));

    assert!(failed.is_empty(), "scenarios failed: {failed:?}");
}
