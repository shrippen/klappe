//! Set objects that move through a scene (LH-24): where an object is at a time, and the set
//! with its objects put there.

use std::borrow::Cow;

use klappe_model::{FloorPlan, ObjectTrack, Point, SetPlan};

use crate::motion::{figure_at, track_at, Placement};

/// An object on its track at `t_s`; `None` without marks. From a mark that names a carrier
/// until the next mark the object keeps the offset it had to that figure at the mark's time,
/// and its turn; while the figure is not in the scene the marks give the place.
///
/// ```text
///  mark 1 (carried by Anna)           mark 2
///  |─────────── follows Anna ─────────|───── stays
/// ```
pub fn object_at(plan: &FloorPlan, track: &ObjectTrack, t_s: f64) -> Option<Placement> {
    let index = track.marks.iter().rposition(|m| m.at_s <= t_s);
    let carried = index
        .map(|i| &track.marks[i])
        .and_then(|m| Some((m, plan.figure(m.carried_by.as_deref()?)?)))
        .and_then(|(m, figure)| Some((m, figure_at(figure, m.at_s)?, figure_at(figure, t_s)?)));

    let Some((mark, start, now)) = carried else {
        return track_at(track, t_s);
    };

    Some(Placement {
        at: Point::new(
            mark.at.x_m + now.at.x_m - start.at.x_m,
            mark.at.y_m + now.at.y_m - start.at.y_m,
        ),
        facing_deg: mark.rotation_deg,
    })
}

/// `set` with every object that has a track in `plan` put where it is at `t_s`; the set
/// itself when none has.
pub fn set_at<'a>(set: &'a SetPlan, plan: &FloorPlan, t_s: f64) -> Cow<'a, SetPlan> {
    let moved = |id: &str| plan.object_track(id).and_then(|t| object_at(plan, t, t_s));
    if !set.objects.iter().any(|o| moved(&o.id).is_some()) {
        return Cow::Borrowed(set);
    }

    let mut placed = set.clone();
    for object in &mut placed.objects {
        let Some(place) = moved(&object.id) else {
            continue;
        };

        object.at = place.at;
        object.rotation_deg = place.facing_deg;
    }

    Cow::Owned(placed)
}
