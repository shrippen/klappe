// Test helpers may unwrap; a failure should stop the test.
#![allow(clippy::unwrap_used)]

use klappe_export::{
    pdf, pdf_with_shots, pdf_with_views, picture_png, shots_csv, LampTexts, Orientation, Page,
    Paper, ShotColumns, ShotOrder, ShotTexts, ShotView, Texts, Theme,
};
use klappe_model::{
    Angle, Arrival, Camera, CameraMark, Departure, FigureTrack, FloorPlan, Mark, Movement, Point,
    Pose, Project, Sensor, SetPlan, Shot, ShotSize,
};
use klappe_stellprobe::{picture, render, vector_view, Cut, DisplayList, Eye, Forms};

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
            axis: "Achsensprung gegen".into(),
        },
        sizes: vec![(ShotSize::Wide, "Totale".into())],
        angles: vec![(Angle::Low, "Untersicht".into())],
        movements: vec![(Movement::Pan, "Schwenk".into())],
        views: "Kamerablicke".into(),
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
            via: Vec::new(),
            curve: Default::default(),
            rig: Default::default(),
            lens_mm: None,
        }],
        follows: None,
    }
}

fn figure(id: &str, x: f64) -> FigureTrack {
    FigureTrack {
        character: id.into(),
        marks: vec![Mark {
            at_s: 0.0,
            at: Point::new(x, 5.0),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
            via: Vec::new(),
            curve: Default::default(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        }],
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
        setup_of: None,
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
        notes: Vec::new(),
        ..Default::default()
    };

    render(&Project::new("Test"), &set, plan, 0.0, &|_| None)
}

fn page() -> Page {
    let texts = Texts {
        figure: "Figur".into(),
        camera: "Kamera".into(),
        light: "Licht".into(),
        axis: "Achse".into(),
        path: "Weg".into(),
        mark: "Mark".into(),
        drawing: Default::default(),
        unit: "m".into(),
        decimal_separator: ',',
        shots: texts(),
        lamps: LampTexts::default(),
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

    let expected = "Nummer,Kamera,Objektiv (mm),Größe,Perspektive,Bewegung,Von (s),Bis (s),Dauer (s),Inhalt,Requisiten,Technik,Team,Notiz,Achsensprung gegen\r\n\
        12.1,A,35,Totale,Untersicht,Schwenk,1,3.5,2.5,\"Sie sagt: \"\"Hallo, Welt\"\"\",\"Käfig, Löffel\",ohne Sonderzeichen,\"Kamera, Ton\",\"Zeile 1\nZeile 2\",\r\n";
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
fn csv_lens_comes_from_the_marks() {
    let mut plan = plan(2);
    let mark = |at_s, lens_mm| CameraMark {
        at_s,
        lens_mm: Some(lens_mm),
        ..plan.cameras[0].marks[0].clone()
    };
    plan.cameras[0].marks = vec![mark(0.0, 35.0), mark(6.0, 85.0)];
    plan.shots[0].lens_mm = Some(50.0);
    plan.shots[0].from_s = 0.0;
    plan.shots[0].to_s = 0.0;
    plan.shots[1].from_s = 0.0;
    plan.shots[1].to_s = 6.0;

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);
    let lens: Vec<&str> = csv
        .lines()
        .skip(1)
        .map(|l| l.split(',').nth(2).unwrap())
        .collect();

    // The shot's own 50 mm does not count; a span reads "35\u{2013}85".
    assert_eq!(lens, ["35", "35\u{2013}85"]);
}

#[test]
fn csv_without_shots_is_the_head_row() {
    let csv = shots_csv(&plan(0), &texts(), ShotOrder::Cut);

    assert_eq!(csv.matches("\r\n").count(), 1);
    assert!(csv.starts_with("Nummer,Kamera,Objektiv (mm),"));
    assert!(csv.ends_with(",Notiz,Achsensprung gegen\r\n"));
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

const A4_PORTRAIT_PT: &str = "[0 0 595.28 841.89]";
const LETTER_PORTRAIT_PT: &str = "[0 0 612 792]";

fn portrait(paper: Paper, shots: usize) -> Vec<u8> {
    let plan = plan(shots);
    let mut page = page();
    page.paper = paper;
    page.orientation = Orientation::Portrait;

    pdf_with_shots(&list(&plan), &page, Theme::Light, &plan, ShotOrder::Cut).unwrap()
}

/// The text of every string a PDF draws, in file order, joined by one space.
fn drawn(text: &str) -> String {
    let mut out = String::new();
    let mut open = false;
    for c in text.chars() {
        match (c, open) {
            ('(', false) => open = true,
            (')', true) => {
                open = false;
                out.push(' ');
            }
            (c, true) => out.push(c),
            _ => {}
        }
    }

    out
}

const LONG_DESCRIPTION: &str = "Elisabeth tritt durch die Tuer, bleibt am Fenster stehen und sieht hinaus auf den Hof, waehrend Wolfram im Hintergrund die Laute stimmt und Tannhaeuser schweigt";

/// Shots 12.1 to 12.23 of camera a, and 12.24 of camera b across the axis from all of them.
fn crossing_plan() -> FloorPlan {
    let mut plan = plan(24);
    plan.figures = vec![figure("e", 2.0), figure("v", 8.0)];
    let mut b = camera("b", "B", 50.0);
    b.marks[0].at = Point::new(8.0, 1.0);
    plan.cameras.push(b);
    plan.shots[23].camera = "b".into();
    plan.shots[23].description = LONG_DESCRIPTION.into();

    plan
}

/// Plan and shot list pages are all upright, every column is there, and nothing is cut that
/// the landscape list shows: a 160 character description and 23 axis partners.
#[test]
fn a_portrait_pdf_has_upright_shot_list_pages() {
    let plan = crossing_plan();
    let mut page = page();
    page.orientation = Orientation::Portrait;
    let bytes = pdf_with_shots(&list(&plan), &page, Theme::Light, &plan, ShotOrder::Cut).unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let words = drawn(&text);
    let all = pages(text.as_bytes());

    assert!(all > 1, "{all} pages");
    assert_eq!(
        text.matches(&format!("/MediaBox {A4_PORTRAIT_PT}")).count(),
        all
    );
    assert!(!text.contains(A4_LANDSCAPE_PT));
    for head in [
        "Nummer",
        "Kamera",
        "Objektiv",
        "Perspektive",
        "Bewegung",
        "Von",
        "Bis",
        "Dauer",
        "Inhalt",
        "Requisiten",
        "Technik",
        "Team",
        "Notiz",
        "Achsensprung",
    ] {
        assert!(words.contains(head), "{head}");
    }
    // A cut cell ends in "…", which the PDF writes as the WinAnsi escape \205.
    assert!(!text.contains("\\205"), "a cell is cut");

    let flat = words.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains(LONG_DESCRIPTION),
        "description is cut or broken"
    );
    let axis = &flat[flat.find("Achsensprung gegen:").unwrap()..];
    for n in 1..=23 {
        assert!(axis.contains(&format!("12.{n}")), "12.{n}");
    }
}

#[test]
fn letter_is_a_us_letter_sheet() {
    let text = String::from_utf8_lossy(&portrait(Paper::Letter, 1)).into_owned();

    assert!(text.contains(&format!("/MediaBox {LETTER_PORTRAIT_PT}")));
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

/// Two figures on y = 5; camera `a` (y = 12) and `b` (y = 1) stand on different sides.
#[test]
fn csv_names_the_shots_the_axis_is_crossed_against() {
    let mut plan = plan(3);
    plan.figures = vec![figure("e", 2.0), figure("v", 8.0)];
    let mut b = camera("b", "B", 50.0);
    b.marks[0].at = Point::new(8.0, 1.0);
    plan.cameras.push(b);
    plan.shots[2].camera = "b".into();

    let csv = shots_csv(&plan, &texts(), ShotOrder::Cut);

    // Only the lone 12.3 is flagged, against 12.1 and 12.2; the list holds a comma, so quoted.
    assert!(csv.lines().nth(1).unwrap().ends_with(","));
    assert!(csv.lines().nth(2).unwrap().ends_with(","));
    assert!(csv.lines().nth(3).unwrap().ends_with(",\"12.1, 12.2\""));
}

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// Where the width sits in a PNG file: after the signature, the IHDR length and its name.
const PNG_WIDTH_AT: usize = 16;

fn camera_eye(shot: &Shot) -> Eye {
    Eye::Shot {
        camera: shot.camera.clone(),
        t_s: shot.from_s,
        aspect: None,
    }
}

fn shot_views(plan: &FloorPlan) -> Vec<ShotView> {
    plan.shots
        .iter()
        .map(|shot| ShotView {
            shot: shot.id.clone(),
            view: vector_view(
                &Project::new("Test"),
                &SetPlan {
                    width_m: 20.0,
                    depth_m: 13.0,
                    ..Default::default()
                },
                plan,
                &Forms::default(),
                &camera_eye(shot),
            )
            .unwrap(),
        })
        .collect()
}

fn pdf_with_camera_views(plan: &FloorPlan) -> Vec<u8> {
    let views = shot_views(plan);

    pdf_with_views(
        &list(plan),
        &page(),
        Theme::Light,
        plan,
        ShotOrder::Cut,
        &views,
    )
    .unwrap()
}

/// LH-86: six views to a page, so 7 shots add 2 pages; each view has the shot's number and
/// size above it, and a view is its faces as filled paths.
#[test]
fn the_pdf_has_camera_pages_with_captions() {
    let plan = plan(7);
    let without = pages(&pdf_of(7));

    let bytes = pdf_with_camera_views(&plan);

    assert_eq!(pages(&bytes), without + 2);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("(Szene 12 \\267 S\\344ngersaal \\267 Plan A \\267 Kamerablicke)"));
    for n in 1..=7 {
        assert!(text.contains(&format!("(12.{n} \\267 Totale)")), "{n}");
    }
    assert!(text.contains("(35 mm)"));
    assert!(text.contains(" f\n") || text.contains(" B\n"));
}

/// Camera pages take the paper in the page's orientation: all pages of the PDF share one size.
#[test]
fn camera_pages_follow_the_orientation() {
    let plan = plan(1);
    let views = shot_views(&plan);
    for (orientation, size) in [
        (Orientation::Portrait, A4_PORTRAIT_PT),
        (Orientation::Landscape, A4_LANDSCAPE_PT),
    ] {
        let mut page = page();
        page.orientation = orientation;

        let bytes = pdf_with_views(
            &list(&plan),
            &page,
            Theme::Light,
            &plan,
            ShotOrder::Cut,
            &views,
        )
        .unwrap();

        let text = String::from_utf8_lossy(&bytes);
        let with_views = pages(&bytes);
        let without = pages(
            &pdf_with_shots(&list(&plan), &page, Theme::Light, &plan, ShotOrder::Cut).unwrap(),
        );
        assert!(with_views > without, "{with_views} pages");
        assert_eq!(
            text.matches(&format!("/MediaBox {size}")).count(),
            with_views
        );
    }
}

/// No view, no page: a plan without camera views prints as before.
#[test]
fn without_views_the_pdf_has_no_camera_pages() {
    let plan = plan(2);

    let bytes = pdf_with_views(
        &list(&plan),
        &page(),
        Theme::Light,
        &plan,
        ShotOrder::Cut,
        &[],
    )
    .unwrap();

    assert_eq!(bytes, pdf_of(2));
    assert!(!String::from_utf8_lossy(&bytes).contains("Kamerablicke"));
}

/// LH-86: the picture as PNG has the picture's size.
#[test]
fn the_camera_picture_is_a_png_of_its_size() {
    let plan = plan(1);
    let set = SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        ..Default::default()
    };
    let eye = camera_eye(&plan.shots[0]);
    let view = picture(
        &Project::new("Test"),
        &set,
        &plan,
        &Forms::default(),
        &eye,
        640,
        Cut::Off,
    )
    .unwrap();

    let bytes = picture_png(&view, Theme::Light, &|_| 1.0).unwrap();

    assert!(bytes.starts_with(&PNG_SIGNATURE));
    let width = u32::from_be_bytes(bytes[PNG_WIDTH_AT..PNG_WIDTH_AT + 4].try_into().unwrap());
    let height = u32::from_be_bytes(
        bytes[PNG_WIDTH_AT + 4..PNG_WIDTH_AT + 8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(
        (width as usize, height as usize),
        (view.frame.width, view.frame.height)
    );
    assert_eq!(width, 640);
    assert_eq!(height, (640.0_f64 / 2.39).round() as u32);
}

/// A setup chosen by hand counts in the setup column and the shooting order (LH-77).
#[test]
fn a_shot_assigned_by_hand_is_listed_in_its_setup() {
    let mut plan = two_setups();
    plan.shots[1].setup_of = Some(plan.shots[0].id.clone());

    let csv = shots_csv(&plan, &texts(), ShotOrder::Shooting);
    let rows: Vec<Vec<&str>> = csv
        .lines()
        .map(|l| l.split(',').take(3).collect())
        .collect();

    assert_eq!(rows[1], ["1", "12.1", "A"]);
    assert_eq!(rows[2], ["1", "12.2", "B"]);
    assert_eq!(rows[3], ["1", "12.3", "A"]);
}
