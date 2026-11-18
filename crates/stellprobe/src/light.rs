//! The lamps of a plan: their symbols and beams on the plan, and what a printed legend says
//! about them (LH-50, LH-51).

use klappe_model::{FloorPlan, Light, LightKind, Point};

use crate::display::{Item, LightGrip, Role, Shape, Source, BODY_RADIUS_M};
use crate::optics::frustum;

/// How far a lamp's beam is drawn.
const BEAM_REACH_M: f64 = 4.0;
/// How far out along the beam's edge its grip sits, as a share of the reach: closer than the
/// direction grip, so a narrow beam does not put one grip on the other (2 m apart at least).
const BEAM_GRIP_SHARE: f64 = 0.5;

/// Where grip `grip` of `light` sits: the direction grip `BEAM_REACH_M` out on its facing,
/// the beam grip halfway out on the beam's edge clockwise of it.
///
/// ```text
///             /
///      beam ·
///          /
///   lamp ·--------- direction grip
///         \
/// ```
pub fn light_grip_place(light: &Light, grip: LightGrip) -> Point {
    let (angle_deg, reach_m) = match grip {
        LightGrip::Direction => (light.facing_deg, BEAM_REACH_M),
        LightGrip::Beam => (
            light.facing_deg + light.beam_deg / 2.0,
            BEAM_REACH_M * BEAM_GRIP_SHARE,
        ),
    };
    let (sin, cos) = angle_deg.to_radians().sin_cos();

    Point::new(light.at.x_m + reach_m * cos, light.at.y_m + reach_m * sin)
}

/// A lamp as a legend names it: the label the plan shows, and what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Lamp {
    /// Id of the lamp, to match it with its items.
    pub id: String,
    pub label: String,
    pub kind: LightKind,
    pub power_w: f64,
    pub color_temp_k: f64,
    pub beam_deg: f64,
}

pub(crate) fn lamps(plan: &FloorPlan) -> Vec<Lamp> {
    plan.lights
        .iter()
        .map(|l| Lamp {
            id: l.id.clone(),
            label: l.label.clone(),
            kind: l.kind,
            power_w: l.power_w,
            color_temp_k: l.color_temp_k,
            beam_deg: l.beam_deg,
        })
        .collect()
}

pub(crate) fn light_items(plan: &FloorPlan, items: &mut Vec<Item>) {
    for light in &plan.lights {
        let source = Some(Source::Light(light.id.clone()));
        let beam = frustum(light.at, light.facing_deg, light.beam_deg, BEAM_REACH_M);

        items.push(Item {
            role: Role::LightBeam,
            shape: Shape::Polygon {
                points: beam.to_vec(),
            },
            label: None,
            source: source.clone(),
            opacity: crate::display::FULL_OPACITY,
        });
        items.push(Item {
            role: Role::Light,
            shape: Shape::Circle {
                center: light.at,
                radius_m: BODY_RADIUS_M,
            },
            label: Some(light.label.clone()),
            source,
            opacity: crate::display::FULL_OPACITY,
        });
    }
}
