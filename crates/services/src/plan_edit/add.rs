//! New elements on the plan: marks, cameras, lights, objects, walls, notes, drawings; also
//! changing a note's text, which shares the note rules.

use klappe_model::{
    Arrival, Camera, CameraMark, Dash, Departure, Drawing, DrawingShape, FigureColor, FigureTrack,
    Fill, Light, LightKind, Mark, Note, ObjectShape, Point, Pose, Sensor, SetObject, Wall,
};

use super::marks::WALK_SPEED_M_S;
use super::{normalize_deg, Scope};
use crate::error::ServiceError;
use crate::field::Field;
use crate::library;

const DEFAULT_CAMERA_HEIGHT_M: f64 = 1.6;

const DEFAULT_SENSOR: Sensor = Sensor::Super35;
const DEFAULT_LENS_MM: f64 = 35.0;
const DEFAULT_ASPECT: f64 = 2.39;

/// An area has an inside from three points on.
const MIN_AREA_POINTS: usize = 3;

/// Side of an object whose kind no library knows.
const DEFAULT_OBJECT_M: f64 = 1.0;
const DEFAULT_WALL_M: f64 = 0.2;

const TUNGSTEN_K: f64 = 3200.0;
const DAYLIGHT_K: f64 = 5600.0;

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
        via: None,
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
pub(super) fn free_letter(cameras: &[Camera]) -> Result<char, ServiceError> {
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
            via: None,
        }],
        follows: None,
    };
    scope.plan.cameras.push(camera);

    Ok(())
}

pub(super) fn add_light(
    scope: &mut Scope<'_>,
    kind: LightKind,
    at: Point,
) -> Result<(), ServiceError> {
    let id = free_id("l", scope.plan.lights.iter().map(|l| l.id.as_str()));
    let defaults = light_defaults(kind);

    scope.plan.lights.push(Light {
        label: id.to_uppercase(),
        id,
        kind,
        at,
        facing_deg: 0.0,
        beam_deg: defaults.beam_deg,
        color_temp_k: defaults.color_temp_k,
        power_w: defaults.power_w,
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
            item.shape,
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
    });

    Ok(())
}

pub(super) fn add_wall(scope: &mut Scope<'_>, from: Point, to: Point) -> Result<(), ServiceError> {
    if from == to {
        return Err(ServiceError::BadValue {
            field: Field::Wall,
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

    set.notes.push(Note { id, at, text });

    Ok(())
}

pub(super) fn edit_note(scope: &mut Scope<'_>, id: &str, text: &str) -> Result<(), ServiceError> {
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
pub(super) fn add_drawing(
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
            field: Field::Drawing,
            value: "empty".to_owned(),
        });
    }

    let id = free_id("d", scope.plan.drawings.iter().map(|d| d.id.as_str()));

    scope.plan.drawings.push(Drawing {
        id,
        shape: shape.clone(),
        color,
        dash,
        fill,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use klappe_model::{LightKind, Point, Project, Sensor};

    use super::super::tests::empty;
    use super::super::{apply, PlanEdit, Snapping};

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
}
