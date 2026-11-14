use std::path::{Path, PathBuf};

use klappe_model::{
    Arrival, Character, Dash, Departure, Drawing, DrawingShape, FigureColor, Fill, FloorPlan,
    LibraryItem, LightKind, Point, Pose, Project, Scene, SetPlan, Wall,
};
use klappe_stellprobe::{hit, within, Item, Role, Source};

use crate::camera_edit;
use crate::error::ServiceError;
use crate::field::Field;
use crate::session::Layer;

mod add;
mod background;
mod check;
mod group;
mod marks;
mod moving;
mod remove;

use add::{
    add_camera, add_drawing, add_light, add_mark, add_note, add_object, add_wall, edit_note,
    free_id, free_letter,
};
use background::{calibrate, move_background, remove_background, set_background};
use check::check;
pub(crate) use marks::mark_info;
use marks::{set_arrival, set_departure, set_hold, set_pace, set_pose};
pub use marks::{MarkInfo, Pace};
use moving::{anchor_pair, carry_cameras, move_to, points_mut, rotate, shift, spin, target, turn};
use remove::remove;

/// What a tool or a drag does to the current plan.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanEdit {
    /// Moves what `source` names to `to`.
    Move {
        source: Source,
        to: Point,
    },
    /// Moves it by a distance: a drag that began beside the middle. With snapping on, its new
    /// place (centre, mark, middle of a wall) lands on the grid.
    Shift {
        source: Source,
        dx_m: f64,
        dy_m: f64,
    },
    /// Turns it by an angle from where it faces now (walls: from their direction), clockwise
    /// for a positive `by_deg`.
    Spin {
        source: Source,
        by_deg: f64,
    },
    /// Turns it to face `facing_deg` (objects: rotation; walls: their direction).
    Turn {
        source: Source,
        facing_deg: f64,
    },
    Remove {
        source: Source,
    },
    /// `Shift` for a group (LH-08): all move by the same distance, the first one snapped. One
    /// edit, so one undo step.
    ShiftGroup {
        sources: Vec<Source>,
        dx_m: f64,
        dy_m: f64,
    },
    /// `Spin` for a group: every element turns by `by_deg` and swings about the middle of the
    /// group's bounding box, so the group turns as one piece.
    SpinGroup {
        sources: Vec<Source>,
        by_deg: f64,
    },
    RemoveGroup {
        sources: Vec<Source>,
    },
    /// Copies the group, set down by a distance. Objects, lamps, notes, drawings, walls and
    /// cameras have copies; a figure, a mark or a grip has none and is left out. An error when
    /// nothing is left to copy.
    Duplicate {
        sources: Vec<Source>,
        dx_m: f64,
        dy_m: f64,
    },
    /// Makes the way straight again (LH-21): to a mark, or all of a figure's or camera's ways.
    Straighten {
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
    /// Sets a new object of library entry `kind` (LH-11), as big as the entry says; a kind
    /// no library knows is a 1 m square.
    AddObject {
        kind: String,
        at: Point,
    },
    /// Gives the object `source` another width and depth (LH-25). Only objects have a size
    /// to change.
    Resize {
        source: Source,
        width_m: f64,
        depth_m: f64,
    },
    /// Sets how tall the object `source` is (LH-17), or takes the height off with `None`. Only
    /// objects have a height.
    SetObjectHeight {
        source: Source,
        height_m: Option<f64>,
    },
    AddWall {
        from: Point,
        to: Point,
    },
    /// Pins a note with `text` to the set.
    AddNote {
        at: Point,
        text: String,
    },
    /// Changes the text of the note `id`.
    EditNote {
        id: String,
        text: String,
    },
    /// Draws a line, arrow, shape or area on the plan of the scene. `fill` only matters for
    /// shapes with an inside.
    AddDrawing {
        shape: DrawingShape,
        color: FigureColor,
        dash: Dash,
        fill: Fill,
    },
    /// Takes the image file `source` (PNG or JPEG) as the set's background, copied into the
    /// project's `bilder/`, as wide as the set from its top left corner. Undo takes the
    /// background back out of the model; the copy stays in `bilder/`, it is the user's file.
    SetBackground {
        source: PathBuf,
    },
    /// Scales the background about `from` so that the stretch from `from` to `to`, as they lie
    /// on the plan now, measures `distance_m` in truth: the way to set the image's scale from
    /// a length known on site.
    Calibrate {
        from: Point,
        to: Point,
        distance_m: f64,
    },
    /// Moves the background without changing its scale.
    MoveBackground {
        dx_m: f64,
        dy_m: f64,
    },
    /// Takes the background off the set; its file stays in `bilder/`.
    RemoveBackground,
    /// Sets the pose of a figure's mark (LH-22): `Source::Mark`, or `Source::Figure` for the
    /// mark at the playhead.
    SetPose {
        source: Source,
        pose: Pose,
    },
    /// Sets how the figure comes to a mark (LH-26): `Source::Mark`, or `Source::Figure` for
    /// the mark at the playhead.
    SetArrival {
        source: Source,
        arrival: Arrival,
    },
    /// Sets whether the figure leaves after the pause at a mark (LH-26): `Source::Mark`, or
    /// `Source::Figure` for the mark at the playhead.
    SetDeparture {
        source: Source,
        departure: Departure,
    },
    /// Sets the pause at a mark of a figure or a camera (LH-63): `Source::Mark`, or
    /// `Source::Figure` and `Source::Camera` for the mark at the playhead. The later marks
    /// of the track move in time by the change, so their ways keep their duration.
    SetHold {
        source: Source,
        hold_s: f64,
    },
    /// Sets the lens height and the tilt of the camera mark at the playhead (LH-34):
    /// `Source::Camera`. Tilt is up from the horizon, positive looks up.
    SetCameraAim {
        source: Source,
        height_m: f64,
        tilt_deg: f64,
    },
    /// Times the way to a figure's mark for `pace` (LH-63): the mark arrives after the pause
    /// at the mark before and the walk along the way (a curve counts by its length). The
    /// later marks move in time by the change. The first mark has no way to it.
    SetPace {
        source: Source,
        pace: Pace,
    },
    /// Couples the camera `camera` to the figure `figure` (LH-37), or lets it go with `None`.
    /// A coupled camera moves and turns with the figure's marks.
    Follow {
        camera: String,
        figure: Option<String>,
    },
    /// Gives the camera `camera` the sensor of `width_mm` (a named format when the width is
    /// its own, else a custom one), and the frame `aspect` when given (LH-32).
    SetSensor {
        camera: String,
        width_mm: f64,
        aspect: Option<f64>,
    },
    /// Gives the camera `camera` the focal length `lens_mm`; shots with a lens of their own
    /// keep it.
    SetLens {
        camera: String,
        lens_mm: f64,
    },
    /// Gives the camera `camera` the frame `aspect`, width to height (LH-38).
    SetAspect {
        camera: String,
        aspect: f64,
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

/// A full turn; angles are kept within [0, 360).
const FULL_TURN_DEG: f64 = 360.0;

/// `at` on the grid when snapping is on.
pub(crate) fn snap(at: Point, snapping: Snapping) -> Point {
    if snapping == Snapping::Off || !at.x_m.is_finite() || !at.y_m.is_finite() {
        return at;
    }

    let on_grid = |v: f64| (v / GRID_M).round() * GRID_M;
    Point::new(on_grid(at.x_m), on_grid(at.y_m))
}

/// The topmost item the user can grab at `at`: the display list is back to front. Items for
/// which `locked` is true are left out.
pub(crate) fn pick(items: &[Item], at: Point, locked: impl Fn(Role) -> bool) -> Option<Source> {
    items
        .iter()
        .rev()
        .filter(|item| grabbable(item.role) && !locked(item.role))
        .find(|item| hit(item, at, PICK_RADIUS_M))
        .and_then(|item| item.source.clone())
}

/// Every element the user can grab whose shown items all lie in the box with corners `a` and
/// `b` (LH-08), in display order. Items for which `locked` is true are left out.
pub(crate) fn pick_in(
    items: &[Item],
    (a, b): (Point, Point),
    locked: impl Fn(Role) -> bool,
) -> Vec<Source> {
    let mut found: Vec<(Source, bool)> = Vec::new();

    for item in items
        .iter()
        .filter(|i| grabbable(i.role) && !locked(i.role))
    {
        let Some(source) = &item.source else {
            continue;
        };
        let inside = within(item, a, b);

        // An element drawn from several items (a figure and its facing wedge) is in only
        // when all of them are.
        match found.iter_mut().find(|(s, _)| s == source) {
            Some((_, all_in)) => *all_in &= inside,
            None => found.push((source.clone(), inside)),
        }
    }

    found
        .into_iter()
        .filter_map(|(source, all_in)| all_in.then_some(source))
        .collect()
}

/// What a click means to take: not the view cone, the axis or a path, which only show.
fn grabbable(role: Role) -> bool {
    !matches!(
        role,
        Role::Background
            | Role::Frustum
            | Role::Axis
            | Role::CameraPath
            | Role::FigurePath(_)
            | Role::LightBeam
    )
}

impl PlanEdit {
    /// The layers the edit changes something on; a locked one refuses it. A group edit can
    /// reach several.
    pub(crate) fn layers(&self) -> Vec<Layer> {
        let layer = match self {
            PlanEdit::Move { source, .. }
            | PlanEdit::Shift { source, .. }
            | PlanEdit::Spin { source, .. }
            | PlanEdit::Turn { source, .. }
            | PlanEdit::Resize { source, .. }
            | PlanEdit::SetPose { source, .. }
            | PlanEdit::SetArrival { source, .. }
            | PlanEdit::SetDeparture { source, .. }
            | PlanEdit::SetHold { source, .. }
            | PlanEdit::SetCameraAim { source, .. }
            | PlanEdit::SetObjectHeight { source, .. }
            | PlanEdit::SetPace { source, .. }
            | PlanEdit::Remove { source }
            | PlanEdit::Straighten { source } => layer_of_source(source),
            PlanEdit::ShiftGroup { sources, .. }
            | PlanEdit::SpinGroup { sources, .. }
            | PlanEdit::RemoveGroup { sources }
            | PlanEdit::Duplicate { sources, .. } => {
                return sources.iter().map(layer_of_source).collect();
            }
            PlanEdit::AddMark { .. } => Layer::Figures,
            PlanEdit::AddCamera { .. }
            | PlanEdit::Follow { .. }
            | PlanEdit::SetSensor { .. }
            | PlanEdit::SetLens { .. }
            | PlanEdit::SetAspect { .. } => Layer::Cameras,
            PlanEdit::AddLight { .. } => Layer::Lights,
            PlanEdit::AddNote { .. } | PlanEdit::EditNote { .. } => Layer::Notes,
            PlanEdit::AddDrawing { .. } => Layer::Drawings,
            PlanEdit::AddObject { .. }
            | PlanEdit::AddWall { .. }
            | PlanEdit::SetBackground { .. }
            | PlanEdit::Calibrate { .. }
            | PlanEdit::MoveBackground { .. }
            | PlanEdit::RemoveBackground => Layer::Set,
        };

        vec![layer]
    }
}

/// The layer an element of the plan is on.
fn layer_of_source(source: &Source) -> Layer {
    match source {
        Source::Object(_) | Source::Wall(_) => Layer::Set,
        Source::Light(_) => Layer::Lights,
        Source::Camera(_) | Source::CameraVia { .. } => Layer::Cameras,
        Source::Figure(_) | Source::Mark { .. } | Source::FigureVia { .. } => Layer::Figures,
        Source::Note(_) => Layer::Notes,
        Source::Drawing(_) => Layer::Drawings,
    }
}

/// The parts of the project one edit works on, borrowed apart.
struct Scope<'a> {
    plan: &'a mut FloorPlan,
    /// `None` when the scene's location is missing.
    set: Option<&'a mut SetPlan>,
    /// Id of the scene's location, for the error when it is missing.
    location: &'a str,
    characters: &'a [Character],
    /// The project's own library entries, for what a new object looks like.
    library: &'a [LibraryItem],
    time_s: f64,
}

/// What `Move` and `Turn` change.
enum Target<'a> {
    Place {
        at: &'a mut Point,
        facing_deg: &'a mut f64,
    },
    Wall(&'a mut Wall),
    /// Has a place but no facing: a note.
    Pin {
        at: &'a mut Point,
    },
    /// Points but no facing: a drawing keeps its shape and is not turned.
    Drawing(&'a mut Drawing),
    /// The bend of the way from mark `a` to mark `b`.
    Curve {
        via: &'a mut Option<Point>,
        a: Point,
        b: Point,
    },
}

/// Applies `edit` to the plan `plan` of scene `scene` at playhead `time_s`; `snapping` is for
/// `Shift`, the other edits take their points as given. `dir` is the project folder, where
/// `SetBackground` copies its image to. On an error the project may be half changed: the
/// caller works on a copy.
pub(crate) fn apply(
    dir: &Path,
    project: &mut Project,
    scene: usize,
    plan: usize,
    time_s: f64,
    snapping: Snapping,
    edit: &PlanEdit,
) -> Result<(), ServiceError> {
    check(edit)?;

    let Project {
        characters,
        locations,
        scenes,
        library,
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
        library,
        time_s,
    };

    match edit {
        PlanEdit::Move { source, to } => {
            carry_cameras(&mut scope, source, |s| move_to(s, source, *to))
        }
        PlanEdit::Shift { source, dx_m, dy_m } => carry_cameras(&mut scope, source, |s| {
            shift(s, source, (*dx_m, *dy_m), snapping)
        }),
        PlanEdit::Spin { source, by_deg } => {
            carry_cameras(&mut scope, source, |s| spin(s, source, *by_deg))
        }
        PlanEdit::Turn { source, facing_deg } => {
            carry_cameras(&mut scope, source, |s| turn(s, source, *facing_deg))
        }
        PlanEdit::Remove { source } => remove(&mut scope, source),
        PlanEdit::ShiftGroup {
            sources,
            dx_m,
            dy_m,
        } => group::shift(&mut scope, sources, (*dx_m, *dy_m), snapping),
        PlanEdit::SpinGroup { sources, by_deg } => group::spin(&mut scope, sources, *by_deg),
        PlanEdit::RemoveGroup { sources } => group::remove_all(&mut scope, sources),
        PlanEdit::Duplicate {
            sources,
            dx_m,
            dy_m,
        } => group::duplicate(&mut scope, sources, (*dx_m, *dy_m)),
        PlanEdit::Straighten { source } => straighten(&mut scope, source),
        PlanEdit::AddMark { character, at } => add_mark(&mut scope, character, *at),
        PlanEdit::AddCamera { at, facing_deg } => add_camera(&mut scope, *at, *facing_deg),
        PlanEdit::AddLight { kind, at } => add_light(&mut scope, *kind, *at),
        PlanEdit::AddObject { kind, at } => add_object(&mut scope, kind, *at),
        PlanEdit::Resize {
            source,
            width_m,
            depth_m,
        } => resize(&mut scope, source, (*width_m, *depth_m)),
        PlanEdit::AddWall { from, to } => add_wall(&mut scope, *from, *to),
        PlanEdit::AddNote { at, text } => add_note(&mut scope, *at, text),
        PlanEdit::AddDrawing {
            shape,
            color,
            dash,
            fill,
        } => add_drawing(&mut scope, shape, (*color, *dash, *fill)),
        PlanEdit::EditNote { id, text } => edit_note(&mut scope, id, text),
        PlanEdit::SetBackground { source } => set_background(&mut scope, dir, source),
        PlanEdit::Calibrate {
            from,
            to,
            distance_m,
        } => calibrate(&mut scope, (*from, *to), *distance_m),
        PlanEdit::MoveBackground { dx_m, dy_m } => move_background(&mut scope, (*dx_m, *dy_m)),
        PlanEdit::RemoveBackground => remove_background(&mut scope),
        PlanEdit::Follow { camera, figure } => follow(&mut scope, camera, figure.as_deref()),
        PlanEdit::SetSensor {
            camera,
            width_mm,
            aspect,
        } => camera_edit::set_sensor(scope.plan, camera, *width_mm, *aspect),
        PlanEdit::SetLens { camera, lens_mm } => {
            camera_edit::set_lens(scope.plan, camera, *lens_mm)
        }
        PlanEdit::SetAspect { camera, aspect } => {
            camera_edit::set_aspect(scope.plan, camera, *aspect)
        }
        PlanEdit::SetPose { source, pose } => set_pose(&mut scope, source, *pose),
        PlanEdit::SetArrival { source, arrival } => set_arrival(&mut scope, source, *arrival),
        PlanEdit::SetDeparture { source, departure } => {
            set_departure(&mut scope, source, *departure)
        }
        PlanEdit::SetHold { source, hold_s } => set_hold(&mut scope, source, *hold_s),
        PlanEdit::SetCameraAim {
            source,
            height_m,
            tilt_deg,
        } => set_camera_aim(&mut scope, source, (*height_m, *tilt_deg)),
        PlanEdit::SetObjectHeight { source, height_m } => {
            set_object_height(&mut scope, source, *height_m)
        }
        PlanEdit::SetPace { source, pace } => set_pace(&mut scope, source, *pace),
    }
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

/// Takes the bend out of the way the source names: to one mark (a mark or its grip), or all
/// ways of a figure or camera. Anything else has no way.
fn straighten(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let plan = &mut *scope.plan;
    let find = |id: &str| plan.figures.iter().position(|f| f.character == id);

    match source {
        Source::Mark { character, index } | Source::FigureVia { character, index } => {
            let track = find(character).ok_or(ServiceError::Gone)?;
            let mark = plan.figures[track].marks.get_mut(*index);
            mark.ok_or(ServiceError::Gone)?.via = None;
        }
        Source::Figure(character) => {
            let track = find(character).ok_or(ServiceError::Gone)?;
            plan.figures[track]
                .marks
                .iter_mut()
                .for_each(|m| m.via = None);
        }
        Source::CameraVia { camera, index } => {
            let camera = plan.cameras.iter_mut().find(|c| c.id == *camera);
            let mark = camera.ok_or(ServiceError::Gone)?.marks.get_mut(*index);
            mark.ok_or(ServiceError::Gone)?.via = None;
        }
        Source::Camera(id) => {
            let camera = plan.cameras.iter_mut().find(|c| c.id == *id);
            camera
                .ok_or(ServiceError::Gone)?
                .marks
                .iter_mut()
                .for_each(|m| m.via = None);
        }
        _ => {}
    }

    Ok(())
}

/// The mark of a figure that `source` names, as its character and index.
fn figure_mark(scope: &Scope<'_>, source: &Source) -> Option<(String, usize)> {
    match source {
        Source::Mark { character, index } => Some((character.clone(), *index)),
        Source::Figure(character) => {
            let track = scope.plan.figure(character)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), scope.time_s)?;
            Some((character.clone(), index))
        }
        _ => None,
    }
}

fn bad_mark(field: Field, value: &str) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: value.to_owned(),
    }
}

