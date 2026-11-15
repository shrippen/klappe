//! The built-in lamps (LH-51).

use std::sync::OnceLock;

use klappe_model::{is_beam_deg, is_power_w, is_temp_k, LightPreset};
use serde::Deserialize;

const BUILTIN: &str = include_str!("../light/standard.toml");

/// `light/standard.toml`: one `[[lampe]]` per lamp.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LightFile {
    lampe: Vec<LightPreset>,
}

/// The lamps that ship with Klappe. Embedded and tested to parse; an unreadable file would
/// only leave the list empty.
pub fn builtin_lights() -> &'static [LightPreset] {
    static LIGHTS: OnceLock<Vec<LightPreset>> = OnceLock::new();

    LIGHTS.get_or_init(|| parse(BUILTIN).unwrap_or_default())
}

fn parse(text: &str) -> Result<Vec<LightPreset>, String> {
    let file: LightFile = toml::from_str(text).map_err(|e| e.to_string())?;
    check(&file.lampe)?;

    Ok(file.lampe)
}

/// Ids are there once; power, colour temperature and beam within the limits of the lamp card.
fn check(lights: &[LightPreset]) -> Result<(), String> {
    for (index, light) in lights.iter().enumerate() {
        if light.id.trim().is_empty() {
            return Err(format!("lamp {}: id is empty", index + 1));
        }
        if lights[..index].iter().any(|l| l.id == light.id) {
            return Err(format!("lamp {:?} is there twice", light.id));
        }

        let power = is_power_w(light.power_w);
        let temp = is_temp_k(light.color_temp_k);
        let beam = is_beam_deg(light.beam_deg);
        if !power || !temp || !beam {
            return Err(format!("lamp {:?} has an unusable value", light.id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::LightKind;

    /// The shipped file parses, passes the checks and holds the lamps LH-51 names.
    #[test]
    fn the_builtin_lamps_load() {
        let lights = builtin_lights();
        let ids: Vec<&str> = lights.iter().map(|l| l.id.as_str()).collect();

        assert_eq!(
            ids,
            [
                "fresnel-650",
                "fresnel-1k",
                "fresnel-2k",
                "led-panel",
                "tube",
                "balloon",
                "reflector",
                "floppy",
                "flag"
            ]
        );
        assert_eq!(parse(BUILTIN).as_deref(), Ok(lights));
    }

    /// Every kind has a lamp, and the Fresnel comes in three powers.
    #[test]
    fn every_kind_has_a_lamp() {
        let lights = builtin_lights();

        for kind in LightKind::ALL {
            assert!(lights.iter().any(|l| l.kind == kind), "{kind:?}");
        }

        let powers: Vec<f64> = lights
            .iter()
            .filter(|l| l.kind == LightKind::Fresnel)
            .map(|l| l.power_w)
            .collect();
        assert_eq!(powers, [650.0, 1000.0, 2000.0]);
    }

    /// The card's limits hold for the library too: a lamp it would refuse is not loaded.
    #[test]
    fn the_cards_limits_hold_for_the_library() {
        let lamp = "[[lampe]]\nid = \"a\"\nkind = \"tube\"\npower_w = 1.0\ncolor_temp_k = 3200.0\nbeam_deg = 30.0\n";
        let cases = [
            lamp.replace("3200.0", "999.0"),
            lamp.replace("3200.0", "20001.0"),
            lamp.replace("power_w = 1.0", "power_w = 100001.0"),
            lamp.replace("power_w = 1.0", "power_w = nan"),
            lamp.replace("beam_deg = 30.0", "beam_deg = nan"),
            lamp.replace("beam_deg = 30.0", "beam_deg = 361.0"),
            lamp.replace("color_temp_k = 3200.0", "color_temp_k = inf"),
        ];

        for text in cases {
            assert!(parse(&text).is_err(), "{text}");
        }
        assert!(parse(&lamp.replace("3200.0", "1000.0")).is_ok());
        assert!(parse(&lamp.replace("3200.0", "20000.0")).is_ok());
        assert!(parse(&lamp.replace("power_w = 1.0", "power_w = 0.0")).is_ok());
        assert!(parse(&lamp.replace("power_w = 1.0", "power_w = 100000.0")).is_ok());
        assert!(parse(&lamp.replace("beam_deg = 30.0", "beam_deg = 360.0")).is_ok());
    }

    #[test]
    fn unusable_entries_are_refused() {
        let lamp = |extra: &str| {
            format!(
                "[[lampe]]\nid = \"a\"\nkind = \"tube\"\npower_w = 1.0\ncolor_temp_k = 3200.0\nbeam_deg = 30.0\n{extra}"
            )
        };
        let cases = [
            lamp("[[lampe]]\nid = \"a\"\nkind = \"tube\"\npower_w = 1.0\ncolor_temp_k = 1.0\nbeam_deg = 1.0\n"),
            lamp("").replace("power_w = 1.0", "power_w = -1.0"),
            lamp("").replace("beam_deg = 30.0", "beam_deg = 0.0"),
            lamp("").replace("beam_deg = 30.0", "beam_deg = 400.0"),
            lamp("").replace("color_temp_k = 3200.0", "color_temp_k = 0.0"),
            lamp("").replace("id = \"a\"", "id = \" \""),
            lamp("").replace("\"tube\"", "\"laser\""),
        ];

        assert!(parse(&lamp("")).is_ok());
        for text in cases {
            assert!(parse(&text).is_err(), "{text}");
        }
    }
}
