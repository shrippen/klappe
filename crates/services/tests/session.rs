//! The session on the example project (`testdata/beispiel.klappe`).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Autosave, BackupState, Layer, LinkKind, ServiceError, Session, ShotField, ShotOrder, TrackKind,
    Visibility,
};
use klappe_stellprobe::Role;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

/// A copy of the example to write to.
fn copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
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

#[test]
fn opens_on_the_first_scene_with_a_plan() {
    let s = open();

    assert_eq!(s.scene().map(|s| s.number.as_str()), Some("12"));
    // The last shot ends at 20 s, after every mark.
    assert_eq!(s.duration_s(), 20.0);
}

#[test]
fn time_stays_within_the_scene() {
    let mut s = open();

    s.set_time(-3.0);
    assert_eq!(s.time_s(), 0.0);
    s.set_time(99.0);
    assert_eq!(s.time_s(), 20.0);
}

#[test]
fn selecting_a_shot_moves_the_playhead() {
    let mut s = open();

    s.select_shot("s12-4").unwrap();

    assert_eq!(s.time_s(), 9.0);
    assert_eq!(s.selected_shot(), Some("s12-4"));
    assert!(s.select_shot("nope").is_err());
}

#[test]
fn shot_rows_take_the_camera_lens_unless_set() {
    let rows = open().shots();

    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].lens_mm, 35.0);
    assert_eq!(rows[1].lens_mm, 40.0);
    assert_eq!(rows[2].camera_label, "B");
}

/// Camera a stands at (8; 11) for 12.1 and at (9; 8) from 6 s: 12.2 (40 mm) and 12.5
/// (40 mm, from 14 s) share a setup; camera b stands at (18; 4) for 12.3 and 12.4.
#[test]
fn shot_rows_know_their_setup() {
    let rows = open().shots();

    let setups: Vec<usize> = rows.iter().map(|r| r.setup).collect();
    assert_eq!(setups, [1, 2, 3, 3, 2]);
}

#[test]
fn shots_in_shooting_order_group_the_setups() {
    let s = open();

    let rows = s.shots_in(ShotOrder::Shooting);

    let numbers: Vec<&str> = rows.iter().map(|r| r.number.as_str()).collect();
    assert_eq!(numbers, ["12.1", "12.2", "12.5", "12.3", "12.4"]);
    let setups: Vec<usize> = rows.iter().map(|r| r.setup).collect();
    assert_eq!(setups, [1, 2, 2, 3, 3]);
    assert_eq!(s.shots_in(ShotOrder::Cut), s.shots());
}

#[test]
fn writes_the_shot_list_in_shooting_order() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-shooting.csv");
    let mut texts = print_texts();
    texts.shots.columns.setup = "Setup".into();
    texts.shots.columns.number = "Nummer".into();

    s.export_csv(&path, &texts, ShotOrder::Shooting).unwrap();

    let csv = std::fs::read_to_string(&path).unwrap();
    let rows: Vec<&str> = csv.split("\r\n").collect();
    assert!(rows[0].starts_with("Setup,Nummer,"));
    assert!(rows[3].starts_with("2,12.5,"), "{}", rows[3]);
}

#[test]
fn shot_rows_tell_the_own_lens_apart() {
    let rows = open().shots();

    assert_eq!(rows[0].own_lens_mm, None);
    assert_eq!(rows[1].own_lens_mm, Some(40.0));
}

#[test]
fn shot_rows_carry_the_angle() {
    let rows = open().shots();

    assert_eq!(rows[0].angle, "eye-level");
    assert_eq!(rows[1].angle, "low");
    assert_eq!(rows[2].angle, "high");
    assert_eq!(rows[4].angle, "dutch");
}

#[test]
fn the_angle_is_edited_undone_and_validated() {
    let mut s = open();

    s.edit_shot("s12-1", ShotField::Angle, "birds-eye").unwrap();
    assert_eq!(s.shots()[0].angle, "birds-eye");

    s.undo();
    assert_eq!(s.shots()[0].angle, "eye-level");

    assert!(s.edit_shot("s12-1", ShotField::Angle, "schief").is_err());
}

