//! Drag targets: what a `Source` names, as the part of the plan a drag changes.

use klappe_model::{Curve, Drawing, Light, Note, PlanImage, Point, SetObject, Wall};
use klappe_stellprobe::{Bend, LightGrip, ObjectGrip, OpeningEnd, Source};

use super::openings::opening_mut;
use super::{current_index, Scope};
use crate::error::ServiceError;

/// What `Move` and `Turn` change.
pub(super) enum Target<'a> {
    Place {
        at: &'a mut Point,
        facing_deg: &'a mut f64,
    },
    Wall(&'a mut Wall),
    /// An opening, which only slides along its wall.
    Slide {
        wall: &'a mut Wall,
        index: usize,
    },
    /// Has a place but no facing: a note.
    Pin {
        at: &'a mut Point,
    },
    /// Points and a turn about the middle of its box.
    Drawing(&'a mut Drawing),
    /// A grip on an end of an opening: moving it sets that end, the other stays.
    OpeningEdge {
        wall: &'a mut Wall,
        index: usize,
        end: OpeningEnd,
    },
    /// A grip on a lamp's beam: moving it turns the lamp or sets its beam, never moves it.
    LightEdge {
        light: &'a mut Light,
        grip: LightGrip,
    },
    /// A grip on the right edge of a note's card: moving it sets the card's width.
    NoteEdge(&'a mut Note),
    /// A grip outside the right edge of an image: moving it sets the image's width.
    ImageEdge(&'a mut PlanImage),
    /// A grip outside the right or bottom edge of an object: moving it sets its width or depth.
    ObjectEdge {
        object: &'a mut SetObject,
        grip: ObjectGrip,
    },
    /// A grip on a point or corner of a drawing: moving it reshapes the drawing.
    Corner {
        drawing: &'a mut Drawing,
        index: usize,
    },
    /// A grip on the edge of an area after point `index`: moving it makes a point there.
    Edge {
        drawing: &'a mut Drawing,
        index: usize,
    },
    /// A grip on the way from mark `a` to mark `b`.
    Curve {
        via: &'a mut Vec<Point>,
        curve: Curve,
        a: Point,
        b: Point,
        bend: Bend,
    },
}

pub(super) fn target<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<Target<'a>, ServiceError> {
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
        Source::FigureVia { .. }
        | Source::CameraVia { .. }
        | Source::DrawingPoint { .. }
        | Source::DrawingEdge { .. }
        | Source::ObjectGrip { .. } => grip_target(scope, source),
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
        Source::Light(_) | Source::LightGrip { .. } => light_target(scope, source),
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
        Source::Opening { .. } | Source::OpeningEdge { .. } => opening_target(scope, source),
        Source::Note(_) | Source::NoteWidth(_) => note_target(scope, source),
        Source::Image(_) | Source::ImageSize(_) => image_target(scope, source),
        Source::Drawing(id) => scope
            .plan
            .drawings
            .iter_mut()
            .find(|d| d.id == *id)
            .map(Target::Drawing)
            .ok_or(ServiceError::Gone),
    }
}

/// A lamp, or the grip on its beam.
fn light_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let (Source::Light(id) | Source::LightGrip { light: id, .. }) = source else {
        return Err(ServiceError::Gone);
    };
    let light = scope
        .plan
        .lights
        .iter_mut()
        .find(|l| l.id == *id)
        .ok_or(ServiceError::Gone)?;

    match source {
        Source::LightGrip { grip, .. } => Ok(Target::LightEdge { light, grip: *grip }),
        _ => Ok(Target::Place {
            at: &mut light.at,
            facing_deg: &mut light.facing_deg,
        }),
    }
}

/// An opening, or the grip on one of its ends.
fn opening_target<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<Target<'a>, ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;

    match source {
        Source::OpeningEdge { end, .. } => Ok(Target::OpeningEdge {
            wall,
            index,
            end: *end,
        }),
        _ => Ok(Target::Slide { wall, index }),
    }
}

