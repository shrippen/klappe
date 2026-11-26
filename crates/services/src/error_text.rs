//! The text of a service error in a language: the one place app and CLI take it from.

use crate::catalog::Language;
use crate::error::ServiceError;
use crate::field::Field;
use crate::session::OutsideFile;

/// The service's error in the given language; details (path, value) appended where they help.
///
/// One arm for every variant and no catch-all: a new error does not build until it has its
/// text here.
pub fn error_text(language: Language, error: &ServiceError) -> String {
    let with = |key: &str, pairs: &[(&str, &str)]| fill(language, key, pairs);
    let named = |key: &str, id: &str| fill(language, key, &[("{detail}", id)]);
    let plain = |key: &str| language.text(key);

    match error {
        ServiceError::NoScene(id) => named("error.noScene", id),
        ServiceError::NoPlan(id) => named("error.noPlan", id),
        ServiceError::NoShot(id) => named("error.noShot", id),
        ServiceError::NoLocation(id) => named("error.noLocation", id),
        ServiceError::NoCharacter(id) => named("error.noCharacter", id),
        ServiceError::NoElement(id) => named("error.noElement", id),
        ServiceError::NoPerson(id) => named("error.noPerson", id),
        ServiceError::NoVersion(id) => named("error.noVersion", id),
        ServiceError::NoDialogue(name) => named("error.noDialogue", name),
        ServiceError::NoLook(id) => named("error.noLook", id),
        ServiceError::NotLookElement(id) => named("error.notLookElement", id),
        ServiceError::BadFdx(why) => named("error.badFdx", why),
        ServiceError::NameUnchanged(name) => named("error.nameUnchanged", name),
        ServiceError::NameAsked(name) => named("error.nameAsked", name),
        ServiceError::NameIsOwnAlias(name) => named("error.nameIsOwnAlias", name),
        ServiceError::NameNotCue(name) => named("error.nameNotCue", name),
        ServiceError::NotEmpty(path) => named("error.notEmpty", &path.to_string_lossy()),
        ServiceError::NoDecision(_) => plain("error.noDecision"),
        ServiceError::NoPlaceAnswer(_) => plain("error.noPlaceAnswer"),
        ServiceError::VersionNameTaken(name) => with("error.versionNameTaken", &[("{name}", name)]),
        ServiceError::PlanNameTaken(name) => with("error.planNameTaken", &[("{name}", name)]),
        ServiceError::CastNumberTaken(number) => with(
            "error.castNumberTaken",
            &[("{number}", &number.to_string())],
        ),
        ServiceError::NumberTaken(number) => with("error.numberTaken", &[("{number}", number)]),
        ServiceError::CameraInUse { camera, shot } => with(
            "error.cameraInUse",
            &[("{camera}", camera), ("{shot}", shot)],
        ),
        ServiceError::LocationInUse { scenes, .. } => {
            with("error.locationInUse", &[("{count}", &scenes.to_string())])
        }
        ServiceError::PlaceInUse { headings } => {
            with("error.placeInUse", &[("{count}", &headings.to_string())])
        }
        ServiceError::CharacterInUse { plans, .. } => {
            with("error.characterInUse", &[("{count}", &plans.to_string())])
        }
        ServiceError::PlanHasShots { shots, .. } => {
            with("error.planHasShots", &[("{count}", &shots.to_string())])
        }
        ServiceError::Locked(layer) => {
            let name = language.text(&format!("layer.{}", layer.name()));
            with("error.locked", &[("{layer}", &name)])
        }
        ServiceError::BadValue { field, value } => bad_value(language, field, value),
        ServiceError::Store(e) => {
            let (key, detail) = store_text(language, e);
            named(key, &detail)
        }
        ServiceError::Export(e) => named(export_key(e), &e.to_string()),
        // The path and why the system refused, e.g. "/a/b.pdf: Datei fehlt".
        ServiceError::Write { path, source } => {
            let why = format!("{}: {}", path.display(), io_reason(language, source));
            named("error.write", &why)
        }
        // Wraps another error: its text first, then where the old example copy went.
        ServiceError::ExampleAside { aside, source } => with(
            "error.exampleAside",
            &[
                ("{reason}", &error_text(language, source)),
                ("{aside}", &aside.to_string_lossy()),
            ],
        ),
        ServiceError::Gone => plain("error.gone"),
        ServiceError::NoFreeCamera => plain("error.noFreeCamera"),
        ServiceError::NotShot => plain("error.notShot"),
        ServiceError::NoCamera => plain("error.noCamera"),
        ServiceError::NoBackup => plain("error.noBackup"),
        ServiceError::EmptyLibrary => plain("error.emptyLibrary"),
        ServiceError::EmptyTemplate => plain("error.emptyTemplate"),
        ServiceError::NoMarks => plain("error.noMarks"),
        ServiceError::NoDuration => plain("error.noDuration"),
        ServiceError::Cancelled => plain("error.cancelled"),
        ServiceError::NoPicture => plain("error.noPicture"),
        ServiceError::NoScript => plain("error.noScript"),
        ServiceError::NoChangedPages => plain("error.noChangedPages"),
        ServiceError::NoLockedPages => plain("error.noLockedPages"),
        ServiceError::NoBookScenes => plain("error.noBookScenes"),
        ServiceError::NoPages => plain("error.noPages"),
        ServiceError::LastPlan(_) => plain("error.lastPlan"),
        ServiceError::FewPoints => plain("error.fewPoints"),
        ServiceError::NotCopyable => plain("error.notCopyable"),
        ServiceError::NoVersions => plain("error.noVersions"),
        ServiceError::EmptyVersionName => plain("error.versionNameEmpty"),
        ServiceError::VersionCamera(_) => plain("error.versionCamera"),
    }
}