#[test]
fn edit_undo_redo() {
    let mut s = open();

    s.edit_shot("s12-1", ShotField::Lens, "85 mm").unwrap();
    assert_eq!(s.shots()[0].lens_mm, 85.0);
    assert!(s.is_dirty());

    s.undo();
    assert_eq!(s.shots()[0].lens_mm, 35.0);
    s.redo();
    assert_eq!(s.shots()[0].lens_mm, 85.0);

    assert!(s.edit_shot("s12-1", ShotField::Size, "riesig").is_err());
    assert!(s.edit_shot("nope", ShotField::Note, "x").is_err());
}

#[test]
fn an_edit_that_changes_nothing_is_no_step() {
    let mut s = open();

    s.edit_shot("s12-1", ShotField::Lens, "").unwrap();

    assert!(!s.can_undo());
}

#[test]
fn hidden_layers_and_tracks_leave_the_plan() {
    let mut s = open();
    let count = |s: &Session, role: Role| {
        s.display_list()
            .items
            .iter()
            .filter(|i| i.role == role)
            .count()
    };

    assert_eq!(count(&s, Role::Frustum), 2);
    s.set_track("b", Visibility::Hidden);
    assert_eq!(count(&s, Role::Frustum), 1);

    s.set_layer(Layer::Cameras, Visibility::Hidden);
    assert_eq!(count(&s, Role::Frustum), 0);
    assert_eq!(s.layer(Layer::Cameras), Visibility::Hidden);
    assert!(s.display_list().items.iter().any(|i| i.role == Role::Wall));
}

#[test]
fn tracks_cut_first_then_figures_cameras_lights() {
    let tracks = open().tracks();
    let kinds: Vec<TrackKind> = tracks.iter().map(|t| t.kind).collect();

    assert_eq!(kinds[0], TrackKind::Cut);
    assert_eq!(tracks[0].segments.len(), 5);
    assert_eq!(tracks[0].segments[0].text, "12.1 · A");

    // Elisabeth: holds 2 s at mark 1, walks until 6 s; holds 4 s at mark 2, walks until 14 s.
    let elisabeth = &tracks[1];
    assert_eq!(elisabeth.kind, TrackKind::Figure);
    assert_eq!(elisabeth.label, "purple");
    let walks: Vec<(f64, f64)> = elisabeth
        .segments
        .iter()
        .map(|s| (s.from_s, s.to_s))
        .collect();
    assert_eq!(walks, vec![(2.0, 6.0), (10.0, 14.0)]);
    assert_eq!(elisabeth.markers.len(), 3);

    assert!(kinds.contains(&TrackKind::Camera));
    assert_eq!(kinds.last(), Some(&TrackKind::Light));
}

#[test]
fn script_lines_carry_their_links() {
    let mut s = open();
    let lines = s.script_lines();

    assert_eq!(lines[0].kind, "action");
    assert_eq!(lines[0].chips[0].kind, LinkKind::Mark);
    assert_eq!(lines[0].chips[0].text, "E · M1");
    assert_eq!(lines[1].chips[0].text, "12.2");

    // At 0 s only the first line's link has come; at 9 s Burgvogt's mark 2 (line 3).
    assert!(lines[0].current);
    s.set_time(9.0);
    let lines = s.script_lines();
    assert!(lines[3].current);
    assert_eq!(lines.iter().filter(|l| l.current).count(), 1);
}

#[test]
fn save_writes_the_edit() {
    let dir = copy("session-save");
    let mut s = Session::open(&dir).unwrap();

    s.edit_shot("s12-3", ShotField::Description, "Burgvogt, Reaktion")
        .unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.shots()[2].description, "Burgvogt, Reaktion");
    assert!(!s.is_dirty());
}

fn print_texts() -> klappe_services::Texts {
    klappe_services::Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: klappe_services::ShotTexts::default(),
    }
}

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

#[test]
fn prints_the_plan_as_png() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-plan.png");

    s.export_png(&path, klappe_services::Paper::A4, print_texts())
        .unwrap();

    assert!(std::fs::read(&path).unwrap().starts_with(PNG_SIGNATURE));
}

