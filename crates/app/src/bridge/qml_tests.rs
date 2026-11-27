//! Runs the QML scenarios in `tests/controller` against the real controller, offscreen: the
//! controller cannot be built from Rust, QML makes it. A scenario ends with `Qt.exit(0)` when
//! it holds and `Qt.exit(1)` (after `console.error`) when it does not; `lib/Watchdog.qml`
//! ends one that hangs.

use cxx::UniquePtr;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::path::{Path, PathBuf};

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// The copy of the example `image_pick.qml` opens, inside the scenarios' folder.
const IMAGE_PROJECT: &str = "bild-projekt";
/// The project `script_untouched.qml` opens: its screenplay has CRLF line ends and a no-break
/// space, which the editor's text area does not give back as it got them.
const CRLF_PROJECT: &str = "crlf-projekt";
const CRLF_SCRIPT: &str = "INT. HALLE - TAG\r\n\r\nSie\u{a0}kommt an.\r\n";
/// The project `script_hervor.qml` opens: a screenplay with emphasis, and lines the writer
/// would write another way.
const HERVOR_PROJECT: &str = "hervor-projekt";
/// Line 2 is typed in; line 3 is not written the way the editor writes it, line 4 holds a
/// backslash and a note, line 5 trailing blanks.
const HERVOR_SCRIPT: &str = "Ein Anfang.\n\nDas ist **fett** und *kursiv*.\n\nNicht ***kanonisch** und *so*  \n\nEin \\*Stern\\* [[Notiz]] und _Strich_.\n";
/// The copy of the example `place_answer.qml` opens.
const PLACE_PROJECT: &str = "orte-projekt";
/// The copy of the example `outside.qml` changes from two controllers.
const OUTSIDE_PROJECT: &str = "aussen-projekt";
/// The copy of the example `looks_window.qml` puts looks on.
const LOOKS_PROJECT: &str = "looks-projekt";
/// The copy of the example `versions_window.qml` locks versions of.
const VERSIONS_PROJECT: &str = "versionen-projekt";
/// The copy of the example `versions_outside.qml` changes from two controllers.
const VERSIONS_OUTSIDE_PROJECT: &str = "versionen-aussen-projekt";
/// The copy of the example `versions_unreadable.qml` opens: its list names a version whose text
/// file is not there.
const VERSIONS_UNREADABLE_PROJECT: &str = "versionen-fehlt-projekt";
/// The list of that copy: one version, no `drehbuch/fassungen/fehlt.fountain`.
const UNREADABLE_LIST: &str = "[[fassung]]\nid = \"fehlt\"\nname = \"Drehfassung\"\ndatum = \"2026-11-25\"\nfarbe = \"white\"\n";
/// The scripted location of the example, and the name the copy has before the scenario renames it.
const PLACE_NAME: &str = "SÄNGERSAAL";
const PLACE_OLD_NAME: &str = "FESTSAAL";
/// The id of the renamed scripted location in the example project; a rename keeps it.
const PLACE_ID: &str = "k3m9q2xa";
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
    copy_dir(&example, &base.join(PLACE_PROJECT));
    copy_dir(&example, &base.join(OUTSIDE_PROJECT));
    copy_dir(&example, &base.join(LOOKS_PROJECT));
    copy_dir(&example, &base.join(VERSIONS_PROJECT));
    copy_dir(&example, &base.join(VERSIONS_OUTSIDE_PROJECT));
    copy_dir(&example, &base.join(VERSIONS_UNREADABLE_PROJECT));

    let unreadable = base.join(VERSIONS_UNREADABLE_PROJECT).join("drehbuch");
    std::fs::create_dir_all(&unreadable).unwrap();
    std::fs::write(unreadable.join("fassungen.toml"), UNREADABLE_LIST).unwrap();

    // The heading says SÄNGERSAAL: the book renamed what the file calls FESTSAAL.
    let places = base.join(PLACE_PROJECT).join("orte.toml");
    let old = std::fs::read_to_string(&places).unwrap();
    assert!(old.contains(PLACE_NAME));
    std::fs::write(places, old.replace(PLACE_NAME, PLACE_OLD_NAME)).unwrap();
}

