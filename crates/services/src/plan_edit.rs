use std::path::{Path, PathBuf};

use klappe_model::is_object_size_m;
use klappe_model::{
    Background, Camera, CameraMark, Character, Dash, Drawing, DrawingShape, FigureColor,
    FigureTrack, Fill, FloorPlan, LibraryItem, Light, LightKind, LinkTarget, Mark, Note,
    ObjectShape, Point, Pose, Project, Scene, Sensor, SetObject, SetPlan, Wall,
};
use klappe_stellprobe::{bend_point, hit, via_through, way_length_m, within, Item, Role, Source};

use crate::error::ServiceError;
use crate::library;
use crate::session::Layer;

mod group;

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
    /// Sets the pause at a mark of a figure or a camera (LH-63): `Source::Mark`, or
    /// `Source::Figure` and `Source::Camera` for the mark at the playhead. The later marks
    /// of the track move in time by the change, so their ways keep their duration.
    SetHold {
        source: Source,
        hold_s: f64,
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
}

/// How fast a figure goes along a way (LH-63).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    Walk,
    Run,
}

impl Pace {
    fn speed_m_s(self) -> f64 {
        match self {
            Pace::Walk => WALK_SPEED_M_S,
            Pace::Run => RUN_SPEED_M_S,
        }
    }
}

/// What the plan says about a mark, for the fields that change it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkInfo {
    /// `None` for a camera's mark.
    pub pose: Option<Pose>,
    pub hold_s: f64,
    /// A way leads to the mark: it is not the first of its track.
    pub has_way: bool,
    /// Speed along that way; `None` when it takes no time.
    pub pace_m_s: Option<f64>,
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

/// Running speed for `Pace::Run`: 14.4 km/h, a jog to a sprint's lower end.
const RUN_SPEED_M_S: f64 = 4.0;

const DEFAULT_CAMERA_HEIGHT_M: f64 = 1.6;
const DEFAULT_SENSOR: Sensor = Sensor::Super35;
const DEFAULT_LENS_MM: f64 = 35.0;
const DEFAULT_ASPECT: f64 = 2.39;

/// A new background lets the plan underneath show through.
const BACKGROUND_OPACITY: f64 = 0.5;

/// An area has an inside from three points on.
const MIN_AREA_POINTS: usize = 3;

/// Side of an object whose kind no library knows.
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
            | PlanEdit::SetHold { source, .. }
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
            PlanEdit::AddCamera { .. } | PlanEdit::Follow { .. } => Layer::Cameras,
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
        PlanEdit::SetPose { source, pose } => set_pose(&mut scope, source, *pose),
        PlanEdit::SetHold { source, hold_s } => set_hold(&mut scope, source, *hold_s),
        PlanEdit::SetPace { source, pace } => set_pace(&mut scope, source, *pace),
    }
}

/// Every point and angle is a finite number: NaN or infinity would be saved as `null`.
fn check(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Move { to: at, .. }
        | PlanEdit::AddMark { at, .. }
        | PlanEdit::AddLight { at, .. }
        | PlanEdit::AddObject { at, .. }
        | PlanEdit::AddNote { at, .. } => check_point(*at),
        PlanEdit::Shift { dx_m, dy_m, .. }
        | PlanEdit::ShiftGroup { dx_m, dy_m, .. }
        | PlanEdit::Duplicate { dx_m, dy_m, .. } => check_point(Point::new(*dx_m, *dy_m)),
        PlanEdit::Spin {
            by_deg: facing_deg, ..
        }
        | PlanEdit::SpinGroup {
            by_deg: facing_deg, ..
        }
        | PlanEdit::Turn { facing_deg, .. } => check_angle(*facing_deg),
        PlanEdit::AddCamera { at, facing_deg } => {
            check_point(*at)?;
            check_angle(*facing_deg)
        }
        PlanEdit::AddWall { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        PlanEdit::Calibrate {
            from,
            to,
            distance_m,
        } => {
            check_point(*from)?;
            check_point(*to)?;
            check_distance(*distance_m)
        }
        PlanEdit::MoveBackground { dx_m, dy_m } => check_point(Point::new(*dx_m, *dy_m)),
        PlanEdit::Resize {
            width_m, depth_m, ..
        } => check_object_size(*width_m, *depth_m),
        PlanEdit::AddDrawing { shape, .. } => check_shape(shape),
        PlanEdit::SetHold { hold_s, .. } => check_hold(*hold_s),
        PlanEdit::Remove { .. }
        | PlanEdit::RemoveGroup { .. }
        | PlanEdit::Straighten { .. }
        | PlanEdit::EditNote { .. }
        | PlanEdit::SetBackground { .. }
        | PlanEdit::RemoveBackground
        | PlanEdit::Follow { .. }
        | PlanEdit::SetPose { .. }
        | PlanEdit::SetPace { .. } => Ok(()),
    }
}

