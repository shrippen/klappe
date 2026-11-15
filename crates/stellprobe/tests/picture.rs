//! The camera's picture as pixels (LH-36, LH-99; 3D-KONZEPT.md, section 5): where things land,
//! what covers what, which names show.
#![allow(clippy::expect_used)]

use klappe_model::{
    Arrival, Camera, CameraMark, Character, Departure, FigureColor, FigureTrack, FloorPlan, Mark,
    Point, Pose, Project, Sensor, SetPlan, Wall,
};
use klappe_stellprobe::{floor_at, picture, source_at, Eye, Orbit, Picture, SceneRole, Source};

const WIDTH_PX: usize = 160;

fn project() -> Project {
    let mut p = Project::new("Test");
    p.characters = ["e", "w"]
        .iter()
        .zip([FigureColor::Purple, FigureColor::Green])
        .map(|(id, color)| Character {
            id: (*id).into(),
            name: id.to_uppercase(),
            short: id.to_uppercase(),
            color,
            group: None,
            size_m: None,
        })
        .collect();
    p
}

/// Camera at (10, 10), 1.6 m high, looking up the plan (towards smaller y), level.
fn plan(lens_mm: f64, figures: &[(&str, f64, f64)]) -> FloorPlan {
    let mut plan = FloorPlan::new("p", "Plan");
    plan.cameras = vec![Camera {
        id: "a".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm,
        aspect: 16.0 / 9.0,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(10.0, 10.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: Vec::new(),
            rig: Default::default(),
            lens_mm: None,
        }],
        follows: None,
    }];
    plan.figures = figures
        .iter()
        .map(|(id, x, y)| FigureTrack {
            character: (*id).into(),
            marks: vec![Mark {
                at_s: 0.0,
                at: Point::new(*x, *y),
                facing_deg: 0.0,
                pose: Pose::Standing,
                hold_s: 0.0,
                via: Vec::new(),
                arrival: Arrival::Walk,
                departure: Departure::Stay,
            }],
        })
        .collect();
    plan
}

fn set() -> SetPlan {
    SetPlan {
        width_m: 20.0,
        depth_m: 20.0,
        ..SetPlan::default()
    }
}

fn shot() -> Eye {
    Eye::Shot {
        camera: "a".into(),
        t_s: 0.0,
    }
}

fn take(set: &SetPlan, plan: &FloorPlan) -> Picture {
    picture(&project(), set, plan, &shot(), WIDTH_PX).expect("picture")
}

/// The columns where the figure of colour `color` shows, left and right.
fn columns(picture: &Picture, color: FigureColor) -> Option<(usize, usize)> {
    let code = SceneRole::Figure(color).code();
    let frame = &picture.frame;
    let xs = (0..frame.role.len())
        .filter(|i| frame.role[*i] == code)
        .map(|i| i % frame.width);

    Some((xs.clone().min()?, xs.max()?))
}

#[test]
fn the_picture_has_the_cameras_aspect() {
    let picture = take(&set(), &plan(35.0, &[]));

    assert_eq!((picture.frame.width, picture.frame.height), (160, 90));
}

#[test]
fn a_figure_ahead_stands_in_the_middle_and_one_to_the_left_on_the_left() {
    let picture = take(&set(), &plan(35.0, &[("e", 10.0, 5.0), ("w", 8.0, 5.0)]));

    let (left, right) = columns(&picture, FigureColor::Purple).expect("ahead");
    assert!(
        left < WIDTH_PX / 2 && right > WIDTH_PX / 2,
        "{left}..{right}"
    );
    let (_, right) = columns(&picture, FigureColor::Green).expect("left");
    assert!(right < left, "{right}");
}

#[test]
fn a_figure_behind_the_camera_is_not_there() {
    let picture = take(&set(), &plan(35.0, &[("e", 10.0, 14.0)]));

    assert_eq!(columns(&picture, FigureColor::Purple), None);
    assert!(picture.labels.is_empty());
}

#[test]
fn a_longer_lens_makes_it_wider() {
    let width = |lens_mm| {
        let (l, r) = columns(
            &take(&set(), &plan(lens_mm, &[("e", 10.0, 5.0)])),
            FigureColor::Purple,
        )
        .expect("figure");
        r - l
    };

    assert!(width(85.0) > width(35.0) * 2);
}

/// A wall between camera and figure hides the figure and its name (LH-99): covering is right
/// per pixel, not by sorting.
#[test]
fn a_wall_in_front_hides_the_figure_and_its_name() {
    let mut set = set();
    set.walls = vec![Wall::new(Point::new(5.0, 7.0), Point::new(15.0, 7.0), 0.2)];
    set.wall_height_m = 3.0;

    let picture = take(&set, &plan(35.0, &[("e", 10.0, 5.0)]));

    assert_eq!(columns(&picture, FigureColor::Purple), None);
    assert!(picture.labels.is_empty());
    let middle = picture.frame.width * picture.frame.height / 2 + picture.frame.width / 2;
    assert_eq!(picture.frame.role[middle], SceneRole::Wall.code());
}

