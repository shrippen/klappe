//! A lamp that gives no light has no power.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use klappe_model::{Light, LightKind};
use klappe_store::{load, save};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

/// A project saved while a reflector could keep a lamp's power loads with 0 W for it.
#[test]
fn a_passive_lamp_loads_without_power() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("passive-power");
    let _ = fs::remove_dir_all(&dir);
    let mut project = load(Path::new(EXAMPLE)).unwrap().project;
    let lights = &mut project.scenes[0].plans[0].lights;
    lights.push(Light {
        id: "reflektor-1".into(),
        label: "Reflektor".into(),
        kind: LightKind::Reflector,
        power_w: 650.0,
        ..lights[0].clone()
    });
    let lit_power = lights[0].power_w;

    save(&dir, &project, None).unwrap();
    let loaded = load(&dir).unwrap();
    let lights = &loaded.project.scenes[0].plans[0].lights;

    assert_eq!(lights[1].power_w, 0.0);
    assert_eq!(lights[0].power_w, lit_power);
}
