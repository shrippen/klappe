//! New elements on the plan: marks, cameras, lights, objects, walls, notes, drawings; also
//! changing a note's text, which shares the note rules.

use klappe_model::{
    is_outline, outline_box, Arrival, Camera, CameraMark, Departure, Drawing, DrawingShape,
    FigureTrack, Light, Mark, Note, ObjectShape, Point, Pose, Sensor, SetObject, Wall,
    FREE_FORM_KIND, NOTE_WIDTH_M,
};

use super::bound::note_mut;
use super::marks::WALK_SPEED_M_S;
use super::{normalize_deg, PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;
use crate::library;
use crate::light_edit;

const DEFAULT_CAMERA_HEIGHT_M: f64 = 1.6;

/// A dolly rolls at walking pace.
const DOLLY_SPEED_M_S: f64 = WALK_SPEED_M_S;

const DEFAULT_SENSOR: Sensor = Sensor::Super35;
const DEFAULT_LENS_MM: f64 = 35.0;
const DEFAULT_ASPECT: f64 = 2.39;

/// An area has an inside from three points on.
const MIN_AREA_POINTS: usize = 3;

/// Side of an object whose kind no library knows.
const DEFAULT_OBJECT_M: f64 = 1.0;
const DEFAULT_WALL_M: f64 = 0.2;

pub(super) fn add_mark(
    scope: &mut Scope<'_>,
    character: &str,
    at: Point,
) -> Result<(), ServiceError> {
    if !scope.characters.iter().any(|c| c.id == character) {
        return Err(ServiceError::BadValue {
            field: Field::Character,
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
        via: Vec::new(),
        curve: Default::default(),
        arrival: Arrival::Walk,
        departure: Departure::Stay,
    });

    Ok(())
}

/// The first id `<prefix><n>` that `taken` does not have.
pub(super) fn free_id<'a>(prefix: &str, taken: impl Iterator<Item = &'a str> + Clone) -> String {
    let n = (1..)
        .find(|n| !taken.clone().any(|id| id == format!("{prefix}{n}")))
        .unwrap_or(1);

    format!("{prefix}{n}")
}

/// The first camera letter no camera of the plan has.
pub(crate) fn free_letter(cameras: &[Camera]) -> Result<char, ServiceError> {
    ('A'..='Z')
        .find(|l| {
            let label = l.to_string();
            !cameras
                .iter()
                .any(|c| c.label == label || c.id == label.to_lowercase())
        })
        .ok_or(ServiceError::NoFreeCamera)
}

pub(super) fn add_camera(
    scope: &mut Scope<'_>,
    at: Point,
    facing_deg: f64,
) -> Result<(), ServiceError> {
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
            via: Vec::new(),
            curve: Default::default(),
            rig: Default::default(),
            lens_mm: None,
        }],
        follows: None,
    };
    scope.plan.cameras.push(camera);

    Ok(())
}

/// Appends a mark to `camera`.
/// The camera arrives, waits, then rolls at `DOLLY_SPEED_M_S`; it keeps its aim, height, tilt
/// and lens, so it goes on looking at the scene.
pub(super) fn add_camera_mark(
    scope: &mut Scope<'_>,
    camera: &str,
    at: Point,
) -> Result<(), ServiceError> {
    let camera = scope
        .plan
        .cameras
        .iter_mut()
        .find(|c| c.id == camera)
        .ok_or(ServiceError::Gone)?;
    let prev = camera.marks.last().ok_or(ServiceError::Gone)?;
    let at_s = prev.at_s + prev.hold_s + prev.at.distance_m(at) / DOLLY_SPEED_M_S;

    camera.marks.push(CameraMark {
        at_s,
        at,
        facing_deg: prev.facing_deg,
        height_m: prev.height_m,
        tilt_deg: prev.tilt_deg,
        hold_s: 0.0,
        via: Vec::new(),
        curve: Default::default(),
        rig: Default::default(),
        lens_mm: prev.lens_mm,
    });

    Ok(())
}

