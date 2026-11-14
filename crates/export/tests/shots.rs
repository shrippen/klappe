// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    pdf, pdf_with_shots, shots_csv, Orientation, Page, Paper, ShotColumns, ShotOrder, ShotTexts,
    Texts, Theme,
};
use klappe_model::{
    Angle, Camera, CameraMark, FloorPlan, Movement, Point, Project, Sensor, SetPlan, Shot, ShotSize,
};
use klappe_stellprobe::{render, DisplayList};

const A4_LANDSCAPE_PT: &str = "[0 0 841.89 595.28]";

fn texts() -> ShotTexts {
    ShotTexts {
        title: "Shotliste".into(),
        columns: ShotColumns {
            setup: "Setup".into(),
            number: "Nummer".into(),
            camera: "Kamera".into(),
            lens_mm: "Objektiv (mm)".into(),
            size: "Größe".into(),
            angle: "Perspektive".into(),
            movement: "Bewegung".into(),
            from_s: "Von (s)".into(),
            to_s: "Bis (s)".into(),
            duration_s: "Dauer (s)".into(),
            description: "Inhalt".into(),
            props: "Requisiten".into(),
            gear: "Technik".into(),
            crew: "Team".into(),
            note: "Notiz".into(),
        },
        sizes: vec![(ShotSize::Wide, "Totale".into())],
        angles: vec![(Angle::Low, "Untersicht".into())],
        movements: vec![(Movement::Pan, "Schwenk".into())],
    }
}

fn camera(id: &str, label: &str, lens_mm: f64) -> Camera {
    Camera {
        id: id.into(),
        label: label.into(),
        sensor: Sensor::Super35,
        lens_mm,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(8.0, 12.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
        }],
        follows: None,
    }
}

fn shot(n: usize) -> Shot {
    Shot {
        id: format!("s{n}"),
        number: format!("12.{n}"),
        camera: "a".into(),
        size: ShotSize::Wide,
        angle: Default::default(),
        movement: Movement::Pan,
        lens_mm: None,
        from_s: n as f64,
        to_s: n as f64 + 2.5,
        description: format!("Elisabeth tritt an den Tisch und sieht auf den Brief {n}"),
        props: "Brief, Kerzenleuchter".into(),
        gear: "Dolly".into(),
        crew: "Kamera, Ton".into(),
        note: "Nach dem ersten Take neu ausleuchten".into(),
    }
}

fn plan(shots: usize) -> FloorPlan {
    let mut p = FloorPlan::new("a", "Plan A");
    p.cameras = vec![camera("a", "A", 35.0)];
    p.shots = (1..=shots).map(shot).collect();
    p
}

fn list(plan: &FloorPlan) -> DisplayList {
    let set = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: Vec::new(),
        objects: Vec::new(),
        background: None,
    };

    render(&Project::new("Test"), &set, plan, 0.0, None)
}

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: texts(),
    };

    Page::new(
        Paper::A4,
        Orientation::Landscape,
        "Szene 12 · Sängersaal · Plan A",
        texts,
    )
}

fn pages(bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(bytes);

    text.matches("/Type /Page /").count()
}

fn pdf_of(shots: usize) -> Vec<u8> {
    let plan = plan(shots);

    pdf_with_shots(&list(&plan), &page(), Theme::Light, &plan, ShotOrder::Cut).unwrap()
}

#[test]
fn csv_quotes_what_rfc_4180_says() {
    let mut plan = plan(1);
    plan.shots[0].description = "Sie sagt: \"Hallo, Welt\"".into();
    plan.shots[0].props = "Käfig, Löffel".into();
    plan.shots[0].note = "Zeile 1\nZeile 2".into();
    plan.shots[0].gear = "ohne Sonderzeichen".into();
    plan.shots[0].angle = Angle::Low;

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);

    let expected = "Nummer,Kamera,Objektiv (mm),Größe,Perspektive,Bewegung,Von (s),Bis (s),Dauer (s),Inhalt,Requisiten,Technik,Team,Notiz\r\n\
        12.1,A,35,Totale,Untersicht,Schwenk,1,3.5,2.5,\"Sie sagt: \"\"Hallo, Welt\"\"\",\"Käfig, Löffel\",ohne Sonderzeichen,\"Kamera, Ton\",\"Zeile 1\nZeile 2\"\r\n";
    assert_eq!(csv, expected);
}

#[test]
fn csv_lens_falls_back_to_the_camera() {
    let mut plan = plan(3);
    plan.shots[1].lens_mm = Some(85.0);
    plan.shots[2].camera = "gone".into();

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);
    let rows: Vec<Vec<&str>> = csv
        .lines()
        .skip(1)
        .map(|l| l.split(',').take(3).collect())
        .collect();

    assert_eq!(rows[0], ["12.1", "A", "35"]);
    assert_eq!(rows[1], ["12.2", "A", "85"]);
    // A camera that is gone: its id stays, the lens is unknown.
    assert_eq!(rows[2], ["12.3", "gone", ""]);
}

#[test]
fn csv_without_shots_is_the_head_row() {
    let csv = shots_csv(&plan(0), &texts(), ShotOrder::Cut);

    assert_eq!(csv.matches("\r\n").count(), 1);
    assert!(csv.starts_with("Nummer,Kamera,Objektiv (mm),"));
    assert!(csv.ends_with(",Notiz\r\n"));
}

