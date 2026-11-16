//! What a click, a box or a lasso on the plan takes: the topmost grabbable item, every element
//! inside. Locked layers are left out by the caller's `locked`.

use klappe_model::Point;
use klappe_stellprobe::{anchor, hit, in_lasso, within, Item, Role, Shape, Source};

/// How far beside a shape a click still takes it: a finger or a mouse is not exact.
const PICK_RADIUS_M: f64 = 0.15;

/// The topmost item the user can grab at `at`: the display list is back to front. Items for
/// which `locked` is true are left out.
pub(crate) fn pick(items: &[Item], at: Point, locked: impl Fn(Role) -> bool) -> Option<Source> {
    items
        .iter()
        .rev()
        // A caption that belongs to nothing (a height, a free form's name) does not hide
        // the body under it.
        .filter(|item| grabbable(item.role) && !locked(item.role) && item.source.is_some())
        .find(|item| hit(item, at, PICK_RADIUS_M))
        .and_then(|item| item.source.clone())
}

/// Every element the user can grab whose shown items all lie in the box with corners `a` and
/// `b` (LH-08), in display order. Items for which `locked` is true are left out.
pub(crate) fn pick_in(
    items: &[Item],
    (a, b): (Point, Point),
    locked: impl Fn(Role) -> bool,
) -> Vec<Source> {
    let mut found: Vec<(Source, bool)> = Vec::new();

    for item in items
        .iter()
        .filter(|i| grabbable(i.role) && !locked(i.role))
    {
        let Some(source) = &item.source else {
            continue;
        };
        let inside = within(item, a, b);

        // An element drawn from several items (a figure and its facing wedge) is in only
        // when all of them are.
        match found.iter_mut().find(|(s, _)| s == source) {
            Some((_, all_in)) => *all_in &= inside,
            None => found.push((source.clone(), inside)),
        }
    }

    found
        .into_iter()
        .filter_map(|(source, all_in)| all_in.then_some(source))
        .collect()
}

/// Every element the user can grab whose reference point lies in the freehand shape `lasso`
/// (LH-08), in display order. The point is the centre of the element's circle (a figure's
/// body), else of its first item. Items for which `locked` is true are left out.
pub(crate) fn pick_lasso(
    items: &[Item],
    lasso: &[Point],
    locked: impl Fn(Role) -> bool,
) -> Vec<Source> {
    let mut points: Vec<(Source, Option<Point>, bool)> = Vec::new();

    for item in items
        .iter()
        .filter(|i| grabbable(i.role) && !locked(i.role))
    {
        let Some(source) = &item.source else {
            continue;
        };
        let round = matches!(item.shape, Shape::Circle { .. });

        match points.iter_mut().find(|(s, _, _)| s == source) {
            Some((_, at, has_circle)) if round && !*has_circle => {
                *at = anchor(item);
                *has_circle = true;
            }
            Some(_) => {}
            None => points.push((source.clone(), anchor(item), round)),
        }
    }

    points
        .into_iter()
        .filter(|(_, at, _)| at.is_some_and(|at| in_lasso(lasso, at)))
        .map(|(source, _, _)| source)
        .collect()
}

/// What a click means to take: not the view cone, the axis or a path, which only show.
fn grabbable(role: Role) -> bool {
    !matches!(
        role,
        Role::Background
            | Role::Frustum
            | Role::Axis
            | Role::CameraPath { .. }
            | Role::FigurePath(_)
            | Role::LightBeam
    )
}