/// Exports are written aside and renamed: no half file, no temp file left, an old file replaced.
#[test]
fn an_export_replaces_the_file_whole() {
    let s = open();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-atomic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("plan.png");
    std::fs::write(&path, "old").unwrap();

    s.export_png(&path, klappe_services::Paper::A4, print_texts())
        .unwrap();

    assert!(std::fs::read(&path).unwrap().starts_with(PNG_SIGNATURE));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
}

/// Review of PR 58: a failed export names the file the user chose, not the temp file beside it.
#[cfg(unix)]
#[test]
fn a_failed_export_names_the_file() {
    use std::os::unix::fs::PermissionsExt;

    let s = open();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-read-only");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    let path = dir.join("plan.png");

    let err = s.export_png(&path, klappe_services::Paper::A4, print_texts());
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();

    match err {
        Err(ServiceError::Write { path: named, .. }) => assert_eq!(named, path),
        other => panic!("{other:?}"),
    }
}

#[test]
fn prints_the_plan_as_svg() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-plan.svg");

    s.export_svg(&path, klappe_services::Paper::A3, print_texts())
        .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("<svg"));
    assert!(text.contains("INT. SÄNGERSAAL"));
    assert!(text.contains("Achse"));
}

#[test]
fn selects_a_plan_by_id() {
    let mut s = open();

    assert!(matches!(
        s.select_plan("nope"),
        Err(ServiceError::NoPlan(_))
    ));
    let id = s.plan().unwrap().id.clone();
    s.select_plan(&id).unwrap();
    assert_eq!(s.plan().unwrap().id, id);
}

#[test]
fn prints_the_plan_as_pdf() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-plan.pdf");
    let texts = print_texts();

    s.export_pdf(&path, klappe_services::Paper::A4, texts, ShotOrder::Cut)
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    // The plan and one sheet of shots.
    assert_eq!(
        String::from_utf8_lossy(&bytes)
            .matches("/Type /Page /")
            .count(),
        2
    );
    assert_eq!(s.plan_title(), "12 · INT. SÄNGERSAAL – TAG · Plan A");
}

#[test]
fn writes_the_shot_list_as_csv() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-shots.csv");
    let mut texts = print_texts();
    texts.shots.columns.number = "Nummer".into();

    s.export_csv(&path, &texts, ShotOrder::Cut).unwrap();

    let csv = std::fs::read_to_string(&path).unwrap();
    let rows: Vec<&str> = csv.split("\r\n").collect();
    assert!(rows[0].starts_with("Nummer,"));
    // Head, one row per shot, and the empty rest after the last line end.
    assert_eq!(rows.len(), s.shots().len() + 2);
    assert!(rows[1].starts_with("12.1,"));
}

#[test]
fn a_lens_outside_a_sensible_range_is_refused() {
    let mut s = open();

    for bad in ["inf", "1e999", "1e-300", "NaN"] {
        let err = s.edit_shot("s12-1", ShotField::Lens, bad);
        assert!(
            matches!(err, Err(ServiceError::BadValue { .. })),
            "{bad}: {err:?}"
        );
    }

    assert_eq!(s.shots()[0].lens_mm, 35.0);
    assert!(!s.can_undo());
}

#[test]
fn an_unknown_camera_is_refused() {
    let mut s = open();

    let err = s.edit_shot("s12-1", ShotField::Camera, "nope");

    assert!(matches!(err, Err(ServiceError::BadValue { .. })), "{err:?}");
    assert_eq!(s.shots()[0].camera_label, "A");
    assert!(!s.can_undo());
}

#[test]
fn a_time_that_is_not_a_number_is_ignored() {
    let mut s = open();
    s.set_time(5.0);

    s.set_time(f64::NAN);

    assert_eq!(s.time_s(), 5.0);
}

#[test]
fn a_blank_shot_number_is_refused() {
    let mut s = open();
    let before = s.shots()[0].number.clone();

    let err = s.edit_shot("s12-1", ShotField::Number, "  ");

    assert!(matches!(err, Err(ServiceError::BadValue { .. })), "{err:?}");
    assert_eq!(s.shots()[0].number, before);
}

