//! The marks of set objects that figures move (LH-24): add, remove, carry.

use klappe_model::{
    normalize_deg, FloorPlan, ObjectMark, ObjectTrack, Point, Project, SetObject, SetPlan,
};
use klappe_stellprobe::{object_at, Placement, Source};

use super::free_id;
use super::marks::WALK_SPEED_M_S;
use super::{bad_mark, current_index, PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// The edits of an object's way: a new mark, the carrier of a mark.
pub(super) fn edit(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddObjectMark { object, at } => add_object_mark(scope, object, *at),
        PlanEdit::SetCarrier { source, figure } => set_carrier(scope, source, figure.as_deref()),
        _ => Ok(()),
    }
}

/// The mark of an object's track that `source` names: the mark itself, or the object for its
/// mark at the playhead. Its object and index.
pub(super) fn object_mark(scope: &Scope<'_>, source: &Source) -> Option<(String, usize)> {
    match source {
        Source::ObjectMark { object, index } => Some((object.clone(), *index)),
        Source::Object(object) => {
            let track = scope.plan.object_track(object)?;
            let index = current_index(track.marks.iter().map(|m| m.at_s), scope.time_s)?;
            Some((object.clone(), index))
        }
        _ => None,
    }
}

/// The mark `source` names, to change.
pub(super) fn object_mark_mut<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<&'a mut ObjectMark, ServiceError> {
    let (object, index) = object_mark(scope, source).ok_or_else(|| bad_mark(Field::Source, ""))?;

    scope
        .plan
        .object_tracks
        .iter_mut()
        .find(|t| t.object == object)
        .and_then(|t| t.marks.get_mut(index))
        .ok_or(ServiceError::Gone)
}

/// The next mark of the object's way at `at`. The first click gives the object a first mark
/// where it stands now and a second at `at`; every later click one after the last. The object
/// moves at walking pace after the pause, and keeps its turn.
pub(super) fn add_object_mark(
    scope: &mut Scope<'_>,
    object: &str,
    at: Point,
) -> Result<(), ServiceError> {
    let time_s = scope.time_s;
    let home = scope
        .set
        .as_deref()
        .and_then(|s| s.objects.iter().find(|o| o.id == object))
        .map(|o| (o.at, o.rotation_deg))
        .ok_or(ServiceError::Gone)?;
    let index = match scope
        .plan
        .object_tracks
        .iter()
        .position(|t| t.object == object)
    {
        Some(index) => index,
        None => {
            scope.plan.object_tracks.push(ObjectTrack {
                object: object.to_owned(),
                marks: Vec::new(),
            });
            scope.plan.object_tracks.len() - 1
        }
    };
    // After a carried span the object is where it is carried to: no jump at creation.
    let track = &scope.plan.object_tracks[index];
    let carried = track
        .marks
        .last()
        .is_some_and(|m| m.carried_by.is_some() && time_s > m.at_s)
        .then(|| object_at(scope.plan, track, time_s))
        .flatten();
    let marks = &mut scope.plan.object_tracks[index].marks;

    if marks.is_empty() {
        marks.push(ObjectMark {
            at_s: time_s,
            at: home.0,
            rotation_deg: normalize_deg(home.1),
            hold_s: 0.0,
            carried_by: None,
        });
    }

    let Some(prev) = marks.last() else {
        return Ok(());
    };
    let walked_s = prev.at_s + prev.hold_s + prev.at.distance_m(at) / WALK_SPEED_M_S;
    let (at_s, at) = match carried {
        Some(place) => (time_s, place.at),
        None => (walked_s, at),
    };
    let rotation_deg = prev.rotation_deg;

    marks.push(ObjectMark {
        at_s,
        at,
        rotation_deg,
        hold_s: 0.0,
        carried_by: None,
    });

    Ok(())
}

/// Takes mark `index` out of the way of `object`; the last one takes the whole track, the
/// object is back where the set puts it.
pub(super) fn remove_object_mark(
    scope: &mut Scope<'_>,
    object: &str,
    index: usize,
) -> Result<(), ServiceError> {
    let tracks = &mut scope.plan.object_tracks;
    let at = tracks.iter().position(|t| t.object == object);
    let track = at.map(|i| &mut tracks[i]).ok_or(ServiceError::Gone)?;
    if index >= track.marks.len() {
        return Err(ServiceError::Gone);
    }

    track.marks.remove(index);
    if track.marks.is_empty() {
        tracks.remove(at.unwrap_or_default());
    }

    Ok(())
}

