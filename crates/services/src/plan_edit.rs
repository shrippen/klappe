use std::path::{Path, PathBuf};

use klappe_model::{
    normalize_deg, Arrival, Character, Curve, Dash, Departure, DoorSide, DrawingShape, FigureColor,
    Fill, FloorPlan, Hinge, LibraryItem, OpeningKind, Point, Pose, Project, Rig, Scene, SetPlan,
};
use klappe_stellprobe::Source;

use crate::camera_edit;
use crate::error::ServiceError;
use crate::field::Field;
use crate::light_edit::LightChange;
use crate::session::Layer;

mod add;
mod background;
mod bound;
mod check;
mod drawings;
mod grid;
mod group;
mod heights;
mod images;
mod marks;
mod moving;
mod object;
mod openings;
mod picking;
mod props;
mod remove;
mod target;

pub(crate) use add::free_letter;
use add::{
    add_camera, add_camera_mark, add_drawing, add_mark, add_note, add_objects, add_walls,
    edit_light, edit_note, free_id,
};
use background::{calibrate, move_background, remove_background, set_background};
use check::check;
use drawings::set_drawing_style;
pub(crate) use grid::snap;
pub use grid::Grid;
use heights::edit_height;
use images::edit_image;
use marks::edit_field;
pub(crate) use marks::{check_tripods, mark_info, way_curve};
pub use marks::{MarkInfo, Pace};
use moving::{anchor_pair, carry_cameras, edit_way, move_to, points_mut, shift, spin, turn};
pub use object::ObjectChange;
use openings::edit_opening;
use remove::remove;

