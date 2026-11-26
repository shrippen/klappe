//! Locked versions of the screenplay and the revision against the last one (DB-40 to DB-44).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Draft, HeaderLine, Language, OnOutside, RevisionColor, Revisions, Saved, SceneNumbering,
    SceneNumbers, ScenePlace, SceneStatus, ScriptPrint, ServiceError, Session,
};

const VERSIONS_FILE: &str = "drehbuch/fassungen.toml";

fn scene(head: &str, tag: &str) -> String {
    let mut s = format!("INT. {head} – TAG\n\n");
    for i in 1..=4 {
        s.push_str(&format!("{tag} geht durch den Raum und tut Ding {i}.\n\n"));
    }
    s
}

fn book(scenes: &[(&str, &str)]) -> String {
    scenes.iter().map(|(h, t)| scene(h, t)).collect()
}

const THREE: [(&str, &str); 3] = [("SAAL", "Alpha"), ("KÜCHE", "Beta"), ("HOF", "Gamma")];

/// An empty project (automatic numbering, English) holding `text`.
fn session(name: &str, text: &str) -> (PathBuf, Session) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Session::create(&dir, "Nachtwache", Language::English).unwrap();
    s.set_script_text(text);

    (dir, s)
}

fn lock(s: &mut Session, name: &str) {
    let saved = s
        .lock_version(name, Language::English, OnOutside::Ask)
        .unwrap();
    assert_eq!(saved, Saved::Done);
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

#[test]
fn locking_saves_numbers_and_keeps_the_text() {
    let (dir, mut s) = session("versions-lock", &book(&THREE));

    lock(&mut s, "  Drehfassung ");

    // Saved like Ctrl+S: the numbers are in the file, and are locked now.
    let saved = read(&dir, "drehbuch.fountain");
    assert!(saved.contains("#1#") && saved.contains("#3#"), "{saved}");
    assert_eq!(s.project().script.scene_numbering, SceneNumbering::Locked);
    // `locked` is the default of `projekt.toml`, so the file says nothing but "not automatic".
    assert!(!read(&dir, "projekt.toml").contains("automatic"));

    let [version] = s.versions() else {
        panic!("one version expected");
    };
    assert_eq!(version.name, "Drehfassung");
    assert_eq!(version.color, RevisionColor::White);
    assert!(klappe_model::is_version_date(&version.date));
    assert_eq!(
        read(&dir, &format!("drehbuch/fassungen/{}.fountain", version.id)),
        saved
    );
    assert!(read(&dir, VERSIONS_FILE).contains("farbe = \"white\""));

    // The pages were laid out, and the next session knows the version.
    assert_eq!(s.script_pages().map(|p| p.pages), Some(1));
    let id = version.id.clone();
    assert_eq!(Session::open(&dir).unwrap().versions()[0].id, id);
}

#[test]
fn the_colours_follow_the_order_of_locking() {
    let (_, mut s) = session("versions-colours", &book(&THREE));

    for name in ["Eins", "Zwei", "Drei"] {
        lock(&mut s, name);
    }

    let colours: Vec<RevisionColor> = s.versions().iter().map(|v| v.color).collect();
    assert_eq!(
        colours,
        vec![
            RevisionColor::White,
            RevisionColor::Blue,
            RevisionColor::Pink
        ]
    );
}

#[test]
fn an_empty_or_used_name_is_refused_and_nothing_is_written() {
    let (dir, mut s) = session("versions-names", &book(&THREE));

    let empty = s.lock_version("  ", Language::English, OnOutside::Ask);
    assert!(matches!(empty, Err(ServiceError::EmptyVersionName)));
    assert!(!dir.join("drehbuch").exists());

    lock(&mut s, "Drehfassung");
    let used = s.lock_version("drehfassung", Language::English, OnOutside::Ask);
    assert!(matches!(used, Err(ServiceError::VersionNameTaken(_))));
    assert_eq!(s.versions().len(), 1);
}

#[test]
fn a_book_without_text_cannot_be_locked() {
    let (_, mut s) = session("versions-empty", "  \n");

    let locked = s.lock_version("Leer", Language::English, OnOutside::Ask);

    assert!(matches!(locked, Err(ServiceError::NoScript)));
}

#[test]
fn a_conflict_locks_nothing() {
    let (dir, mut s) = session("versions-conflict", &book(&THREE));
    s.save(OnOutside::Ask).unwrap();
    s.set_script_text(&book(&THREE[..2]));
    std::fs::write(dir.join("drehbuch.fountain"), book(&THREE[1..])).unwrap();

    let saved = s
        .lock_version("Drehfassung", Language::English, OnOutside::Ask)
        .unwrap();

    assert_eq!(saved, Saved::Conflict);
    assert!(s.versions().is_empty());
    assert!(!dir.join(VERSIONS_FILE).exists());
    assert_eq!(
        s.project().script.scene_numbering,
        SceneNumbering::Automatic
    );
}

#[test]
fn a_version_can_be_deleted() {
    let (dir, mut s) = session("versions-delete", &book(&THREE));
    lock(&mut s, "Eins");
    lock(&mut s, "Zwei");
    let id = s.versions()[0].id.clone();

    s.remove_version(&id).unwrap();

    assert_eq!(s.versions().len(), 1);
    assert_eq!(s.versions()[0].name, "Zwei");
    assert!(!dir
        .join(format!("drehbuch/fassungen/{id}.fountain"))
        .exists());
    assert!(!read(&dir, VERSIONS_FILE).contains(&id));
    assert!(matches!(
        s.remove_version(&id),
        Err(ServiceError::NoVersion(_))
    ));
}

#[test]
fn a_broken_list_does_not_open() {
    let (dir, mut s) = session("versions-broken", &book(&THREE));
    lock(&mut s, "Eins");
    drop(s);

    std::fs::write(dir.join(VERSIONS_FILE), "<<<<<<< HEAD\n").unwrap();

    assert!(Session::open(&dir).is_err());
}

#[test]
fn the_revision_tells_new_and_changed_scenes() {
    let (_, mut s) = session("versions-revision", &book(&THREE));
    assert!(matches!(s.revision(), Err(ServiceError::NoVersions)));
    lock(&mut s, "Drehfassung");

    // A scene at the start and the kitchen's first line changed.
    let text = s.script_text().to_owned();
    let changed = text.replace("Beta geht durch den Raum und tut Ding 1.", "Beta schläft.");
    let new = format!("{}{}", scene("NEU", "Delta"), changed);
    s.set_script_text(&new);

    let revision = s.revision().unwrap();
    let kinds: Vec<SceneStatus> = revision.scenes.iter().map(|d| d.status).collect();
    assert_eq!(
        kinds,
        vec![
            SceneStatus::New,
            SceneStatus::Same,
            SceneStatus::Changed,
            SceneStatus::Same
        ]
    );
    assert!(revision.scenes.iter().all(|d| d.place == ScenePlace::Kept));

    // Against the version by id is the same.
    let id = s.versions()[0].id.clone();
    let direct = s
        .compare_drafts(&Draft::Version(id.clone()), &Draft::Current)
        .unwrap();
    assert_eq!(direct, revision);
    assert!(matches!(
        s.compare_drafts(&Draft::Version("nope".into()), &Draft::Current),
        Err(ServiceError::NoVersion(_))
    ));

    // Two versions compare with each other.
    lock(&mut s, "Nachdreh");
    let id2 = s.versions()[1].id.clone();
    let between = s
        .compare_drafts(&Draft::Version(id), &Draft::Version(id2))
        .unwrap();
    let statuses =
        |c: &klappe_services::Comparison| c.scenes.iter().map(|d| d.status).collect::<Vec<_>>();
    assert_eq!(statuses(&between), statuses(&revision));
    assert!(s
        .revision()
        .unwrap()
        .scenes
        .iter()
        .all(|d| d.status == SceneStatus::Same));
}

const PRINT: ScriptPrint = ScriptPrint {
    scene_numbers: SceneNumbers::Hidden,
    header: HeaderLine::Off,
    language: Language::English,
    revisions: Revisions::On,
};

fn pdf_of(s: &mut Session, name: &str, print: &ScriptPrint) -> String {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    s.export_script_pdf(&path, print).unwrap();

    String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned()
}

#[test]
fn the_pdf_marks_changed_lines_only_with_a_locked_version() {
    let (_, mut s) = session("versions-pdf", &book(&THREE));

    // No version yet: no marks, no header.
    let pdf = pdf_of(&mut s, "versions-pdf-none.pdf", &PRINT);
    assert!(
        !pdf.contains("(*) Tj") && !pdf.contains("Revision"),
        "{pdf}"
    );

    lock(&mut s, "Drehfassung");
    let text = s
        .script_text()
        .replace("Beta geht durch den Raum und tut Ding 2.", "Beta schläft.");
    s.set_script_text(&text);

    // The second version would be blue.
    let pdf = pdf_of(&mut s, "versions-pdf-on.pdf", &PRINT);
    assert_eq!(pdf.matches("(*) Tj").count(), 1, "{pdf}");
    assert!(pdf.contains("(Blue Revision"), "{pdf}");

    let off = ScriptPrint {
        revisions: Revisions::Off,
        ..PRINT
    };
    let pdf = pdf_of(&mut s, "versions-pdf-off.pdf", &off);
    assert!(
        !pdf.contains("(*) Tj") && !pdf.contains("Revision"),
        "{pdf}"
    );
}

#[test]
fn the_next_colour_follows_the_last_version_even_after_a_delete() {
    let (_, mut s) = session("versions-after-delete", &book(&THREE));
    lock(&mut s, "Eins");
    lock(&mut s, "Zwei");
    let first = s.versions()[0].id.clone();
    s.remove_version(&first).unwrap();

    lock(&mut s, "Drei");

    let colours: Vec<RevisionColor> = s.versions().iter().map(|v| v.color).collect();
    assert_eq!(colours, vec![RevisionColor::Blue, RevisionColor::Pink]);

    // The header names the colour after the last one: yellow.
    let pdf = pdf_of(&mut s, "versions-after-delete.pdf", &PRINT);
    assert!(pdf.contains("(Yellow Revision"), "{pdf}");
}
