//! The lamp list under the legend (LH-51).
#![allow(clippy::unwrap_used)]

use klappe_export::{png, ExportError};
use klappe_export::{svg, LampTexts, Orientation, Page, Paper, ShotTexts, Texts, Theme};
use klappe_model::LightKind;
use klappe_stellprobe::{DisplayList, Lamp};

fn lamp(id: &str, kind: LightKind, power_w: f64, beam_deg: f64) -> Lamp {
    Lamp {
        id: id.into(),
        label: id.to_uppercase(),
        kind,
        power_w,
        color_temp_k: 5600.0,
        beam_deg,
    }
}

fn list() -> DisplayList {
    DisplayList {
        items: Vec::new(),
        lamps: vec![
            lamp("l1", LightKind::LedPanel, 200.0, 60.0),
            lamp("l2", LightKind::Flag, 0.0, 22.5),
        ],
        width_m: 10.0,
        depth_m: 10.0,
    }
}

fn page(lamps: LampTexts) -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        drawing: Default::default(),
        lamps,
    };

    Page::new(Paper::A4, Orientation::Landscape, "Titel", texts)
}

fn words() -> LampTexts {
    LampTexts {
        line: "{label}: {kind}, {power} W, {temp} K, {beam}°".into(),
        passive: "{label}: {kind}, {beam}°".into(),
        more: "+ {n} weitere".into(),
        kinds: vec![(LightKind::LedPanel, "LED-Panel".into())],
    }
}

fn lines_with<'a>(svg: &'a str, class: &str) -> Vec<&'a str> {
    let needle = format!("class=\"{class}\"");
    svg.lines().filter(|l| l.contains(&needle)).collect()
}

fn frame_height(svg: &str) -> f64 {
    let line = lines_with(svg, "frame")[0];
    let start = line.find(" height=\"").unwrap() + " height=\"".len();

    line[start..].split('"').next().unwrap().parse().unwrap()
}

/// Each lamp is named with kind, power, colour and beam; a flag has neither power nor colour.
#[test]
fn the_lamp_list_names_each_lamp() {
    let out = svg(&list(), &page(words()), Theme::Light).unwrap();
    let lines = lines_with(&out, "lamp-label");

    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert!(
        lines[0].contains("L1: LED-Panel, 200 W, 5600 K, 60°"),
        "{}",
        lines[0]
    );
    // A kind the words do not name shows its file name.
    assert!(lines[1].contains("L2: flag, 22,5°"), "{}", lines[1]);
}

/// The short form follows the kind, not the power: a bounce with a power set still has no
/// power or colour to name.
#[test]
fn a_reflector_with_power_is_still_passive() {
    let mut list = list();
    list.lamps = vec![lamp("r1", LightKind::Reflector, 650.0, 90.0)];

    let out = svg(&list, &page(words()), Theme::Light).unwrap();
    let lines = lines_with(&out, "lamp-label");

    assert!(lines[0].contains("R1: reflector, 90°"), "{}", lines[0]);
}

/// Without words for it there is no lamp list, and the plan keeps its room.
#[test]
fn no_lamp_words_no_lamp_list() {
    let out = svg(&list(), &page(LampTexts::default()), Theme::Light).unwrap();

    assert!(lines_with(&out, "lamp-label").is_empty());
}

/// The list takes room from the plan: one row of 16 pt, shared by the lamps side by side.
#[test]
fn the_lamp_list_takes_a_row_each() {
    let without = svg(&list(), &page(LampTexts::default()), Theme::Light).unwrap();
    let with = svg(&list(), &page(words()), Theme::Light).unwrap();

    let shrink = frame_height(&without) - frame_height(&with);

    assert!((shrink - 16.0).abs() < 0.011, "{shrink}");
}

/// The plan keeps at least this share of the room it has without lamps.
const MIN_PLAN_SHARE: f64 = 0.7;

fn many(count: usize) -> DisplayList {
    let mut list = list();
    list.lamps = (0..count)
        .map(|i| lamp(&format!("l{i}"), LightKind::LedPanel, 200.0, 60.0))
        .collect();

    list
}

/// 30 and 60 lamps print on every page shape; the plan keeps most of its room and the lamps
/// that do not fit are counted.
#[test]
fn many_lamps_leave_the_plan_its_room() {
    for orientation in [Orientation::Landscape, Orientation::Portrait] {
        for count in [30, 60] {
            let mut bare = page(LampTexts::default());
            let mut full = page(words());
            bare.orientation = orientation;
            full.orientation = orientation;

            let without = svg(&many(count), &bare, Theme::Light).unwrap();
            let with = svg(&many(count), &full, Theme::Light).unwrap();
            let share = frame_height(&with) / frame_height(&without);
            let lines = lines_with(&with, "lamp-label");

            assert!(share >= MIN_PLAN_SHARE, "{orientation:?} {count}: {share}");
            assert!(lines.len() < count, "{orientation:?} {count}");
            let more = format!("+ {} weitere", count - (lines.len() - 1));
            assert!(
                lines[lines.len() - 1].contains(&more),
                "{}",
                lines[lines.len() - 1]
            );
        }
    }
}

/// Every printed format copes with many lamps.
#[test]
fn many_lamps_print_as_png() {
    let out: Result<Vec<u8>, ExportError> = png(&many(60), &page(words()), Theme::Light);

    assert!(out.is_ok());
}
