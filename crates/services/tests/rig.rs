//! The rig and the lens of a camera mark (LH-33), on the example project: camera `a` has marks
//! at 0 s (8, 11) with a pause of 6 s, and at 6 s (9, 8); shot 12.1 runs on camera `a` from 0
//! to 6 s without a movement.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::Rig;
use klappe_services::{
    Field, Hint, Layer, Lock, Movement, PlanEdit, Point, ServiceError, Session, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn camera() -> Source {
    Source::Camera("a".into())
}

fn set_rig(s: &mut Session, rig: Rig) -> Result<(), ServiceError> {
    s.edit_plan(PlanEdit::SetRig {
        source: camera(),
        rig,
    })
}

fn set_lens(s: &mut Session, lens_mm: Option<f64>) -> Result<(), ServiceError> {
    s.edit_plan(PlanEdit::SetMarkLens {
        source: camera(),
        lens_mm,
    })
}

/// Rig and lens of both marks of camera A.
fn gear(s: &Session) -> Vec<(Rig, Option<f64>)> {
    let camera = s.plan().unwrap().camera("a").unwrap();

    camera.marks.iter().map(|m| (m.rig, m.lens_mm)).collect()
}

fn is_bad(result: Result<(), ServiceError>, field: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field: f, .. }) if f == field)
}

#[test]
fn the_mark_at_the_playhead_gets_a_rig_and_a_lens() {
    let mut s = open();
    s.set_time(7.0);

    set_rig(&mut s, Rig::Crane).unwrap();
    set_lens(&mut s, Some(85.0)).unwrap();

    assert_eq!(gear(&s), [(Rig::Dolly, None), (Rig::Crane, Some(85.0))]);
    let info = s.mark_info(&camera()).unwrap();
    assert_eq!((info.rig, info.lens_mm), (Some(Rig::Crane), Some(85.0)));

    s.undo();
    s.undo();
    assert_eq!(gear(&s), [(Rig::Dolly, None), (Rig::Dolly, None)]);

    s.redo();
    s.redo();
    assert_eq!(gear(&s)[1], (Rig::Crane, Some(85.0)));
}

#[test]
fn a_lens_is_taken_away_with_none_and_keeps_its_limits() {
    let mut s = open();

    set_lens(&mut s, Some(50.0)).unwrap();
    set_lens(&mut s, None).unwrap();
    assert_eq!(gear(&s)[0].1, None);

    for lens_mm in [0.0, -5.0, 5000.1, f64::NAN, f64::INFINITY] {
        assert!(
            is_bad(set_lens(&mut s, Some(lens_mm)), Field::Lens),
            "{lens_mm}"
        );
    }
    assert_eq!(gear(&s)[0].1, None);
}

#[test]
fn a_figure_has_no_rig_or_lens() {
    let mut s = open();
    let figure = Source::Figure("elisabeth".into());

    let rig = s.edit_plan(PlanEdit::SetRig {
        source: figure.clone(),
        rig: Rig::Crane,
    });
    let lens = s.edit_plan(PlanEdit::SetMarkLens {
        source: figure.clone(),
        lens_mm: Some(50.0),
    });

    assert!(is_bad(rig, Field::Source));
    assert!(is_bad(lens, Field::Source));
    let info = s.mark_info(&figure).unwrap();
    assert_eq!((info.rig, info.lens_mm), (None, None));
}

#[test]
fn a_locked_camera_layer_refuses_rig_and_lens() {
    let mut s = open();
    s.set_lock(Layer::Cameras, Lock::Locked);

    assert!(matches!(
        set_rig(&mut s, Rig::Crane),
        Err(ServiceError::Locked(Layer::Cameras))
    ));
    assert!(matches!(
        set_lens(&mut s, Some(50.0)),
        Err(ServiceError::Locked(Layer::Cameras))
    ));
    assert_eq!(gear(&s), [(Rig::Dolly, None), (Rig::Dolly, None)]);
}

/// A tripod stays where the mark before is: mark 2 is at another place, so it cannot be one.
#[test]
fn a_tripod_cannot_be_set_where_the_camera_moves() {
    let mut s = open();
    s.set_time(7.0);

    assert!(is_bad(set_rig(&mut s, Rig::Tripod), Field::Rig));
    assert_eq!(gear(&s)[1].0, Rig::Dolly);
}