#[test]
fn script_text_is_replaced_and_saved() {
    let dir = copy("session-script");
    let mut s = Session::open(&dir).unwrap();
    assert!(s.script_text().starts_with("INT. SÄNGERSAAL"));

    let text = "INT. SÄNGERSAAL – TAG #12#\n\nNeu.\n";
    s.set_script_text(text);
    assert!(s.is_dirty());
    assert_eq!(s.script_lines()[0].text, "Neu.");
    s.save().unwrap();

    assert_eq!(Session::open(&dir).unwrap().script_text(), text);
}

#[test]
fn save_script_leaves_other_files_alone() {
    let dir = copy("session-save-script");
    let mut s = Session::open(&dir).unwrap();
    let foreign = dir.join("szenen/neu.json");
    std::fs::write(&foreign, "{}\n").unwrap();
    let project = std::fs::read(dir.join("projekt.toml")).unwrap();

    s.set_script_text("Neu.\n");
    s.save_script().unwrap();

    assert!(foreign.exists());
    assert_eq!(std::fs::read(dir.join("projekt.toml")).unwrap(), project);
    let script = std::fs::read_to_string(dir.join("drehbuch.fountain")).unwrap();
    assert_eq!(script, "Neu.\n");
}

/// Shot 12.2 (camera a, 6 s to 14 s) has 40 mm; the camera has 35 mm.
#[test]
fn lens_now_follows_the_shot() {
    let mut s = open();

    s.set_time(9.0);
    assert_eq!(s.lens_now_mm("a"), Some(40.0));

    s.set_time(0.0);
    assert_eq!(s.lens_now_mm("a"), Some(35.0));
    assert_eq!(s.lens_now_mm("nobody"), None);
}

fn change_lens(s: &mut Session) {
    s.edit_shot("s12-1", ShotField::Lens, "85 mm").unwrap();
}

#[test]
fn autosave_survives_a_crash_and_restores() {
    let dir = copy("session-autosave");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    let saved = std::fs::read(dir.join("szenen/szene-012.json")).unwrap();

    assert_eq!(s.autosave().unwrap(), Autosave::Written);
    drop(s); // the crash: no save

    // Project files are untouched.
    assert_eq!(
        std::fs::read(dir.join("szenen/szene-012.json")).unwrap(),
        saved
    );
    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.backup(), &BackupState::Found);
    assert_eq!(s.shots()[0].lens_mm, 35.0);

    s.restore_backup().unwrap();

    assert_eq!(s.shots()[0].lens_mm, 85.0);
    assert_eq!(s.backup(), &BackupState::None);
    assert!(s.is_dirty());
}

#[test]
fn undo_follows_restore() {
    let dir = copy("session-restore-undo");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    let mut s = Session::open(&dir).unwrap();

    s.restore_backup().unwrap();
    assert!(s.can_undo());
    s.undo();

    assert_eq!(s.shots()[0].lens_mm, 35.0);
}

#[test]
fn restore_takes_the_script_too() {
    let dir = copy("session-restore-script");
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text("Neu.\n");
    s.autosave().unwrap();
    let mut s = Session::open(&dir).unwrap();

    s.restore_backup().unwrap();

    assert_eq!(s.script_text(), "Neu.\n");
}

#[test]
fn save_removes_the_backup() {
    let dir = copy("session-save-backup");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    assert!(dir.join(".sicherung").exists());

    s.save().unwrap();

    assert!(!dir.join(".sicherung").exists());
    assert_eq!(Session::open(&dir).unwrap().backup(), &BackupState::None);
}

#[test]
fn autosave_writes_only_after_a_change() {
    let dir = copy("session-autosave-clean");
    let mut s = Session::open(&dir).unwrap();

    assert_eq!(s.autosave().unwrap(), Autosave::Clean);
    assert!(!dir.join(".sicherung").exists());

    change_lens(&mut s);
    assert_eq!(s.autosave().unwrap(), Autosave::Written);
    assert_eq!(s.autosave().unwrap(), Autosave::Clean);

    s.save().unwrap();
    assert_eq!(s.autosave().unwrap(), Autosave::Clean);
    assert!(!dir.join(".sicherung").exists());
}

