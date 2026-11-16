//! Runs the QML scenarios in `tests/controller` against the real controller, offscreen: the
//! controller cannot be built from Rust, QML makes it. A scenario ends with `Qt.exit(0)` when
//! it holds and `Qt.exit(1)` (after `console.error`) when it does not; `lib/Watchdog.qml`
//! ends one that hangs.

use cxx_qt_lib::{QQmlApplicationEngine, QString, QUrl};
use std::path::{Path, PathBuf};

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// The copy of the example `image_pick.qml` opens, inside the scenarios' folder.
const IMAGE_PROJECT: &str = "bild-projekt";
/// Where the width sits in a PNG file; the height follows.
const PNG_WIDTH_AT: usize = 16;

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

/// The example, copied for `image_pick.qml`: placing an image writes into `bilder/`, which must
/// not happen in `testdata`.
fn copy_example(base: &Path) {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

    copy_dir(&example, &base.join(IMAGE_PROJECT));
}

/// What `image_pick.qml` picked lies in the copy's `bilder/`, byte for byte, both files.
fn check_picked_images(base: &Path) {
    let testdata = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata");

    for name in ["skizze.png", "skizze2.png"] {
        let copied = std::fs::read(base.join(IMAGE_PROJECT).join("bilder").join(name));

        assert_eq!(
            copied.unwrap_or_default(),
            std::fs::read(testdata.join(name)).unwrap(),
            "{name} is not in bilder/"
        );
    }
}

/// Letter portrait is 612 by 792 pt.
const LETTER_PORTRAIT_MEDIA_BOX: &str = "/MediaBox [0 0 612 792]";

/// ODS and XLSX are ZIP files; the frames folder holds one PNG per mark of the example; the
/// PNGs of the 3D view (800 x 450 on screen) and of the camera are 1920 wide.
fn check_list_exports(base: &Path) {
    let png_size = |name: &str| {
        let bytes = std::fs::read(base.join(name)).unwrap();
        assert!(bytes.starts_with(&PNG_SIGNATURE), "{name} is no PNG");
        let number = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());

        (number(PNG_WIDTH_AT), number(PNG_WIDTH_AT + 4))
    };
    assert_eq!(png_size("3D.png"), (1920, 1080));
    assert_eq!(png_size("Kamera.png").0, 1920);

    for name in ["Liste.ods", "Liste.xlsx"] {
        let bytes = std::fs::read(base.join(name)).unwrap();
        assert!(bytes.starts_with(b"PK"), "{name} is no ZIP");
    }

    let upright =
        String::from_utf8_lossy(&std::fs::read(base.join("Hoch.pdf")).unwrap()).into_owned();
    assert!(
        upright.contains(LETTER_PORTRAIT_MEDIA_BOX),
        "no portrait page"
    );

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let marks = klappe_services::Session::open(&example).unwrap().marks_s();
    let mut frames: Vec<_> = std::fs::read_dir(base.join("frames"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    frames.sort();

    assert!(!marks.is_empty());
    assert_eq!(frames.len(), marks.len());
    assert!(frames
        .iter()
        .all(|f| f.extension().is_some_and(|e| e == "png")));
}

#[test]
fn controller_scenarios_hold() {
    std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    std::env::set_var("QML_XHR_ALLOW_FILE_READ", "1");
    let mut app = crate::application::new();
    // The desktop's native file dialogs need a QApplication (src/application.rs).
    assert!(crate::application::is_widgets(), "not a QApplication");
    let mut failed = Vec::new();

    // `tests/controller/export_to.qml` writes here; a run that stopped early may have left it.
    let base = std::env::temp_dir().join("klappe-export-test");
    let _ = std::fs::remove_dir_all(&base);
    copy_example(&base);

    // `main_creates.qml` makes the whole window: its recent list, previews and settings go to the
    // test folder, never to the user's.
    for (var, dir) in [
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_CONFIG_HOME", "config"),
    ] {
        std::env::set_var(var, base.join(dir));
    }

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

    if failed.is_empty() {
        check_list_exports(&base);
        check_picked_images(&base);
    }
    let _ = std::fs::remove_dir_all(&base);

    assert!(failed.is_empty(), "scenarios failed: {failed:?}");
}
