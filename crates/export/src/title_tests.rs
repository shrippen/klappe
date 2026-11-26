//! A title longer than the sheet is wide stays inside the right margin, on every output that
//! has one: the plan, the shot list and the camera pages.
#![allow(clippy::unwrap_used)]

use klappe_model::{Camera, CameraMark, FloorPlan, Movement, Point, Rig, Sensor, Shot, ShotSize};
use klappe_stellprobe::{DisplayList, ShotOrder, VectorView};

use crate::canvas::Prim;
use crate::layout::PT_PER_MM;
use crate::metrics::text_width_pt;
use crate::page::ShotView;
use crate::palette::palette;
use crate::{camera_pages, layout, table, Orientation, Page, Paper, ShotTexts, Texts, Theme};

const TITLE_CLASS: &str = "title";
const REPEATS: usize = 4;

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        arrival: "Arrival".into(),
        departure: "Departure".into(),
        drawing: Default::default(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: ShotTexts::default(),
        lamps: crate::page::LampTexts::default(),
    };
    let title = "Szene 12 · Sängersaal im alten Herrenhaus am See · Plan A ".repeat(REPEATS);

    Page::new(Paper::A4, Orientation::Landscape, title, texts)
}

fn plan() -> FloorPlan {
    let mut plan = FloorPlan::new("a", "Plan A");
    plan.cameras = vec![Camera {
        id: "a".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(8.0, 12.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: Vec::new(),
            curve: Default::default(),
            rig: Rig::default(),
            lens_mm: None,
        }],
        follows: None,
        color: None,
    }];
    plan.shots = vec![Shot {
        id: "s1".into(),
        number: "12.1".into(),
        camera: "a".into(),
        size: ShotSize::Wide,
        angle: Default::default(),
        movement: Movement::Pan,
        lens_mm: None,
        from_s: 0.0,
        to_s: 2.5,
        description: String::new(),
        props: String::new(),
        gear: String::new(),
        crew: String::new(),
        note: String::new(),
        setup_of: None,
        short_name: String::new(),
        image: None,
        version_of: None,
    }];

    plan
}

fn list() -> DisplayList {
    DisplayList {
        lamps: Vec::new(),
        items: Vec::new(),
        width_m: 20.0,
        depth_m: 13.0,
    }
}

/// Every title text of `prims` ends inside the margin of `width_pt`.
fn assert_inside(prims: &[Prim], width_pt: f64, page: &Page) {
    let right = width_pt - page.margin_mm * PT_PER_MM;
    let mut titles = 0;
    for prim in prims {
        let Prim::Text {
            class,
            at,
            size_pt,
            weight,
            text,
            ..
        } = prim
        else {
            continue;
        };
        if class != TITLE_CLASS {
            continue;
        }

        titles += 1;
        let end = at.0 + text_width_pt(text, *size_pt, *weight);
        assert!(end <= right, "{text:?} ends at {end}, margin at {right}");
    }
    assert!(titles > 0, "no title written");
}

#[test]
fn a_long_plan_title_stays_in_the_margin() {
    let page = page();

    let doc = layout::compose(&list(), &page, Theme::Light).unwrap();

    assert_inside(&doc.chrome, doc.width_pt, &page);
}

#[test]
fn a_long_shot_list_title_stays_in_the_margin() {
    let page = page();
    let pal = palette(Theme::Light).unwrap();

    let sheets = table::sheets(&plan(), &page, pal, ShotOrder::Cut).unwrap();

    assert_inside(&sheets[0].prims, sheets[0].width_pt, &page);
}

#[test]
fn a_long_camera_pages_title_stays_in_the_margin() {
    let page = page();
    let pal = palette(Theme::Light).unwrap();
    let view = ShotView {
        shot: "s1".into(),
        view: VectorView {
            height: 0.5,
            faces: Vec::new(),
        },
        strength: Vec::new(),
    };

    let sheets = camera_pages::sheets(&plan(), &page, pal, ShotOrder::Cut, &[view]).unwrap();

    assert_inside(&sheets[0].prims, sheets[0].width_pt, &page);
}