/// The lamp edits: a new lamp of a library entry, or a change of one.
pub(super) fn edit_light(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddLight { preset, at } => add_light(scope, preset, *at),
        PlanEdit::SetLight { light, change } => light_edit::set_light(scope.plan, light, change),
        _ => Ok(()),
    }
}

fn add_light(scope: &mut Scope<'_>, preset: &str, at: Point) -> Result<(), ServiceError> {
    let preset = light_edit::preset(preset)?;
    let id = free_id("l", scope.plan.lights.iter().map(|l| l.id.as_str()));

    scope.plan.lights.push(Light {
        label: id.to_uppercase(),
        id,
        kind: preset.kind,
        at,
        facing_deg: 0.0,
        beam_deg: preset.beam_deg,
        color_temp_k: preset.color_temp_k,
        power_w: preset.power_w,
    });

    Ok(())
}

pub(super) fn add_object(scope: &mut Scope<'_>, kind: &str, at: Point) -> Result<(), ServiceError> {
    let kind = kind.trim();
    if kind.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Kind,
            value: String::new(),
        });
    }

    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let id = free_id("o", set.objects.iter().map(|o| o.id.as_str()));

    // An old project may name a kind no library has: a square it is.
    let (label, width_m, depth_m, shape, height_m) = match library::find(scope.library, kind) {
        Some(item) => (
            item.label.clone(),
            item.width_m,
            item.depth_m,
            item.shape.clone(),
            item.height_m,
        ),
        None => (
            String::new(),
            DEFAULT_OBJECT_M,
            DEFAULT_OBJECT_M,
            ObjectShape::Rect,
            None,
        ),
    };

    set.objects.push(SetObject {
        id,
        kind: kind.to_owned(),
        label,
        at,
        width_m,
        depth_m,
        rotation_deg: 0.0,
        shape,
        height_m,
        base_m: None,
        mirrored: false,
    });

    Ok(())
}

/// A library object or a free form pushed on the set.
pub(super) fn add_objects(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddObject { kind, at } => add_object(scope, kind, *at),
        PlanEdit::AddShape { points } => add_shape(scope, points),
        _ => Ok(()),
    }
}

/// A free form (LH-92) pushed on the set as an object of the kind `form`. The object sits at
/// the centre of the outline's box and keeps the points relative to it. A shape with no area
/// has nothing to draw.
fn add_shape(scope: &mut Scope<'_>, points: &[Point]) -> Result<(), ServiceError> {
    if !is_outline(points) {
        return Err(ServiceError::BadValue {
            field: Field::Points,
            value: points.len().to_string(),
        });
    }

    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let id = free_id("o", set.objects.iter().map(|o| o.id.as_str()));
    let (at, width_m, depth_m) = outline_box(points);
    let points = points
        .iter()
        .map(|p| Point::new(p.x_m - at.x_m, p.y_m - at.y_m))
        .collect();

    set.objects.push(SetObject {
        id,
        kind: FREE_FORM_KIND.to_owned(),
        label: String::new(),
        at,
        width_m,
        depth_m,
        rotation_deg: 0.0,
        shape: ObjectShape::Polygon { points },
        height_m: None,
        base_m: None,
        mirrored: false,
    });

    Ok(())
}

/// A wall, or the four walls of a room (LH-96), pushed on the set. A room goes round from
/// `from` to the corner across, so each wall ends exactly where the next begins.
pub(super) fn add_walls(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    let (corners, ring) = match edit {
        PlanEdit::AddRoom { from, to } => (room_corners(*from, *to), true),
        PlanEdit::AddWall { from, to } => (vec![*from, *to], false),
        _ => return Ok(()),
    };

    // A wall without length or a room without area has nothing to draw.
    if corners[0] == corners[1] || (ring && corners[0].y_m == corners[2].y_m) {
        return Err(ServiceError::BadValue {
            field: Field::Wall,
            value: "0".to_owned(),
        });
    }

    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let count = if ring { corners.len() } else { 1 };

    for i in 0..count {
        let to = corners[(i + 1) % corners.len()];
        set.walls.push(Wall::new(corners[i], to, DEFAULT_WALL_M));
    }

    Ok(())
}

