use klappe_model::{FigureColor, FloorPlan, Point, Project, SetObject, SetPlan};

use crate::axis::axis_at;
use crate::motion::{camera_at, figure_at};
use crate::optics::{frustum, horizontal_fov_deg};

/// What the GUI draws and the export writes, back to front, in metres (planung/KONZEPT.md,
/// section 4). Colours are roles; the GUI maps them to Kante.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayList {
    pub items: Vec<Item>,
    pub width_m: f64,
    pub depth_m: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub role: Role,
    pub shape: Shape,
    /// Text drawn next to the shape, e.g. a figure's name or a mark's number.
    pub label: Option<String>,
    /// What the item belongs to, for selection and tooltips.
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Line {
        from: Point,
        to: Point,
        width_m: f64,
    },
    Polyline {
        points: Vec<Point>,
    },
    Polygon {
        points: Vec<Point>,
    },
    Circle {
        center: Point,
        radius_m: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Wall,
    Object,
    LightBeam,
    Light,
    Axis,
    CameraPath,
    Frustum,
    Camera,
    FigurePath(FigureColor),
    Mark(FigureColor),
    Figure(FigureColor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Object(String),
    Light(String),
    Camera(String),
    Figure(String),
    Mark { character: String, index: usize },
}

/// Figure body radius on the plan.
const FIGURE_RADIUS_M: f64 = 0.35;
/// Mark circles are smaller than the figure.
const MARK_RADIUS_M: f64 = 0.22;
/// Camera and lamp bodies.
const BODY_RADIUS_M: f64 = 0.25;
/// How far a lamp's beam is drawn.
const BEAM_REACH_M: f64 = 4.0;
/// How far a camera's view is drawn when the set gives no size.
const VIEW_REACH_FALLBACK_M: f64 = 8.0;
/// The facing wedge: how far it reaches beyond the body and how wide it is.
const NOSE_M: f64 = 0.25;
const NOSE_HALF_DEG: f64 = 35.0;

/// The display list of a floor plan on its location's set at time `t_s` (LH-60).
pub fn render(project: &Project, set: &SetPlan, plan: &FloorPlan, t_s: f64) -> DisplayList {
    let mut items = Vec::new();
    let reach_m = set_diagonal_m(set).unwrap_or(VIEW_REACH_FALLBACK_M);

    set_items(set, &mut items);
    light_items(plan, &mut items);
    axis_items(plan, t_s, reach_m, &mut items);
    camera_items(plan, t_s, reach_m, &mut items);
    figure_items(project, plan, t_s, &mut items);

    DisplayList {
        items,
        width_m: set.width_m,
        depth_m: set.depth_m,
    }
}

fn set_diagonal_m(set: &SetPlan) -> Option<f64> {
    let d = set.width_m.hypot(set.depth_m);
    (d > 0.0).then_some(d)
}

fn set_items(set: &SetPlan, items: &mut Vec<Item>) {
    for wall in &set.walls {
        items.push(Item {
            role: Role::Wall,
            shape: Shape::Line {
                from: wall.from,
                to: wall.to,
                width_m: wall.thickness_m,
            },
            label: None,
            source: None,
        });
    }

    for object in &set.objects {
        items.push(Item {
            role: Role::Object,
            shape: Shape::Polygon {
                points: object_corners(object),
            },
            label: Some(object.label.clone()).filter(|l| !l.is_empty()),
            source: Some(Source::Object(object.id.clone())),
        });
    }
}

/// The four corners of a set object, rotated about its centre.
fn object_corners(o: &SetObject) -> Vec<Point> {
    let (hw, hd) = (o.width_m / 2.0, o.depth_m / 2.0);
    let (sin, cos) = o.rotation_deg.to_radians().sin_cos();

    [(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)]
        .iter()
        .map(|(x, y)| Point::new(o.at.x_m + x * cos - y * sin, o.at.y_m + x * sin + y * cos))
        .collect()
}

fn light_items(plan: &FloorPlan, items: &mut Vec<Item>) {
    for light in &plan.lights {
        let source = Some(Source::Light(light.id.clone()));
        let beam = frustum(light.at, light.facing_deg, light.beam_deg, BEAM_REACH_M);

        items.push(Item {
            role: Role::LightBeam,
            shape: Shape::Polygon {
                points: beam.to_vec(),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Light,
            shape: Shape::Circle {
                center: light.at,
                radius_m: BODY_RADIUS_M,
            },
            label: Some(light.label.clone()),
            source,
        });
    }
}

/// The axis drawn across the whole set, not only between the two figures.
fn axis_items(plan: &FloorPlan, t_s: f64, reach_m: f64, items: &mut Vec<Item>) {
    let Some(axis) = axis_at(plan, t_s) else {
        return;
    };
    let length = axis.a.distance_m(axis.b);
    if length == 0.0 {
        return;
    }

    let k = reach_m / length;
    let from = axis.a.lerp(axis.b, -k);
    let to = axis.a.lerp(axis.b, 1.0 + k);
    items.push(Item {
        role: Role::Axis,
        shape: Shape::Line {
            from,
            to,
            width_m: 0.0,
        },
        label: None,
        source: None,
    });
}

fn camera_items(plan: &FloorPlan, t_s: f64, reach_m: f64, items: &mut Vec<Item>) {
    for camera in &plan.cameras {
        let source = Some(Source::Camera(camera.id.clone()));

        if camera.marks.len() > 1 {
            items.push(Item {
                role: Role::CameraPath,
                shape: Shape::Polyline {
                    points: camera.marks.iter().map(|m| m.at).collect(),
                },
                label: None,
                source: source.clone(),
            });
        }

        let Some(place) = camera_at(camera, t_s) else {
            continue;
        };
        let fov = horizontal_fov_deg(camera.sensor, camera.lens_mm);
        let view = frustum(place.at, place.facing_deg, fov, reach_m);

        items.push(Item {
            role: Role::Frustum,
            shape: Shape::Polygon {
                points: view.to_vec(),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Camera,
            shape: Shape::Polygon {
                points: nose(place.at, place.facing_deg, BODY_RADIUS_M),
            },
            label: Some(camera.label.clone()),
            source,
        });
    }
}

fn figure_items(project: &Project, plan: &FloorPlan, t_s: f64, items: &mut Vec<Item>) {
    for track in &plan.figures {
        let Some(character) = project.character(&track.character) else {
            continue;
        };
        let color = character.color;

        if track.marks.len() > 1 {
            items.push(Item {
                role: Role::FigurePath(color),
                shape: Shape::Polyline {
                    points: track.marks.iter().map(|m| m.at).collect(),
                },
                label: None,
                source: Some(Source::Figure(track.character.clone())),
            });
        }

        // Numbered marks 1, 2, 3 …
        for (index, mark) in track.marks.iter().enumerate() {
            items.push(Item {
                role: Role::Mark(color),
                shape: Shape::Circle {
                    center: mark.at,
                    radius_m: MARK_RADIUS_M,
                },
                label: Some((index + 1).to_string()),
                source: Some(Source::Mark {
                    character: track.character.clone(),
                    index,
                }),
            });
        }

        let Some(place) = figure_at(track, t_s) else {
            continue;
        };
        let source = Some(Source::Figure(track.character.clone()));
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Polygon {
                points: nose(place.at, place.facing_deg, FIGURE_RADIUS_M),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Circle {
                center: place.at,
                radius_m: FIGURE_RADIUS_M,
            },
            label: Some(character.name.clone()),
            source,
        });
    }
}

/// A wedge in front of a body of `radius_m`, pointing where it faces.
fn nose(at: Point, facing_deg: f64, radius_m: f64) -> Vec<Point> {
    let point = |deg: f64, r: f64| {
        let a = deg.to_radians();
        Point::new(at.x_m + r * a.cos(), at.y_m + r * a.sin())
    };

    vec![
        point(facing_deg - NOSE_HALF_DEG, radius_m),
        point(facing_deg, radius_m + NOSE_M),
        point(facing_deg + NOSE_HALF_DEG, radius_m),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{
        Camera, CameraMark, Character, FigureTrack, Light, LightKind, Mark, Pose, Sensor, Wall,
    };

    fn project() -> Project {
        let mut p = Project::new("Test");
        p.characters = vec![
            Character {
                id: "e".into(),
                name: "Elisabeth".into(),
                short: "E".into(),
                color: FigureColor::Purple,
            },
            Character {
                id: "v".into(),
                name: "Burgvogt".into(),
                short: "B".into(),
                color: FigureColor::Blue,
            },
        ];
        p
    }

    fn set() -> SetPlan {
        SetPlan {
            width_m: 20.0,
            depth_m: 13.0,
            walls: vec![Wall {
                from: Point::new(0.0, 0.0),
                to: Point::new(20.0, 0.0),
                thickness_m: 0.2,
            }],
            objects: vec![],
        }
    }

    fn mark(at_s: f64, x: f64, y: f64) -> Mark {
        Mark {
            at_s,
            at: Point::new(x, y),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
        }
    }

    fn plan() -> FloorPlan {
        let mut p = FloorPlan::new("a", "Plan A");
        p.figures = vec![
            FigureTrack {
                character: "e".into(),
                marks: vec![mark(0.0, 2.0, 6.0), mark(8.0, 7.0, 6.0)],
            },
            FigureTrack {
                character: "v".into(),
                marks: vec![mark(0.0, 12.0, 6.0)],
            },
        ];
        p.cameras = vec![Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks: vec![CameraMark {
                at_s: 0.0,
                at: Point::new(8.0, 12.0),
                facing_deg: -90.0,
                height_m: 1.6,
                tilt_deg: 0.0,
                hold_s: 0.0,
            }],
            follows: None,
        }];
        p.lights = vec![Light {
            id: "l1".into(),
            label: "L1".into(),
            kind: LightKind::LedPanel,
            at: Point::new(18.0, 2.0),
            facing_deg: 135.0,
            beam_deg: 60.0,
            color_temp_k: 5600.0,
            power_w: 200.0,
        }];
        p
    }

    fn count(list: &DisplayList, role: Role) -> usize {
        list.items.iter().filter(|i| i.role == role).count()
    }

    #[test]
    fn draws_every_part_of_the_plan() {
        let list = render(&project(), &set(), &plan(), 0.0);

        assert_eq!(count(&list, Role::Wall), 1);
        assert_eq!(count(&list, Role::Light), 1);
        assert_eq!(count(&list, Role::Axis), 1);
        assert_eq!(count(&list, Role::Frustum), 1);
        assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 2);
        assert_eq!(count(&list, Role::Figure(FigureColor::Blue)), 2);
    }

    #[test]
    fn figure_moves_with_time() {
        let at = |t: f64| {
            render(&project(), &set(), &plan(), t)
                .items
                .into_iter()
                .find_map(|i| match (i.role, i.shape) {
                    (Role::Figure(FigureColor::Purple), Shape::Circle { center, .. }) => {
                        Some(center)
                    }
                    _ => None,
                })
        };

        assert_eq!(at(0.0), Some(Point::new(2.0, 6.0)));
        assert_eq!(at(4.0), Some(Point::new(4.5, 6.0)));
    }

    #[test]
    fn walls_come_first_figures_last() {
        let list = render(&project(), &set(), &plan(), 0.0);

        assert_eq!(list.items.first().map(|i| i.role), Some(Role::Wall));
        assert!(matches!(
            list.items.last().map(|i| i.role),
            Some(Role::Figure(_))
        ));
    }

    #[test]
    fn unknown_character_is_skipped() {
        let mut p = plan();
        p.figures[0].character = "nobody".into();

        let list = render(&project(), &set(), &p, 0.0);

        assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 0);
    }
}