fn check_shape(shape: &DrawingShape) -> Result<(), ServiceError> {
    match shape {
        DrawingShape::Line { from, to }
        | DrawingShape::Arrow { from, to }
        | DrawingShape::Rect { from, to }
        | DrawingShape::Ellipse { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        DrawingShape::Area { points } => points.iter().try_for_each(|p| check_point(*p)),
    }
}

/// A pause is a finite time, not negative.
fn check_hold(hold_s: f64) -> Result<(), ServiceError> {
    if hold_s.is_finite() && hold_s >= 0.0 {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "hold_s",
        value: hold_s.to_string(),
    })
}

/// A length in truth: more than nothing, and a number.
fn check_distance(distance_m: f64) -> Result<(), ServiceError> {
    if distance_m.is_finite() && distance_m > 0.0 {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "distance",
        value: distance_m.to_string(),
    })
}

/// An object's sides: numbers, more than nothing, not absurd.
fn check_object_size(width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
    if is_object_size_m(width_m) && is_object_size_m(depth_m) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: "size",
        value: format!("{width_m} x {depth_m}"),
    })
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
        Source::FigureVia { character, index } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            curve(&mut track.marks, *index, |m| (m.at, &mut m.via))
        }
        Source::CameraVia { camera, index } => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *camera)
                .ok_or(ServiceError::Gone)?;
            curve(&mut camera.marks, *index, |m| (m.at, &mut m.via))
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
        Source::Note(id) => {
            let note = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.notes.iter_mut().find(|n| n.id == *id))
                .ok_or(ServiceError::Gone)?;
            Ok(Target::Pin { at: &mut note.at })
        }
        Source::Drawing(id) => scope
            .plan
            .drawings
            .iter_mut()
            .find(|d| d.id == *id)
            .map(Target::Drawing)
            .ok_or(ServiceError::Gone),
    }
}

/// The bend of the way to mark `index` of `marks`; `place` gives a mark's place and its `via`.
/// The first mark has no way to it.
fn curve<'a, M>(
    marks: &'a mut [M],
    index: usize,
    place: impl Fn(&'a mut M) -> (Point, &'a mut Option<Point>),
) -> Result<Target<'a>, ServiceError> {
    let before = index.checked_sub(1).ok_or(ServiceError::Gone)?;
    let (head, tail) = marks.split_at_mut(index);
    let a = head.get_mut(before).ok_or(ServiceError::Gone)?;
    let this = tail.first_mut().ok_or(ServiceError::Gone)?;

    let (a, _) = place(a);
    let (b, via) = place(this);

    Ok(Target::Curve { via, a, b })
}

fn middle(wall: &Wall) -> Point {
    wall.from.lerp(wall.to, 0.5)
}

/// Every point of a drawing, to read or to move.
fn points_mut(shape: &mut DrawingShape) -> Vec<&mut Point> {
    match shape {
        DrawingShape::Line { from, to }
        | DrawingShape::Arrow { from, to }
        | DrawingShape::Rect { from, to }
        | DrawingShape::Ellipse { from, to } => vec![from, to],
        DrawingShape::Area { points } => points.iter_mut().collect(),
    }
}

