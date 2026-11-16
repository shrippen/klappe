//! Notes bound to an element (LH-118): a label of an object, a figure or a mark. A bound note
//! keeps its own place (`at`) and moves by what its element moved in an edit, so the pair
//! stays as it was set. Edits the note itself takes part in (a group moved together, the note
//! dragged) leave it where they put it; an element that is gone lets the note go.
//!
//! ```text
//!  object   ─► note in the set's notes      bound = "object:<id>"
//!  figure   ─► label in the plan's labels   bound = "figure:<character>"
//!  mark     ─► label in the plan's labels   bound = "mark:<character>:<index>"
//!
//!  edit:   held ──► the edit ──► follow
//!          note a, element e      e moved by (dx, dy)  ─►  note.at += (dx, dy)
//! ```
//!
//! An object belongs to the location, like the set's notes. A figure and a mark belong to a
//! plan, so their labels live in it and go with it (a copy of the plan, a deleted plan or
//! scene): no key names a plan or a scene.

use klappe_model::{FloorPlan, Note, Point, Project, SetPlan, NOTE_WIDTH_M};
use klappe_stellprobe::{figure_at, Source};

use super::add::{free_id, note_text};
use super::Scope;
use crate::error::ServiceError;
use crate::field::Field;

/// Gap between an element and the label set beside it, right and up.
const LABEL_GAP_M: f64 = 0.3;

/// Where an element is, and for a mark how many marks its figure has (its index means
/// something else once one went).
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

/// What a bound note is bound to.
enum Target<'a> {
    Object(&'a str),
    Figure(&'a str),
    Mark { character: &'a str, index: usize },
}

impl<'a> Target<'a> {
    /// Reads `object:<id>`, `figure:<character>` or `mark:<character>:<index>`.
    fn parse(key: &'a str) -> Option<Self> {
        let (kind, rest) = key.split_once(':')?;

        match kind {
            "object" => Some(Self::Object(rest)),
            "figure" => Some(Self::Figure(rest)),
            "mark" => {
                let (character, index) = rest.rsplit_once(':')?;
                Some(Self::Mark {
                    character,
                    index: index.parse().ok()?,
                })
            }
            _ => None,
        }
    }
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
    match Target::parse(key)? {
        Target::Object(id) => {
            let object = set.objects.iter().find(|o| o.id == id)?;
            Some(Anchor {
                at: object.at,
                marks: 0,
            })
        }
        Target::Figure(character) => {
            let track = plan.figures.iter().find(|f| f.character == character)?;
            Some(Anchor {
                at: figure_at(track, time_s)?.at,
                marks: 0,
            })
        }
        Target::Mark { character, index } => {
            let track = plan.figures.iter().find(|f| f.character == character)?;
            let mark = track.marks.get(index)?;
            Some(Anchor {
                at: mark.at,
                marks: track.marks.len(),
            })
        }
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

/// The bound notes of the scene's set and the plan's labels, with where they and their
/// elements are.
pub(super) fn held(project: &Project, at: (usize, usize), time_s: f64) -> Vec<Held> {
    let Some((set, plan)) = parts(project, at) else {
        return Vec::new();
    };

    set.notes
        .iter()
        .chain(&plan.labels)
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
    let Project {
        locations, scenes, ..
    } = project;
    let Some(scene) = scenes.get_mut(at.0) else {
        return;
    };
    let Some(set) = locations
        .iter_mut()
        .find(|l| l.id == scene.location)
        .map(|l| &mut l.set)
    else {
        return;
    };
    let Some(plan) = scene.plans.get_mut(at.1) else {
        return;
    };

    for (h, now) in held.iter().zip(after) {
        let notes = set.notes.iter_mut().chain(plan.labels.iter_mut());
        let Some(note) = { notes }.find(|n| n.id == h.note) else {
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

/// A copy of the note `id` set down by `by_m`, free. A label's copy is a label of the plan, a
/// note's a note of the set.
pub(super) fn copy_note(
    scope: &mut Scope<'_>,
    id: &str,
    by_m: (f64, f64),
) -> Result<(), ServiceError> {
    let mut note = note_mut(scope, id).ok_or(ServiceError::Gone)?.clone();
    let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
    let in_plan = scope.plan.labels.iter().any(|n| n.id == id);

    note.at = Point::new(note.at.x_m + by_m.0, note.at.y_m + by_m.1);
    note.bound = None;
    match in_plan {
        true => {
            note.id = free_id("t", scope.plan.labels.iter().map(|n| n.id.as_str()));
            scope.plan.labels.push(note);
        }
        false => {
            note.id = free_id("n", set.notes.iter().map(|n| n.id.as_str()));
            set.notes.push(note);
        }
    }

    Ok(())
}

/// The note `id`: one of the set's, else one of the plan's labels.
pub(super) fn note_mut<'a>(scope: &'a mut Scope<'_>, id: &str) -> Option<&'a mut Note> {
    let in_set = scope
        .set
        .as_deref()
        .is_some_and(|s| s.notes.iter().any(|n| n.id == id));

    match in_set {
        true => scope
            .set
            .as_deref_mut()?
            .notes
            .iter_mut()
            .find(|n| n.id == id),
        false => scope.plan.labels.iter_mut().find(|n| n.id == id),
    }
}

/// Takes the note `id` away: one of the set's, else one of the plan's labels.
pub(super) fn remove_note(scope: &mut Scope<'_>, id: &str) -> Result<(), ServiceError> {
    if let Some(set) = scope.set.as_deref_mut() {
        if let Some(index) = set.notes.iter().position(|n| n.id == id) {
            set.notes.remove(index);
            return Ok(());
        }
    }

    let index = scope.plan.labels.iter().position(|n| n.id == id);

    scope.plan.labels.remove(index.ok_or(ServiceError::Gone)?);

    Ok(())
}

fn bad_source() -> ServiceError {
    ServiceError::BadValue {
        field: Field::Source,
        value: String::new(),
    }
}

/// A note with `text` beside the element `source`, bound to it: right of and above an
/// object's box, off a figure's or mark's place. An object's goes to the set's notes, a
/// figure's or mark's to the plan's labels.
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
    let note = |id: String| Note {
        id,
        at: Point::new(
            at.x_m + reach_m + LABEL_GAP_M,
            at.y_m - reach_m - LABEL_GAP_M,
        ),
        text,
        width_m: NOTE_WIDTH_M,
        bound: Some(key),
    };

    match source {
        Source::Object(_) => {
            let id = free_id("n", set.notes.iter().map(|n| n.id.as_str()));
            set.notes.push(note(id));
        }
        _ => {
            let id = free_id("t", scope.plan.labels.iter().map(|n| n.id.as_str()));
            scope.plan.labels.push(note(id));
        }
    }

    Ok(())
}

/// Lets the note `source` names go.
pub(super) fn unbind(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let Source::Note(id) = source else {
        return Err(bad_source());
    };

    note_mut(scope, id).ok_or(ServiceError::Gone)?.bound = None;

    Ok(())
}