pub(crate) use picking::{pick, pick_in, pick_lasso};

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
    /// Copies the group, set down by a distance. Objects, lamps, notes, images, drawings, walls and
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
    /// Appends a mark to the camera `camera` (LH-33): it keeps the aim, height, tilt and lens
    /// of the last mark and comes by dolly.
    /// Adds a mark to the way of the set object `object` at `at` (LH-24): the first click also
    /// marks where the object stands now, at the playhead.
    AddObjectMark {
        object: String,
        at: Point,
    },
    /// Lets the figure `figure` carry the object from its mark until the next one (LH-24), or
    /// sets it down with `None`: `Source::ObjectMark`, or `Source::Object` for its mark at the
    /// playhead.
    SetCarrier {
        source: Source,
        figure: Option<String>,
    },
    AddCameraMark {
        camera: String,
        at: Point,
    },
    /// Sets a mark before the first one of the figure or camera `source` names, at `at`
    /// (LH-105): where it comes from. The old marks move later by the way from the new mark to
    /// the old first one, so the old first mark keeps its pause and the later ways their
    /// duration. A figure that appeared at the old first mark appears at the new one.
    AddMarkBefore {
        source: Source,
        at: Point,
    },
    /// Moves a way at once (LH-107): both the marks it runs between, and its bends, by a
    /// distance. `source` names the way: a mark (or a figure, a camera: its mark at the
    /// playhead) or a grip on the way. With snapping on, the mark it leads to lands on the grid.
    /// The first mark has no way.
    ShiftWay {
        source: Source,
        dx_m: f64,
        dy_m: f64,
    },
    /// Makes the way to a mark a smooth curve or straight pieces through its bends (LH-108);
    /// `source` names it as `ShiftWay` does.
    SetCurve {
        source: Source,
        curve: Curve,
    },
    /// Sets a new lamp of the built-in library entry `preset` (LH-51).
    AddLight {
        preset: String,
        at: Point,
    },
    /// Changes one thing of the lamp `light` (LH-50); its facing is `Turn`.
    SetLight {
        light: String,
        change: LightChange,
    },
    /// Sets a new object of library entry `kind` (LH-11), as big as the entry says; a kind
    /// no library knows is a 1 m square.
    AddObject {
        kind: String,
        at: Point,
    },
    /// Sets a free form (LH-92): kind `form`, outline `points` in plan metres, no height yet.
    /// Refused with under three points or no area.
    AddShape {
        points: Vec<Point>,
    },
    /// Gives the object `source` another width and depth (LH-25). Only objects have a size
    /// to change.
    Resize {
        source: Source,
        width_m: f64,
        depth_m: f64,
    },
    /// Changes the object `source` (LH-114 to LH-116), or pins a note to an element (LH-118).
    SetObject {
        source: Source,
        change: ObjectChange,
    },
    /// Sets how tall the object `source` is (LH-17), or takes the height off with `None`. Only
    /// objects have a height.
    SetObjectHeight {
        source: Source,
        height_m: Option<f64>,
    },
    /// Sets how tall the wall `source` is (LH-90), or back to the room's height with `None`.
    SetWallHeight {
        source: Source,
        height_m: Option<f64>,
    },
    /// Sets where the object `source` begins above the floor (LH-90), or back to standing on
    /// what is below with `None`.
    SetObjectBase {
        source: Source,
        base_m: Option<f64>,
    },
    AddWall {
        from: Point,
        to: Point,
    },
    /// Four walls round the rectangle with the opposite corners `from` and `to` (LH-96).
    /// Refused when the rectangle has no area.
    AddRoom {
        from: Point,
        to: Point,
    },
    /// Sets a door or window, of the standard width of its kind, in the wall nearest `at`
    /// (LH-10). Refused when no wall is near, the wall is too short or the opening would meet
    /// another.
    AddOpening {
        kind: OpeningKind,
        at: Point,
    },
    /// Gives the opening `source` another width, about its centre (LH-10).
    SetOpeningWidth {
        source: Source,
        width_m: f64,
    },
    /// Sets how the door `source` swings (LH-10): the side of the wall it opens to and the end
    /// of the gap its hinge is at; `None` keeps what it has. Refused for a window.
    SetOpeningSwing {
        source: Source,
        side: Option<DoorSide>,
        hinge: Option<Hinge>,
    },
    /// Sets the sill of the opening `source` (LH-90), or back to its kind's with `None`. Refused
    /// at or above its head.
    SetOpeningSill {
        source: Source,
        sill_m: Option<f64>,
    },
    /// Sets the head of the opening `source` (LH-90), or back to its kind's with `None`.
    /// Refused at or below its sill.
    SetOpeningHead {
        source: Source,
        head_m: Option<f64>,
    },
    /// Pins a note with `text` to the set.
    AddNote {
        at: Point,
        text: String,
    },
    /// Takes the image file `source` (PNG or JPEG) into the project's `bilder/` and places it,
    /// centred on `at`, as a photo or sketch (LH-18), `IMAGE_WIDTH_M` wide. Undo takes the
    /// image back out of the model; the copy stays in `bilder/`, it is the user's file.
    AddImage {
        source: PathBuf,
        at: Point,
    },
    /// Gives the image `source` names another opacity (`MIN_IMAGE_OPACITY` to 1).
    SetImageOpacity {
        source: Source,
        opacity: f64,
    },
    /// Swaps the file of the image `source` names for the image file `with` (PNG or JPEG),
    /// copied into `bilder/`; place, width, turn and opacity stay. Undo gives the old file
    /// back; both copies stay in `bilder/`.
    ReplaceImage {
        source: Source,
        with: PathBuf,
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
    /// Gives the drawing `source` another colour, line and fill (LH-19). `fill` only matters
    /// for shapes with an inside.
    SetDrawingStyle {
        source: Source,
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
    /// Sets the rig of the camera mark at the playhead (LH-33): `Source::Camera`. A tripod
    /// keeps the place of the mark before; the edit is refused when this mark lies elsewhere.
    SetRig {
        source: Source,
        rig: Rig,
    },
    /// Sets the lens at the camera mark at the playhead (LH-33), or takes it away with `None`:
    /// `Source::Camera`. Between marks with a lens the focal length runs with time.
    SetMarkLens {
        source: Source,
        lens_mm: Option<f64>,
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
            | PlanEdit::SetCarrier { source, .. }
            | PlanEdit::SetCameraAim { source, .. }
            | PlanEdit::SetRig { source, .. }
            | PlanEdit::SetMarkLens { source, .. }
            | PlanEdit::SetObjectHeight { source, .. }
            | PlanEdit::SetOpeningWidth { source, .. }
            | PlanEdit::SetOpeningSwing { source, .. }
            | PlanEdit::SetWallHeight { source, .. }
            | PlanEdit::SetObjectBase { source, .. }
            | PlanEdit::SetOpeningSill { source, .. }
            | PlanEdit::SetOpeningHead { source, .. }
            | PlanEdit::SetPace { source, .. }
            | PlanEdit::AddMarkBefore { source, .. }
            | PlanEdit::ShiftWay { source, .. }
            | PlanEdit::SetCurve { source, .. }
            | PlanEdit::SetDrawingStyle { source, .. }
            | PlanEdit::SetImageOpacity { source, .. }
            | PlanEdit::ReplaceImage { source, .. }
            | PlanEdit::Remove { source }
            | PlanEdit::Straighten { source } => layer_of_source(source),
            PlanEdit::ShiftGroup { sources, .. }
            | PlanEdit::SpinGroup { sources, .. }
            | PlanEdit::RemoveGroup { sources }
            | PlanEdit::Duplicate { sources, .. } => {
                return sources.iter().map(layer_of_source).collect();
            }
            PlanEdit::AddMark { .. } => Layer::Figures,
            PlanEdit::AddObjectMark { .. } => Layer::Set,
            PlanEdit::AddCamera { .. }
            | PlanEdit::AddCameraMark { .. }
            | PlanEdit::Follow { .. }
            | PlanEdit::SetSensor { .. }
            | PlanEdit::SetLens { .. }
            | PlanEdit::SetAspect { .. } => Layer::Cameras,
            PlanEdit::AddLight { .. } | PlanEdit::SetLight { .. } => Layer::Lights,
            PlanEdit::AddNote { .. } | PlanEdit::EditNote { .. } | PlanEdit::AddImage { .. } => {
                Layer::Notes
            }
            PlanEdit::SetObject { source, change } => match change {
                ObjectChange::Label(_) => Layer::Notes,
                _ => layer_of_source(source),
            },
            PlanEdit::AddDrawing { .. } => Layer::Drawings,
            PlanEdit::AddObject { .. }
            | PlanEdit::AddShape { .. }
            | PlanEdit::AddWall { .. }
            | PlanEdit::AddRoom { .. }
            | PlanEdit::AddOpening { .. }
            | PlanEdit::SetBackground { .. }
            | PlanEdit::Calibrate { .. }
            | PlanEdit::MoveBackground { .. }
            | PlanEdit::RemoveBackground => Layer::Set,
        };

        vec![layer]
    }
}