/// The first point of a drawing and the middle of its bounding box (the middle of a line,
/// the centre of a rectangle or ellipse); `None` for a shape without points.
fn anchors(drawing: &mut Drawing) -> Option<(Point, Point)> {
    let points = points_mut(&mut drawing.shape);
    let first = **points.first()?;
    let (mut min, mut max) = (first, first);

    for p in &points {
        min = Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m));
        max = Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m));
    }

    Some((first, min.lerp(max, 0.5)))
}

fn move_to(scope: &mut Scope<'_>, source: &Source, to: Point) -> Result<(), ServiceError> {
    match target(scope, source)? {
        Target::Place { at, .. } | Target::Pin { at } => *at = to,
        // `to` is where the way passes halfway, which is what the grip sits on.
        Target::Curve { via, a, b } => *via = Some(via_through(a, to, b)),
        Target::Drawing(drawing) => {
            // The whole drawing keeps its shape; the middle of its box lands on `to`.
            let Some((_, mid)) = anchors(drawing) else {
                return Ok(());
            };
            let (dx, dy) = (to.x_m - mid.x_m, to.y_m - mid.y_m);

            for p in points_mut(&mut drawing.shape) {
                *p = Point::new(p.x_m + dx, p.y_m + dy);
            }
        }
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

/// What `shift` reads off an element: the point that snaps and the point that moves.
fn anchor_pair(target: Target<'_>) -> Result<(Point, Point), ServiceError> {
    match target {
        Target::Place { at, .. } | Target::Pin { at } => Ok((*at, *at)),
        Target::Curve { via, a, b } => {
            let grip = bend_point(a, *via, b);
            Ok((grip, grip))
        }
        Target::Wall(wall) => Ok((wall.from, middle(wall))),
        Target::Drawing(drawing) => anchors(drawing).ok_or(ServiceError::Gone),
    }
}

/// Moves by `by_m` (dx, dy) from where the element is now; the new place is snapped. A wall
/// snaps its start, not its middle: a wall 3.25 m long keeps both ends where they meet others.
/// A drawing does the same with its first point.
fn shift(
    scope: &mut Scope<'_>,
    source: &Source,
    by_m: (f64, f64),
    snapping: Snapping,
) -> Result<(), ServiceError> {
    let (anchor, middle_now) = anchor_pair(target(scope, source)?)?;
    let landed = snap(
        Point::new(anchor.x_m + by_m.0, anchor.y_m + by_m.1),
        snapping,
    );
    let (dx, dy) = (landed.x_m - anchor.x_m, landed.y_m - anchor.y_m);

    move_to(
        scope,
        source,
        Point::new(middle_now.x_m + dx, middle_now.y_m + dy),
    )
}

fn spin(scope: &mut Scope<'_>, source: &Source, by_deg: f64) -> Result<(), ServiceError> {
    let now_deg = match target(scope, source)? {
        Target::Place { facing_deg, .. } => *facing_deg,
        Target::Wall(wall) => wall.from.bearing_deg(wall.to),
        // Nothing to turn.
        Target::Pin { .. } | Target::Drawing(_) | Target::Curve { .. } => return Ok(()),
    };

    turn(scope, source, now_deg + by_deg)
}

fn turn(scope: &mut Scope<'_>, source: &Source, facing_deg: f64) -> Result<(), ServiceError> {
    let facing_deg = normalize_deg(facing_deg);

    match target(scope, source)? {
        Target::Place { facing_deg: f, .. } => *f = facing_deg,
        // Nothing to turn.
        Target::Pin { .. } | Target::Drawing(_) | Target::Curve { .. } => {}
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
        // A grip is no element of its own: removing it takes the bend out.
        Source::FigureVia { .. } | Source::CameraVia { .. } => straighten(scope, source),
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
        Source::Note(id) => {
            let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
            let index = set.notes.iter().position(|n| n.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            set.notes.remove(index);
            Ok(())
        }
        Source::Drawing(id) => {
            let index = scope.plan.drawings.iter().position(|d| d.id == *id);
            let index = index.ok_or(ServiceError::Gone)?;
            scope.plan.drawings.remove(index);
            Ok(())
        }
    }
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

    // The new first mark has no way to it: its bend would reappear on a later first mark.
    if index == 0 {
        track.marks[0].via = None;
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
        via: None,
    });

    Ok(())
}

/// The first camera letter no camera of the plan has.
fn free_letter(cameras: &[Camera]) -> Result<char, ServiceError> {
    ('A'..='Z')
        .find(|l| {
            let label = l.to_string();
            !cameras
                .iter()
                .any(|c| c.label == label || c.id == label.to_lowercase())
        })
        .ok_or(ServiceError::NoFreeCamera)
}

fn add_camera(scope: &mut Scope<'_>, at: Point, facing_deg: f64) -> Result<(), ServiceError> {
    let cameras = &scope.plan.cameras;
    let letter = free_letter(cameras)?;
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
            via: None,
        }],
        follows: None,
    };
    scope.plan.cameras.push(camera);

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

