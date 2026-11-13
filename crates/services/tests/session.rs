//! The session on the example project (`testdata/beispiel.klappe`).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, LinkKind, ServiceError, Session, ShotField, TrackKind, Visibility};
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

#[test]
fn prints_the_plan_as_pdf() {
    let s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("session-plan.pdf");
    let texts = klappe_services::Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
    };

    s.export_pdf(&path, klappe_services::Paper::A4, texts)
        .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
    assert_eq!(s.plan_title(), "12 · INT. SÄNGERSAAL – TAG · Plan A");
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