#[test]
fn a_backup_equal_to_the_project_is_removed_on_open() {
    let dir = copy("session-backup-equal");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.undo();
    assert_eq!(s.autosave().unwrap(), Autosave::Written);
    assert!(dir.join(".sicherung").exists());

    let s = Session::open(&dir).unwrap();

    assert_eq!(s.backup(), &BackupState::None);
    assert!(!dir.join(".sicherung").exists());
}

#[test]
fn an_unreadable_backup_does_not_stop_open() {
    let dir = copy("session-backup-broken");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    std::fs::write(dir.join(".sicherung/projekt.toml"), "kaputt = [").unwrap();

    let mut s = Session::open(&dir).unwrap();

    let BackupState::Unreadable(message) = s.backup() else {
        panic!("expected Unreadable, got {:?}", s.backup());
    };
    assert!(message.contains("projekt.toml"), "{message}");
    assert!(s.restore_backup().is_err());

    s.discard_backup().unwrap();
    assert_eq!(s.backup(), &BackupState::None);
    assert!(!dir.join(".sicherung").exists());
}

#[test]
fn autosave_keeps_an_unanswered_backup() {
    let dir = copy("session-backup-held");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    let mut s = Session::open(&dir).unwrap();
    let backup = std::fs::read(dir.join(".sicherung/szenen/szene-012.json")).unwrap();

    s.edit_shot("s12-1", ShotField::Lens, "50 mm").unwrap();

    assert_eq!(s.autosave().unwrap(), Autosave::Held);
    assert_eq!(
        std::fs::read(dir.join(".sicherung/szenen/szene-012.json")).unwrap(),
        backup
    );
}

#[test]
fn save_script_alone_removes_a_script_only_backup() {
    let dir = copy("session-script-only");
    let mut s = Session::open(&dir).unwrap();
    s.set_script_text("Neu.\n");
    s.autosave().unwrap();
    assert!(dir.join(".sicherung").exists());

    s.save_script().unwrap();

    assert!(!dir.join(".sicherung").exists());
    assert!(!s.is_dirty());
    assert_eq!(s.autosave().unwrap(), Autosave::Clean);
    assert_eq!(Session::open(&dir).unwrap().backup(), &BackupState::None);
}

#[test]
fn save_script_rewrites_the_backup_when_the_project_changed() {
    let dir = copy("session-script-and-project");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.set_script_text("Neu.\n");
    s.autosave().unwrap();

    s.save_script().unwrap();

    assert!(s.is_dirty());
    assert_eq!(s.autosave().unwrap(), Autosave::Clean);
    // Reopened: the backup differs by the lens only; the script is saved already.
    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.backup(), &BackupState::Found);
    assert_eq!(s.script_text(), "Neu.\n");
    s.restore_backup().unwrap();
    assert_eq!(s.shots()[0].lens_mm, 85.0);
    assert_eq!(s.script_text(), "Neu.\n");
}

#[test]
fn save_keeps_an_unanswered_backup() {
    let dir = copy("session-save-held");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    let mut s = Session::open(&dir).unwrap();
    assert_eq!(s.backup(), &BackupState::Found);

    s.edit_shot("s12-1", ShotField::Lens, "50 mm").unwrap();
    s.save().unwrap();

    assert!(dir.join(".sicherung").exists());
    assert_eq!(s.backup(), &BackupState::Found);
    assert!(!s.is_dirty());
    s.restore_backup().unwrap();
    assert_eq!(s.shots()[0].lens_mm, 85.0);
}

#[test]
fn a_half_deleted_backup_is_not_found() {
    let dir = copy("session-backup-half-deleted");
    let mut s = Session::open(&dir).unwrap();
    change_lens(&mut s);
    s.autosave().unwrap();
    std::fs::remove_file(dir.join(".sicherung/fertig")).unwrap();
    std::fs::remove_dir_all(dir.join(".sicherung/szenen")).unwrap();

    assert_eq!(Session::open(&dir).unwrap().backup(), &BackupState::None);
}
