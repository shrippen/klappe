use klappe_model::{
    Camera, CameraMark, Character, FigureTrack, FloorPlan, Light, LightKind, LinkTarget, Mark,
    Point, Pose, Project, Scene, Sensor, SetObject, SetPlan, Wall,
};
use klappe_stellprobe::{hit, Item, Role, Source};

use crate::error::ServiceError;

/// What a tool or a drag does to the current plan.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanEdit {
    /// Moves what `source` names to `to`.
    Move {
        source: Source,
        to: Point,
    },
    /// Turns it to face `facing_deg` (objects: rotation; walls: their direction).
    Turn {
        source: Source,
        facing_deg: f64,
    },
    Remove {
        source: Source,
    },
    /// Appends a mark to a character's track; adds the track if the plan has none.
    AddMark {
        character: String,
        at: Point,
    },
    AddCamera {
        at: Point,
        facing_deg: f64,
    },
    AddLight {
        kind: LightKind,
        at: Point,
    },
    AddObject {
        kind: String,
        at: Point,
    },
    AddWall {
        from: Point,
        to: Point,
    },
}

/// Whether points snap to the grid (LH-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Snapping {
    On,
    Off,
}

/// Grid spacing of the plan.
const GRID_M: f64 = 0.25;

/// How far beside a shape a click still takes it: a finger or a mouse is not exact.
const PICK_RADIUS_M: f64 = 0.15;

/// Walking speed that sets the time of a new mark: an easy pace on set, 5 km/h.
const WALK_SPEED_M_S: f64 = 1.4;

const DEFAULT_CAMERA_HEIGHT_M: f64 = 1.6;
const DEFAULT_SENSOR: Sensor = Sensor::Super35;
const DEFAULT_LENS_MM: f64 = 35.0;
const DEFAULT_ASPECT: f64 = 2.39;

const DEFAULT_OBJECT_M: f64 = 1.0;
const DEFAULT_WALL_M: f64 = 0.2;

const TUNGSTEN_K: f64 = 3200.0;
const DAYLIGHT_K: f64 = 5600.0;

/// A full turn; angles are kept within [0, 360).
const FULL_TURN_DEG: f64 = 360.0;

/// Beam, colour and power a new lamp starts with; the user changes them afterwards.
struct LightDefaults {
    beam_deg: f64,
    color_temp_k: f64,
    power_w: f64,
}

fn light_defaults(kind: LightKind) -> LightDefaults {
    let (beam_deg, color_temp_k, power_w) = match kind {
        LightKind::Fresnel => (30.0, TUNGSTEN_K, 650.0),
        LightKind::LedPanel => (90.0, DAYLIGHT_K, 200.0),
        LightKind::Tube => (120.0, DAYLIGHT_K, 55.0),
        LightKind::Balloon => (120.0, TUNGSTEN_K, 1200.0),
        // Bounce and flag make no light of their own.
        LightKind::Reflector => (40.0, DAYLIGHT_K, 0.0),
        LightKind::Flag => (20.0, DAYLIGHT_K, 0.0),
    };

    LightDefaults {
        beam_deg,
        color_temp_k,
        power_w,
    }
}

/// `at` on the grid when snapping is on.
pub(crate) fn snap(at: Point, snapping: Snapping) -> Point {
    if snapping == Snapping::Off || !at.x_m.is_finite() || !at.y_m.is_finite() {
        return at;
    }

    let on_grid = |v: f64| (v / GRID_M).round() * GRID_M;
    Point::new(on_grid(at.x_m), on_grid(at.y_m))
}

/// The topmost item the user can grab at `at`: the display list is back to front.
pub(crate) fn pick(items: &[Item], at: Point) -> Option<Source> {
    items
        .iter()
        .rev()
        .filter(|item| grabbable(item.role))
        .find(|item| hit(item, at, PICK_RADIUS_M))
        .and_then(|item| item.source.clone())
}

/// What a click means to take: not the view cone, the axis or a path, which only show.
fn grabbable(role: Role) -> bool {
    !matches!(
        role,
        Role::Frustum | Role::Axis | Role::CameraPath | Role::FigurePath(_) | Role::LightBeam
    )
}