/// The text of `key` with each placeholder of `pairs` replaced, e.g. `{count}` by "3".
fn fill(language: Language, key: &str, pairs: &[(&str, &str)]) -> String {
    let mut text = language.text(key);
    for (placeholder, value) in pairs {
        text = text.replace(placeholder, value);
    }

    text
}

/// A store error: a merge conflict and the format versions have their own text, naming the
/// file; a missing file or one without permission says so in the language; so do the refusals
/// a user's action causes (image, trash); the rest says what the store says.
fn store_text(language: Language, error: &klappe_store::StoreError) -> (&'static str, String) {
    use klappe_store::StoreError;

    match error {
        StoreError::Conflict { path, line } => {
            ("error.conflict", format!("{}:{line}", path.display()))
        }
        StoreError::Io { path, source } => (
            "error.store",
            format!("{}: {}", path.display(), io_reason(language, source)),
        ),
        StoreError::NewerFormat { path, .. } => ("error.newerFormat", path.display().to_string()),
        StoreError::OlderFormat { path, .. } => ("error.olderFormat", path.display().to_string()),
        StoreError::ImageType { path } => ("error.imageType", path.display().to_string()),
        StoreError::BadImage { path, .. } => ("error.badImage", path.display().to_string()),
        StoreError::Trash { path, .. } => ("error.trash", path.display().to_string()),
        StoreError::ImageUnprintable { path, reason } => {
            let key = match reason {
                klappe_store::Unprintable::Channels => "error.imageCmyk",
                klappe_store::Unprintable::Depth => "error.imageDepth",
            };
            (key, path.display().to_string())
        }
        StoreError::ImageName { name } => ("error.imageName", name.clone()),
        other => ("error.store", other.to_string()),
    }
}

/// Why the system refused a file: the common reasons in the language, e.g. "Datei fehlt"; the
/// system's own words for the rest.
fn io_reason(language: Language, error: &std::io::Error) -> String {
    let key = match error.kind() {
        std::io::ErrorKind::NotFound => "io.notFound",
        std::io::ErrorKind::PermissionDenied => "io.denied",
        std::io::ErrorKind::IsADirectory => "io.isFolder",
        std::io::ErrorKind::StorageFull => "io.full",
        std::io::ErrorKind::ReadOnlyFilesystem => "io.readOnly",
        _ => return error.to_string(),
    };

    language.text(key)
}

/// The key of an export error: the one a user can act on has its own text.
fn export_key(error: &klappe_export::ExportError) -> &'static str {
    match error {
        klappe_export::ExportError::ZipTooBig => "error.zipTooBig",
        klappe_export::ExportError::NoVideoProgram(_) => "error.noFfmpeg",
        klappe_export::ExportError::Video(_) => "error.video",
        _ => "error.export",
    }
}

