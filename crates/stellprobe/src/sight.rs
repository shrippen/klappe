use klappe_model::{Camera, FigureColor, FigureTrack, FloorPlan, Point, Pose, Project, SetPlan};

use crate::display::object_corners;
use crate::hit::in_lasso;
use crate::motion::{aim_at, camera_at, figure_at};
use crate::optics::{horizontal_fov_deg, lens_at};

/// Image width; the height is `1 / aspect`.
pub const FRAME_WIDTH: f64 = 1.0;
/// Walls have no height in the model.
const WALL_HEIGHT_M: f64 = 2.6;
/// An object without `height_m`.
const OBJECT_HEIGHT_M: f64 = 0.9;
const STANDING_M: f64 = 1.75;
const SITTING_M: f64 = 1.25;
const LYING_M: f64 = 0.4;
/// Width of a figure's placeholder.
const FIGURE_WIDTH_M: f64 = 0.5;
/// Nothing nearer than this is drawn.
const NEAR_M: f64 = 0.1;
/// Keeps `tan` finite when the camera looks straight up or down.
const MAX_TILT_DEG: f64 = 89.0;

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

type Vec3 = [f64; 3];

/// A shape in the set, in metres: x and y as on the plan, z up.
struct Face {
    role: ViewRole,
    corners: Vec<Vec3>,
    label: Option<String>,
}

/// The camera: where it stands, where it looks, how wide it sees.
struct Eye {
    at: [f64; 2],
    height_m: f64,
    forward: [f64; 2],
    right: [f64; 2],
    tilt: (f64, f64),
    focal: f64,
    frame_height: f64,
}

impl Eye {
    /// `p` in the camera's frame: right, up, depth.
    fn local(&self, p: Vec3) -> Vec3 {
        let (dx, dy, up) = (p[0] - self.at[0], p[1] - self.at[1], p[2] - self.height_m);
        let ahead = dx * self.forward[0] + dy * self.forward[1];
        let (sin, cos) = self.tilt;

        [
            dx * self.right[0] + dy * self.right[1],
            up * cos - ahead * sin,
            ahead * cos + up * sin,
        ]
    }

    /// The image point of a point in the camera's frame; the depth is above `NEAR_M`.
    fn image(&self, p: Vec3) -> [f64; 2] {
        [
            FRAME_WIDTH / 2.0 + p[0] / p[2] * self.focal,
            self.frame_height / 2.0 - p[1] / p[2] * self.focal,
        ]
    }

    fn horizon_y(&self) -> f64 {
        self.frame_height / 2.0 + self.tilt.0 / self.tilt.1 * self.focal
    }
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
    let camera = plan.camera(camera)?;
    let eye = eye_of(camera, plan, t_s)?;

    let mut faces = Vec::new();
    wall_faces(set, &mut faces);
    object_faces(set, &eye, &mut faces);
    figure_faces(project, plan, t_s, &eye, &mut faces);