/// A note, or the grip on its right edge.
fn note_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let (Source::Note(id) | Source::NoteWidth(id)) = source else {
        return Err(ServiceError::Gone);
    };
    let note = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.notes.iter_mut().find(|n| n.id == *id))
        .ok_or(ServiceError::Gone)?;

    match source {
        Source::NoteWidth(_) => Ok(Target::NoteEdge(note)),
        _ => Ok(Target::Pin { at: &mut note.at }),
    }
}

/// An image, or the grip outside its right edge.
fn image_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    let (Source::Image(id) | Source::ImageSize(id)) = source else {
        return Err(ServiceError::Gone);
    };
    let image = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.images.iter_mut().find(|i| i.id == *id))
        .ok_or(ServiceError::Gone)?;

    match source {
        Source::ImageSize(_) => Ok(Target::ImageEdge(image)),
        _ => Ok(Target::Place {
            at: &mut image.at,
            facing_deg: &mut image.rotation_deg,
        }),
    }
}

/// The grip a `FigureVia` or `CameraVia` names, on its way.
fn grip_target<'a>(scope: &'a mut Scope<'_>, source: &Source) -> Result<Target<'a>, ServiceError> {
    match source {
        Source::FigureVia {
            character,
            index,
            bend,
        } => {
            let track = scope
                .plan
                .figures
                .iter_mut()
                .find(|f| f.character == *character)
                .ok_or(ServiceError::Gone)?;
            curve(&mut track.marks, (*index, *bend), |m| {
                (m.at, &mut m.via, m.curve)
            })
        }
        Source::CameraVia {
            camera,
            index,
            bend,
        } => {
            let camera = scope
                .plan
                .cameras
                .iter_mut()
                .find(|c| c.id == *camera)
                .ok_or(ServiceError::Gone)?;
            curve(&mut camera.marks, (*index, *bend), |m| {
                (m.at, &mut m.via, m.curve)
            })
        }
        Source::DrawingPoint { drawing, index } => {
            let drawing = drawing_mut(scope, drawing)?;

            Ok(Target::Corner {
                drawing,
                index: *index,
            })
        }
        Source::DrawingEdge { drawing, index } => {
            let drawing = drawing_mut(scope, drawing)?;

            Ok(Target::Edge {
                drawing,
                index: *index,
            })
        }
        Source::ObjectGrip { object, grip } => {
            let object = scope
                .set
                .as_deref_mut()
                .and_then(|s| s.objects.iter_mut().find(|o| o.id == *object))
                .ok_or(ServiceError::Gone)?;

            Ok(Target::ObjectEdge {
                object,
                grip: *grip,
            })
        }
        _ => Err(ServiceError::Gone),
    }
}

fn drawing_mut<'a>(scope: &'a mut Scope<'_>, id: &str) -> Result<&'a mut Drawing, ServiceError> {
    scope
        .plan
        .drawings
        .iter_mut()
        .find(|d| d.id == id)
        .ok_or(ServiceError::Gone)
}

/// The grip `bend` on the way to mark `index` of `marks`; `place` gives a mark's place and its
/// `via`. The first mark has no way to it.
fn curve<'a, M>(
    marks: &'a mut [M],
    (index, bend): (usize, Bend),
    place: impl Fn(&'a mut M) -> (Point, &'a mut Vec<Point>, Curve),
) -> Result<Target<'a>, ServiceError> {
    let before = index.checked_sub(1).ok_or(ServiceError::Gone)?;
    let (head, tail) = marks.split_at_mut(index);
    let a = head.get_mut(before).ok_or(ServiceError::Gone)?;
    let this = tail.first_mut().ok_or(ServiceError::Gone)?;

    let (a, ..) = place(a);
    let (b, via, curve) = place(this);

    Ok(Target::Curve {
        via,
        curve,
        a,
        b,
        bend,
    })
}