/// The corners of the rectangle between two opposite ones, in order round it.
fn room_corners(from: Point, to: Point) -> Vec<Point> {
    vec![
        from,
        Point::new(to.x_m, from.y_m),
        to,
        Point::new(from.x_m, to.y_m),
    ]
}

/// A note's text as kept: trimmed, and more than nothing.
pub(super) fn note_text(text: &str) -> Result<String, ServiceError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Note,
            value: String::new(),
        });
    }

    Ok(text.to_owned())
}

pub(super) fn add_note(scope: &mut Scope<'_>, at: Point, text: &str) -> Result<(), ServiceError> {
    let text = note_text(text)?;
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let id = free_id("n", set.notes.iter().map(|n| n.id.as_str()));

    set.notes.push(Note {
        id,
        at,
        text,
        width_m: NOTE_WIDTH_M,
        bound: None,
    });

    Ok(())
}

pub(super) fn edit_note(scope: &mut Scope<'_>, id: &str, text: &str) -> Result<(), ServiceError> {
    let text = note_text(text)?;
    let note = note_mut(scope, id).ok_or(ServiceError::Gone)?;
    note.text = text;

    Ok(())
}

/// A drawing needs something to draw: a stretch, a box with a width and a height, three
/// points for an area.
pub(super) fn check_drawable(shape: &DrawingShape) -> Result<(), ServiceError> {
    let drawable = match shape {
        DrawingShape::Line { from, to } | DrawingShape::Arrow { from, to } => from != to,
        DrawingShape::Rect { from, to } | DrawingShape::Ellipse { from, to } => {
            from.x_m != to.x_m && from.y_m != to.y_m
        }
        DrawingShape::Area { points } => points.len() >= MIN_AREA_POINTS,
    };
    if drawable {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Drawing,
        value: "empty".to_owned(),
    })
}

/// Applies `AddDrawing`; any other edit is none of its business.
pub(super) fn add_drawing(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    let PlanEdit::AddDrawing {
        shape,
        color,
        dash,
        fill,
    } = edit
    else {
        return Ok(());
    };
    check_drawable(shape)?;

    let id = free_id("d", scope.plan.drawings.iter().map(|d| d.id.as_str()));

    scope.plan.drawings.push(Drawing {
        id,
        shape: shape.clone(),
        color: *color,
        dash: *dash,
        fill: *fill,
        rotation_deg: 0.0,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use klappe_model::{Point, Project, Sensor};

    use super::super::tests::empty;
    use super::super::{apply, Grid, PlanEdit, Snapping};

    fn add_camera(project: &mut Project) {
        let edit = PlanEdit::AddCamera {
            at: Point::new(1.0, 2.0),
            facing_deg: 0.0,
        };
        apply(
            Path::new("."),
            project,
            0,
            0,
            3.0,
            (Snapping::On, Grid::default()),
            &edit,
        )
        .unwrap();
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

        let light = |preset: &str| PlanEdit::AddLight {
            preset: preset.into(),
            at: Point::new(0.0, 0.0),
        };
        for _ in 0..2 {
            apply(
                Path::new("."),
                &mut project,
                0,
                0,
                0.0,
                (Snapping::On, Grid::default()),
                &light("flag"),
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
            (Snapping::On, Grid::default()),
            &light("flag"),
        )
        .unwrap();

        let ids: Vec<_> = project.scenes[0].plans[0]
            .lights
            .iter()
            .map(|l| l.id.as_str())
            .collect();
        assert_eq!(ids, ["l2", "l1"]);
    }
}
