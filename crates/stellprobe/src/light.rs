//! The lamps of a plan: their symbols and beams on the plan, and what a printed legend says
//! about them (LH-50, LH-51).

use klappe_model::{FloorPlan, LightKind};

use crate::display::{Item, Role, Shape, Source, BODY_RADIUS_M};
use crate::optics::frustum;

/// How far a lamp's beam is drawn.
const BEAM_REACH_M: f64 = 4.0;

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
        });
        items.push(Item {
            role: Role::Light,
            shape: Shape::Circle {
                center: light.at,
                radius_m: BODY_RADIUS_M,
            },
            label: Some(light.label.clone()),
            source,
        });
    }
}
