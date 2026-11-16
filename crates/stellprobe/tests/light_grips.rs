//! The grips on a selected lamp's beam (LH-50).
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, Light, LightKind, Point, Project, SetPlan};
use klappe_stellprobe::{handles, light_grip_place, LightGrip, Role, Shape, Source};

/// A lamp at (1, 2) facing down (90°) with a 60° beam; the beam is drawn 4 m long.
fn lamp() -> Light {
    Light {
        id: "l".into(),
        label: "L".into(),
        kind: LightKind::LedPanel,
        at: Point::new(1.0, 2.0),
        facing_deg: 90.0,
        beam_deg: 60.0,
        color_temp_k: 5600.0,
        power_w: 100.0,
    }
}

fn grips(selected: &Source) -> Vec<(Role, Option<Source>, Point)> {
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.lights.push(lamp());

    handles(&Project::new("Test"), &SetPlan::default(), &plan, selected)
        .into_iter()
        .filter_map(|i| match i.shape {
            Shape::Circle { center, .. } => Some((i.role, i.source, center)),
            _ => None,
        })
        .collect()
}

fn source(grip: LightGrip) -> Option<Source> {
    Some(Source::LightGrip {
        light: "l".into(),
        grip,
    })
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

#[test]
fn the_grips_sit_on_the_facing_and_on_the_beam_edge() {
    // 90° + 30° = 120° for the edge, halfway out (2 m): (1 - 1, 2 + 2 * sin 120°).
    let edge = Point::new(1.0 - 1.0, 2.0 + 2.0 * 120.0_f64.to_radians().sin());

    assert!(near(
        light_grip_place(&lamp(), LightGrip::Direction),
        Point::new(1.0, 6.0)
    ));
    assert!(near(light_grip_place(&lamp(), LightGrip::Beam), edge));
}

#[test]
fn a_selected_lamp_shows_both_grips_and_one_of_them_does_too() {
    let direction = (Role::Handle, source(LightGrip::Direction));
    let beam = (Role::Handle, source(LightGrip::Beam));

    for selected in [Source::Light("l".into()), source(LightGrip::Beam).unwrap()] {
        let shown: Vec<_> = grips(&selected).into_iter().map(|g| (g.0, g.1)).collect();

        assert_eq!(shown.len(), 2);
        assert!(shown.contains(&direction) && shown.contains(&beam));
    }
}

#[test]
fn nothing_else_selected_shows_no_lamp_grip() {
    assert!(grips(&Source::Light("other".into())).is_empty());
    assert!(grips(&Source::Camera("a".into())).is_empty());
}
