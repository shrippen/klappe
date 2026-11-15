//! The lamp card: what a kind, library lamp, facing, beam, power or colour temperature typed
//! or picked for the selected lamp means as a plan edit (LH-50, LH-51). The session checks the
//! limits.

use klappe_services::{
    read_number, Field, LightChange, LightKind, PlanEdit, Range, ServiceError, Source, Unit,
};

use crate::tools::Typed;

/// A field of the lamp card, as QML names it to `setLightField`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightField {
    /// A kind's file name, e.g. `led-panel`.
    Kind,
    /// A built-in lamp, by id: sets kind, beam, power and colour together.
    Preset,
    /// Direction in degrees, clockwise from the right.
    Facing,
    /// Beam angle in degrees.
    Beam,
    /// Power in watts.
    Power,
    /// Colour temperature in kelvin.
    Temp,
}

impl LightField {
    pub fn named(name: &str) -> Option<LightField> {
        match name {
            "kind" => Some(LightField::Kind),
            "preset" => Some(LightField::Preset),
            "facing" => Some(LightField::Facing),
            "beam" => Some(LightField::Beam),
            "power" => Some(LightField::Power),
            "temp" => Some(LightField::Temp),
            _ => None,
        }
    }
}

/// The value typed or chosen for `field` of the selected lamp, as an edit. `None` when
/// something other than a lamp is selected or a kind is unknown.
pub fn light_edit(selected: Option<&Source>, field: LightField, value: &str) -> Typed {
    let Some(Source::Light(id)) = selected else {
        return None;
    };
    let light = id.clone();
    let set = |change: Result<LightChange, ServiceError>| {
        change.map(|change| PlanEdit::SetLight {
            light: light.clone(),
            change,
        })
    };

    match field {
        LightField::Kind => Some(set(Ok(LightChange::Kind(LightKind::from_name(value)?)))),
        LightField::Preset => Some(set(Ok(LightChange::Preset(value.to_owned())))),
        LightField::Facing => {
            let facing = read_number(Field::Angle, value, Unit::Degree, Range::Any);

            Some(facing.map(|facing_deg| PlanEdit::Turn {
                source: Source::Light(id.clone()),
                facing_deg,
            }))
        }
        LightField::Beam => {
            let beam = read_number(Field::Beam, value, Unit::Degree, Range::Positive);

            Some(set(beam.map(LightChange::Beam)))
        }
        LightField::Power => {
            let power = read_number(Field::Power, value, Unit::Watt, Range::NotNegative);

            Some(set(power.map(LightChange::Power)))
        }
        LightField::Temp => {
            let temp = read_number(Field::Temperature, value, Unit::Kelvin, Range::Positive);

            Some(set(temp.map(LightChange::Temp)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::edit_of;

    fn lamp() -> Source {
        Source::Light("l1".into())
    }

    fn pick(field: LightField, text: &str) -> Option<PlanEdit> {
        edit_of(light_edit(Some(&lamp()), field, text))
    }

    fn set(change: LightChange) -> Option<PlanEdit> {
        Some(PlanEdit::SetLight {
            light: "l1".into(),
            change,
        })
    }

    #[test]
    fn typed_numbers_take_comma_point_and_unit() {
        assert_eq!(
            pick(LightField::Beam, "22,5°"),
            set(LightChange::Beam(22.5))
        );
        assert_eq!(
            pick(LightField::Power, "1000 W"),
            set(LightChange::Power(1000.0))
        );
        assert_eq!(
            pick(LightField::Temp, "5600"),
            set(LightChange::Temp(5600.0))
        );
        assert_eq!(pick(LightField::Power, "0"), set(LightChange::Power(0.0)));
    }

    #[test]
    fn a_kind_or_a_lamp_is_named() {
        assert_eq!(
            pick(LightField::Kind, "tube"),
            set(LightChange::Kind(LightKind::Tube))
        );
        assert_eq!(pick(LightField::Kind, "laser"), None);
        assert_eq!(
            pick(LightField::Preset, "fresnel-1k"),
            set(LightChange::Preset("fresnel-1k".into()))
        );
    }

    #[test]
    fn the_facing_is_a_turn() {
        assert_eq!(
            pick(LightField::Facing, "-90"),
            Some(PlanEdit::Turn {
                source: lamp(),
                facing_deg: -90.0
            })
        );
    }

    #[test]
    fn what_is_no_value_is_no_edit() {
        let cases = [
            (LightField::Beam, ["", "0", "-5", "breit", "NaN", "1 W"]),
            (LightField::Power, ["", "-1", "viel", "inf", "5 K", "1e999"]),
            (LightField::Temp, ["", "0", "-1", "warm", "5 W", "NaN"]),
            (
                LightField::Facing,
                ["", "links", "NaN", "inf", "5 W", "1e999"],
            ),
        ];

        for (field, texts) in cases {
            for text in texts {
                assert_eq!(pick(field, text), None, "{field:?} {text:?}");
            }
        }
    }

    #[test]
    fn only_a_lamp_has_a_card() {
        for selected in [None, Some(Source::Camera("a".into()))] {
            let typed = light_edit(selected.as_ref(), LightField::Beam, "30");

            assert!(typed.is_none());
        }
    }
}