    // Farthest first, so a near shape covers what lies behind it.
    let far = |face: &Face| -> f64 {
        let n = face.corners.len() as f64;
        let x = face.corners.iter().map(|c| c[0]).sum::<f64>() / n - eye.at[0];
        let y = face.corners.iter().map(|c| c[1]).sum::<f64>() / n - eye.at[1];
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

fn eye_of(camera: &Camera, plan: &FloorPlan, t_s: f64) -> Option<Eye> {
    let place = camera_at(camera, t_s)?;
    let (height_m, tilt_deg) = aim_at(camera, t_s)?;
    let fov = horizontal_fov_deg(camera.sensor, lens_at(camera, &plan.shots, t_s));
    if fov <= 0.0 || camera.aspect <= 0.0 {
        return None;
    }

    let (sin, cos) = place.facing_deg.to_radians().sin_cos();
    let tilt = tilt_deg.clamp(-MAX_TILT_DEG, MAX_TILT_DEG).to_radians();

    Some(Eye {
        at: [place.at.x_m, place.at.y_m],
        height_m,
        forward: [cos, sin],
        // On the plan y grows downwards, so the right hand of one facing +x is +y.
        right: [-sin, cos],
        tilt: tilt.sin_cos(),
        focal: FRAME_WIDTH / 2.0 / (fov.to_radians() / 2.0).tan(),
        frame_height: FRAME_WIDTH / camera.aspect,
    })
}

fn wall_faces(set: &SetPlan, faces: &mut Vec<Face>) {
    for wall in &set.walls {
        let (a, b) = (wall.from, wall.to);
        faces.push(Face {
            role: ViewRole::Wall,
            corners: vec![
                [a.x_m, a.y_m, 0.0],
                [b.x_m, b.y_m, 0.0],
                [b.x_m, b.y_m, WALL_HEIGHT_M],
                [a.x_m, a.y_m, WALL_HEIGHT_M],
            ],
            label: None,
        });
    }
}

/// An object is a box: its outline raised to its height; four sides and a lid.
/// One that stands under the camera (tripod, dolly, crane) is left out: it would fill the picture.
fn object_faces(set: &SetPlan, eye: &Eye, faces: &mut Vec<Face>) {
    let camera = Point::new(eye.at[0], eye.at[1]);

    for object in &set.objects {
        let height_m = object.height_m.unwrap_or(OBJECT_HEIGHT_M);
        let outline = object_corners(object);
        if in_lasso(&outline, camera) {
            continue;
        }

        for (i, a) in outline.iter().enumerate() {
            let b = outline[(i + 1) % outline.len()];
            faces.push(Face {
                role: ViewRole::Object,
                corners: vec![
                    [a.x_m, a.y_m, 0.0],
                    [b.x_m, b.y_m, 0.0],
                    [b.x_m, b.y_m, height_m],
                    [a.x_m, a.y_m, height_m],
                ],
                label: None,
            });
        }
        faces.push(Face {
            role: ViewRole::Object,
            corners: outline.iter().map(|p| [p.x_m, p.y_m, height_m]).collect(),
            label: None,
        });
    }
}

/// A figure is an upright slab turned to the camera, as tall as its pose.
fn figure_faces(project: &Project, plan: &FloorPlan, t_s: f64, eye: &Eye, faces: &mut Vec<Face>) {
    for track in &plan.figures {
        let Some(character) = project.character(&track.character) else {
            continue;
        };
        let Some(place) = figure_at(track, t_s) else {
            continue;
        };

        let half = FIGURE_WIDTH_M / 2.0;
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

    match pose.unwrap_or_default() {
        Pose::Standing => STANDING_M,
        Pose::Sitting => SITTING_M,
        Pose::Lying => LYING_M,
    }
}

/// The ground: the frame below the horizon.
fn floor_item(eye: &Eye, items: &mut Vec<ViewItem>) {
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
fn flatten(eye: &Eye, corners: &[Vec3]) -> Option<Vec<[f64; 2]>> {
    let local: Vec<Vec3> = corners.iter().map(|c| eye.local(*c)).collect();
    let near = clip(
        &local,
        |p| p[2] - NEAR_M,
        |a, b| {
            let t = (NEAR_M - a[2]) / (b[2] - a[2]);
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, NEAR_M]
        },
    );
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

/// Sutherland-Hodgman: keeps the part of `polygon` where `side` is not negative; `cut` gives
/// the point where an edge crosses the line.
fn clip<const N: usize>(
    polygon: &[[f64; N]],
    side: impl Fn(&[f64; N]) -> f64,
    cut: impl Fn(&[f64; N], &[f64; N]) -> [f64; N],
) -> Vec<[f64; N]> {
    let mut kept = Vec::new();
    for (i, a) in polygon.iter().enumerate() {
        let b = &polygon[(i + 1) % polygon.len()];
        let (a_in, b_in) = (side(a) >= 0.0, side(b) >= 0.0);
        if a_in {
            kept.push(*a);
        }
        if a_in != b_in {
            kept.push(cut(a, b));
        }
    }

    kept
}

#[cfg(test)]
mod tests {
    use klappe_model::{
        Arrival, CameraMark, Character, Departure, Mark, Point, Sensor, SetObject, Wall,
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
                via: None,
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
            via: None,
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
    fn wall_and_object_ahead_are_drawn_and_far_first() {
        let mut set = set();
        set.walls = vec![Wall {
            from: Point::new(5.0, 2.0),
            to: Point::new(15.0, 2.0),
            thickness_m: 0.2,
            openings: Vec::new(),
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