/// Runs `change`; if it moved or turned a mark of a figure that cameras follow (LH-37), those
/// cameras' marks within the mark's time window (arrival to the end of the pause) take the
/// same move and turn about the mark, so distance and angle to the figure stay. In the same
/// edit, hence one undo step.
fn carry_cameras(
    scope: &mut Scope<'_>,
    source: &Source,
    change: impl FnOnce(&mut Scope<'_>) -> Result<(), ServiceError>,
) -> Result<(), ServiceError> {
    let Some((character, index)) = figure_mark(scope, source) else {
        return change(scope);
    };
    let mark_of = |plan: &FloorPlan| plan.figure(&character)?.marks.get(index).cloned();
    let before = mark_of(scope.plan);

    change(scope)?;

    let (Some(before), Some(after)) = (before, mark_of(scope.plan)) else {
        return Ok(());
    };
    let (dx_m, dy_m) = (after.at.x_m - before.at.x_m, after.at.y_m - before.at.y_m);
    let turn_deg = normalize_deg(after.facing_deg - before.facing_deg);
    let window_s = before.at_s..=before.at_s + before.hold_s;

    let followers = scope
        .plan
        .cameras
        .iter_mut()
        .filter(|c| c.follows.as_deref() == Some(character.as_str()));
    for camera in followers {
        for camera_mark in camera
            .marks
            .iter_mut()
            .filter(|m| window_s.contains(&m.at_s))
        {
            // The bend of the move goes along, or the move turns into a loop (LH-21).
            camera_mark.at = carry(camera_mark.at, (dx_m, dy_m), after.at, turn_deg);
            camera_mark.via = camera_mark
                .via
                .map(|via| carry(via, (dx_m, dy_m), after.at, turn_deg));
            camera_mark.facing_deg = normalize_deg(camera_mark.facing_deg + turn_deg);
        }
    }

    Ok(())
}

/// `point` shifted by `delta_m`, then turned by `deg` about `centre`: where a following
/// camera's point goes when its figure moves and turns.
fn carry(point: Point, delta_m: (f64, f64), centre: Point, deg: f64) -> Point {
    let moved = Point::new(point.x_m + delta_m.0, point.y_m + delta_m.1);

    rotate(moved, centre, deg)
}

/// `point` turned clockwise on the plan by `deg` about `centre`.
fn rotate(point: Point, centre: Point, deg: f64) -> Point {
    if deg == 0.0 {
        return point;
    }

    let (sin, cos) = deg.to_radians().sin_cos();
    let (x_m, y_m) = (point.x_m - centre.x_m, point.y_m - centre.y_m);

    Point::new(
        centre.x_m + x_m * cos - y_m * sin,
        centre.y_m + x_m * sin + y_m * cos,
    )
}

fn bad_mark(field: &'static str, value: &str) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: value.to_owned(),
    }
}

