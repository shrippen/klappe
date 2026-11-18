//! Sunlight on the plan (LH-52): the patches it casts on the floor through the set's windows,
//! the arrow of its direction and the arrow of north.

use klappe_model::{OpeningKind, Point, SetPlan, Site, SunTime};

use crate::display::{DisplayList, Item, Role, Shape};
use crate::height::{opening_span, wall_top};
use crate::sun::{sun_at, Sun};

/// Below this the sun's patches would run past any set: a window of 1.2 m height throws a
/// patch 34 m long at 2°.
const LOW_SUN_DEG: f64 = 2.0;
/// A wall this close to the beam's direction (sine of the angle between them) lets a sliver of
/// light through, not a patch.
const GRAZING_SINE: f64 = 0.05;
/// The arrows stand one above the other in the top right corner of the set, this far from its
/// edges and from each other.
const ARROW_LENGTH_M: f64 = 1.2;
const ARROW_MARGIN_M: f64 = 1.0;
const SUN_ARROW_STEP_M: f64 = 1.6;
const SHAFT_HALF_M: f64 = 0.04;
const HEAD_LENGTH_M: f64 = 0.3;
const HEAD_HALF_M: f64 = 0.15;
const NORTH_LABEL: &str = "N";

/// Adds the sun at `time` over `site` to `list` of `set`: patches under the walls and
/// everything else, both arrows on top of the set. Before the first item that is no background
/// or picture, so the floor is lit below what stands on it.
pub fn add_sun(list: &mut DisplayList, set: &SetPlan, site: &Site, time: &SunTime) {
    let sun = sun_at(site, time);
    let mut items = if sun.is_up() {
        patch_items(set, site, &sun)
    } else {
        Vec::new()
    };
    items.extend(arrow_items(set, site, &sun));

    let first = list
        .items
        .iter()
        .position(|i| !matches!(i.role, Role::Background | Role::Picture))
        .unwrap_or(list.items.len());
    list.items.splice(first..first, items);
}

/// The window's opening projected along the sun's rays onto the floor: a parallelogram from the
/// shadow of its sill to the shadow of its head.
///
/// ```text
///  sun  \          window        floor
///        \   head ─┐
///         \        │ opening
///          \ sill ─┘
///  wall ────────────────────────────────►
///                  [ near ][ patch ]       near = sill / tan(elevation)
///                                          patch = (head - sill) / tan(elevation)
/// ```
fn patch_items(set: &SetPlan, site: &Site, sun: &Sun) -> Vec<Item> {
    if sun.elevation_deg < LOW_SUN_DEG {
        return Vec::new();
    }

    let travel = sun.toward_on_plan(site);
    let travel = Point::new(-travel.x_m, -travel.y_m);
    let run_per_height = 1.0 / sun.elevation_deg.to_radians().tan();
    let mut items = Vec::new();

    for wall in &set.walls {
        let length_m = wall.length_m();
        if length_m <= 0.0 {
            continue;
        }

        let along = (
            (wall.to.x_m - wall.from.x_m) / length_m,
            (wall.to.y_m - wall.from.y_m) / length_m,
        );
        let sine = (travel.x_m * along.1 - travel.y_m * along.0).abs();
        if sine < GRAZING_SINE {
            continue;
        }

        for opening in wall
            .openings
            .iter()
            .filter(|o| o.kind == OpeningKind::Window)
        {
            let (sill_m, head_m) = opening_span(opening);
            let head_m = head_m.min(wall_top(set, wall));
            let (from_m, to_m) = opening.span_m();
            let shift = |at_m: f64, height_m: f64| {
                let at = wall.point_at(at_m);
                let run_m = height_m * run_per_height;

                Point::new(at.x_m + travel.x_m * run_m, at.y_m + travel.y_m * run_m)
            };

            items.push(Item {
                role: Role::SunPatch,
                shape: Shape::Polygon {
                    points: vec![
                        shift(from_m, sill_m),
                        shift(to_m, sill_m),
                        shift(to_m, head_m),
                        shift(from_m, head_m),
                    ],
                },
                label: None,
                source: None,
            });
        }
    }

    items
}

/// North, always; the sun's direction of travel, while it is up.
fn arrow_items(set: &SetPlan, site: &Site, sun: &Sun) -> Vec<Item> {
    let anchor = |step: f64| {
        let x_m = (set.width_m - ARROW_MARGIN_M).max(ARROW_MARGIN_M);

        Point::new(x_m, ARROW_MARGIN_M + step)
    };
    let north = Sun {
        azimuth_deg: 0.0,
        elevation_deg: 0.0,
    }
    .toward_on_plan(site);
    let mut items = vec![Item {
        role: Role::North,
        shape: Shape::Polygon {
            points: arrow(anchor(0.0), north),
        },
        label: Some(NORTH_LABEL.to_owned()),
        source: None,
    }];

    if sun.is_up() {
        let toward = sun.toward_on_plan(site);
        items.push(Item {
            role: Role::SunRay,
            shape: Shape::Polygon {
                points: arrow(
                    anchor(SUN_ARROW_STEP_M),
                    Point::new(-toward.x_m, -toward.y_m),
                ),
            },
            label: Some(format!(
                "{:.0}° / {:.0}°",
                sun.azimuth_deg, sun.elevation_deg
            )),
            source: None,
        });
    }

    items
}