/// What `place_answer.qml` answered is in `orte.toml`: the old name is called the heading's
/// now, the id stayed.
fn check_place_answer(base: &Path) {
    let places = std::fs::read_to_string(base.join(PLACE_PROJECT).join("orte.toml")).unwrap();

    assert!(places.contains(PLACE_NAME), "{places}");
    assert!(!places.contains(PLACE_OLD_NAME), "{places}");
    assert!(places.contains(PLACE_ID), "{places}");
}

/// An empty project with the screenplay of `CRLF_SCRIPT`, for `script_untouched.qml`.
fn write_crlf_project(base: &Path) {
    let dir = base.join(CRLF_PROJECT);
    let session = klappe_services::Session::create(&dir, "CRLF", klappe_services::Language::German);

    assert!(session.is_ok());
    std::fs::write(dir.join("drehbuch.fountain"), CRLF_SCRIPT).unwrap();
}

/// A project with the screenplay of `HERVOR_SCRIPT`, for `script_hervor.qml`.
fn write_hervor_project(base: &Path) {
    let dir = base.join(HERVOR_PROJECT);
    let session =
        klappe_services::Session::create(&dir, "Hervor", klappe_services::Language::German);

    assert!(session.is_ok());
    std::fs::write(dir.join("drehbuch.fountain"), HERVOR_SCRIPT).unwrap();
}

/// `script_hervor.qml` typed in line 2 and saved: that line is another, every other line of the
/// file is as it was.
fn check_hervor_file(base: &Path) {
    let saved =
        std::fs::read_to_string(base.join(HERVOR_PROJECT).join("drehbuch.fountain")).unwrap();
    let (old, new): (Vec<_>, Vec<_>) = (
        HERVOR_SCRIPT.split('\n').collect(),
        saved.split('\n').collect(),
    );

    assert_eq!(old.len(), new.len(), "{saved:?}");
    for (at, (was, now)) in old.iter().zip(&new).enumerate() {
        match at {
            2 => assert_eq!(*now, "Das ist **fexxtt** und *kursiv*."),
            _ => assert_eq!(was, now, "line {at}"),
        }
    }
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

/// This run's own temporary folder. The scenarios find their files under the temporary folder
/// (`StandardPaths.TempLocation`); a fixed one there was cleared by a run in another worktree
/// at the same time.
fn own_temp() -> PathBuf {
    std::env::temp_dir().join(format!("klappe-qml-{}", std::process::id()))
}

/// Runs the scenario `file` to its `Qt.exit`; the exit code.
fn run_scenario(app: &mut UniquePtr<QGuiApplication>, file: &Path) -> i32 {
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

    app.as_mut().map_or(1, |a| a.exec())
}

/// The editor loads a script of 120 pages and takes a key within the budgets of D-36; run by
/// `make perf` in a release build, where the timings mean something.
#[test]
#[ignore = "timing, run by `make perf` in release"]
fn the_editor_of_120_pages_is_quick() {
    std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    std::env::set_var("QML_XHR_ALLOW_FILE_READ", "1");
    let mut app = crate::application::new();
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/perf/script_editor.qml");

    assert_eq!(run_scenario(&mut app, &file), 0);
}

#[test]
fn controller_scenarios_hold() {
    std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    std::env::set_var("QML_XHR_ALLOW_FILE_READ", "1");
    let mut app = crate::application::new();
    // The desktop's native file dialogs need a QApplication (src/application.rs).
    assert!(crate::application::is_widgets(), "not a QApplication");
    let mut failed = Vec::new();

    // The scenarios' temporary folder is this run's own, so runs side by side keep apart.
    let own = own_temp();
    std::fs::create_dir_all(&own).unwrap();
    std::env::set_var("TMPDIR", &own);

    // `tests/controller/export_to.qml` writes here; a run that stopped early may have left it.
    let base = own.join("klappe-export-test");
    let _ = std::fs::remove_dir_all(&base);
    copy_example(&base);
    write_crlf_project(&base);
    write_hervor_project(&base);

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
        if run_scenario(&mut app, &file) != 0 {
            failed.push(file);
        }
    }

    if failed.is_empty() {
        check_list_exports(&base);
        check_picked_images(&base);
        check_place_answer(&base);
        check_hervor_file(&base);
    }
    let _ = std::fs::remove_dir_all(&base);
    // Every scenario writes below `base`; other tests of this process (`manage.rs`) may still
    // keep folders here and remove their own, so only an empty folder goes.
    let _ = std::fs::remove_dir(&own);

    assert!(failed.is_empty(), "scenarios failed: {failed:?}");
}
