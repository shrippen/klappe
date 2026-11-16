//! Notes bound to an element (LH-118): a label of an object, a figure or a mark. A bound note
//! keeps its own place (`at`) and moves by what its element moved in an edit, so the pair
//! stays as it was set. Edits the note itself takes part in (a group moved together, the note
//! dragged) leave it where they put it; an element that is gone lets the note go.
//!
//! ```text
//!  edit:   held ──► the edit ──► follow
//!          note a, element e      e moved by (dx, dy)  ─►  note.at += (dx, dy)
//! ```

use klappe_model::{FloorPlan, Note, Point, Project, SetPlan, NOTE_WIDTH_M};
use klappe_stellprobe::{figure_at, Source};

use super::add::{free_id, note_text};
use super::Scope;
use crate::error::ServiceError;
use crate::field::Field;

/// Gap between an element and the label set beside it, right and up.
const LABEL_GAP_M: f64 = 0.3;

/// Where an element is, and how many marks its figure has (a mark's index means something
/// else once one went).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Anchor {
    at: Point,
    marks: usize,
}

/// A bound note and its element before an edit.
#[derive(Debug, Clone)]
pub(super) struct Held {
    note: String,
    at: Point,
    key: String,
    anchor: Option<Anchor>,
}

/// The key a note keeps for `source`; `None` for what cannot be labelled.
fn key_of(source: &Source) -> Option<String> {
    match source {
        Source::Object(id) => Some(format!("object:{id}")),
        Source::Figure(character) => Some(format!("figure:{character}")),
        Source::Mark { character, index } => Some(format!("mark:{character}:{index}")),
        _ => None,
    }
}

fn anchor(set: &SetPlan, plan: &FloorPlan, time_s: f64, key: &str) -> Option<Anchor> {
    let (kind, rest) = key.split_once(':')?;

    match kind {
        "object" => {
            let object = set.objects.iter().find(|o| o.id == rest)?;
            Some(Anchor {
                at: object.at,
                marks: 0,
            })
        }
        "figure" => {
            let track = plan.figures.iter().find(|f| f.character == rest)?;
            Some(Anchor {
                at: figure_at(track, time_s)?.at,
                marks: track.marks.len(),
            })
        }
        "mark" => {
            let (character, index) = rest.rsplit_once(':')?;
            let track = plan.figures.iter().find(|f| f.character == character)?;
            let mark = track.marks.get(index.parse::<usize>().ok()?)?;
            Some(Anchor {
                at: mark.at,
                marks: track.marks.len(),
            })
        }
        _ => None,
    }
}

/// The set and plan of the scene on view.
fn parts(project: &Project, (scene, plan): (usize, usize)) -> Option<(&SetPlan, &FloorPlan)> {
    let scene = project.scenes.get(scene)?;
    let set = &project
        .locations
        .iter()
        .find(|l| l.id == scene.location)?
        .set;

    Some((set, scene.plans.get(plan)?))
}

/// The bound notes of the scene's set, with where they and their elements are.
pub(super) fn held(project: &Project, at: (usize, usize), time_s: f64) -> Vec<Held> {
    let Some((set, plan)) = parts(project, at) else {
        return Vec::new();
    };

    set.notes
        .iter()
        .filter_map(|n| {
            let key = n.bound.clone()?;
            Some(Held {
                note: n.id.clone(),
                at: n.at,
                anchor: anchor(set, plan, time_s, &key),
                key,
            })
        })
        .collect()
}

/// Moves each held note by what its element moved; lets go of a note whose element is gone.
pub(super) fn follow(project: &mut Project, at: (usize, usize), time_s: f64, held: &[Held]) {
    let Some((set, plan)) = parts(project, at) else {
        return;
    };
    let after: Vec<Option<Anchor>> = held
        .iter()
        .map(|h| anchor(set, plan, time_s, &h.key))
        .collect();
    let location = project.scenes[at.0].location.clone();
    let Some(set) = project
        .locations
        .iter_mut()
        .find(|l| l.id == location)
        .map(|l| &mut l.set)
    else {
        return;
    };

    for (h, now) in held.iter().zip(after) {
        let Some(note) = set.notes.iter_mut().find(|n| n.id == h.note) else {
            continue;
        };
        // The note moved by itself or let go in this edit, or its element was missing before.
        if note.at != h.at || note.bound.is_none() {
            continue;
        }
        let Some(was) = h.anchor else {
            continue;
        };

        match now {
            Some(now) if now.marks == was.marks => {
                let (dx_m, dy_m) = (now.at.x_m - was.at.x_m, now.at.y_m - was.at.y_m);

                note.at = Point::new(note.at.x_m + dx_m, note.at.y_m + dy_m);
            }
            _ => note.bound = None,
        }
    }
}

fn bad_source() -> ServiceError {
    ServiceError::BadValue {
        field: Field::Source,
        value: String::new(),
    }
}

/// A note with `text` beside the element `source`, bound to it: right of and above an
/// object's box, off a figure's or mark's place.
pub(super) fn add_label(
    scope: &mut Scope<'_>,
    source: &Source,
    text: &str,
) -> Result<(), ServiceError> {
    let text = note_text(text)?;
    let key = key_of(source).ok_or_else(bad_source)?;
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let at = anchor(set, scope.plan, scope.time_s, &key)
        .ok_or(ServiceError::Gone)?
        .at;
    let reach_m = match source {
        Source::Object(id) => set
            .objects
            .iter()
            .find(|o| o.id == *id)
            .map_or(0.0, |o| o.width_m.max(o.depth_m) / 2.0),
        _ => 0.0,
    };
    let id = free_id("n", set.notes.iter().map(|n| n.id.as_str()));

    set.notes.push(Note {
        id,
        at: Point::new(
            at.x_m + reach_m + LABEL_GAP_M,
            at.y_m - reach_m - LABEL_GAP_M,
        ),
        text,
        width_m: NOTE_WIDTH_M,
        bound: Some(key),
    });

    Ok(())
}

/// Lets the note `source` names go.
pub(super) fn unbind(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let Source::Note(id) = source else {
        return Err(bad_source());
    };
    let note = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.notes.iter_mut().find(|n| n.id == *id))
        .ok_or(ServiceError::Gone)?;

    note.bound = None;

    Ok(())
}
