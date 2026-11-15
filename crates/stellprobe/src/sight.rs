use klappe_model::{FigureColor, FigureTrack, FloorPlan, Point, Project, SetPlan, Wall};

use crate::display::{object_corners, FIGURE_RADIUS_M};
use crate::eye::{clip, clip_near, Eye, Pinhole, FRAME_WIDTH};
use crate::figure::body_radius_m;
use crate::height::{figure_height_m, object_extent, opening_span, wall_top};
use crate::hit::in_lasso;
use crate::motion::figure_at;
use crate::opening::solid_spans;

/// Width of a standard figure's placeholder; a figure with its own size is wider in proportion.
const FIGURE_WIDTH_M: f64 = 0.5;

/// What a camera sees at one moment (LH-36): a pinhole projection of the set, flat shapes
/// back to front. Image units: x from 0 to `FRAME_WIDTH`, y from 0 (top) to `height`.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraView {
    pub height: f64,
    /// Where the horizon lies; outside `0..height` when the camera looks far up or down.
    pub horizon_y: f64,
    pub items: Vec<ViewItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewItem {
    pub role: ViewRole,
    /// A closed polygon, clipped to the frame.
    pub points: Vec<[f64; 2]>,
    /// A figure's name.
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewRole {
    /// Everything below the horizon.
    Floor,
    Wall,
    Object,
    Figure(FigureColor),
}

use crate::eye::Vec3;

/// A shape in the set, in metres: x and y as on the plan, z up.
struct Face {
    role: ViewRole,
    corners: Vec<Vec3>,
    label: Option<String>,
}

/// What `camera` sees at `t_s`; `None` for an unknown camera and one without marks.
///
/// ```text
///   plan (from above)         image
///        figure                 ┌─────────────┐
///          ▲                    │    ▮        │   a figure ahead stands in the middle,
///          │ facing             │ ───────────  │   one on the plan's left on the left,
///        camera                 └─────────────┘   one behind the camera is not there
/// ```
pub fn camera_view(
    project: &Project,
    set: &SetPlan,
    plan: &FloorPlan,
    camera: &str,
    t_s: f64,
) -> Option<CameraView> {
    let eye = Pinhole::of(
        &Eye::Shot {
            camera: camera.to_owned(),
            t_s,
        },
        plan,
    )?;

    let mut faces = Vec::new();
    wall_faces(set, &mut faces);
    object_faces(set, &eye, &mut faces);
    figure_faces(project, plan, t_s, &eye, &mut faces);

    // Farthest first, so a near shape covers what lies behind it.
    let far = |face: &Face| -> f64 {
        let n = face.corners.len() as f64;
        let x = face.corners.iter().map(|c| c[0]).sum::<f64>() / n - eye.ground()[0];
        let y = face.corners.iter().map(|c| c[1]).sum::<f64>() / n - eye.ground()[1];
        x.hypot(y)
    };
    faces.sort_by(|a, b| far(b).total_cmp(&far(a)));

    let mut items = Vec::new();
    floor_item(&eye, &mut items);
    for face in faces {
        let Some(points) = flatten(&eye, &face.corners) else {
            continue;
        };
        items.push(ViewItem {
            role: face.role,
            points,
            label: face.label,
        });
    }

    Some(CameraView {
        height: eye.frame_height,
        horizon_y: eye.horizon_y(),
        items,
    })
}

/// A wall stands between its openings to its own top; below a sill and above a head the
/// wall goes on, e.g. a window from 0.90 to 2.10 m leaves wall below and above it.
fn wall_faces(set: &SetPlan, faces: &mut Vec<Face>) {
    for wall in &set.walls {
        let top_m = wall_top(set, wall);
        for (start_m, end_m) in solid_spans(wall) {
            faces.push(wall_face(wall, (start_m, end_m), (0.0, top_m)));
        }

        for opening in &wall.openings {
            let (start_m, end_m) = opening.span_m();
            let span = (start_m.max(0.0), end_m.min(wall.length_m()));
            let (sill_m, head_m) = opening_span(opening);
            if sill_m > 0.0 {
                faces.push(wall_face(wall, span, (0.0, sill_m.min(top_m))));
            }
            if head_m < top_m {
                faces.push(wall_face(wall, span, (head_m.max(0.0), top_m)));
            }
        }
    }
}

/// The piece of `wall` from `along` start to end, between the heights `low` and `high`.
fn wall_face(wall: &Wall, along: (f64, f64), (low_m, high_m): (f64, f64)) -> Face {
    let (a, b) = (wall.point_at(along.0), wall.point_at(along.1));

    Face {
        role: ViewRole::Wall,
        corners: vec![
            [a.x_m, a.y_m, low_m],
            [b.x_m, b.y_m, low_m],
            [b.x_m, b.y_m, high_m],
            [a.x_m, a.y_m, high_m],
        ],
        label: None,
    }
}

/// An object is a box: its outline from its underside to its top; four sides, a lid and,
/// when it floats (a shelf), a bottom.
/// One that stands under the camera (tripod, dolly, crane) is left out: it would fill the picture.
fn object_faces(set: &SetPlan, eye: &Pinhole, faces: &mut Vec<Face>) {
    let [x, y] = eye.ground();
    let camera = Point::new(x, y);

    for (index, object) in set.objects.iter().enumerate() {
        let (base_m, top_m) = object_extent(set, index);
        let outline = object_corners(object);
        if in_lasso(&outline, camera) {
            continue;
        }

        for (i, a) in outline.iter().enumerate() {
            let b = outline[(i + 1) % outline.len()];
            faces.push(Face {
                role: ViewRole::Object,
                corners: vec![
                    [a.x_m, a.y_m, base_m],
                    [b.x_m, b.y_m, base_m],
                    [b.x_m, b.y_m, top_m],
                    [a.x_m, a.y_m, top_m],
                ],
                label: None,
            });
        }

        let mut levels = vec![top_m];
        if base_m > 0.0 {
            levels.push(base_m);
        }
        for level_m in levels {
            faces.push(Face {
                role: ViewRole::Object,
                corners: outline.iter().map(|p| [p.x_m, p.y_m, level_m]).collect(),
                label: None,
            });
        }
    }
}

/// A figure is an upright slab turned to the camera, as tall as its pose.
fn figure_faces(
    project: &Project,
    plan: &FloorPlan,
    t_s: f64,
    eye: &Pinhole,
    faces: &mut Vec<Face>,
) {
    for track in &plan.figures {
        let Some(character) = project.character(&track.character) else {
            continue;
        };
        let Some(place) = figure_at(track, t_s) else {
            continue;
        };

        // Shoulder width scaled like the plan's circle, e.g. a 2.4 m horse: 0.5 * 2.4 / 0.7.
        let half = FIGURE_WIDTH_M / 2.0 * body_radius_m(character) / FIGURE_RADIUS_M;
        let (x, y) = (place.at.x_m, place.at.y_m);
        let (rx, ry) = (eye.right[0] * half, eye.right[1] * half);
        let top = pose_height_m(track, t_s);
        faces.push(Face {
            role: ViewRole::Figure(character.color),
            corners: vec![
                [x - rx, y - ry, 0.0],
                [x + rx, y + ry, 0.0],
                [x + rx, y + ry, top],
                [x - rx, y - ry, top],
            ],
            label: Some(character.title()),
        });
    }
}

/// The pose of the last mark reached by `t_s`, else of the first.
fn pose_height_m(track: &FigureTrack, t_s: f64) -> f64 {
    let reached = track.marks.iter().rev().find(|m| m.at_s <= t_s);
    let pose = reached.or(track.marks.first()).map(|m| m.pose);

    figure_height_m(pose.unwrap_or_default())
}

/// The ground: the frame below the horizon.
fn floor_item(eye: &Pinhole, items: &mut Vec<ViewItem>) {
    let top = eye.horizon_y().max(0.0);
    if top >= eye.frame_height {
        return;
    }

    items.push(ViewItem {
        role: ViewRole::Floor,
        points: vec![
            [0.0, top],
            [FRAME_WIDTH, top],
            [FRAME_WIDTH, eye.frame_height],
            [0.0, eye.frame_height],
        ],
        label: None,
    });
}

/// The face in the image, cut at the near plane and the frame; `None` when nothing is left.
fn flatten(eye: &Pinhole, corners: &[Vec3]) -> Option<Vec<[f64; 2]>> {
    let local: Vec<Vec3> = corners.iter().map(|c| eye.local(*c)).collect();
    let near = clip_near(&local);
    let flat: Vec<[f64; 2]> = near.iter().map(|p| eye.image(*p)).collect();

    // Left, right, top and bottom edge of the frame.
    let height = eye.frame_height;
    let mut shape = flat;
    for (axis, edge, inside) in [
        (0, 0.0, 1.0),
        (0, FRAME_WIDTH, -1.0),
        (1, 0.0, 1.0),
        (1, height, -1.0),
    ] {
        shape = clip(
            &shape,
            |p| (p[axis] - edge) * inside,
            |a, b| {
                let t = (edge - a[axis]) / (b[axis] - a[axis]);
                [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
            },
        );
    }

    (shape.len() >= 3).then_some(shape)
}

#[cfg(test)]
mod tests {
    use klappe_model::{
        Arrival, Camera, CameraMark, Character, Departure, Mark, Opening, OpeningKind, Point, Pose,
        Sensor, SetObject, Wall,
    };

    use super::*;

    fn project() -> Project {
        let mut p = Project::new("Test");
        p.characters = vec![Character {
            id: "e".into(),
            name: "Elisabeth".into(),
            short: "E".into(),
            color: FigureColor::Purple,
            group: None,
            size_m: None,
        }];
        p
    }

    fn camera(lens_mm: f64, height_m: f64, tilt_deg: f64, aspect: f64) -> Camera {
        Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm,
            aspect,
            marks: vec![CameraMark {
                at_s: 0.0,
                at: Point::new(10.0, 10.0),
                facing_deg: -90.0,
                height_m,
                tilt_deg,
                hold_s: 0.0,
                via: Vec::new(),
                rig: Default::default(),
                lens_mm: None,
            }],
            follows: None,
        }
    }

    fn mark(x: f64, y: f64, pose: Pose) -> Mark {
        Mark {
            at_s: 0.0,
            at: Point::new(x, y),
            facing_deg: 0.0,
            pose,
            hold_s: 0.0,
            via: Vec::new(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        }
    }

    fn plan_with(camera: Camera, figures: &[(f64, f64, Pose)]) -> FloorPlan {
        let mut plan = FloorPlan::new("p", "Plan");
        plan.cameras = vec![camera];
        plan.figures = figures
            .iter()
            .map(|(x, y, pose)| FigureTrack {
                character: "e".into(),
                marks: vec![mark(*x, *y, *pose)],
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

    fn view_of(plan: &FloorPlan, set: &SetPlan) -> CameraView {
        camera_view(&project(), set, plan, "a", 0.0).expect("view")
    }

    fn figures(view: &CameraView) -> Vec<&ViewItem> {
        view.items
            .iter()
            .filter(|i| matches!(i.role, ViewRole::Figure(_)))
            .collect()
    }

    /// The middle of the shape's width.
    fn centre_x(item: &ViewItem) -> f64 {
        let xs = item.points.iter().map(|p| p[0]);
        (xs.clone().fold(f64::MAX, f64::min) + xs.fold(f64::MIN, f64::max)) / 2.0
    }

    fn width(item: &ViewItem) -> f64 {
        let xs = item.points.iter().map(|p| p[0]);
        xs.clone().fold(f64::MIN, f64::max) - xs.fold(f64::MAX, f64::min)
    }

    #[test]
    fn figure_ahead_is_in_the_middle() {
        let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[(10.0, 5.0, Pose::Standing)]);
        let view = view_of(&plan, &set());

        let found = figures(&view);
        assert_eq!(found.len(), 1);
        assert!((centre_x(found[0]) - FRAME_WIDTH / 2.0).abs() < 1e-9);
        assert_eq!(found[0].label.as_deref(), Some("Elisabeth"));
    }

    #[test]
    fn figure_left_on_the_plan_is_left_in_the_image() {
        let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[(9.0, 5.0, Pose::Standing)]);
        let view = view_of(&plan, &set());

        assert!(centre_x(figures(&view)[0]) < FRAME_WIDTH / 2.0);
    }

    #[test]
    fn figure_right_on_the_plan_is_right_in_the_image() {
        let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[(11.0, 5.0, Pose::Standing)]);
        let view = view_of(&plan, &set());

        assert!(centre_x(figures(&view)[0]) > FRAME_WIDTH / 2.0);
    }

    #[test]
    fn figure_behind_the_camera_is_not_there() {
        let plan = plan_with(
            camera(35.0, 1.6, 0.0, 2.39),
            &[(10.0, 15.0, Pose::Standing)],
        );

        assert!(figures(&view_of(&plan, &set())).is_empty());
    }

    #[test]
    fn longer_lens_makes_it_larger() {
        let at = [(10.0, 5.0, Pose::Standing)];
        let wide = view_of(&plan_with(camera(25.0, 1.6, 0.0, 2.39), &at), &set());
        let long = view_of(&plan_with(camera(85.0, 1.6, 0.0, 2.39), &at), &set());

        assert!(width(figures(&long)[0]) > width(figures(&wide)[0]));
    }

    #[test]
    fn height_and_tilt_move_the_horizon() {
        let level = view_of(&plan_with(camera(35.0, 1.6, 0.0, 2.39), &[]), &set());
        let up = view_of(&plan_with(camera(35.0, 1.6, 10.0, 2.39), &[]), &set());

        assert!((level.horizon_y - level.height / 2.0).abs() < 1e-9);
        // Tilted up, the horizon sinks in the image.
        assert!(up.horizon_y > level.horizon_y);
    }

    #[test]
    fn height_alone_keeps_the_horizon_but_lifts_the_figure_feet() {
        let at = [(10.0, 5.0, Pose::Standing)];
        let low = view_of(&plan_with(camera(35.0, 1.0, 0.0, 1.0), &at), &set());
        let high = view_of(&plan_with(camera(35.0, 2.0, 0.0, 1.0), &at), &set());
        let feet = |v: &CameraView| {
            figures(v)[0]
                .points
                .iter()
                .map(|p| p[1])
                .fold(f64::MIN, f64::max)
        };

        assert_eq!(low.horizon_y, high.horizon_y);
        assert!(feet(&high) > feet(&low));
    }

    #[test]
    fn aspect_sets_the_height() {
        let plan = |aspect| plan_with(camera(35.0, 1.6, 0.0, aspect), &[]);

        assert!((view_of(&plan(2.0), &set()).height - 0.5).abs() < 1e-9);
        assert!((view_of(&plan(1.0), &set()).height - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sitting_is_lower_than_standing() {
        let top = |pose| {
            let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[(10.0, 5.0, pose)]);
            let view = view_of(&plan, &set());
            figures(&view)[0]
                .points
                .iter()
                .map(|p| p[1])
                .fold(f64::MAX, f64::min)
        };

        assert!(top(Pose::Sitting) > top(Pose::Standing));
    }

    #[test]
    fn a_larger_figure_is_wider_by_its_size() {
        let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[(10.0, 5.0, Pose::Standing)]);
        let standard = width(figures(&view_of(&plan, &set()))[0]);

        // A horse: 2.4 m across on the plan against 0.7 m for a person.
        let mut horse = project();
        horse.characters[0].size_m = Some(2.4);
        let view = camera_view(&horse, &set(), &plan, "a", 0.0).expect("view");
        let wide = width(figures(&view)[0]);

        assert!((wide / standard - 2.4 / (2.0 * FIGURE_RADIUS_M)).abs() < 1e-9);
    }

    #[test]
    fn wall_and_object_ahead_are_drawn_and_far_first() {
        let mut set = set();
        set.walls = vec![Wall {
            from: Point::new(5.0, 2.0),
            to: Point::new(15.0, 2.0),
            thickness_m: 0.2,
            openings: Vec::new(),
            height_m: None,
        }];
        set.objects = vec![SetObject {
            id: "t".into(),
            kind: "table".into(),
            label: String::new(),
            at: Point::new(10.0, 6.0),
            width_m: 1.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
            shape: Default::default(),
            height_m: None,
            base_m: None,
        }];
        let view = view_of(&plan_with(camera(35.0, 1.6, 0.0, 1.78), &[]), &set);

        let roles: Vec<_> = view.items.iter().map(|i| i.role).collect();
        let wall = roles
            .iter()
            .position(|r| *r == ViewRole::Wall)
            .expect("wall");
        let object = roles
            .iter()
            .position(|r| *r == ViewRole::Object)
            .expect("object");
        assert_eq!(roles[0], ViewRole::Floor);
        assert!(wall < object);
    }

    /// A window from 0.90 to 2.10 m, the camera at 1.6 m: wall below the window and above it,
    /// the lower piece under the horizon, the upper one over it.
    #[test]
    fn a_window_has_wall_below_and_above() {
        let mut set = set();
        let mut wall = Wall::new(Point::new(5.0, 2.0), Point::new(15.0, 2.0), 0.2);
        wall.openings = vec![Opening::new(OpeningKind::Window, 5.0, 4.0)];
        set.walls = vec![wall];
        let view = view_of(&plan_with(camera(35.0, 1.6, 0.0, 1.78), &[]), &set);

        let centre: Vec<_> = view
            .items
            .iter()
            .filter(|i| i.role == ViewRole::Wall)
            .filter(|i| {
                let left = i.points.iter().any(|p| p[0] < FRAME_WIDTH / 2.0);
                let right = i.points.iter().any(|p| p[0] > FRAME_WIDTH / 2.0);
                left && right
            })
            .collect();
        assert_eq!(centre.len(), 2);
        assert!(centre
            .iter()
            .any(|i| i.points.iter().all(|p| p[1] >= view.horizon_y)));
        assert!(centre
            .iter()
            .any(|i| i.points.iter().all(|p| p[1] <= view.horizon_y)));
    }

    /// A door from 8 m to 12 m in a wall from 5 m to 15 m: the camera at x 10 looks through it,
    /// under the wall above the door (2.10 to 2.60 m), which lies above the horizon.
    #[test]
    fn a_door_leaves_a_gap_in_the_wall() {
        let mut set = set();
        set.walls = vec![Wall {
            from: Point::new(5.0, 2.0),
            to: Point::new(15.0, 2.0),
            thickness_m: 0.2,
            openings: vec![Opening::new(OpeningKind::Door, 5.0, 4.0)],
            height_m: None,
        }];
        let view = view_of(&plan_with(camera(35.0, 1.6, 0.0, 1.78), &[]), &set);

        let walls: Vec<_> = view
            .items
            .iter()
            .filter(|i| i.role == ViewRole::Wall)
            .collect();
        assert_eq!(walls.len(), 3);
        for wall in walls {
            if wall.points.iter().all(|p| p[1] < view.horizon_y) {
                continue;
            }
            let xs = wall.points.iter().map(|p| p[0]);
            let (left, right) = (
                xs.clone().fold(f64::MAX, f64::min),
                xs.fold(f64::MIN, f64::max),
            );
            assert!(
                right < FRAME_WIDTH / 2.0 || left > FRAME_WIDTH / 2.0,
                "{left}..{right}"
            );
        }
    }

    /// Tripod, dolly or crane stand under the camera: the picture must stay free.
    #[test]
    fn object_under_the_camera_does_not_cover_the_view() {
        let mut set = set();
        set.objects = vec![SetObject {
            id: "tripod".into(),
            kind: "tripod".into(),
            label: String::new(),
            at: Point::new(10.0, 10.0),
            width_m: 1.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
            shape: Default::default(),
            height_m: Some(1.6),
            base_m: None,
        }];
        let plan = plan_with(camera(35.0, 1.5, 0.0, 1.78), &[(10.0, 5.0, Pose::Standing)]);
        let view = view_of(&plan, &set);

        assert!(!view.items.iter().any(|i| i.role == ViewRole::Object));
        assert_eq!(figures(&view).len(), 1);
    }

    /// Image numbers worked out by hand. Camera (10, 10) facing -90 degrees, 1.6 m high, Super 35,
    /// 35 mm, aspect 1: f = 35 / 24.89 = 1.40619 of the frame width. A standing figure 5 m ahead
    /// is 0.5 m wide and 1.75 m tall:
    ///   x    = 0.5 +- 0.25 / 5 * f                = 0.42969 .. 0.57031
    ///   feet = 0.5 + 1.6 / 5 * f                  = 0.94998
    ///   head = 0.5 - (1.75 - 1.6) / 5 * f         = 0.45781
    ///   horizon with tilt t: 0.5 + tan(t) * f      = 0.74795 (+10), 0.25205 (-10)
    ///   feet with tilt -10: depth 5 cos10 - 1.6 sin10 = 5.20188, up -1.6 cos10 + 5 sin10 = -0.70746
    ///                       y = 0.5 + 0.70746 / 5.20188 * f = 0.69124
    const TOLERANCE: f64 = 1e-5;

    /// Left, right, top and bottom of the one figure.
    fn figure_box(tilt_deg: f64) -> (CameraView, [f64; 4]) {
        let plan = plan_with(
            camera(35.0, 1.6, tilt_deg, 1.0),
            &[(10.0, 5.0, Pose::Standing)],
        );
        let view = view_of(&plan, &set());
        let points = &figures(&view)[0].points;
        let range = |axis: usize, pick: fn(f64, f64) -> f64, start: f64| {
            points.iter().map(|p| p[axis]).fold(start, pick)
        };
        let bounds = [
            range(0, f64::min, f64::MAX),
            range(0, f64::max, f64::MIN),
            range(1, f64::min, f64::MAX),
            range(1, f64::max, f64::MIN),
        ];

        (view, bounds)
    }

    #[test]
    fn standing_figure_has_the_hand_computed_image() {
        let (view, [left, right, head, feet]) = figure_box(0.0);

        assert!((left - 0.42969).abs() < TOLERANCE, "left {left}");
        assert!((right - 0.57031).abs() < TOLERANCE, "right {right}");
        assert!((head - 0.45781).abs() < TOLERANCE, "head {head}");
        assert!((feet - 0.94998).abs() < TOLERANCE, "feet {feet}");
        assert!((view.horizon_y - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn tilting_up_puts_the_horizon_at_0_74795() {
        let (view, _) = figure_box(10.0);

        assert!(
            (view.horizon_y - 0.74795).abs() < TOLERANCE,
            "{}",
            view.horizon_y
        );
    }

    #[test]
    fn tilting_down_puts_the_horizon_at_0_25205_and_the_feet_at_0_69124() {
        let (view, [_, _, _, feet]) = figure_box(-10.0);

        assert!(
            (view.horizon_y - 0.25205).abs() < TOLERANCE,
            "{}",
            view.horizon_y
        );
        assert!((feet - 0.69124).abs() < TOLERANCE, "feet {feet}");
    }

    #[test]
    fn unknown_camera_has_no_view() {
        let plan = plan_with(camera(35.0, 1.6, 0.0, 2.39), &[]);

        assert!(camera_view(&project(), &set(), &plan, "x", 0.0).is_none());
    }
}
