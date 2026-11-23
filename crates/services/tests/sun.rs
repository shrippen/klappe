//! Sunlight on the plan (LH-52), on the example project: the Sängersaal is 20 x 13 m, its south
//! wall (index 2, y = 13) has a door; a window is added in its middle.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{OpeningKind, Site, SunTime};
use klappe_services::{
    Field, Language, Layer, Paper, PlanEdit, Point, ProjectEdit, Role, ServiceError, Session,
    Shape, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

/// The example with a window in the south wall at x = 10.
fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.edit_plan(PlanEdit::AddOpening {
        kind: OpeningKind::Window,
        at: Point::new(10.0, 13.0),
    })
    .unwrap();

    s
}

const BERLIN: Site = Site {
    latitude_deg: 52.52,
    longitude_deg: 13.405,
    north_deg: 0.0,
};

/// Solar noon of the June solstice in Berlin by the clocks of summer time.
fn noon() -> SunTime {
    SunTime {
        year: 2026,
        month: 6,
        day: 21,
        hour: 13,
        minute: 8,
        utc_offset_min: Some(120),
    }
}

fn set_location(s: &mut Session, site: Option<Site>) -> Result<(), ServiceError> {
    let location = s.project().locations[0].clone();

    s.edit_project(ProjectEdit::EditLocation {
        id: location.id,
        name: location.name,
        width_m: location.set.width_m,
        depth_m: location.set.depth_m,
        wall_height_m: location.set.wall_height_m,
        site,
    })
}

fn set_scene(s: &mut Session, sun: Option<SunTime>) -> Result<(), ServiceError> {
    let scene = s.project().scenes[0].clone();

    s.edit_project(ProjectEdit::EditScene {
        id: scene.id,
        number: scene.number,
        heading: scene.heading,
        location: scene.location,
        sun,
    })
}

/// The example with a window, in Berlin at noon.
fn lit() -> Session {
    let mut lit = open();
    set_location(&mut lit, Some(BERLIN)).unwrap();
    set_scene(&mut lit, Some(noon())).unwrap();

    lit
}

fn roles(s: &Session, role: Role) -> usize {
    s.display_list()
        .items
        .iter()
        .filter(|i| i.role == role)
        .count()
}

fn bad(result: Result<(), ServiceError>, wanted: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field, .. }) if field == wanted)
}

#[test]
fn without_a_place_and_a_date_the_plan_has_no_sun_and_no_north() {
    let mut s = open();
    assert_eq!(roles(&s, Role::North), 0);

    set_location(&mut s, Some(BERLIN)).unwrap();
    assert_eq!(roles(&s, Role::North), 0, "a place alone sets no sun");

    set_scene(&mut s, Some(noon())).unwrap();
    set_location(&mut s, None).unwrap();
    assert_eq!(roles(&s, Role::North), 0, "a date alone sets no sun");
}

/// At solar noon of the solstice the sun in Berlin stands 61° high in the south: the window in
/// the south wall lights the floor 0.5 to 1.2 m inside, 1.2 m wide, straight ahead of it.
#[test]
fn the_window_throws_a_patch_into_the_room() {
    let mut s = open();
    set_location(&mut s, Some(BERLIN)).unwrap();
    set_scene(&mut s, Some(noon())).unwrap();

    let list = s.display_list();
    let patches: Vec<_> = list
        .items
        .iter()
        .filter(|i| i.role == Role::SunPatch)
        .collect();
    assert_eq!(patches.len(), 1);
    let Shape::Polygon { points } = &patches[0].shape else {
        panic!("a patch is a polygon");
    };

    let (x_min, x_max) = points.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| {
        (lo.min(p.x_m), hi.max(p.x_m))
    });
    let (y_min, y_max) = points.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| {
        (lo.min(p.y_m), hi.max(p.y_m))
    });
    assert!(
        (x_min - 9.4).abs() < 0.1 && (x_max - 10.6).abs() < 0.1,
        "{points:?}"
    );
    assert!((13.0 - y_max - 0.5).abs() < 0.05, "{points:?}");
    assert!((13.0 - y_min - 1.17).abs() < 0.05, "{points:?}");

    assert_eq!(roles(&s, Role::North), 1);
    assert_eq!(roles(&s, Role::SunRay), 1);
}

#[test]
fn at_night_there_is_no_patch_and_no_ray_but_north() {
    let mut s = lit();
    set_scene(&mut s, Some(SunTime { hour: 1, ..noon() })).unwrap();

    assert_eq!(roles(&s, Role::SunPatch), 0);
    assert_eq!(roles(&s, Role::SunRay), 0);
    assert_eq!(roles(&s, Role::North), 1);
}

#[test]
fn the_sun_follows_the_lights_layer() {
    let mut s = lit();
    assert_eq!(roles(&s, Role::SunPatch), 1);

    s.set_layer(Layer::Lights, Visibility::Hidden);
    assert_eq!(roles(&s, Role::SunPatch), 0);
    assert_eq!(roles(&s, Role::SunRay), 0);
    assert_eq!(roles(&s, Role::North), 0);
}