/// The parts of the project one edit works on, borrowed apart.
struct Scope<'a> {
    plan: &'a mut FloorPlan,
    /// `None` when the scene's location is missing.
    set: Option<&'a mut SetPlan>,
    /// Id of the scene's location, for the error when it is missing.
    location: &'a str,
    characters: &'a [Character],
    time_s: f64,
}

/// What `Move` and `Turn` change.
enum Target<'a> {
    Place {
        at: &'a mut Point,
        facing_deg: &'a mut f64,
    },
    Wall(&'a mut Wall),
}

/// Applies `edit` to the plan `plan` of scene `scene` at playhead `time_s`. On an error the
/// project may be half changed: the caller works on a copy.
pub(crate) fn apply(
    project: &mut Project,
    scene: usize,
    plan: usize,
    time_s: f64,
    edit: &PlanEdit,
) -> Result<(), ServiceError> {
    check(edit)?;

    let Project {
        characters,
        locations,
        scenes,
        ..
    } = project;
    let Scene {
        location, plans, ..
    } = scenes
        .get_mut(scene)
        .ok_or_else(|| ServiceError::NoScene(scene.to_string()))?;
    let plan = plans
        .get_mut(plan)
        .ok_or_else(|| ServiceError::NoScene(scene.to_string()))?;
    let set = locations
        .iter_mut()
        .find(|l| l.id == *location)
        .map(|l| &mut l.set);

    let mut scope = Scope {
        plan,
        set,
        location,
        characters,
        time_s,
    };

    match edit {
        PlanEdit::Move { source, to } => move_to(&mut scope, source, *to),
        PlanEdit::Turn { source, facing_deg } => turn(&mut scope, source, *facing_deg),
        PlanEdit::Remove { source } => remove(&mut scope, source),
        PlanEdit::AddMark { character, at } => add_mark(&mut scope, character, *at),
        PlanEdit::AddCamera { at, facing_deg } => add_camera(&mut scope, *at, *facing_deg),
        PlanEdit::AddLight { kind, at } => add_light(&mut scope, *kind, *at),
        PlanEdit::AddObject { kind, at } => add_object(&mut scope, kind, *at),
        PlanEdit::AddWall { from, to } => add_wall(&mut scope, *from, *to),
    }
}

/// Every point and angle is a finite number: NaN or infinity would be saved as `null`.
fn check(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Move { to: at, .. }
        | PlanEdit::AddMark { at, .. }
        | PlanEdit::AddLight { at, .. }
        | PlanEdit::AddObject { at, .. } => check_point(*at),
        PlanEdit::Turn { facing_deg, .. } => check_angle(*facing_deg),
        PlanEdit::AddCamera { at, facing_deg } => {
            check_point(*at)?;
            check_angle(*facing_deg)
        }
        PlanEdit::AddWall { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        PlanEdit::Remove { .. } => Ok(()),
    }
}

fn check_point(p: Point) -> Result<(), ServiceError> {
    if p.x_m.is_finite() && p.y_m.is_finite() {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "point",
        value: format!("{}, {}", p.x_m, p.y_m),
    })
}

fn check_angle(deg: f64) -> Result<(), ServiceError> {
    if deg.is_finite() {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "angle",
        value: deg.to_string(),
    })
}

/// An angle within [0, 360): -90 becomes 270. `rem_euclid` can return 360 for a tiny negative
/// number, so that case is folded back.
fn normalize_deg(deg: f64) -> f64 {
    let n = deg.rem_euclid(FULL_TURN_DEG);
    if n >= FULL_TURN_DEG {
        return 0.0;
    }
    n
}

/// The mark a figure or camera stands at, or is leaving, at `time_s`: the last one that has
/// begun, else the first.
fn current_index(times_s: impl Iterator<Item = f64>, time_s: f64) -> Option<usize> {
    let times: Vec<f64> = times_s.collect();
    if times.is_empty() {
        return None;
    }

    Some(times.iter().rposition(|t| *t <= time_s).unwrap_or(0))
}

