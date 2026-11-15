//! Lamps: the built-in library, kind, beam, power and colour (LH-50, LH-51), on the example
//! project, whose scene has the LED panel `led-1` (60° beam, 5600 K, 200 W).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{Light, LightKind, Point};
use klappe_services::{
    Field, Hint, Language, Layer, LightChange, Lock, Paper, PlanEdit, ServiceError, Session,
    Source, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn light(s: &Session, id: &str) -> Light {
    s.plan()
        .unwrap()
        .lights
        .iter()
        .find(|l| l.id == id)
        .unwrap()
        .clone()
}

fn set(id: &str, change: LightChange) -> PlanEdit {
    PlanEdit::SetLight {
        light: id.into(),
        change,
    }
}

fn bad(result: Result<(), ServiceError>, field: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field: f, .. }) if f == field)
}

/// The session offers the shipped lamps: three Fresnels, LED, tube, balloon, bounce, flags.
#[test]
fn the_session_offers_the_lamps() {
    let s = open();
    let lights = s.light_presets();
    let fresnel = lights.iter().filter(|l| l.kind == LightKind::Fresnel);

    assert_eq!(
        fresnel.map(|l| l.power_w).collect::<Vec<_>>(),
        [650.0, 1000.0, 2000.0]
    );
    assert!(lights.iter().any(|l| l.id == "floppy"));
}

/// A lamp from the library brings kind, beam, power and colour; undo takes it away.
#[test]
fn a_library_lamp_sets_all_its_values() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddLight {
        preset: "fresnel-2k".into(),
        at: Point::new(2.0, 2.0),
    })
    .unwrap();

    let new = light(&s, "l1");
    assert_eq!(new.kind, LightKind::Fresnel);
    assert_eq!(
        (new.power_w, new.color_temp_k, new.beam_deg),
        (2000.0, 3200.0, 30.0)
    );

    s.undo();
    assert_eq!(s.plan().unwrap().lights.len(), 1);
}

#[test]
fn an_unknown_library_lamp_is_refused() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::AddLight {
        preset: "laser".into(),
        at: Point::new(2.0, 2.0),
    });

    assert!(bad(result, Field::Kind));
    assert_eq!(s.plan().unwrap().lights.len(), 1);
}

/// Beam, power and colour of an LED change; one undo step each.
#[test]
fn beam_power_and_colour_change_and_undo() {
    let mut s = open();

    s.edit_plan(set("led-1", LightChange::Beam(45.0))).unwrap();
    s.edit_plan(set("led-1", LightChange::Power(100.0)))
        .unwrap();
    s.edit_plan(set("led-1", LightChange::Temp(3200.0)))
        .unwrap();

    let l = light(&s, "led-1");
    assert_eq!(
        (l.beam_deg, l.power_w, l.color_temp_k),
        (45.0, 100.0, 3200.0)
    );

    s.undo();
    assert_eq!(light(&s, "led-1").color_temp_k, 5600.0);
    s.undo();
    s.undo();
    assert_eq!(light(&s, "led-1").beam_deg, 60.0);
}

/// A Fresnel keeps its colour; an LED takes another.
#[test]
fn only_an_led_takes_another_colour() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddLight {
        preset: "fresnel-650".into(),
        at: Point::new(2.0, 2.0),
    })
    .unwrap();

    let result = s.edit_plan(set("l1", LightChange::Temp(5600.0)));

    assert!(bad(result, Field::Temperature));
    assert_eq!(light(&s, "l1").color_temp_k, 3200.0);
}

/// A kind with a fixed colour brings it: LED to Fresnel shows 3200 K in hint and print, the
/// power stays, and back to an LED the colour stays settable.
#[test]
fn a_fixed_colour_kind_sets_its_colour() {
    let mut s = open();

    s.edit_plan(set("led-1", LightChange::Kind(LightKind::Fresnel)))
        .unwrap();

    let l = light(&s, "led-1");
    assert_eq!((l.color_temp_k, l.power_w), (3200.0, 200.0));
    let Some(Hint::Light { temp_k, .. }) = s.light_hint("led-1") else {
        panic!("no lamp hint");
    };
    assert_eq!(temp_k, 3200.0);
    let printed = s.display_list().lamps;
    assert_eq!(printed[0].color_temp_k, 3200.0);

    // Fresnel to bounce: the library's bounce is daylight.
    s.edit_plan(set("led-1", LightChange::Kind(LightKind::Reflector)))
        .unwrap();
    assert_eq!(light(&s, "led-1").color_temp_k, 5600.0);

    s.edit_plan(set("led-1", LightChange::Kind(LightKind::LedPanel)))
        .unwrap();
    s.edit_plan(set("led-1", LightChange::Temp(4000.0)))
        .unwrap();
    assert_eq!(light(&s, "led-1").color_temp_k, 4000.0);
}