fn set_pose(scope: &mut Scope<'_>, source: &Source, pose: Pose) -> Result<(), ServiceError> {
    let (character, index) = figure_mark(scope, source).ok_or_else(|| bad_mark("source", ""))?;
    let mark = scope
        .plan
        .figures
        .iter_mut()
        .find(|f| f.character == character)
        .and_then(|f| f.marks.get_mut(index))
        .ok_or(ServiceError::Gone)?;

    mark.pose = pose;

    Ok(())
}

/// The arrival and the pause of the mark `source` names, with those of the marks after it in
/// the same track.
struct Timing<'a> {
    /// Per mark of the track: `(at_s, hold_s)`.
    marks: Vec<(&'a mut f64, &'a mut f64)>,
    index: usize,
}

impl Timing<'_> {
    /// Moves the marks from `first` on by `delta_s`.
    fn shift_from(&mut self, first: usize, delta_s: f64) {
        for (at_s, _) in self.marks.iter_mut().skip(first) {
            **at_s += delta_s;
        }
    }
}

fn timing<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Timing<'a>, ServiceError> {
    let time_s = scope.time_s;
    let (index, marks) = match source {
        Source::Mark { character, index } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let marks = track.marks.iter_mut();

            (
                Some(*index),
                marks
                    .map(|m| (&mut m.at_s, &mut m.hold_s))
                    .collect::<Vec<_>>(),
            )
        }
        Source::Figure(character) => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s);
            let marks = track.marks.iter_mut();

            (index, marks.map(|m| (&mut m.at_s, &mut m.hold_s)).collect())
        }
        Source::Camera(id) => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *id)
                .ok_or(ServiceError::Gone)?;
            let index = current_index(camera.marks.iter().map(|m| m.at_s), time_s);
            let marks = camera.marks.iter_mut();

            (index, marks.map(|m| (&mut m.at_s, &mut m.hold_s)).collect())
        }
        _ => return Err(bad_mark("source", "")),
    };
    let index = index
        .filter(|i| *i < marks.len())
        .ok_or(ServiceError::Gone)?;

    Ok(Timing { marks, index })
}

fn set_hold(scope: &mut Scope<'_>, source: &Source, hold_s: f64) -> Result<(), ServiceError> {
    let mut timing = timing(scope, source)?;
    let index = timing.index;
    let delta_s = hold_s - *timing.marks[index].1;

    *timing.marks[index].1 = hold_s;
    timing.shift_from(index + 1, delta_s);

    Ok(())
}

fn set_pace(scope: &mut Scope<'_>, source: &Source, pace: Pace) -> Result<(), ServiceError> {
    let (character, index) = figure_mark(scope, source).ok_or_else(|| bad_mark("source", ""))?;
    let before = index
        .checked_sub(1)
        .ok_or_else(|| bad_mark("pace", "first mark"))?;
    let marks = &scope
        .plan
        .figure(&character)
        .ok_or(ServiceError::Gone)?
        .marks;
    let (Some(prev), Some(mark)) = (marks.get(before), marks.get(index)) else {
        return Err(ServiceError::Gone);
    };

    // Arrive after the pause at the mark before and the walk along the way.
    let walk_s = way_length_m(prev.at, mark.via, mark.at) / pace.speed_m_s();
    let delta_s = prev.at_s + prev.hold_s + walk_s - mark.at_s;

    timing(scope, source)?.shift_from(index, delta_s);

    Ok(())
}