fn target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let time_s = scope.time_s;

    match source {
        Source::Mark { character, index } => {
            let mark = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .and_then(|f| f.marks.get_mut(*index))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::Figure(character) => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s)
                .ok_or(ServiceError::Gone)?;
            let mark = track.marks.get_mut(index).ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::Camera(id) => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *id)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s)
                .ok_or(ServiceError::Gone)?;
            let mark = camera.marks.get_mut(index).ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut mark.at,
                facing_deg: &mut mark.facing_deg,
            })
        }
        Source::Light(id) => {
            let light = scope
                .plan
                .lights
                .iter_mut()
                .find(|l| l.id == *id)
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut light.at,
                facing_deg: &mut light.facing_deg,
            })
        }
        Source::Object(id) => {
            let object = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Place {
                at: &mut object.at,
                facing_deg: &mut object.rotation_deg,
            })
        }
        Source::Wall(index) => {
            let wall = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.walls.get_mut(*index))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Wall(wall))
        }
    }
}

fn middle(wall: &Wall) -> Point {
    wall.from.lerp(wall.to, 0.5)
}

fn move_to(scope: &mut Scope<'_>, source: &Source, to: Point) -> Result<(), ServiceError> {
    match target(scope, source)? {
        Target::Place { at, .. } => *at = to,
        Target::Wall(wall) => {
            // The whole wall keeps its shape; its middle lands on `to`.
            let mid = middle(wall);
            let (dx, dy) = (to.x_m - mid.x_m, to.y_m - mid.y_m);
            wall.from = Point::new(wall.from.x_m + dx, wall.from.y_m + dy);
            wall.to = Point::new(wall.to.x_m + dx, wall.to.y_m + dy);
        }
    }

    Ok(())
}

fn turn(scope: &mut Scope<'_>, source: &Source, facing_deg: f64) -> Result<(), ServiceError> {
    let facing_deg = normalize_deg(facing_deg);

    match target(scope, source)? {
        Target::Place { facing_deg: f, .. } => *f = facing_deg,
        Target::Wall(wall) => {
            // Turns about its middle, keeping its length; from -> to then points to `facing_deg`.
            let mid = middle(wall);
            let half_m = wall.from.distance_m(wall.to) / 2.0;
            let (sin, cos) = facing_deg.to_radians().sin_cos();
            wall.from = Point::new(mid.x_m - half_m * cos, mid.y_m - half_m * sin);
            wall.to = Point::new(mid.x_m + half_m * cos, mid.y_m + half_m * sin);
        }
    }

    Ok(())
}