#[test]
fn a_tripod_mark_cannot_be_moved_away_but_can_turn() {
    let mut s = open();
    s.set_time(7.0);
    let before = Point::new(8.0, 11.0);

    s.edit_plan(PlanEdit::Move {
        source: camera(),
        to: before,
    })
    .unwrap();
    set_rig(&mut s, Rig::Tripod).unwrap();

    let away = PlanEdit::Move {
        source: camera(),
        to: Point::new(9.0, 8.0),
    };
    assert!(is_bad(s.edit_plan(away.clone()), Field::Rig));
    assert!(s.preview(&away).is_err());
    // The mark before cannot leave it behind either.
    s.set_time(0.0);
    let leave = PlanEdit::Move {
        source: camera(),
        to: Point::new(1.0, 1.0),
    };
    assert!(is_bad(s.edit_plan(leave), Field::Rig));

    s.set_time(7.0);
    s.edit_plan(PlanEdit::Turn {
        source: camera(),
        facing_deg: 10.0,
    })
    .unwrap();
    let camera_a = s.plan().unwrap().camera("a").unwrap();
    assert_eq!(camera_a.marks[1].at, before);
    assert_eq!(camera_a.marks[1].facing_deg, 10.0);
}

/// Another rig moves freely.
#[test]
fn other_rigs_move_freely() {
    let mut s = open();
    s.set_time(7.0);

    set_rig(&mut s, Rig::Track).unwrap();
    s.edit_plan(PlanEdit::Move {
        source: camera(),
        to: Point::new(12.0, 8.0),
    })
    .unwrap();
}

/// The lens of the marks runs with time and reaches the view and the tooltip.
#[test]
fn the_lens_of_the_marks_is_the_lens_now() {
    let mut s = open();
    set_lens(&mut s, Some(35.0)).unwrap();
    s.set_time(7.0);
    set_lens(&mut s, Some(85.0)).unwrap();

    s.set_time(3.0);
    assert_eq!(s.lens_now_mm("a"), Some(35.0));
    s.set_time(6.0);
    assert_eq!(s.lens_now_mm("a"), Some(85.0));
    let Some(Hint::Camera { lens_mm, rig, .. }) = s.camera_hint("a") else {
        panic!("no hint");
    };
    assert_eq!((lens_mm, rig.as_str()), (85.0, "dolly"));

    s.set_time(0.0);
    let Some(Hint::Camera { rig, .. }) = s.camera_hint("a") else {
        panic!("no hint");
    };
    assert_eq!(rig, "");
}

/// A shot without a movement of its own gets what the marks do; one with its own gets no hint.
#[test]
fn a_shot_without_a_movement_gets_a_hint() {
    let mut s = open();
    let hint = |s: &Session| s.shots().into_iter().find(|r| r.id == "s12-1").unwrap();

    assert_eq!(hint(&s).movement, Movement::Static.name());
    assert_eq!(hint(&s).movement_hint, "");

    set_lens(&mut s, Some(35.0)).unwrap();
    s.set_time(7.0);
    set_lens(&mut s, Some(85.0)).unwrap();

    assert_eq!(hint(&s).movement_hint, "zoom");
    // The list keeps what is set.
    assert_eq!(hint(&s).movement, "static");
}

/// The tooltip's height is the running one: the later mark's at its time, not the first's.
#[test]
fn the_tooltip_height_is_the_one_at_the_playhead() {
    let mut s = open();
    s.set_time(7.0);
    s.edit_plan(PlanEdit::SetCameraAim {
        source: camera(),
        height_m: 2.4,
        tilt_deg: 0.0,
    })
    .unwrap();
    let height = |s: &Session| match s.camera_hint("a") {
        Some(Hint::Camera { height_m, .. }) => height_m,
        _ => panic!("no hint"),
    };

    assert_eq!(height(&s), 2.4);
    s.set_time(0.0);
    assert_eq!(height(&s), 1.6);
}

/// Camera A's last mark: (9, 8) at 6 s, no pause, aim and height as the file has them. A mark
/// at (9, 5.2) is 2.8 m away: 2.8 / 1.4 = 2 s, so 8 s.
#[test]
fn a_new_camera_mark_arrives_by_dolly_and_keeps_the_aim() {
    let mut s = open();
    let before = s.plan().unwrap().camera("a").unwrap().marks[1].clone();

    s.edit_plan(PlanEdit::AddCameraMark {
        camera: "a".into(),
        at: Point::new(9.0, 5.2),
    })
    .unwrap();

    let marks = &s.plan().unwrap().camera("a").unwrap().marks;
    let new = &marks[2];
    assert_eq!(marks.len(), 3);
    assert!((new.at_s - (before.at_s + before.hold_s + 2.0)).abs() < 1e-9);
    assert_eq!(new.at, Point::new(9.0, 5.2));
    assert_eq!(
        (new.facing_deg, new.height_m, new.tilt_deg, new.lens_mm),
        (
            before.facing_deg,
            before.height_m,
            before.tilt_deg,
            before.lens_mm
        )
    );
    assert_eq!(
        (new.rig, new.hold_s, new.via.clone()),
        (Rig::Dolly, 0.0, vec![])
    );

    s.undo();
    assert_eq!(s.plan().unwrap().camera("a").unwrap().marks.len(), 2);
}