/// An arrow centred on `centre` that points along the unit vector `dir`: the shaft, then the
/// head.
fn arrow(centre: Point, dir: Point) -> Vec<Point> {
    let side = Point::new(-dir.y_m, dir.x_m);
    let at = |along_m: f64, across_m: f64| {
        Point::new(
            centre.x_m + dir.x_m * along_m + side.x_m * across_m,
            centre.y_m + dir.y_m * along_m + side.y_m * across_m,
        )
    };
    let (tail_m, tip_m) = (-ARROW_LENGTH_M / 2.0, ARROW_LENGTH_M / 2.0);
    let neck_m = tip_m - HEAD_LENGTH_M;

    vec![
        at(tail_m, SHAFT_HALF_M),
        at(neck_m, SHAFT_HALF_M),
        at(neck_m, HEAD_HALF_M),
        at(tip_m, 0.0),
        at(neck_m, -HEAD_HALF_M),
        at(neck_m, -SHAFT_HALF_M),
        at(tail_m, -SHAFT_HALF_M),
    ]
}

#[cfg(test)]
mod tests {
    use klappe_model::{Opening, Wall};

    use super::*;

    const SITE: Site = Site {
        latitude_deg: 52.52,
        longitude_deg: 13.405,
        north_deg: 0.0,
    };

    fn set_with(wall: Wall) -> SetPlan {
        SetPlan {
            width_m: 10.0,
            depth_m: 8.0,
            walls: vec![wall],
            ..SetPlan::default()
        }
    }

    /// A wall with a window of the kind's usual sill (0.9 m) and head (2.1 m) in its middle.
    fn walled(from: (f64, f64), to: (f64, f64), kind: OpeningKind) -> SetPlan {
        let mut wall = Wall::new(Point::new(from.0, from.1), Point::new(to.0, to.1), 0.2);
        wall.openings
            .push(Opening::new(kind, wall.length_m() / 2.0, 1.2));

        set_with(wall)
    }