/// The refused field and its value; without a value, no colon with nothing after it.
fn bad_value(language: Language, field: &Field, value: &str) -> String {
    let name = language.text(&format!("field.{}", field.name()));
    let key = if value.is_empty() {
        "error.badField"
    } else {
        "error.badValue"
    };

    language
        .text(key)
        .replace("{field}", &name)
        .replace("{value}", value)
}

/// The kinds of `files` as words, each once, in the order they first come ("Drehbuch, Pläne").
pub fn outside_files_text(language: Language, files: &[OutsideFile]) -> String {
    let mut words: Vec<String> = Vec::new();
    for file in files {
        let key = match file.kind {
            klappe_store::FileKind::Project => "outside.file.project",
            klappe_store::FileKind::Figures => "outside.file.figures",
            klappe_store::FileKind::Library => "outside.file.library",
            klappe_store::FileKind::Templates => "outside.file.templates",
            klappe_store::FileKind::Location => "outside.file.location",
            klappe_store::FileKind::Register => "outside.file.register",
            klappe_store::FileKind::Scene => "outside.file.scene",
            klappe_store::FileKind::Breakdown => "outside.file.breakdown",
            klappe_store::FileKind::Team => "outside.file.team",
            klappe_store::FileKind::Contacts => "outside.file.contacts",
            klappe_store::FileKind::Script => "outside.file.script",
        };
        let word = language.text(key);
        if !words.contains(&word) {
            words.push(word);
        }
    }

    words.join(", ")
}

/// What loaded but is odd, one sentence per note, for the status line after opening.
pub fn load_notes_text(language: Language, notes: &[klappe_store::LoadNote]) -> String {
    let lines: Vec<String> = notes
        .iter()
        .map(|note| match note {
            klappe_store::LoadNote::OrphanScene { id } => {
                language.text("note.orphanScene").replace("{id}", id)
            }
        })
        .collect();

    lines.join(" ")
}