/// What the fields of a mark show: `source` is a figure's mark, or a figure or camera for
/// its mark at the playhead. `None` for anything else.
pub(crate) fn mark_info(plan: &FloorPlan, time_s: f64, source: &Source) -> Option<MarkInfo> {
    match source {
        Source::Mark { character, index } => figure_info(plan.figure(character)?, *index),
        Source::Figure(character) => {
            let track = plan.figure(character)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), time_s)?;

            figure_info(track, index)
        }
        Source::Camera(id) => {
            let marks = &plan.camera(id)?.marks;
            let index = current_index(marks.iter().map(|m| m.at_s), time_s)?;
            let mark = marks.get(index)?;
            let pace_m_s = index
                .checked_sub(1)
                .and_then(|before| marks.get(before))
                .and_then(|prev| {
                    pace_to(
                        (prev.at, prev.at_s + prev.hold_s),
                        (mark.at, mark.via, mark.at_s),
                    )
                });

            Some(MarkInfo {
                pose: None,
                hold_s: mark.hold_s,
                has_way: index > 0,
                pace_m_s,
            })
        }
        _ => None,
    }
}

fn figure_info(track: &FigureTrack, index: usize) -> Option<MarkInfo> {
    let mark = track.marks.get(index)?;
    let pace_m_s = index
        .checked_sub(1)
        .and_then(|before| track.marks.get(before))
        .and_then(|prev| {
            pace_to(
                (prev.at, prev.at_s + prev.hold_s),
                (mark.at, mark.via, mark.at_s),
            )
        });

    Some(MarkInfo {
        pose: Some(mark.pose),
        hold_s: mark.hold_s,
        has_way: index > 0,
        pace_m_s,
    })
}

/// Speed of the way from where and when a mark is left to the next mark's place, bend and
/// arrival; `None` when the way takes no time.
fn pace_to(leave: (Point, f64), arrive: (Point, Option<Point>, f64)) -> Option<f64> {
    let walk_s = arrive.2 - leave.1;
    if walk_s <= 0.0 {
        return None;
    }

    Some(way_length_m(leave.0, arrive.1, arrive.0) / walk_s)
}