    fn polygons(items: &[Item], role: Role) -> Vec<Vec<Point>> {
        items
            .iter()
            .filter(|i| i.role == role)
            .filter_map(|i| match &i.shape {
                Shape::Polygon { points } => Some(points.clone()),
                _ => None,
            })
            .collect()
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// Sun due south at 30°, a window in the south wall (y = 8) of a plan with north up: the
    /// light travels up the plan. The patch starts sill / tan(30°) = 1.559 m from the wall
    /// and ends head / tan(30°) = 3.637 m from it, as wide as the window.
    #[test]
    fn the_sun_from_the_south_throws_a_patch_into_the_room() {
        let set = walled((0.0, 8.0), (10.0, 8.0), OpeningKind::Window);
        let sun = Sun {
            azimuth_deg: 180.0,
            elevation_deg: 30.0,
        };

        let items = patch_items(&set, &SITE, &sun);
        let patches = polygons(&items, Role::SunPatch);

        assert_eq!(patches.len(), 1);
        let ys: Vec<f64> = patches[0].iter().map(|p| p.y_m).collect();
        let xs: Vec<f64> = patches[0].iter().map(|p| p.x_m).collect();
        let tan = 30f64.to_radians().tan();

        assert!(near(ys[0], 8.0 - 0.9 / tan) && near(ys[1], ys[0]), "{ys:?}");
        assert!(near(ys[2], 8.0 - 2.1 / tan) && near(ys[3], ys[2]), "{ys:?}");
        assert!(near(xs[0], 4.4) && near(xs[1], 5.6), "{xs:?}");
        assert!(near(ys[0] - ys[2], (2.1 - 0.9) / tan));
    }

    /// The plan's north to the right: the sun in the south is on the left, so a window in the left
    /// wall lights the floor to its right.
    #[test]
    fn the_plans_north_turns_the_sun() {
        let set = walled((0.0, 0.0), (0.0, 8.0), OpeningKind::Window);
        let site = Site {
            north_deg: 90.0,
            ..SITE
        };
        let sun = Sun {
            azimuth_deg: 180.0,
            elevation_deg: 45.0,
        };

        let patches = polygons(&patch_items(&set, &site, &sun), Role::SunPatch);

        let xs: Vec<f64> = patches[0].iter().map(|p| p.x_m).collect();
        // At 45° the shadow of a height is as long as the height.
        assert!(near(xs[0], 0.9) && near(xs[2], 2.1), "{xs:?}");
    }

    #[test]
    fn a_door_a_low_sun_and_a_grazing_sun_cast_nothing() {
        let south = Sun {
            azimuth_deg: 180.0,
            elevation_deg: 30.0,
        };
        let door = walled((0.0, 8.0), (10.0, 8.0), OpeningKind::Door);
        assert!(patch_items(&door, &SITE, &south).is_empty());

        let window = walled((0.0, 8.0), (10.0, 8.0), OpeningKind::Window);
        let low = Sun {
            elevation_deg: 1.0,
            ..south
        };
        assert!(patch_items(&window, &SITE, &low).is_empty());

        // Sun in the east, wall running east-west: the light slides along the wall.
        let along = Sun {
            azimuth_deg: 90.0,
            elevation_deg: 30.0,
        };
        assert!(patch_items(&window, &SITE, &along).is_empty());
    }

    /// Noon of the June solstice in Berlin: the sun is up and south, with its patch in the
    /// list under the walls and the wall's own items; at midnight only north is shown.
    #[test]
    fn the_list_gets_the_patch_under_the_set_and_the_arrows_by_day_only() {
        let set = walled((0.0, 8.0), (10.0, 8.0), OpeningKind::Window);
        let wall_item = Item {
            role: Role::Wall,
            shape: Shape::Polyline { points: vec![] },
            label: None,
            source: None,
        };
        let background = Item {
            role: Role::Background,
            ..wall_item.clone()
        };
        let list = DisplayList {
            items: vec![background, wall_item],
            ..DisplayList::default()
        };
        let day = SunTime {
            year: 2026,
            month: 6,
            day: 21,
            hour: 13,
            minute: 8,
            utc_offset_min: Some(120),
        };

        let mut by_day = list.clone();
        add_sun(&mut by_day, &set, &SITE, &day);
        let roles: Vec<Role> = by_day.items.iter().map(|i| i.role).collect();
        assert_eq!(
            roles,
            [
                Role::Background,
                Role::SunPatch,
                Role::North,
                Role::SunRay,
                Role::Wall
            ]
        );

        let night = SunTime { hour: 1, ..day };
        let mut by_night = list;
        add_sun(&mut by_night, &set, &SITE, &night);
        let roles: Vec<Role> = by_night.items.iter().map(|i| i.role).collect();
        assert_eq!(roles, [Role::Background, Role::North, Role::Wall]);
    }

    /// North up is an arrow whose tip is the topmost point; north to the right, the rightmost.
    #[test]
    fn the_north_arrow_points_to_north() {
        let sun = Sun {
            azimuth_deg: 180.0,
            elevation_deg: -10.0,
        };
        let set = set_with(Wall::new(Point::new(0.0, 0.0), Point::new(1.0, 0.0), 0.2));

        let up = polygons(&arrow_items(&set, &SITE, &sun), Role::North);
        let tip = up[0].iter().min_by(|a, b| a.y_m.total_cmp(&b.y_m)).unwrap();
        assert!(
            near(tip.x_m, 9.0) && near(tip.y_m, 1.0 - ARROW_LENGTH_M / 2.0),
            "{tip:?}"
        );

        let right_site = Site {
            north_deg: 90.0,
            ..SITE
        };
        let right = polygons(&arrow_items(&set, &right_site, &sun), Role::North);
        let tip = right[0]
            .iter()
            .max_by(|a, b| a.x_m.total_cmp(&b.x_m))
            .unwrap();
        assert!(near(tip.x_m, 9.0 + ARROW_LENGTH_M / 2.0), "{tip:?}");
    }

    /// A set narrower than two arrows side by side keeps them apart.
    #[test]
    fn the_arrows_on_a_narrow_set_do_not_overlap() {
        let sun = Sun {
            azimuth_deg: 180.0,
            elevation_deg: 30.0,
        };
        let set = SetPlan {
            width_m: 2.0,
            depth_m: 8.0,
            ..SetPlan::default()
        };
        let items = arrow_items(&set, &SITE, &sun);
        let centre = |role| {
            let points = &polygons(&items, role)[0];
            let n = points.len() as f64;

            Point::new(
                points.iter().map(|p| p.x_m).sum::<f64>() / n,
                points.iter().map(|p| p.y_m).sum::<f64>() / n,
            )
        };
        let (north, ray) = (centre(Role::North), centre(Role::SunRay));
        let apart = (north.x_m - ray.x_m).hypot(north.y_m - ray.y_m);

        assert!(apart >= ARROW_LENGTH_M, "{apart}");
    }
}