/// The pause of the mark before counts, and its lens carries over.
#[test]
fn a_new_camera_mark_waits_for_the_pause_and_takes_the_lens() {
    let mut s = open();
    let last = s.plan().unwrap().camera("a").unwrap().marks[1].clone();
    s.set_time(last.at_s);
    set_lens(&mut s, Some(85.0)).unwrap();
    s.edit_plan(PlanEdit::SetHold {
        source: camera(),
        hold_s: 3.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::AddCameraMark {
        camera: "a".into(),
        at: last.at,
    })
    .unwrap();

    let marks = &s.plan().unwrap().camera("a").unwrap().marks;
    assert_eq!(marks[2].lens_mm, Some(85.0));
    assert!((marks[2].at_s - (last.at_s + 3.0)).abs() < 1e-9);
}

#[test]
fn a_camera_mark_needs_a_known_camera_and_an_open_layer() {
    let mut s = open();
    let add = |camera: &str| PlanEdit::AddCameraMark {
        camera: camera.into(),
        at: Point::new(1.0, 1.0),
    };

    assert!(matches!(s.edit_plan(add("z")), Err(ServiceError::Gone)));
    assert!(!s.can_undo());

    s.set_lock(Layer::Cameras, Lock::Locked);
    assert!(matches!(
        s.edit_plan(add("a")),
        Err(ServiceError::Locked(Layer::Cameras))
    ));
    assert_eq!(s.plan().unwrap().camera("a").unwrap().marks.len(), 2);
}

/// With a lens on the marks the list shows theirs: 12.1 runs 35 to 85 mm, 12.2 stays on
/// 85 mm; its own 40 mm no longer counts.
#[test]
fn the_shot_list_shows_the_lens_of_the_marks() {
    let mut s = open();
    set_lens(&mut s, Some(35.0)).unwrap();
    s.set_time(7.0);
    set_lens(&mut s, Some(85.0)).unwrap();

    let rows = s.shots();

    assert_eq!(
        (rows[0].lens_mm, rows[0].lens_to_mm, rows[0].lens_from_marks),
        (35.0, Some(85.0), true)
    );
    assert_eq!(
        (rows[1].lens_mm, rows[1].lens_to_mm, rows[1].lens_from_marks),
        (85.0, None, true)
    );
    // The setup follows the lens at the start: 35, 85, 85 mm.
    assert_ne!(rows[0].setup, rows[1].setup);
}

/// The example with camera A leaving mark 1 at 2 s: it runs to mark 2 from 2 to 6 s.
fn example_with_a_way() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("rig-way");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let scene = dir.join("szenen/szene-012.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&scene).unwrap()).unwrap();
    json["plans"][0]["cameras"][0]["marks"][0]["hold_s"] = 2.0.into();
    std::fs::write(&scene, json.to_string()).unwrap();

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

/// On the way the tooltip names the rig of the section: that of the mark the camera runs to.
#[test]
fn the_tooltip_rig_is_the_one_of_the_way() {
    let mut s = Session::open(&example_with_a_way()).unwrap();
    s.set_time(7.0);
    set_rig(&mut s, Rig::Crane).unwrap();
    let rig = |s: &Session| match s.camera_hint("a") {
        Some(Hint::Camera { rig, .. }) => rig,
        _ => panic!("no hint"),
    };

    // Pausing at mark 1 (0 to 2 s), running to mark 2 (2 to 6 s), standing there.
    s.set_time(1.0);
    assert_eq!(rig(&s), "");
    s.set_time(4.0);
    assert_eq!(rig(&s), "crane");
    s.set_time(7.0);
    assert_eq!(rig(&s), "crane");
}

/// The example with camera A turning on the spot: mark 2 at the place of mark 1, from 2 s.
fn example_with_a_pan() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("rig-pan");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let scene = dir.join("szenen/szene-012.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&scene).unwrap()).unwrap();
    let marks = &mut json["plans"][0]["cameras"][0]["marks"];
    marks[0]["hold_s"] = 2.0.into();
    marks[1]["at"] = marks[0]["at"].clone();
    std::fs::write(&scene, json.to_string()).unwrap();

    dir
}

/// Without a change of place the tooltip still names the rig of the section, then of the mark.
#[test]
fn the_tooltip_rig_holds_without_a_change_of_place() {
    let mut s = Session::open(&example_with_a_pan()).unwrap();
    s.set_time(7.0);
    set_rig(&mut s, Rig::Tripod).unwrap();
    let rig = |s: &mut Session, t_s: f64| {
        s.set_time(t_s);
        match s.camera_hint("a") {
            Some(Hint::Camera { rig, .. }) => rig,
            _ => panic!("no hint"),
        }
    };

    // Turning from 2 to 6 s, standing at mark 2 afterwards.
    assert_eq!(rig(&mut s, 4.0), "tripod");
    assert_eq!(rig(&mut s, 7.0), "tripod");
}
