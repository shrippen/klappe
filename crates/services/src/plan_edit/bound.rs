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

/// What a bound note is bound to. Figures and marks are of one plan (a variant has its own
/// tracks, the notes are the set's); an edit in another plan leaves such a note alone.
enum Target<'a> {
    Object(&'a str),
    Figure {
        scene: &'a str,
        plan: &'a str,
        character: &'a str,
    },
    Mark {
        scene: &'a str,
        plan: &'a str,
        character: &'a str,
        index: usize,
    },
}

impl<'a> Target<'a> {
    /// Reads `object:<id>`, `figure:<scene>:<plan>:<character>` or
    /// `mark:<scene>:<plan>:<character>:<index>`; ids have no `:`.
    fn parse(key: &'a str) -> Option<Self> {
        let (kind, rest) = key.split_once(':')?;

        match kind {
            "object" => Some(Self::Object(rest)),
            "figure" => {
                let (scene, rest) = rest.split_once(':')?;
                let (plan, character) = rest.split_once(':')?;
                Some(Self::Figure {
                    scene,
                    plan,
                    character,
                })
            }
            "mark" => {
                let (scene, rest) = rest.split_once(':')?;
                let (plan, rest) = rest.split_once(':')?;
                let (character, index) = rest.rsplit_once(':')?;
                let index = index.parse().ok()?;
                Some(Self::Mark {
                    scene,
                    plan,
                    character,
                    index,
                })
            }
            _ => None,
        }
    }

    /// The scene and plan the target belongs to; `None` for an object, which is the set's.
    fn owner(&self) -> Option<(&str, &str)> {
        match self {
            Self::Object(_) => None,
            Self::Figure { scene, plan, .. } | Self::Mark { scene, plan, .. } => {
                Some((scene, plan))
            }
        }
    }
}

/// The key a note keeps for `source` on the plan `plan` of the scene `scene`; `None` for what
/// cannot be labelled.
fn key_of(source: &Source, scene: &str, plan: &str) -> Option<String> {
    match source {
        Source::Object(id) => Some(format!("object:{id}")),
        Source::Figure(character) => Some(format!("figure:{scene}:{plan}:{character}")),
        Source::Mark { character, index } => {
            Some(format!("mark:{scene}:{plan}:{character}:{index}"))
        }
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
        Target::Figure { character, .. } => {
            let track = plan.figures.iter().find(|f| f.character == character)?;
            Some(Anchor {
                at: figure_at(track, time_s)?.at,
                marks: 0,
            })
        }
        Target::Mark {
            character, index, ..
        } => {
            let track = plan.figures.iter().find(|f| f.character == character)?;
            let mark = track.marks.get(index)?;
            Some(Anchor {
                at: mark.at,
                marks: track.marks.len(),
            })
        }
    }
}

/// The set, plan and scene id of the scene on view.
type Parts<'a> = (&'a SetPlan, &'a FloorPlan, &'a str);

fn parts(project: &Project, (scene, plan): (usize, usize)) -> Option<Parts<'_>> {
    let scene = project.scenes.get(scene)?;
    let set = &project
        .locations
        .iter()
        .find(|l| l.id == scene.location)?
        .set;

    Some((set, scene.plans.get(plan)?, &scene.id))
}

/// The bound notes of the scene's set, with where they and their elements are.
pub(super) fn held(project: &Project, at: (usize, usize), time_s: f64) -> Vec<Held> {
    let Some((set, plan, scene)) = parts(project, at) else {
        return Vec::new();
    };

    set.notes
        .iter()
        .filter_map(|n| {
            let key = n.bound.clone()?;
            // A figure or mark of another plan is none of this edit's business.
            if Target::parse(&key)?
                .owner()
                .is_some_and(|o| o != (scene, plan.id.as_str()))
            {
                return None;
            }

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
    let Some((set, plan, _)) = parts(project, at) else {
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
    let key = key_of(source, scope.scene, &scope.plan.id).ok_or_else(bad_source)?;
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

/// Lets go of the labels of the figures and marks of plan `plan` of scene `scene` (it is
/// deleted).
pub(crate) fn unbind_plan(project: &mut Project, scene: usize, plan: &str) {
    let Some(scene) = project.scenes.get(scene) else {
        return;
    };
    let (scene, location) = (scene.id.clone(), scene.location.clone());
    let Some(set) = project
        .locations
        .iter_mut()
        .find(|l| l.id == location)
        .map(|l| &mut l.set)
    else {
        return;
    };

    for note in &mut set.notes {
        let gone = note.bound.as_deref().and_then(Target::parse);
        if gone.is_some_and(|t| t.owner() == Some((scene.as_str(), plan))) {
            note.bound = None;
        }
    }
}
