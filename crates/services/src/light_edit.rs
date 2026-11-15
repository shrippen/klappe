//! Edits of a lamp: kind, beam, power, colour temperature (LH-50) and the library lamps it
//! starts from (LH-51).

use klappe_model::{FloorPlan, Light, LightKind, LightPreset};

use crate::error::ServiceError;
use crate::field::Field;

/// Widest beam is the full circle.
const MAX_BEAM_DEG: f64 = 360.0;
/// Strongest lamp: more is a typing error.
const MAX_POWER_W: f64 = 100_000.0;
/// Colour temperatures of candle to blue sky.
const MIN_TEMP_K: f64 = 1000.0;
const MAX_TEMP_K: f64 = 20_000.0;

/// Colour of a tungsten kind and of a daylight kind, for a kind the library lacks.
const TUNGSTEN_K: f64 = 3200.0;
const DAYLIGHT_K: f64 = 5600.0;

/// What a lamp's card can change.
#[derive(Debug, Clone, PartialEq)]
pub enum LightChange {
    /// Only the kind; beam and power stay. A kind with a fixed colour brings it.
    Kind(LightKind),
    /// Kind, beam, power and colour of the library lamp with this id.
    Preset(String),
    Beam(f64),
    Power(f64),
    Temp(f64),
}

/// The built-in lamp `id`.
pub(crate) fn preset(id: &str) -> Result<&'static LightPreset, ServiceError> {
    klappe_store::builtin_lights()
        .iter()
        .find(|l| l.id == id)
        .ok_or_else(|| ServiceError::BadValue {
            field: Field::Kind,
            value: id.to_owned(),
        })
}

/// The colour a kind of fixed colour has: its first library lamp's, else the kind's own.
fn fixed_temp_k(kind: LightKind) -> f64 {
    let library = klappe_store::builtin_lights().iter().find(|l| l.kind == kind);
    if let Some(preset) = library {
        return preset.color_temp_k;
    }

    match kind {
        LightKind::Fresnel | LightKind::Balloon => TUNGSTEN_K,
        _ => DAYLIGHT_K,
    }
}

fn bad(field: Field, value: f64) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: value.to_string(),
    }
}

fn check(change: &LightChange) -> Result<(), ServiceError> {
    match change {
        LightChange::Kind(_) | LightChange::Preset(_) => Ok(()),
        LightChange::Beam(v) if *v > 0.0 && *v <= MAX_BEAM_DEG => Ok(()),
        LightChange::Beam(v) => Err(bad(Field::Beam, *v)),
        LightChange::Power(v) if (0.0..=MAX_POWER_W).contains(v) => Ok(()),
        LightChange::Power(v) => Err(bad(Field::Power, *v)),
        LightChange::Temp(v) if (MIN_TEMP_K..=MAX_TEMP_K).contains(v) => Ok(()),
        LightChange::Temp(v) => Err(bad(Field::Temperature, *v)),
    }
}

fn light<'a>(plan: &'a mut FloorPlan, id: &str) -> Result<&'a mut Light, ServiceError> {
    plan.lights
        .iter_mut()
        .find(|l| l.id == id)
        .ok_or(ServiceError::Gone)
}

pub(crate) fn set_light(
    plan: &mut FloorPlan,
    id: &str,
    change: &LightChange,
) -> Result<(), ServiceError> {
    check(change)?;

    let preset = match change {
        LightChange::Preset(preset_id) => Some(preset(preset_id)?),
        _ => None,
    };
    let light = light(plan, id)?;

    match (change, preset) {
        (LightChange::Kind(kind), _) => {
            light.kind = *kind;
            if !kind.tunable_color() {
                light.color_temp_k = fixed_temp_k(*kind);
            }
        }
        (_, Some(preset)) => {
            light.kind = preset.kind;
            light.beam_deg = preset.beam_deg;
            light.power_w = preset.power_w;
            light.color_temp_k = preset.color_temp_k;
        }
        (LightChange::Beam(deg), _) => light.beam_deg = *deg,
        (LightChange::Power(watt), _) => light.power_w = *watt,
        // A tungsten lamp is what it is; only LEDs are set to another colour.
        (LightChange::Temp(kelvin), _) if light.kind.tunable_color() => {
            light.color_temp_k = *kelvin
        }
        (LightChange::Temp(kelvin), _) => return Err(bad(Field::Temperature, *kelvin)),
        (LightChange::Preset(_), None) => {}
    }

    Ok(())
}
