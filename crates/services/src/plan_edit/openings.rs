//! Doors and windows in walls (LH-10): set, slide along the wall, resize.

use klappe_model::{DoorSide, Hinge, Opening, OpeningKind, Point, Wall};
use klappe_stellprobe::{OpeningEnd, Source};

use super::{PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// How far from a wall's edge a click still sets an opening in it.
const OPENING_REACH_M: f64 = 0.3;
/// The narrowest an opening gets when an end is dragged.
const OPENING_MIN_WIDTH_M: f64 = 0.3;

fn refused(field: Field, value: String) -> ServiceError {
    ServiceError::BadValue { field, value }
}

/// Where an opening `width_m` wide, wanted centred `at_m` along `wall`, goes: pushed back
/// inside the wall, and refused when it does not fit or meets another opening. `skip` is the
/// opening that is being moved or resized itself.
fn centre_m(
    wall: &Wall,
    skip: Option<usize>,
    at_m: f64,
    width_m: f64,
) -> Result<f64, ServiceError> {
    let room_m = wall.length_m() - width_m;
    if room_m < 0.0 {
        return Err(refused(Field::Width, width_m.to_string()));
    }

    let centre_m = at_m.clamp(width_m / 2.0, width_m / 2.0 + room_m);
    let (start_m, end_m) = (centre_m - width_m / 2.0, centre_m + width_m / 2.0);

    let meets = wall
        .openings
        .iter()
        .enumerate()
        .filter(|(i, _)| Some(*i) != skip)
        .map(|(_, o)| o.span_m())
        .any(|(s, e)| start_m < e && s < end_m);
    if meets {
        return Err(refused(Field::Wall, String::new()));
    }

    Ok(centre_m)
}

/// Applies `AddOpening`, `SetOpeningWidth` or `SetOpeningSwing`; any other edit is none of its business.
pub(super) fn edit_opening(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddOpening { kind, at } => add_opening(scope, *kind, *at),
        PlanEdit::SetOpeningWidth { source, width_m } => set_width(scope, source, *width_m),
        PlanEdit::SetOpeningSwing {
            source,
            side,
            hinge,
        } => set_swing(scope, source, *side, *hinge),
        _ => Ok(()),
    }
}

/// Sets a door or window, as wide as its kind says, in the wall nearest `at`.
fn add_opening(scope: &mut Scope<'_>, kind: OpeningKind, at: Point) -> Result<(), ServiceError> {
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;

    let nearest = set
        .walls
        .iter_mut()
        .map(|wall| {
            let (along_m, off_m) = wall.foot(at);
            (wall, along_m, off_m)
        })
        .filter(|(wall, _, off_m)| *off_m <= wall.thickness_m / 2.0 + OPENING_REACH_M)
        .min_by(|a, b| a.2.total_cmp(&b.2));
    let Some((wall, along_m, _)) = nearest else {
        return Err(refused(Field::Wall, String::new()));
    };

    let width_m = kind.default_width_m();
    let at_m = centre_m(wall, None, along_m, width_m)?;
    wall.openings.push(Opening::new(kind, at_m, width_m));

    Ok(())
}

/// The opening `source` names, or the one its grip is on, with its wall.
pub(super) fn opening_mut<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<(&'a mut Wall, usize), ServiceError> {
    let (Source::Opening { wall, index } | Source::OpeningEdge { wall, index, .. }) = source else {
        return Err(refused(Field::Source, String::new()));
    };
    let wall = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.walls.get_mut(*wall))
        .ok_or(ServiceError::Gone)?;
    if *index >= wall.openings.len() {
        return Err(ServiceError::Gone);
    }

    Ok((wall, *index))
}

/// Slides the opening along its wall to where `to` is nearest the wall.
pub(super) fn slide(wall: &mut Wall, index: usize, to: Point) -> Result<(), ServiceError> {
    let (along_m, _) = wall.foot(to);
    let width_m = wall.openings[index].width_m;

    wall.openings[index].at_m = centre_m(wall, Some(index), along_m, width_m)?;

    Ok(())
}

/// Where `end` of the opening is, in metres from the wall's `from`.
pub(super) fn edge_m(opening: &Opening, end: OpeningEnd) -> f64 {
    let (start_m, end_m) = opening.span_m();

    match end {
        OpeningEnd::Start => start_m,
        OpeningEnd::End => end_m,
    }
}

/// Moves one end of the opening to where `to` is along the wall; the other end stays. The
/// opening keeps its minimum width, and `to` must not lie beyond the wall.
pub(super) fn drag_edge(
    wall: &mut Wall,
    index: usize,
    end: OpeningEnd,
    to: Point,
) -> Result<(), ServiceError> {
    let length_m = wall.length_m();
    let (dx, dy) = (wall.to.x_m - wall.from.x_m, wall.to.y_m - wall.from.y_m);
    let along_m = match length_m == 0.0 {
        true => 0.0,
        false => ((to.x_m - wall.from.x_m) * dx + (to.y_m - wall.from.y_m) * dy) / length_m,
    };
    if !(0.0..=length_m).contains(&along_m) {
        return Err(refused(Field::Wall, String::new()));
    }

    let (start_m, end_m) = wall.openings[index].span_m();
    let (start_m, end_m) = match end {
        OpeningEnd::Start => (along_m.min(end_m - OPENING_MIN_WIDTH_M), end_m),
        OpeningEnd::End => (start_m, along_m.max(start_m + OPENING_MIN_WIDTH_M)),
    };
    // The minimum must not push an end past the wall: `centre_m` would move the other end.
    if start_m < 0.0 || end_m > length_m {
        return Err(refused(Field::Wall, String::new()));
    }
    let width_m = end_m - start_m;

    wall.openings[index].at_m = centre_m(wall, Some(index), start_m + width_m / 2.0, width_m)?;
    wall.openings[index].width_m = width_m;

    Ok(())
}

/// Gives the opening another width about its centre.
fn set_width(scope: &mut Scope<'_>, source: &Source, width_m: f64) -> Result<(), ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;
    let at_m = wall.openings[index].at_m;

    wall.openings[index].at_m = centre_m(wall, Some(index), at_m, width_m)?;
    wall.openings[index].width_m = width_m;

    Ok(())
}

/// Sets the side and hinge of a door; what is `None` stays.
fn set_swing(
    scope: &mut Scope<'_>,
    source: &Source,
    side: Option<DoorSide>,
    hinge: Option<Hinge>,
) -> Result<(), ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;
    let opening = &mut wall.openings[index];
    if opening.kind != OpeningKind::Door {
        return Err(refused(Field::Source, String::new()));
    }

    opening.side = side.unwrap_or(opening.side);
    opening.hinge = hinge.unwrap_or(opening.hinge);

    Ok(())
}