#[test]
fn a_place_and_a_date_are_one_undo_step_each_and_refused_when_wrong() {
    let mut s = open();

    set_location(&mut s, Some(BERLIN)).unwrap();
    assert_eq!(s.undo_step(), Some("EditLocation"));
    set_scene(&mut s, Some(noon())).unwrap();
    assert_eq!(s.undo_step(), Some("EditScene"));
    assert_eq!(roles(&s, Role::North), 1);

    s.undo();
    assert_eq!(s.project().scenes[0].sun, None);
    assert_eq!(roles(&s, Role::North), 0);

    let off_globe = Site {
        latitude_deg: 91.0,
        ..BERLIN
    };
    assert!(bad(set_location(&mut s, Some(off_globe)), Field::Latitude));
    let no_day = SunTime {
        day: 31,
        month: 4,
        ..noon()
    };
    assert!(bad(set_scene(&mut s, Some(no_day)), Field::Date));
    assert_eq!(s.project().scenes[0].sun, None);
}

#[test]
fn north_is_kept_between_0_and_360() {
    let mut s = open();

    set_location(
        &mut s,
        Some(Site {
            north_deg: -90.0,
            ..BERLIN
        }),
    )
    .unwrap();

    assert_eq!(s.project().locations[0].site.unwrap().north_deg, 270.0);
}

/// Folding 370.1 into the circle leaves 10.100000000000023; the form would show the residue.
#[test]
fn folding_north_leaves_no_rounding_residue() {
    let mut s = open();

    for (typed, kept) in [(370.1, 10.1), (-232.2, 127.8)] {
        set_location(
            &mut s,
            Some(Site {
                north_deg: typed,
                ..BERLIN
            }),
        )
        .unwrap();

        assert_eq!(s.project().locations[0].site.unwrap().north_deg, kept);
    }
}

#[test]
fn the_form_rows_carry_the_place_and_the_date() {
    let mut s = lit();
    set_scene(
        &mut s,
        Some(SunTime {
            utc_offset_min: None,
            ..noon()
        }),
    )
    .unwrap();

    let overview = s.overview();

    assert_eq!(overview.locations[0].latitude, Some(52.52));
    assert_eq!(overview.locations[0].longitude, Some(13.405));
    assert_eq!(overview.locations[0].north, 0.0);
    assert_eq!(overview.scenes[0].sun.sun_date, "2026-06-21");
    assert_eq!(overview.scenes[0].sun.sun_time, "13:08");
    assert_eq!(overview.scenes[0].sun.sun_offset, "");
}

#[test]
fn the_printout_shows_the_patch_like_any_plan_item() {
    let s = lit();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sun.svg");

    s.export_svg(&path, Paper::A4, Language::English.print_texts())
        .unwrap();
    let svg = std::fs::read_to_string(&path).unwrap();

    assert!(svg.contains("sun-patch"), "{svg}");
    assert!(svg.contains("sun-ray"));
    assert!(svg.contains("north"));
}

#[test]
fn place_and_date_survive_saving() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sun-save");
    let _ = std::fs::remove_dir_all(&dir);
    let mut s = Session::create(&dir, "Sonne", Language::German).unwrap();
    s.edit_project(ProjectEdit::AddLocation {
        name: "Saal".into(),
        width_m: 10.0,
        depth_m: 8.0,
        wall_height_m: 2.6,
        site: Some(BERLIN),
    })
    .unwrap();
    s.edit_project(ProjectEdit::AddScene {
        number: "1".into(),
        heading: "INT. SAAL - TAG".into(),
        location: Some("saal".into()),
    })
    .unwrap();
    set_scene(&mut s, Some(noon())).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let back = Session::open(&dir).unwrap();

    assert_eq!(back.project().locations[0].site, Some(BERLIN));
    assert_eq!(back.project().scenes[0].sun, Some(noon()));
}

/// The video shows the sun as the plan does: its frame is the plan's at the same time.
#[test]
fn the_video_shows_the_sun() {
    let installed = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !installed {
        eprintln!("skipped: ffmpeg is not installed (no job without it)");
        return;
    }
    let mut s = open();
    set_location(&mut s, Some(BERLIN)).unwrap();
    set_scene(&mut s, Some(noon())).unwrap();
    let texts = klappe_services::Language::English.print_texts();
    let mut job = s.video_job_at("ffmpeg", &texts, VIDEO_FPS).unwrap();

    let frame = job.frame(0).unwrap();

    s.set_time(0.0);
    let list = s.display_list();
    assert_eq!(roles(&s, Role::SunRay), 1);
    let setup = klappe_services::PaperSetup::default();
    let mut page = klappe_export::Page::new(setup.size, setup.orientation, s.plan_title(), texts);
    page.fade = s.item_fades(&list);
    let expected = klappe_export::frame(&list, &page, klappe_services::Theme::Light).unwrap();
    assert!(
        frame == expected,
        "the video frame differs from the plan with its sun"
    );
}

/// Frames per second of the video in the test; few, so the job is quick.
const VIDEO_FPS: u32 = 5;