/// Lets the figure `figure` carry the object from the mark `source` names until its next mark,
/// or sets it down with `None`. The figure must be one of the plan.
pub(super) fn set_carrier(
    scope: &mut Scope<'_>,
    source: &Source,
    figure: Option<&str>,
) -> Result<(), ServiceError> {
    if let Some(id) = figure {
        if scope.plan.figure(id).is_none() {
            return Err(bad_mark(Field::Character, id));
        }
    }

    object_mark_mut(scope, source)?.carried_by = figure.map(str::to_owned);

    Ok(())
}

/// The object ids a track names in any plan of the location of scene `scene`, orphans
/// included: a new object must not take one over.
pub(super) fn tracked_ids(project: &Project, scene: usize) -> Vec<String> {
    let Some(location) = project.scenes.get(scene).map(|s| &s.location) else {
        return Vec::new();
    };

    project
        .scenes
        .iter()
        .filter(|s| s.location == *location)
        .flat_map(|s| &s.plans)
        .flat_map(|p| &p.object_tracks)
        .map(|t| t.object.clone())
        .collect()
}

/// The first `o<n>` that neither the set's objects nor a track name.
pub(super) fn free_object_id(set: &SetPlan, tracked: &[String]) -> String {
    let taken = set.objects.iter().map(|o| o.id.as_str());

    free_id("o", taken.chain(tracked.iter().map(String::as_str)))
}

/// The ids of the objects in the set of scene `scene`'s location.
pub(super) fn object_ids(project: &Project, scene: usize) -> Vec<String> {
    let location = project.scenes.get(scene).map(|s| &s.location);

    project
        .locations
        .iter()
        .filter(|l| Some(&l.id) == location)
        .flat_map(|l| l.set.objects.iter().map(|o| o.id.clone()))
        .collect()
}

/// The objects of the set before an edit that may remove some, for `forget_removed`.
pub(super) fn watch_removal(
    project: &Project,
    scene: usize,
    edit: &PlanEdit,
) -> Option<Vec<String>> {
    matches!(edit, PlanEdit::Remove { .. } | PlanEdit::RemoveGroup { .. })
        .then(|| object_ids(project, scene))
}

/// Takes the tracks of the objects an edit removed from the set out of every plan of that
/// location: an object removed in one scene leaves no way behind in another. `before` is
/// `watch_removal` from before the edit; a track that named no object already stays.
pub(super) fn forget_removed(project: &mut Project, scene: usize, before: Option<&[String]>) {
    let Some(before) = before else {
        return;
    };
    let now = object_ids(project, scene);
    let Some(location) = project.scenes.get(scene).map(|s| s.location.clone()) else {
        return;
    };

    for plan in project
        .scenes
        .iter_mut()
        .filter(|s| s.location == location)
        .flat_map(|s| &mut s.plans)
    {
        plan.object_tracks
            .retain(|t| now.contains(&t.object) || !before.contains(&t.object));
    }
}

/// Where object `id` is shown at `time_s` when it has a way.
pub(super) fn shown(plan: &FloorPlan, id: &str, time_s: f64) -> Option<Placement> {
    object_at(plan, plan.object_track(id)?, time_s)
}

/// Puts `object` where it is shown (a copy of a moving object starts there), turned as shown.
pub(super) fn put_at(object: &mut SetObject, shown: Option<Placement>) {
    if let Some(place) = shown {
        object.at = place.at;
        object.rotation_deg = place.facing_deg;
    }
}

/// A figure that is gone from the plan carries nothing any more.
pub(super) fn drop_carrier(plan: &mut FloorPlan, figure: &str) {
    for track in &mut plan.object_tracks {
        for mark in &mut track.marks {
            if mark.carried_by.as_deref() == Some(figure) {
                mark.carried_by = None;
            }
        }
    }
}