/// The layer an element of the plan is on.
pub fn layer_of_source(source: &Source) -> Layer {
    match source {
        Source::Object(_)
        | Source::ObjectMark { .. }
        | Source::Wall(_)
        | Source::Opening { .. } => Layer::Set,
        Source::Light(_) => Layer::Lights,
        Source::Camera(_) | Source::CameraVia { .. } => Layer::Cameras,
        Source::Figure(_) | Source::Mark { .. } | Source::FigureVia { .. } => Layer::Figures,
        Source::Note(_) | Source::Image(_) => Layer::Notes,
        Source::Drawing(_) => Layer::Drawings,
        // A grip is on the layer of the element it belongs to.
        grip => grip.owner().as_ref().map_or(Layer::Set, layer_of_source),
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

/// Applies `edit` to the plan `plan` of scene `scene` at playhead `time_s`; `snap_to` is for
/// `Shift`, the other edits take their points as given. `dir` is the project folder, where
/// `SetBackground` copies its image to. On an error the project may be half changed: the
/// caller works on a copy.
pub(crate) fn apply(
    dir: &Path,
    project: &mut Project,
    scene: usize,
    plan: usize,
    time_s: f64,
    snap_to: (Snapping, Grid),
    edit: &PlanEdit,
) -> Result<(), ServiceError> {
    check(edit)?;

    let at = (scene, plan);
    let held = bound::held(project, at, time_s);
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

    let done = match edit {
        PlanEdit::Move { source, to } => {
            carry_cameras(&mut scope, source, |s| move_to(s, source, *to))
        }
        PlanEdit::Shift { source, dx_m, dy_m } => carry_cameras(&mut scope, source, |s| {
            shift(s, source, (*dx_m, *dy_m), snap_to)
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
        } => group::shift(&mut scope, sources, (*dx_m, *dy_m), snap_to),
        PlanEdit::SpinGroup { sources, by_deg } => group::spin(&mut scope, sources, *by_deg),
        PlanEdit::RemoveGroup { sources } => group::remove_all(&mut scope, sources),
        PlanEdit::Duplicate {
            sources,
            dx_m,
            dy_m,
        } => group::duplicate(&mut scope, sources, (*dx_m, *dy_m)),
        PlanEdit::Straighten { source } => straighten(&mut scope, source),
        PlanEdit::AddMarkBefore { .. } | PlanEdit::ShiftWay { .. } | PlanEdit::SetCurve { .. } => {
            edit_way(&mut scope, edit, snap_to)
        }
        PlanEdit::AddMark { character, at } => add_mark(&mut scope, character, *at),
        PlanEdit::AddCamera { at, facing_deg } => add_camera(&mut scope, *at, *facing_deg),
        PlanEdit::AddCameraMark { camera, at } => add_camera_mark(&mut scope, camera, *at),
        PlanEdit::AddObjectMark { .. } | PlanEdit::SetCarrier { .. } => {
            props::edit(&mut scope, edit)
        }
        PlanEdit::AddLight { .. } | PlanEdit::SetLight { .. } => edit_light(&mut scope, edit),
        PlanEdit::AddObject { .. } | PlanEdit::AddShape { .. } => add_objects(&mut scope, edit),
        PlanEdit::Resize { .. } | PlanEdit::SetObject { .. } => object::edit(&mut scope, edit),
        PlanEdit::AddWall { .. } | PlanEdit::AddRoom { .. } => add_walls(&mut scope, edit),
        PlanEdit::AddOpening { .. }
        | PlanEdit::SetOpeningWidth { .. }
        | PlanEdit::SetOpeningSwing { .. } => edit_opening(&mut scope, edit),
        PlanEdit::SetWallHeight { .. }
        | PlanEdit::SetObjectHeight { .. }
        | PlanEdit::SetObjectBase { .. }
        | PlanEdit::SetOpeningSill { .. }
        | PlanEdit::SetOpeningHead { .. } => edit_height(&mut scope, edit),
        PlanEdit::AddNote { at, text } => add_note(&mut scope, *at, text),
        PlanEdit::AddDrawing { .. } => add_drawing(&mut scope, edit),
        PlanEdit::SetDrawingStyle { .. } => set_drawing_style(&mut scope, edit),
        PlanEdit::EditNote { id, text } => edit_note(&mut scope, id, text),
        PlanEdit::AddImage { .. }
        | PlanEdit::SetImageOpacity { .. }
        | PlanEdit::ReplaceImage { .. } => edit_image(&mut scope, dir, edit),
        PlanEdit::SetBackground { source } => set_background(&mut scope, dir, source),
        PlanEdit::Calibrate {
            from,
            to,
            distance_m,
        } => calibrate(&mut scope, (*from, *to), *distance_m),
        PlanEdit::MoveBackground { dx_m, dy_m } => move_background(&mut scope, (*dx_m, *dy_m)),
        PlanEdit::RemoveBackground => remove_background(&mut scope),
        PlanEdit::Follow { camera, figure } => follow(&mut scope, camera, figure.as_deref()),
        PlanEdit::SetSensor { .. } | PlanEdit::SetLens { .. } | PlanEdit::SetAspect { .. } => {
            camera_edit::edit(scope.plan, edit)
        }
        PlanEdit::SetPose { .. }
        | PlanEdit::SetArrival { .. }
        | PlanEdit::SetDeparture { .. }
        | PlanEdit::SetHold { .. }
        | PlanEdit::SetRig { .. }
        | PlanEdit::SetMarkLens { .. }
        | PlanEdit::SetPace { .. }
        | PlanEdit::SetCameraAim { .. } => edit_field(&mut scope, edit),
    };

    bound::follow(project, at, time_s, &held);

    done
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

/// Takes the bends out of the way the source names: to one mark (a mark or its grip), or all
/// ways of a figure or camera. Anything else has no way.
fn straighten(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let plan = &mut *scope.plan;
    let find = |id: &str| plan.figures.iter().position(|f| f.character == id);

    match source {
        Source::Mark { character, index }
        | Source::FigureVia {
            character, index, ..
        } => {
            let track = find(character).ok_or(ServiceError::Gone)?;
            let mark = plan.figures[track].marks.get_mut(*index);
            mark.ok_or(ServiceError::Gone)?.via.clear();
        }
        Source::Figure(character) => {
            let track = find(character).ok_or(ServiceError::Gone)?;
            plan.figures[track]
                .marks
                .iter_mut()
                .for_each(|m| m.via.clear());
        }
        Source::CameraVia { camera, index, .. } => {
            let camera = plan.cameras.iter_mut().find(|c| c.id == *camera);
            let mark = camera.ok_or(ServiceError::Gone)?.marks.get_mut(*index);
            mark.ok_or(ServiceError::Gone)?.via.clear();
        }
        Source::Camera(id) => {
            let camera = plan.cameras.iter_mut().find(|c| c.id == *id);
            camera
                .ok_or(ServiceError::Gone)?
                .marks
                .iter_mut()
                .for_each(|m| m.via.clear());
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

#[cfg(test)]
mod tests;