/// Another kind keeps the values; another library lamp replaces all of them.
#[test]
fn kind_and_library_lamp_change_the_lamp() {
    let mut s = open();

    s.edit_plan(set("led-1", LightChange::Kind(LightKind::Tube)))
        .unwrap();
    let l = light(&s, "led-1");
    assert_eq!(
        (l.kind, l.beam_deg, l.power_w),
        (LightKind::Tube, 60.0, 200.0)
    );

    s.edit_plan(set("led-1", LightChange::Preset("balloon".into())))
        .unwrap();
    let l = light(&s, "led-1");
    assert_eq!(
        (l.kind, l.beam_deg, l.power_w, l.color_temp_k),
        (LightKind::Balloon, 120.0, 1200.0, 3200.0)
    );
}

/// What is no beam, power or colour changes nothing.
#[test]
fn unusable_values_are_refused() {
    let mut s = open();
    let cases = [
        (LightChange::Beam(0.0), Field::Beam),
        (LightChange::Beam(361.0), Field::Beam),
        (LightChange::Beam(f64::NAN), Field::Beam),
        (LightChange::Power(-1.0), Field::Power),
        (LightChange::Power(f64::INFINITY), Field::Power),
        (LightChange::Temp(10.0), Field::Temperature),
        (LightChange::Temp(1e6), Field::Temperature),
        (LightChange::Preset("laser".into()), Field::Kind),
    ];

    for (change, field) in cases {
        let result = s.edit_plan(set("led-1", change.clone()));

        assert!(bad(result, field), "{change:?}");
    }
    assert_eq!(light(&s, "led-1").beam_deg, 60.0);
}

#[test]
fn a_lamp_that_is_gone_is_refused() {
    let mut s = open();

    let result = s.edit_plan(set("x", LightChange::Beam(10.0)));

    assert!(matches!(result, Err(ServiceError::Gone)));
}

/// The facing is turned as for every element.
#[test]
fn the_facing_turns_to_a_direction() {
    let mut s = open();

    s.edit_plan(PlanEdit::Turn {
        source: Source::Light("led-1".into()),
        facing_deg: -90.0,
    })
    .unwrap();

    assert_eq!(light(&s, "led-1").facing_deg, 270.0);
}

/// The lights layer's lock stops every lamp edit.
#[test]
fn a_locked_layer_refuses_lamp_edits() {
    let mut s = open();
    s.set_lock(Layer::Lights, Lock::Locked);

    let result = s.edit_plan(set("led-1", LightChange::Beam(45.0)));

    assert!(matches!(result, Err(ServiceError::Locked(Layer::Lights))));
    assert_eq!(light(&s, "led-1").beam_deg, 60.0);
}

/// The printed plan lists the lamp with kind, power, colour and beam, in the catalogue's
/// words; a hidden lights layer takes it out again.
#[test]
fn the_printout_names_the_lamps() {
    let mut s = open();
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("lamps.svg");
    let printed = |s: &Session| {
        s.export_svg(&path, Paper::A4, Language::English.print_texts())
            .unwrap();

        std::fs::read_to_string(&path).unwrap()
    };

    assert!(printed(&s).contains("LED-Panel: LED panel, 200 W, 5600 K, 60°"));

    s.set_layer(Layer::Lights, Visibility::Hidden);
    assert!(!printed(&s).contains("lamp-label"));
}

/// The shot pictures of a scene with 60 lamps still print: the lamp list is cut, not the plan.
#[test]
fn sixty_lamps_print_in_the_frames() {
    let mut s = open();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("lamps-frames");
    let _ = std::fs::remove_dir_all(&dir);

    for i in 0..60 {
        s.edit_plan(PlanEdit::AddLight {
            preset: "led-panel".into(),
            at: Point::new(1.0 + f64::from(i) * 0.1, 2.0),
        })
        .unwrap();
    }

    let files = s
        .export_frames(&dir, Paper::A4, &Language::English.print_texts())
        .unwrap();

    assert!(!files.is_empty());
}

/// A bounce or flag makes no light, whatever power it carries: the tooltip says so.
#[test]
fn a_hint_tells_whether_the_lamp_gives_light() {
    let mut s = open();
    s.edit_plan(set("led-1", LightChange::Kind(LightKind::Reflector)))
        .unwrap();
    s.edit_plan(set("led-1", LightChange::Power(650.0)))
        .unwrap();

    let Some(Hint::Light { gives_light, .. }) = s.light_hint("led-1") else {
        panic!("no lamp hint");
    };

    assert!(!gives_light);
}

/// The timeline's light track names a colour for a lamp that gives light, none for a bounce.
#[test]
fn a_passive_lamp_has_no_colour_on_the_timeline() {
    let mut s = open();
    let text = |s: &Session| {
        let track = s.tracks().into_iter().find(|t| t.id == "led-1").unwrap();
        track.segments[0].text.clone()
    };
    assert_eq!(text(&s), "5600 K");

    s.edit_plan(set("led-1", LightChange::Kind(LightKind::Reflector)))
        .unwrap();

    assert_eq!(text(&s), "");
}