fn remove(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    match source {
        Source::Mark { character, index } => remove_mark(scope.plan, character, *index),
        Source::Figure(character) => {
            let known = scope.plan.figure(character).is_some();
            if !known {
                return Err(ServiceError::Gone);
            }

            remove_figure(scope.plan, character);
            Ok(())
        }
        Source::Camera(id) => remove_camera(scope.plan, id),
        Source::Light(id) => {
            let index = scope.plan.lights.iter().position(|l| l.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            scope.plan.lights.remove(index);
            Ok(())
        }
        Source::Object(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let index = set.objects.iter().position(|o| o.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            set.objects.remove(index);
            Ok(())
        }
        Source::Wall(index) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            if *index >= set.walls.len() {
                return Err(ServiceError::Gone);
            }

            set.walls.remove(*index);
            Ok(())
        }
    }
}

fn remove_mark(plan: &mut FloorPlan, character: &str, index: usize) -> Result<(), ServiceError> {
    let track = plan
        .figures
        .iter_mut()
        .find(|f| f.character == character)
        .ok_or(ServiceError::Gone)?;
    if index >= track.marks.len() {
        return Err(ServiceError::Gone);
    }

    track.marks.remove(index);
    if track.marks.is_empty() {
        remove_figure(plan, character);
        return Ok(());
    }

    // Links to the removed mark go; links to later marks move up by one.
    plan.links.retain(|l| {
        !matches!(&l.target, LinkTarget::Mark { character: c, index: i } if c == character && *i == index)
    });
    for link in &mut plan.links {
        let LinkTarget::Mark {
            character: c,
            index: i,
        } = &mut link.target
        else {
            continue;
        };
        if c == character && *i > index {
            *i -= 1;
        }
    }

    Ok(())
}

/// The whole track, with everything that pointed at the figure.
fn remove_figure(plan: &mut FloorPlan, character: &str) {
    plan.figures.retain(|f| f.character != character);
    plan.links
        .retain(|l| !matches!(&l.target, LinkTarget::Mark { character: c, .. } if c == character));

    let names_it = |axis: &klappe_model::AxisPair| axis.a == character || axis.b == character;
    if plan.axis.as_ref().is_some_and(names_it) {
        plan.axis = None;
    }

    for camera in &mut plan.cameras {
        if camera.follows.as_deref() == Some(character) {
            camera.follows = None;
        }
    }
}

fn remove_camera(plan: &mut FloorPlan, id: &str) -> Result<(), ServiceError> {
    let index = plan
        .cameras
        .iter()
        .position(|c| c.id == id)
        .ok_or(ServiceError::Gone)?;
    if let Some(shot) = plan.shots.iter().find(|s| s.camera == id) {
        return Err(ServiceError::CameraInUse {
            camera: plan.cameras[index].label.clone(),
            shot: shot.number.clone(),
        });
    }

    plan.cameras.remove(index);
    Ok(())
}

fn add_mark(scope: &mut Scope<'_>, character: &str, at: Point) -> Result<(), ServiceError> {
    if !scope.characters.iter().any(|c| c.id == character) {
        return Err(ServiceError::BadValue {
            field: "character",
            value: character.to_owned(),
        });
    }

    let first_s = scope.time_s;
    let index = match scope
        .plan
        .figures
        .iter()
        .position(|f| f.character == character)
    {
        Some(index) => index,
        None => {
            scope.plan.figures.push(FigureTrack {
                character: character.to_owned(),
                marks: Vec::new(),
            });
            scope.plan.figures.len() - 1
        }
    };
    let marks = &mut scope.plan.figures[index].marks;

    // The figure arrives, waits, then walks at `WALK_SPEED_M_S` and faces its way.
    let (at_s, facing_deg) = match marks.last() {
        None => (first_s, 0.0),
        Some(prev) => (
            prev.at_s + prev.hold_s + prev.at.distance_m(at) / WALK_SPEED_M_S,
            normalize_deg(prev.at.bearing_deg(at)),
        ),
    };
    marks.push(Mark {
        at_s,
        at,
        facing_deg,
        pose: Pose::Standing,
        hold_s: 0.0,
    });

    Ok(())
}

fn add_camera(scope: &mut Scope<'_>, at: Point, facing_deg: f64) -> Result<(), ServiceError> {
    let cameras = &scope.plan.cameras;
    let letter = ('A'..='Z')
        .find(|l| {
            let label = l.to_string();
            !cameras
                .iter()
                .any(|c| c.label == label || c.id == label.to_lowercase())
        })
        .ok_or(ServiceError::NoFreeCamera)?;
    let (sensor, lens_mm, aspect) = match cameras.first() {
        Some(first) => (first.sensor, first.lens_mm, first.aspect),
        None => (DEFAULT_SENSOR, DEFAULT_LENS_MM, DEFAULT_ASPECT),
    };

    let camera = Camera {
        id: letter.to_ascii_lowercase().to_string(),
        label: letter.to_string(),
        sensor,
        lens_mm,
        aspect,
        marks: vec![CameraMark {
            at_s: scope.time_s,
            at,
            facing_deg: normalize_deg(facing_deg),
            height_m: DEFAULT_CAMERA_HEIGHT_M,
            tilt_deg: 0.0,
            hold_s: 0.0,
        }],
        follows: None,
    };
    scope.plan.cameras.push(camera);

    Ok(())
}

fn add_light(scope: &mut Scope<'_>, kind: LightKind, at: Point) -> Result<(), ServiceError> {
    let lights = &scope.plan.lights;
    let n = (1..)
        .find(|n| !lights.iter().any(|l| l.id == format!("l{n}")))
        .unwrap_or(1);
    let defaults = light_defaults(kind);

    scope.plan.lights.push(Light {
        id: format!("l{n}"),
        label: format!("L{n}"),
        kind,
        at,
        facing_deg: 0.0,
        beam_deg: defaults.beam_deg,
        color_temp_k: defaults.color_temp_k,
        power_w: defaults.power_w,
    });

    Ok(())
}

fn add_object(scope: &mut Scope<'_>, kind: &str, at: Point) -> Result<(), ServiceError> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err(ServiceError::BadValue {
            field: "kind",
            value: String::new(),
        });
    }

    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let n = (1..)
        .find(|n| !set.objects.iter().any(|o| o.id == format!("o{n}")))
        .unwrap_or(1);

    set.objects.push(SetObject {
        id: format!("o{n}"),
        kind: kind.to_owned(),
        label: String::new(),
        at,
        width_m: DEFAULT_OBJECT_M,
        depth_m: DEFAULT_OBJECT_M,
        rotation_deg: 0.0,
    });

    Ok(())
}

