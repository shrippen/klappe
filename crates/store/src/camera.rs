//! The built-in camera presets: sensor formats and lens sets (LH-32).

use std::sync::OnceLock;

use klappe_model::{CameraPresets, LensSet};
use serde::Deserialize;

const BUILTIN: &str = include_str!("../camera/standard.toml");

/// `camera/standard.toml`: one `[[kamera]]` per body, one `[[objektiv]]` per lens set.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresetFile {
    kamera: Vec<klappe_model::CameraBody>,
    objektiv: Vec<LensSet>,
}

/// The presets that ship with Klappe. Embedded and tested to parse; an unreadable file would
/// only leave the lists empty.
pub fn builtin_camera_presets() -> &'static CameraPresets {
    static PRESETS: OnceLock<CameraPresets> = OnceLock::new();

    PRESETS.get_or_init(|| parse(BUILTIN).unwrap_or_default())
}

fn parse(text: &str) -> Result<CameraPresets, String> {
    let file: PresetFile = toml::from_str(text).map_err(|e| e.to_string())?;
    let presets = CameraPresets {
        bodies: file.kamera,
        lens_sets: file.objektiv,
    };
    check(&presets)?;

    Ok(presets)
}

fn is_length(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Ids are there once; sizes are lengths; a lens set is primes or a zoom, with the zoom's
/// shortest end not above its longest.
fn check(presets: &CameraPresets) -> Result<(), String> {
    for (index, body) in presets.bodies.iter().enumerate() {
        if body.id.trim().is_empty() || body.name.trim().is_empty() {
            return Err(format!("body {}: id or name is empty", index + 1));
        }
        if !is_length(body.width_mm) || !is_length(body.aspect) {
            return Err(format!("body {:?}: width or aspect is no length", body.id));
        }
        if presets.bodies[..index].iter().any(|b| b.id == body.id) {
            return Err(format!("body {:?} is there twice", body.id));
        }
    }

    for (index, set) in presets.lens_sets.iter().enumerate() {
        if set.id.trim().is_empty() {
            return Err(format!("lens set {}: id is empty", index + 1));
        }
        if presets.lens_sets[..index].iter().any(|s| s.id == set.id) {
            return Err(format!("lens set {:?} is there twice", set.id));
        }

        let stops = set.stops_mm();
        let primes_xor_zoom = set.primes_mm.is_empty() != set.zoom_mm.is_none();
        let ordered = set.zoom_mm.is_none_or(|[min, max]| min <= max);
        if stops.is_empty() || !primes_xor_zoom || !ordered || !stops.iter().all(|s| is_length(*s))
        {
            return Err(format!("lens set {:?} is not usable", set.id));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped file parses, passes the checks and holds the formats LH-32 names.
    #[test]
    fn the_builtin_presets_load() {
        let presets = builtin_camera_presets();
        let ids: Vec<&str> = presets.bodies.iter().map(|b| b.id.as_str()).collect();

        assert_eq!(
            ids,
            [
                "super35",
                "full-frame",
                "micro-four-thirds",
                "aps-c",
                "super16",
                "large-format"
            ]
        );
        assert!(presets.lens_sets.iter().any(|s| s.primes_mm.len() == 7));
        assert!(presets.lens_sets.iter().any(|s| s.zoom_mm.is_some()));
        assert_eq!(parse(BUILTIN).as_ref(), Ok(presets));
    }

    /// The bodies of a named `Sensor` have the width the model knows it by.
    #[test]
    fn named_formats_match_the_model() {
        use klappe_model::Sensor;

        let width = |id: &str| {
            let body = builtin_camera_presets().bodies.iter().find(|b| b.id == id);
            body.map(|b| b.width_mm)
        };

        assert_eq!(width("super35"), Some(Sensor::Super35.width_mm()));
        assert_eq!(width("full-frame"), Some(Sensor::FullFrame.width_mm()));
        assert_eq!(
            width("micro-four-thirds"),
            Some(Sensor::MicroFourThirds.width_mm())
        );
    }

    fn broken(extra: &str) -> Result<CameraPresets, String> {
        parse(&format!(
            "[[kamera]]\nid = \"a\"\nname = \"A\"\nwidth_mm = 10.0\naspect = 2.0\n{extra}"
        ))
    }

    #[test]
    fn unusable_entries_are_refused() {
        let body = "[[kamera]]\nid = \"b\"\nname = \"B\"\nwidth_mm = ";
        let set = "[[objektiv]]\nid = \"s\"\n";
        let cases = [
            // A file without lens sets is no preset file.
            "".to_owned(),
            format!("{body}-1.0\naspect = 2.0\n{set}primes_mm = [50.0]\n"),
            format!("{body}10.0\naspect = 0.0\n{set}primes_mm = [50.0]\n"),
            format!("[[kamera]]\nid = \"a\"\nname = \"Twice\"\nwidth_mm = 9.0\naspect = 2.0\n{set}primes_mm = [50.0]\n"),
            format!("{set}\n"),
            format!("{set}primes_mm = [0.0]\n"),
            format!("{set}primes_mm = [50.0]\nzoom_mm = [24.0, 70.0]\n"),
            format!("{set}zoom_mm = [70.0, 24.0]\n"),
            format!("{set}zoom_mm = [24.0, nan]\n"),
        ];

        for case in cases {
            assert!(broken(&case).is_err(), "{case}");
        }
        assert!(broken(&format!("{set}primes_mm = [50.0]\n")).is_ok());
    }
}