/// What the status line says about the book: scenes to match, scenes struck from it and
/// names no character answers to;
/// empty when there is nothing to say.
pub fn book_report_text(language: Language, report: &crate::BookReport) -> String {
    let count = |key: &str, n: usize| language.text(key).replace("{count}", &n.to_string());
    let mut lines = Vec::new();

    if report.pending > 0 {
        lines.push(count("book.pending", report.pending));
    }
    if report.not_in_book > 0 {
        lines.push(count("book.notInBook", report.not_in_book));
    }
    if report.figures > 0 {
        lines.push(count("book.figures", report.figures));
    }
    if report.places > 0 {
        lines.push(count("book.places", report.places));
    }
    if report.twin_numbers > 0 {
        lines.push(count("book.twinNumbers", report.twin_numbers));
    }

    lines.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Layer;
    use std::path::{Path, PathBuf};

    /// Review of PR 286: a project that does not open said why in the system's English
    /// ("No such file or directory (os error 2)", "format version 9 is newer …").
    #[test]
    fn store_errors_speak_the_language() {
        let path = PathBuf::from("/p/projekt.toml");
        let missing = ServiceError::Store(klappe_store::StoreError::Io {
            path: path.clone(),
            source: std::io::Error::from(std::io::ErrorKind::NotFound),
        });
        let newer = ServiceError::Store(klappe_store::StoreError::NewerFormat {
            path: path.clone(),
            found: 9,
            supported: 1,
        });
        let folder = ServiceError::Write {
            path,
            source: std::io::Error::from(std::io::ErrorKind::IsADirectory),
        };

        for (error, german) in [
            (missing, "Datei fehlt"),
            (newer, "neuer"),
            (folder, "ist ein Ordner"),
        ] {
            let text = error_text(Language::German, &error);
            assert!(text.contains(german), "{text}");
            assert!(!text.contains("os error"), "{text}");
        }
    }

    /// Audit 18, finding 10: choosing `bild.gif` showed the store's English text in the German
    /// window, behind "Projekt konnte nicht gelesen oder gespeichert werden".
    #[test]
    fn refusals_of_user_actions_speak_the_language() {
        use klappe_store::StoreError;

        let gif = klappe_store::add_image(Path::new("/p"), Path::new("/q/bild.gif")).unwrap_err();
        let path = PathBuf::from("/q/bild.png");
        let cases = [
            (gif, "bild.gif", "Bilder"),
            (
                StoreError::BadImage {
                    path: path.clone(),
                    message: "not a readable PNG or JPEG image".to_owned(),
                },
                "bild.png",
                "Bild",
            ),
            (
                StoreError::Trash {
                    path: path.clone(),
                    message: "no trash".to_owned(),
                },
                "bild.png",
                "Papierkorb",
            ),
        ];

        for (error, detail, german) in cases {
            let text = error_text(Language::German, &ServiceError::Store(error));

            assert!(text.contains(german) && text.contains(detail), "{text}");
            assert!(!text.contains("konnte nicht gelesen"), "{text}");
            assert!(
                !text.contains("supported") && !text.contains("no trash"),
                "{text}"
            );
            assert!(
                !text.contains("not a readable") && !text.contains("file name"),
                "{text}"
            );
        }
    }

    /// Review of PR 306: a CMYK JPEG said "kein brauchbares Bild", hiding that it only has to
    /// become RGB; a project file with a bad id lost the frame text of a broken project.
    #[test]
    fn image_and_name_refusals_say_what_to_do() {
        use klappe_store::StoreError;

        let dir = std::env::temp_dir().join(format!("klappe-cmyk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cmyk = dir.join("cmyk.jpg");
        // SOI, SOF0: 8 bits, 1 x 1 pixel, 4 components.
        let head = [
            0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x14, 0x08, 0x00, 0x01, 0x00, 0x01, 0x04, 0x01, 0x11,
            0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00, 0x04, 0x11, 0x00,
        ];
        std::fs::write(&cmyk, head).unwrap();
        let error = klappe_store::add_image(&dir, &cmyk).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();

        let text = error_text(Language::German, &ServiceError::Store(error));
        assert!(text.contains("CMYK") && text.contains("RGB"), "{text}");
        assert!(!text.contains("kein brauchbares"), "{text}");

        let named = klappe_store::add_image(Path::new("/p"), Path::new("/q/.foto.png"));
        let text = error_text(Language::German, &ServiceError::Store(named.unwrap_err()));
        assert!(
            text.contains(".foto.png") && text.contains("Bild"),
            "{text}"
        );
        assert!(!text.contains("konnte nicht gelesen"), "{text}");

        let broken = StoreError::BadId {
            id: "../x".to_owned(),
        };
        let text = error_text(Language::German, &ServiceError::Store(broken));
        assert!(text.starts_with("Projekt konnte nicht gelesen"), "{text}");
    }

    /// A 12-bit JPEG (SOF1) reads fine but the PDF cannot print it.
    #[test]
    fn a_12_bit_jpeg_says_8_bit() {
        let dir = std::env::temp_dir().join(format!("klappe-12bit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let deep = dir.join("tief.jpg");
        // SOI, SOF1: 12 bits, 1 x 1 pixel, 3 components.
        let head = [
            0xFF, 0xD8, 0xFF, 0xC1, 0x00, 0x11, 0x0C, 0x00, 0x01, 0x00, 0x01, 0x03, 0x01, 0x11,
            0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00,
        ];
        std::fs::write(&deep, head).unwrap();
        let error = klappe_store::add_image(&dir, &deep).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();

        let text = error_text(Language::German, &ServiceError::Store(error));
        assert!(text.contains("8 Bit"), "{text}");
        assert!(!text.contains("kein brauchbares"), "{text}");
    }

    /// Review of PR 37: a refused tilt said "field.tilt", the catalogue had no such key.
    #[test]
    fn every_refused_field_has_a_name() {
        for language in [Language::German, Language::English] {
            let raw: Vec<_> = Field::ALL
                .iter()
                .map(|f| f.name())
                .filter(|name| language.text(&format!("field.{name}")) == format!("field.{name}"))
                .collect();

            assert!(raw.is_empty(), "{language:?}: {raw:?}");
        }
    }

    /// Q2-04: a refusal with no value to show said "Ungültiger Wert für Element:".
    #[test]
    fn a_refusal_without_value_has_no_empty_tail() {
        for language in [Language::German, Language::English] {
            for field in Field::ALL {
                let error = ServiceError::BadValue {
                    field,
                    value: String::new(),
                };
                let text = error_text(language, &error);

                assert!(!text.trim_end().ends_with(':'), "{language:?}: {text}");
            }
        }
    }

    #[test]
    fn a_refused_copy_names_no_kind() {
        let error = ServiceError::NotCopyable;

        assert_eq!(
            error_text(Language::German, &error),
            "Die Auswahl lässt sich nicht duplizieren"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The selection cannot be duplicated"
        );
    }

    #[test]
    fn errors_are_translated_with_details() {
        let write = ServiceError::Write {
            path: PathBuf::from("/a/b.pdf"),
            source: std::io::Error::other("kaputt"),
        };
        let bad = ServiceError::BadValue {
            field: Field::Lens,
            value: "5000".to_owned(),
        };

        assert_eq!(
            error_text(Language::German, &write),
            "Kann nicht schreiben: /a/b.pdf: kaputt"
        );
        assert_eq!(
            error_text(Language::English, &write),
            "Cannot write: /a/b.pdf: kaputt"
        );
        assert_eq!(
            error_text(Language::German, &bad),
            "Ungültiger Wert für Brennweite (mm): 5000"
        );
        assert_eq!(
            error_text(Language::English, &bad),
            "Invalid value for Focal length (mm): 5000"
        );
    }

    #[test]
    fn plan_refusals_are_translated() {
        let shots = ServiceError::PlanHasShots {
            id: "plan-a".to_owned(),
            shots: 5,
        };
        let taken = ServiceError::PlanNameTaken("Plan B".to_owned());
        let last = ServiceError::LastPlan("plan-a".to_owned());
        let few = ServiceError::FewPoints;

        assert_eq!(
            error_text(Language::German, &shots),
            "Der Plan enthält 5 Shot(s): lösche sie zuerst"
        );
        assert_eq!(
            error_text(Language::English, &taken),
            "This scene already has a plan called Plan B"
        );
        assert_eq!(
            error_text(Language::English, &last),
            "A scene keeps its last plan"
        );
        assert_eq!(
            error_text(Language::German, &few),
            "Eine Fläche braucht mindestens drei Punkte."
        );
        assert_eq!(
            error_text(Language::English, &few),
            "An area needs at least three points."
        );
    }

    /// Review of PR 10: label and shot number, no code text.
    #[test]
    fn a_camera_in_use_names_label_and_shot() {
        let error = ServiceError::CameraInUse {
            camera: "B".to_owned(),
            shot: "12.3".to_owned(),
        };

        assert_eq!(
            error_text(Language::German, &error),
            "Kamera B wird von Shot 12.3 genutzt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "Camera B is used by shot 12.3"
        );
    }

    #[test]
    fn a_locked_layer_is_named_in_the_message() {
        let error = ServiceError::Locked(Layer::Notes);

        assert_eq!(
            error_text(Language::German, &error),
            "Die Ebene Notizen ist gesperrt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The layer Notes is locked"
        );
    }

    #[test]
    fn an_object_in_use_names_the_count() {
        let error = ServiceError::LocationInUse {
            id: "l".to_owned(),
            scenes: 3,
        };

        assert_eq!(
            error_text(Language::German, &error),
            "Der Drehort wird von 3 Szene(n) genutzt"
        );
        assert_eq!(
            error_text(Language::English, &error),
            "The location is used by 3 scene(s)"
        );
    }

    #[test]
    fn the_book_report_is_worded_in_both_languages() {
        let report = crate::BookReport {
            pending: 2,
            not_in_book: 1,
            figures: 3,
            places: 3,
            twin_numbers: 4,
        };

        for language in [Language::German, Language::English] {
            let text = book_report_text(language, &report);
            assert!(
                text.contains('2')
                    && text.contains('1')
                    && text.contains('3')
                    && text.contains('4')
                    && !text.contains("{count}")
            );
            assert!(!text.contains("book."), "{text}");
        }
        assert_eq!(book_report_text(Language::English, &Default::default()), "");
    }

    #[test]
    fn every_layer_has_a_name_in_both_languages() {
        for layer in Layer::ALL {
            let key = format!("layer.{}", layer.name());
            for language in [Language::German, Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }

    /// Every error the session makes, in two lists only to keep each function short (part 1).
    fn bare_errors() -> Vec<ServiceError> {
        vec![
            ServiceError::NoScene("s".to_owned()),
            ServiceError::NoDecision("s".to_owned()),
            ServiceError::NoShot("x".to_owned()),
            ServiceError::Gone,
            ServiceError::NoPages,
            ServiceError::NoLocation("l".to_owned()),
            ServiceError::NotShot,
            ServiceError::NoFreeCamera,
            ServiceError::NoCamera,
            ServiceError::NoBackup,
            ServiceError::EmptyTemplate,
            ServiceError::Locked(Layer::Set),
            ServiceError::NoCharacter("c".to_owned()),
            ServiceError::NameUnchanged("c".to_owned()),
            ServiceError::NameAsked("c".to_owned()),
            ServiceError::NameIsOwnAlias("c".to_owned()),
            ServiceError::NameNotCue("c".to_owned()),
            ServiceError::NoElement("e".to_owned()),
            ServiceError::NoLook("l".to_owned()),
            ServiceError::NotLookElement("e".to_owned()),
            ServiceError::BadFdx("line 1".to_owned()),
            ServiceError::NumberTaken("12".to_owned()),
            ServiceError::LocationInUse {
                id: "l".to_owned(),
                scenes: 2,
            },
            ServiceError::PlaceInUse { headings: 2 },
            ServiceError::CharacterInUse {
                id: "c".to_owned(),
                plans: 1,
            },
            ServiceError::NotEmpty(PathBuf::from("/a")),
            ServiceError::ExampleAside {
                aside: PathBuf::from("/a.alt"),
                source: Box::new(ServiceError::NoMarks),
            },
            ServiceError::NoMarks,
            ServiceError::NoPicture,
            ServiceError::NoScript,
            ServiceError::NoChangedPages,
            ServiceError::NoLockedPages,
            ServiceError::NoDialogue("d".to_owned()),
            ServiceError::NoBookScenes,
            ServiceError::NoDuration,
            ServiceError::Cancelled,
            ServiceError::NotCopyable,
            ServiceError::Export(klappe_export::ExportError::NoVideoProgram("ffmpeg".into())),
            ServiceError::Export(klappe_export::ExportError::Video("x".into())),
            ServiceError::Export(klappe_export::ExportError::ZipTooBig),
            ServiceError::Export(klappe_export::ExportError::EmptyPlan),
        ]
    }

    /// Every error the session makes, part 2.
    fn named_errors() -> Vec<ServiceError> {
        vec![
            ServiceError::NoPlaceAnswer(1),
            ServiceError::NoPlan("p".to_owned()),
            ServiceError::NoPerson("p".to_owned()),
            ServiceError::NoVersion("v".to_owned()),
            ServiceError::NoVersions,
            ServiceError::EmptyVersionName,
            ServiceError::VersionNameTaken("v".to_owned()),
            ServiceError::CastNumberTaken(3),
            ServiceError::CameraInUse {
                camera: "A".to_owned(),
                shot: "1".to_owned(),
            },
            ServiceError::LastPlan("p".to_owned()),
            ServiceError::FewPoints,
            ServiceError::PlanHasShots {
                id: "p".to_owned(),
                shots: 2,
            },
            ServiceError::PlanNameTaken("p".to_owned()),
            ServiceError::VersionCamera("1".to_owned()),
            ServiceError::EmptyLibrary,
            ServiceError::BadValue {
                field: Field::Angle,
                value: "x".to_owned(),
            },
            ServiceError::BadValue {
                field: Field::Angle,
                value: String::new(),
            },
            ServiceError::Write {
                path: PathBuf::from("/a"),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            },
            ServiceError::Store(klappe_store::StoreError::Io {
                path: PathBuf::from("/a"),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            }),
            ServiceError::Store(klappe_store::StoreError::Conflict {
                path: PathBuf::from("/a"),
                line: 1,
            }),
            ServiceError::Store(klappe_store::StoreError::ImageName {
                name: "x".to_owned(),
            }),
        ]
    }

    #[test]
    fn every_error_kind_has_a_text_in_both_languages() {
        let errors: Vec<_> = bare_errors().into_iter().chain(named_errors()).collect();

        for error in &errors {
            for language in [Language::German, Language::English] {
                let text = error_text(language, error);
                assert!(!text.is_empty(), "{error:?}");
                assert!(!text.starts_with("error."), "{text}");
                assert_ne!(text, error.to_string());
            }
        }
    }
}
