//! The lamp list under the legend (LH-51).
#![allow(clippy::unwrap_used)]

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
        lamps,
    };

    Page::new(Paper::A4, Orientation::Landscape, "Titel", texts)
}

fn words() -> LampTexts {
    LampTexts {
        line: "{label}: {kind}, {power} W, {temp} K, {beam}°".into(),
        passive: "{label}: {kind}, {beam}°".into(),
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

/// Without words for it there is no lamp list, and the plan keeps its room.
#[test]
fn no_lamp_words_no_lamp_list() {
    let out = svg(&list(), &page(LampTexts::default()), Theme::Light).unwrap();

    assert!(lines_with(&out, "lamp-label").is_empty());
}

/// The list takes room from the plan: one row of 16 pt per lamp.
#[test]
fn the_lamp_list_takes_a_row_each() {
    let without = svg(&list(), &page(LampTexts::default()), Theme::Light).unwrap();
    let with = svg(&list(), &page(words()), Theme::Light).unwrap();

    let shrink = frame_height(&without) - frame_height(&with);

    assert!((shrink - 2.0 * 16.0).abs() < 0.011, "{shrink}");
}