/// Couples `camera` to `figure`, a figure of this plan, or lets it go.
fn follow(scope: &mut Scope<'_>, camera: &str, figure: Option<&str>) -> Result<(), ServiceError> {
    if let Some(character) = figure {
        if scope.plan.figure(character).is_none() {
            return Err(ServiceError::BadValue {
                field: "follows",
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

    // An old project may name a kind no library has: a square it is.
    let (label, width_m, depth_m, shape) = match library::find(scope.library, kind) {
        Some(item) => (item.label.clone(), item.width_m, item.depth_m, item.shape),
        None => (
            String::new(),
            DEFAULT_OBJECT_M,
            DEFAULT_OBJECT_M,
            ObjectShape::Rect,
        ),
    };

    set.objects.push(SetObject {
        id: format!("o{n}"),
        kind: kind.to_owned(),
        label,
        at,
        width_m,
        depth_m,
        rotation_deg: 0.0,
        shape,
    });

    Ok(())
}

fn resize(scope: &mut Scope<'_>, source: &Source, size_m: (f64, f64)) -> Result<(), ServiceError> {
    let Source::Object(id) = source else {
        return Err(ServiceError::BadValue {
            field: "source",
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

/// A note's text as kept: trimmed, and more than nothing.
fn note_text(text: &str) -> Result<String, ServiceError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ServiceError::BadValue {
            field: "note",
            value: String::new(),
        });
    }

    Ok(text.to_owned())
}

fn add_note(scope: &mut Scope<'_>, at: Point, text: &str) -> Result<(), ServiceError> {
    let text = note_text(text)?;
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let n = (1..)
        .find(|n| !set.notes.iter().any(|note| note.id == format!("n{n}")))
        .unwrap_or(1);

    set.notes.push(Note {
        id: format!("n{n}"),
        at,
        text,
    });

    Ok(())
}

fn edit_note(scope: &mut Scope<'_>, id: &str, text: &str) -> Result<(), ServiceError> {
    let text = note_text(text)?;
    let note = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.notes.iter_mut().find(|n| n.id == id))
        .ok_or(ServiceError::Gone)?;
    note.text = text;

    Ok(())
}

/// A drawing needs something to draw: a stretch, a box with a width and a height, three
/// points for an area.
fn add_drawing(
    scope: &mut Scope<'_>,
    shape: &DrawingShape,
    (color, dash, fill): (FigureColor, Dash, Fill),
) -> Result<(), ServiceError> {
    let drawable = match shape {
        DrawingShape::Line { from, to } | DrawingShape::Arrow { from, to } => from != to,
        DrawingShape::Rect { from, to } | DrawingShape::Ellipse { from, to } => {
            from.x_m != to.x_m && from.y_m != to.y_m
        }
        DrawingShape::Area { points } => points.len() >= MIN_AREA_POINTS,
    };
    if !drawable {
        return Err(ServiceError::BadValue {
            field: "drawing",
            value: "empty".to_owned(),
        });
    }

    let drawings = &scope.plan.drawings;
    let n = (1..)
        .find(|n| !drawings.iter().any(|d| d.id == format!("d{n}")))
        .unwrap_or(1);

    scope.plan.drawings.push(Drawing {
        id: format!("d{n}"),
        shape: shape.clone(),
        color,
        dash,
        fill,
    });

    Ok(())
}

fn set_background(scope: &mut Scope<'_>, dir: &Path, source: &Path) -> Result<(), ServiceError> {
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    if !(set.width_m.is_finite() && set.width_m > 0.0) {
        return Err(ServiceError::BadValue {
            field: "set width",
            value: set.width_m.to_string(),
        });
    }

    set.background = Some(Background {
        file: klappe_store::add_image(dir, source)?,
        at: Point::default(),
        width_m: set.width_m,
        rotation_deg: 0.0,
        opacity: BACKGROUND_OPACITY,
    });

    Ok(())
}

/// The set's background, or `Gone` when it has none (or no set).
fn background<'a>(scope: &'a mut Scope<'_>) -> Result<&'a mut Background, ServiceError> {
    scope
        .set
        .as_deref_mut()
        .and_then(|s| s.background.as_mut())
        .ok_or(ServiceError::Gone)
}

/// Scaling the image by k about `from` keeps `from` where it is and turns every length into
/// k times as much: width, and the distance of its corner `at` from `from`. The rotation is
/// untouched, a scaling does not turn anything.
fn calibrate(
    scope: &mut Scope<'_>,
    (from, to): (Point, Point),
    distance_m: f64,
) -> Result<(), ServiceError> {
    let shown_m = from.distance_m(to);
    if shown_m == 0.0 {
        return Err(ServiceError::BadValue {
            field: "points",
            value: "same".to_owned(),
        });
    }

    let k = distance_m / shown_m;
    let bg = background(scope)?;
    bg.at = from.lerp(bg.at, k);
    bg.width_m *= k;

    Ok(())
}

fn move_background(scope: &mut Scope<'_>, by_m: (f64, f64)) -> Result<(), ServiceError> {
    let bg = background(scope)?;
    bg.at = Point::new(bg.at.x_m + by_m.0, bg.at.y_m + by_m.1);

    Ok(())
}

fn remove_background(scope: &mut Scope<'_>) -> Result<(), ServiceError> {
    background(scope)?;
    if let Some(set) = scope.set.as_deref_mut() {
        set.background = None;
    }

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
        apply(Path::new("."), project, 0, 0, 3.0, Snapping::On, &edit).unwrap();
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
            apply(
                Path::new("."),
                &mut project,
                0,
                0,
                0.0,
                Snapping::On,
                &light(LightKind::Flag),
            )
            .unwrap();
        }
        project.scenes[0].plans[0].lights.remove(0);
        apply(
            Path::new("."),
            &mut project,
            0,
            0,
            0.0,
            Snapping::On,
            &light(LightKind::Flag),
        )
        .unwrap();

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
            apply(Path::new("."), &mut project, 0, 0, 0.0, Snapping::On, &edit),
            Err(ServiceError::NoLocation(ref id)) if id == "l"
        ));
    }
}