/// A low wall (1 m) hides the legs but not the head; the name stays.
#[test]
fn a_low_wall_hides_only_what_is_below_its_top() {
    let mut set = set();
    let mut wall = Wall::new(Point::new(5.0, 7.0), Point::new(15.0, 7.0), 0.2);
    wall.height_m = Some(1.0);
    set.walls = vec![wall];

    let picture = take(&set, &plan(35.0, &[("e", 10.0, 5.0)]));

    assert!(columns(&picture, FigureColor::Purple).is_some());
    assert_eq!(picture.labels.len(), 1);
    assert_eq!(picture.labels[0].text, "E");
}

/// Each pixel knows what it shows: a click on the figure hits the figure.
#[test]
fn a_pixel_names_its_source() {
    let picture = take(&set(), &plan(35.0, &[("e", 10.0, 5.0)]));
    let code = SceneRole::Figure(FigureColor::Purple).code();
    let i = picture
        .frame
        .role
        .iter()
        .position(|r| *r == code)
        .expect("figure");

    let source = &picture.sources[picture.frame.source[i] as usize];
    assert_eq!(source, &Source::Figure("e".into()));
}

#[test]
fn the_floor_lies_below_the_horizon_only() {
    let picture = take(&set(), &plan(35.0, &[]));
    let frame = &picture.frame;
    let floor = SceneRole::Floor.code();

    assert_ne!(frame.role[0], floor);
    assert_eq!(frame.role[frame.role.len() - 1], floor);
}

/// The first row of the figure of colour `color`.
fn top_row(picture: &Picture, color: FigureColor) -> Option<usize> {
    let code = SceneRole::Figure(color).code();
    let frame = &picture.frame;

    (0..frame.role.len())
        .find(|i| frame.role[*i] == code)
        .map(|i| i / frame.width)
}

#[test]
fn sitting_is_lower_than_standing() {
    let top = |pose| {
        let mut plan = plan(35.0, &[("e", 10.0, 5.0)]);
        plan.figures[0].marks[0].pose = pose;

        top_row(&take(&set(), &plan), FigureColor::Purple).expect("figure")
    };

    assert!(top(Pose::Sitting) > top(Pose::Standing));
}

/// The free view over the set's middle, 4:3.
fn orbit_eye(orbit: Orbit) -> Eye {
    Eye::Orbit {
        orbit,
        t_s: 0.0,
        aspect: 4.0 / 3.0,
    }
}

/// The free view opens on the middle of the set: the floor point in the picture's centre is
/// the set's centre (LH-94).
#[test]
fn the_free_view_looks_at_the_middle_of_the_set() {
    let set = set();
    let eye = orbit_eye(Orbit::fitting(set.width_m, set.depth_m));
    let width = WIDTH_PX as f64;

    let middle = floor_at(
        &plan(35.0, &[]),
        &eye,
        width,
        (width / 2.0, width * 3.0 / 8.0),
    );

    let middle = middle.expect("floor");
    assert!(
        (middle.x_m - 10.0).abs() < 1e-6 && (middle.y_m - 10.0).abs() < 1e-6,
        "{middle:?}"
    );
}

/// Above the horizon there is no floor to click.
#[test]
fn the_sky_has_no_floor() {
    let set = set();
    let mut orbit = Orbit::fitting(set.width_m, set.depth_m);
    orbit.pitch_deg = 5.0;

    assert_eq!(
        floor_at(&plan(35.0, &[]), &orbit_eye(orbit), 160.0, (80.0, 0.0)),
        None
    );
}

/// The wheel brings the floor under the pointer closer, and it stays under the pointer.
#[test]
fn zooming_keeps_the_point_under_the_pointer() {
    let set = set();
    let plan = plan(35.0, &[]);
    let mut orbit = Orbit::fitting(set.width_m, set.depth_m);
    let pointer = (40.0, 90.0);
    let before = floor_at(&plan, &orbit_eye(orbit), 160.0, pointer).expect("floor");

    orbit.zoom(2.0, Some(before));

    let after = floor_at(&plan, &orbit_eye(orbit), 160.0, pointer).expect("floor");
    assert!(before.distance_m(after) < 1e-6, "{before:?} {after:?}");
}

/// The free view shows the camera as a body you can click; its own picture does not.
#[test]
fn a_camera_shows_in_the_free_view_but_not_in_its_own_picture() {
    let set = set();
    let plan = plan(35.0, &[]);
    let free = picture(
        &project(),
        &set,
        &plan,
        &orbit_eye(Orbit::fitting(20.0, 20.0)),
        WIDTH_PX,
    );
    let own = take(&set, &plan);
    let camera = SceneRole::Camera.code();

    let free = free.expect("picture");
    let i = free
        .frame
        .role
        .iter()
        .position(|r| *r == camera)
        .expect("camera body");
    assert_eq!(
        source_at(&free, (i % free.frame.width, i / free.frame.width)),
        Some(&Source::Camera("a".into()))
    );
    assert!(!own.frame.role.contains(&camera));
}