#[test]
fn csv_numbers_use_a_point_in_every_language() {
    let mut plan = plan(1);
    plan.shots[0].lens_mm = Some(27.5);

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);

    assert!(csv.contains(",27.5,"), "{csv}");
    assert!(!csv.contains("27,5"));
}

#[test]
fn csv_keeps_cut_order() {
    let mut plan = plan(3);
    plan.shots.swap(0, 2);

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);
    let numbers: Vec<&str> = csv
        .lines()
        .skip(1)
        .map(|l| l.split(',').next().unwrap())
        .collect();

    assert_eq!(numbers, ["12.3", "12.2", "12.1"]);
}

#[test]
fn no_shots_leave_the_plan_alone() {
    let plan = plan(0);
    let with = pdf_with_shots(&list(&plan), &page(), Theme::Light, &plan, ShotOrder::Cut).unwrap();
    let without = pdf(&list(&plan), &page(), Theme::Light).unwrap();

    assert_eq!(pages(&with), 1);
    assert_eq!(with, without);
}

#[test]
fn a_few_shots_make_one_landscape_sheet() {
    let bytes = pdf_of(3);
    let text = String::from_utf8_lossy(&bytes);

    assert_eq!(pages(&bytes), 2);
    assert_eq!(
        text.matches(A4_LANDSCAPE_PT).count(),
        2,
        "plan and sheet are A4 landscape"
    );
    assert!(text.contains("Shotliste"));
    assert!(text.contains("(12.3)"));
}

#[test]
fn many_shots_break_onto_more_sheets() {
    let few = pages(&pdf_of(3));
    let many = pages(&pdf_of(120));
    let text = String::from_utf8_lossy(&pdf_of(120)).into_owned();

    assert!(many > few + 1, "{many} pages");
    // The head row comes back on every sheet.
    assert_eq!(text.matches("(Requisiten)").count(), many - 1);
    assert!(text.contains("(12.120)"));
}

#[test]
fn a_huge_cell_is_cut_and_still_fits() {
    let mut plan = plan(1);
    plan.shots[0].note = "wort ".repeat(5000);
    plan.shots[0].description = "Donaudampfschifffahrtsgesellschaftskapitän".repeat(20);

    let bytes = pdf_with_shots(&list(&plan), &page(), Theme::Light, &plan, ShotOrder::Cut).unwrap();

    assert_eq!(pages(&bytes), 2);
}

/// Shots 12.1 and 12.3 are camera a at (8; 12) with 35 mm, 12.2 is camera b elsewhere:
/// setups 1, 2, 1; shooting order 12.1, 12.3, 12.2.
fn two_setups() -> FloorPlan {
    let mut plan = plan(3);
    let mut b = camera("b", "B", 35.0);
    b.marks[0].at = Point::new(2.0, 2.0);
    plan.cameras.push(b);
    plan.shots[1].camera = "b".into();

    plan
}

#[test]
fn csv_in_shooting_order_groups_setups() {
    let csv = shots_csv(&two_setups(), &texts(), ShotOrder::Shooting);
    let rows: Vec<Vec<&str>> = csv
        .lines()
        .map(|l| l.split(',').take(3).collect())
        .collect();

    assert_eq!(rows[0], ["Setup", "Nummer", "Kamera"]);
    assert_eq!(rows[1], ["1", "12.1", "A"]);
    assert_eq!(rows[2], ["1", "12.3", "A"]);
    assert_eq!(rows[3], ["2", "12.2", "B"]);
}

#[test]
fn csv_in_cut_order_has_no_setup_column() {
    let csv = shots_csv(&two_setups(), &texts(), ShotOrder::Cut);

    assert!(csv.starts_with("Nummer,Kamera,"), "{csv}");
}

#[test]
fn pdf_in_shooting_order_has_setup_headings() {
    let plan = two_setups();
    let bytes = pdf_with_shots(
        &list(&plan),
        &page(),
        Theme::Light,
        &plan,
        ShotOrder::Shooting,
    )
    .unwrap();
    let text = String::from_utf8_lossy(&bytes);
    let at = |needle: &str| text.find(needle).unwrap();

    assert!(at("(Setup 1)") < at("(12.1)"));
    assert!(at("(12.1)") < at("(12.3)"));
    assert!(at("(12.3)") < at("(Setup 2)"));
    assert!(at("(Setup 2)") < at("(12.2)"));
    assert_eq!(text.matches("(Setup ").count(), 2);
}

#[test]
fn pdf_in_cut_order_has_no_setup_headings() {
    let plan = two_setups();
    let bytes = pdf_with_shots(&list(&plan), &page(), Theme::Light, &plan, ShotOrder::Cut).unwrap();

    assert!(!String::from_utf8_lossy(&bytes).contains("(Setup "));
}

#[test]
fn a_setup_heading_repeats_on_each_sheet() {
    // 120 shots of one setup fill several sheets; each sheet says which setup it shows.
    let plan = plan(120);
    let bytes = pdf_with_shots(
        &list(&plan),
        &page(),
        Theme::Light,
        &plan,
        ShotOrder::Shooting,
    )
    .unwrap();
    let sheets = pages(&bytes) - 1;

    assert!(sheets > 1, "{sheets} sheets");
    assert_eq!(
        String::from_utf8_lossy(&bytes).matches("(Setup 1)").count(),
        sheets
    );
}