fn set_camera_aim(
    scope: &mut Scope<'_>,
    source: &Source,
    (height_m, tilt_deg): (f64, f64),
) -> Result<(), ServiceError> {
    let Source::Camera(id) = source else {
        return Err(bad_mark(Field::Source, ""));
    };
    let time_s = scope.time_s;
    let camera = scope
        .plan
        .cameras
        .iter_mut()
        .find(|c| c.id == *id)
        .ok_or(ServiceError::Gone)?;
    let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s);
    let mark = index
        .and_then(|i| camera.marks.get_mut(i))
        .ok_or(ServiceError::Gone)?;

    mark.height_m = height_m;
    mark.tilt_deg = tilt_deg;

    Ok(())
}

fn set_object_height(
    scope: &mut Scope<'_>,
    source: &Source,
    height_m: Option<f64>,
) -> Result<(), ServiceError> {
    let Source::Object(id) = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let object = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
        .ok_or(ServiceError::Gone)?;

    object.height_m = height_m;

    Ok(())
}

/// Couples `camera` to `figure`, a figure of this plan, or lets it go.
fn follow(scope: &mut Scope<'_>, camera: &str, figure: Option<&str>) -> Result<(), ServiceError> {
    if let Some(character) = figure {
        if scope.plan.figure(character).is_none() {
            return Err(ServiceError::BadValue {
                field: Field::Follows,
                value: character.to_owned(),
            });
        }
    }

    let camera = scope
        .plan
        .cameras
        .iter_mut()
        .find(|c| c.id == camera)
        .ok_or(ServiceError::Gone)?;
    camera.follows = figure.map(str::to_owned);

    Ok(())
}

fn resize(scope: &mut Scope<'_>, source: &Source, size_m: (f64, f64)) -> Result<(), ServiceError> {
    let Source::Object(id) = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let object = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
        .ok_or(ServiceError::Gone)?;

    (object.width_m, object.depth_m) = size_m;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::Location;

    /// One scene with an empty plan and no cameras, on an empty set.
    pub(super) fn empty() -> Project {
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
            apply(Path::new("."), &mut project, 0, 0, 0.0, Snapping::On, &edit),
            Err(ServiceError::NoLocation(ref id)) if id == "l"
        ));
    }
}