fn add_wall(scope: &mut Scope<'_>, from: Point, to: Point) -> Result<(), ServiceError> {
    if from == to {
        return Err(ServiceError::BadValue {
            field: "wall",
            value: "0".to_owned(),
        });
    }

    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    set.walls.push(Wall {
        from,
        to,
        thickness_m: DEFAULT_WALL_M,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::Location;

    /// One scene with an empty plan and no cameras, on an empty set.
    fn empty() -> Project {
        let mut project = Project::new("Test");
        project.locations = vec![Location {
            id: "l".into(),
            name: "L".into(),
            set: SetPlan::default(),
        }];
        project.scenes = vec![Scene {
            id: "s".into(),
            number: "1".into(),
            heading: String::new(),
            location: "l".into(),
            plans: vec![FloorPlan::new("p", "Plan A")],
        }];
        project
    }

    fn add_camera(project: &mut Project) {
        let edit = PlanEdit::AddCamera {
            at: Point::new(1.0, 2.0),
            facing_deg: 0.0,
        };
        apply(project, 0, 0, 3.0, &edit).unwrap();
    }

    #[test]
    fn the_first_camera_has_the_default_lens() {
        let mut project = empty();

        add_camera(&mut project);

        let camera = &project.scenes[0].plans[0].cameras[0];
        assert_eq!(camera.label, "A");
        assert_eq!(camera.sensor, Sensor::Super35);
        assert_eq!((camera.lens_mm, camera.aspect), (35.0, 2.39));
        assert_eq!(camera.marks[0].at_s, 3.0);
    }

    /// A freed id is taken again: after removing C, the next camera is C once more.
    #[test]
    fn letters_and_light_ids_fill_gaps() {
        let mut project = empty();
        for _ in 0..3 {
            add_camera(&mut project);
        }
        project.scenes[0].plans[0].cameras.remove(2);
        add_camera(&mut project);

        let labels: Vec<_> = project.scenes[0].plans[0]
            .cameras
            .iter()
            .map(|c| c.label.as_str())
            .collect();
        assert_eq!(labels, ["A", "B", "C"]);

        let light = |kind| PlanEdit::AddLight {
            kind,
            at: Point::new(0.0, 0.0),
        };
        for _ in 0..2 {
            apply(&mut project, 0, 0, 0.0, &light(LightKind::Flag)).unwrap();
        }
        project.scenes[0].plans[0].lights.remove(0);
        apply(&mut project, 0, 0, 0.0, &light(LightKind::Flag)).unwrap();

        let ids: Vec<_> = project.scenes[0].plans[0]
            .lights
            .iter()
            .map(|l| l.id.as_str())
            .collect();
        assert_eq!(ids, ["l2", "l1"]);
    }

    #[test]
    fn angles_stay_below_a_full_turn() {
        assert_eq!(normalize_deg(-1e-20), 0.0);
        assert_eq!(normalize_deg(-90.0), 270.0);
        assert_eq!(normalize_deg(720.0), 0.0);
    }

    #[test]
    fn a_missing_location_refuses_set_edits() {
        let mut project = empty();
        project.locations.clear();
        let edit = PlanEdit::AddWall {
            from: Point::new(0.0, 0.0),
            to: Point::new(1.0, 0.0),
        };

        assert!(matches!(
            apply(&mut project, 0, 0, 0.0, &edit),
            Err(ServiceError::NoLocation(ref id)) if id == "l"
        ));
    }
}
