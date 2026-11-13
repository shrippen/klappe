use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::*;
use klappe_store::{load, save, save_script, StoreError};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const SCRIPT: &str = "INT. SÄNGERSAAL – TAG #12#\n\nElisabeth tritt an den Tisch.\n\nELISABETH\nIhr kommt spät.\n\nBURGVOGT\nDer Weg war lang.\n\nElisabeth wendet sich ab.\n";

fn wall(x1: f64, y1: f64, x2: f64, y2: f64) -> Wall {
    Wall {
        from: Point::new(x1, y1),
        to: Point::new(x2, y2),
        thickness_m: 0.4,
    }
}

fn object(id: &str, kind: &str, label: &str, at: Point, w: f64, d: f64) -> SetObject {
    SetObject {
        id: id.into(),
        kind: kind.into(),
        label: label.into(),
        at,
        width_m: w,
        depth_m: d,
        rotation_deg: 0.0,
    }
}

fn mark(at_s: f64, x: f64, y: f64, facing_deg: f64, hold_s: f64) -> Mark {
    Mark {
        at_s,
        at: Point::new(x, y),
        facing_deg,
        pose: Pose::Standing,
        hold_s,
    }
}

fn cam_mark(at_s: f64, x: f64, y: f64, facing_deg: f64, hold_s: f64) -> CameraMark {
    CameraMark {
        at_s,
        at: Point::new(x, y),
        facing_deg,
        height_m: 1.6,
        tilt_deg: 0.0,
        hold_s,
    }
}

fn shot(
    number: &str,
    camera: &str,
    size: ShotSize,
    movement: Movement,
    from_s: f64,
    to_s: f64,
) -> Shot {
    Shot {
        id: format!("s{}", number.replace('.', "-")),
        number: number.into(),
        camera: camera.into(),
        size,
        movement,
        lens_mm: None,
        from_s,
        to_s,
        description: String::new(),
        props: String::new(),
        gear: String::new(),
        crew: String::new(),
        note: String::new(),
    }
}

fn plan() -> FloorPlan {
    let mut plan = FloorPlan::new("plan-a", "Plan A");

    plan.figures = vec![
        FigureTrack {
            character: "elisabeth".into(),
            marks: vec![
                mark(0.0, 3.0, 6.5, 0.0, 2.0),
                mark(6.0, 10.0, 6.5, 0.0, 4.0),
                mark(14.0, 16.0, 3.0, 270.0, 0.0),
            ],
        },
        FigureTrack {
            character: "burgvogt".into(),
            marks: vec![
                mark(0.0, 17.0, 9.5, 180.0, 5.0),
                mark(9.0, 12.0, 6.5, 180.0, 0.0),
            ],
        },
    ];

    let mut dolly = shot("12.2", "a", ShotSize::Medium, Movement::Dolly, 6.0, 14.0);
    dolly.lens_mm = Some(40.0);
    dolly.description = "Fahrt auf Elisabeth".into();
    dolly.gear = "Dolly, 4 m Schiene".into();

    plan.cameras = vec![
        Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks: vec![
                cam_mark(0.0, 8.0, 11.0, 330.0, 6.0),
                cam_mark(6.0, 9.0, 8.0, 320.0, 0.0),
            ],
            follows: None,
        },
        Camera {
            id: "b".into(),
            label: "B".into(),
            sensor: Sensor::Super35,
            lens_mm: 50.0,
            aspect: 2.39,
            marks: vec![cam_mark(0.0, 18.0, 4.0, 200.0, 0.0)],
            follows: Some("burgvogt".into()),
        },
    ];

    plan.lights = vec![Light {
        id: "led-1".into(),
        label: "LED-Panel".into(),
        kind: LightKind::LedPanel,
        at: Point::new(1.0, 10.0),
        facing_deg: 330.0,
        beam_deg: 60.0,
        color_temp_k: 5600.0,
        power_w: 200.0,
    }];

    plan.shots = vec![
        shot("12.1", "a", ShotSize::Wide, Movement::Static, 0.0, 6.0),
        dolly,
        shot("12.3", "b", ShotSize::CloseUp, Movement::Static, 6.0, 9.0),
        shot(
            "12.4",
            "b",
            ShotSize::MediumClose,
            Movement::Static,
            9.0,
            14.0,
        ),
        shot(
            "12.5",
            "a",
            ShotSize::MediumWide,
            Movement::Handheld,
            14.0,
            20.0,
        ),
    ];

    plan.axis = Some(AxisPair {
        a: "elisabeth".into(),
        b: "burgvogt".into(),
    });

    plan.links = vec![
        ScriptLink {
            element: 0,
            target: LinkTarget::Mark {
                character: "elisabeth".into(),
                index: 0,
            },
        },
        ScriptLink {
            element: 1,
            target: LinkTarget::Shot { id: "s12-2".into() },
        },
        ScriptLink {
            element: 3,
            target: LinkTarget::Mark {
                character: "burgvogt".into(),
                index: 1,
            },
        },
    ];

    plan
}

fn example() -> Project {
    let mut project = Project::new("Die Wartburg-Szene");

    project.characters = vec![
        Character {
            id: "elisabeth".into(),
            name: "Elisabeth".into(),
            short: "E".into(),
            color: FigureColor::Purple,
        },
        Character {
            id: "burgvogt".into(),
            name: "Burgvogt".into(),
            short: "B".into(),
            color: FigureColor::Blue,
        },
    ];

    project.locations = vec![Location {
        id: "saengersaal".into(),
        name: "Sängersaal".into(),
        set: SetPlan {
            width_m: 20.0,
            depth_m: 13.0,
            walls: vec![
                wall(0.0, 0.0, 20.0, 0.0),
                wall(20.0, 0.0, 20.0, 13.0),
                wall(20.0, 13.0, 0.0, 13.0),
                wall(0.0, 13.0, 0.0, 0.0),
            ],
            objects: vec![
                object("tisch", "table", "Tafel", Point::new(10.0, 6.5), 6.0, 1.2),
                object(
                    "saeule-1",
                    "pillar",
                    "Säule",
                    Point::new(6.0, 4.0),
                    0.8,
                    0.8,
                ),
                object(
                    "saeule-2",
                    "pillar",
                    "Säule",
                    Point::new(14.0, 4.0),
                    0.8,
                    0.8,
                ),
            ],
        },
    }];

    project.scenes = vec![Scene {
        id: "szene-012".into(),
        number: "12".into(),
        heading: "INT. SÄNGERSAAL – TAG".into(),
        location: "saengersaal".into(),
        plans: vec![plan()],
    }];

    project
}

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// Rewrites the checked-in example: `cargo test -p klappe-store -- --ignored`.
#[test]
#[ignore]
fn generate_example() {
    let dir = Path::new(EXAMPLE);
    let _ = fs::remove_dir_all(dir);

    save(dir, &example(), Some(SCRIPT)).unwrap();
}

#[test]
fn loads_example() {
    let loaded = load(Path::new(EXAMPLE)).unwrap();
    let project = &loaded.project;

    assert_eq!(project.title, "Die Wartburg-Szene");
    assert_eq!(project.characters[0].color, FigureColor::Purple);
    assert_eq!(project.characters[1].color, FigureColor::Blue);

    let hall = project.location("saengersaal").unwrap();
    assert_eq!((hall.set.width_m, hall.set.depth_m), (20.0, 13.0));
    assert_eq!(hall.set.walls.len(), 4);
    assert_eq!(hall.set.objects.len(), 3);

    let scene = project.scene("szene-012").unwrap();
    assert_eq!(scene.number, "12");
    assert_eq!(scene.heading, "INT. SÄNGERSAAL – TAG");
    assert_eq!(scene.location, "saengersaal");

    let plan = &scene.plans[0];
    assert_eq!(plan.figure("elisabeth").unwrap().marks.len(), 3);
    assert_eq!(plan.camera("a").unwrap().sensor, Sensor::Super35);
    assert_eq!(plan.camera("a").unwrap().marks.len(), 2);
    assert_eq!(plan.camera("b").unwrap().lens_mm, 50.0);
    assert_eq!(plan.lights[0].kind, LightKind::LedPanel);
    assert_eq!(plan.axis.as_ref().unwrap().b, "burgvogt");
    assert_eq!(plan.links.len(), 3);

    let numbers: Vec<_> = plan.shots.iter().map(|s| s.number.as_str()).collect();
    assert_eq!(numbers, ["12.1", "12.2", "12.3", "12.4", "12.5"]);

    assert_eq!(loaded.script.as_deref(), Some(SCRIPT));
}

#[test]
fn example_matches_generator() {
    let loaded = load(Path::new(EXAMPLE)).unwrap();

    assert_eq!(loaded.project, example());
}

#[test]
fn round_trip() {
    let dir = test_dir("round-trip");

    save(&dir, &example(), Some(SCRIPT)).unwrap();
    let loaded = load(&dir).unwrap();

    assert_eq!(loaded.project, example());
    assert_eq!(loaded.script.as_deref(), Some(SCRIPT));
}

#[test]
fn files_end_with_newline_and_leave_no_temp() {
    let dir = test_dir("newline");
    save(&dir, &example(), None).unwrap();

    for rel in [
        "projekt.toml",
        "figuren.toml",
        "drehorte/saengersaal.toml",
        "szenen/szene-012.json",
    ] {
        let text = fs::read_to_string(dir.join(rel)).unwrap();
        assert!(text.ends_with('\n'), "{rel}");
    }

    let temp = fs::read_dir(dir.join("szenen")).unwrap().count();
    assert_eq!(temp, 1);
}

#[test]
fn removes_stale_files_only() {
    let dir = test_dir("stale");
    let mut project = example();
    let mut extra = project.scenes[0].clone();
    extra.id = "szene-013".into();
    project.scenes.push(extra);
    let mut place = project.locations[0].clone();
    place.id = "hof".into();
    project.locations.push(place);
    save(&dir, &project, None).unwrap();
    fs::write(dir.join("drehorte/hof.png"), b"bg").unwrap();
    fs::write(dir.join("notizen.txt"), b"n").unwrap();

    save(&dir, &example(), None).unwrap();

    assert!(!dir.join("szenen/szene-013.json").exists());
    assert!(!dir.join("drehorte/hof.toml").exists());
    assert!(dir.join("szenen/szene-012.json").exists());
    assert!(dir.join("drehorte/saengersaal.toml").exists());
    assert!(dir.join("drehorte/hof.png").exists());
    assert!(dir.join("notizen.txt").exists());
}

#[test]
fn bad_ids_are_rejected() {
    for id in ["", "a/b", "a\\b", "..", "a..b", ".hidden", "../x"] {
        let dir = test_dir("bad-id");
        let mut project = example();
        project.scenes[0].id = id.into();

        let result = save(&dir, &project, None);
        assert!(matches!(result, Err(StoreError::BadId { .. })), "{id:?}");
        assert!(!dir.exists(), "nothing written for {id:?}");

        let mut project = example();
        project.locations[0].id = id.into();
        let result = save(&dir, &project, None);
        assert!(matches!(result, Err(StoreError::BadId { .. })), "{id:?}");
    }
}

#[test]
fn duplicate_ids_are_rejected() {
    let dir = test_dir("dup-id");
    let mut project = example();
    let scene = project.scenes[0].clone();
    project.scenes.push(scene);

    let result = save(&dir, &project, None);

    assert!(matches!(result, Err(StoreError::DuplicateId { .. })));
}

#[test]
fn newer_format_is_rejected() {
    let dir = test_dir("newer");
    save(&dir, &example(), None).unwrap();
    let newer = FORMAT_VERSION + 1;
    fs::write(
        dir.join("projekt.toml"),
        format!("format_version = {newer}\ntitle = \"x\"\n"),
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(matches!(error, StoreError::NewerFormat { found, .. } if found == newer));
    assert!(error.to_string().contains("projekt.toml"));
}

#[test]
fn missing_script_is_none() {
    let dir = test_dir("no-script");
    save(&dir, &example(), None).unwrap();

    let loaded = load(&dir).unwrap();

    assert_eq!(loaded.script, None);
}

#[test]
fn none_keeps_existing_script() {
    let dir = test_dir("keep-script");
    save(&dir, &example(), Some(SCRIPT)).unwrap();

    save(&dir, &example(), None).unwrap();

    assert_eq!(load(&dir).unwrap().script.as_deref(), Some(SCRIPT));
}

#[test]
fn missing_project_file_names_it() {
    let dir = test_dir("empty");
    fs::create_dir_all(&dir).unwrap();

    let error = load(&dir).unwrap_err();

    assert!(error.to_string().contains("projekt.toml"));
}

#[test]
fn id_must_match_file_name() {
    let dir = test_dir("mismatch");
    save(&dir, &example(), None).unwrap();
    fs::rename(
        dir.join("szenen/szene-012.json"),
        dir.join("szenen/andere.json"),
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(matches!(error, StoreError::IdMismatch { .. }));
}

#[test]
fn broken_file_names_it() {
    let dir = test_dir("broken");
    save(&dir, &example(), None).unwrap();
    fs::write(dir.join("szenen/szene-012.json"), "{").unwrap();

    let error = load(&dir).unwrap_err();

    assert!(error.to_string().contains("szene-012.json"));
}

#[test]
fn float_round_trip_is_exact() {
    let dir = test_dir("float");
    let mut project = example();
    project.scenes[0].plans[0].figures[0].marks[0].at.x_m = 14.815548635762573;

    save(&dir, &project, None).unwrap();

    assert_eq!(load(&dir).unwrap().project, project);
}

#[test]
fn non_finite_numbers_are_refused_before_any_write() {
    for value in [f64::NAN, f64::INFINITY] {
        let dir = test_dir("non-finite");
        save(&dir, &example(), None).unwrap();
        let before = fs::read_to_string(dir.join("projekt.toml")).unwrap();

        let mut project = example();
        project.title = "Neu".into();
        project.scenes[0].plans[0].figures[0].marks[0].at_s = value;
        let result = save(&dir, &project, None);

        assert!(matches!(result, Err(StoreError::Encode { .. })), "{value}");
        assert_eq!(
            fs::read_to_string(dir.join("projekt.toml")).unwrap(),
            before
        );
        assert_eq!(load(&dir).unwrap().project, example());
    }
}

#[test]
fn newer_format_wins_over_missing_title() {
    let dir = test_dir("newer-no-title");
    fs::create_dir_all(&dir).unwrap();
    let newer = FORMAT_VERSION + 1;
    fs::write(
        dir.join("projekt.toml"),
        format!("format_version = {newer}\n"),
    )
    .unwrap();

    let error = load(&dir).unwrap_err();

    assert!(matches!(error, StoreError::NewerFormat { .. }), "{error}");
}

#[test]
fn windows_invalid_ids_are_rejected() {
    let ids = [
        "12:A", "con", "CON", "aux", "nul", "prn", "com1", "LPT9", "con.txt", "a.", "a ", "a<b",
        "a>b", "a\"b", "a|b", "a?b", "a*b",
    ];
    for id in ids {
        let dir = test_dir("win-id");
        let mut project = example();
        project.scenes[0].id = id.into();

        let result = save(&dir, &project, None);

        assert!(matches!(result, Err(StoreError::BadId { .. })), "{id:?}");
    }
}

#[test]
fn ids_equal_ignoring_case_are_duplicates() {
    let dir = test_dir("case-id");
    let mut project = example();
    let mut scene = project.scenes[0].clone();
    scene.id = "SZENE-012".into();
    project.scenes.push(scene);
    let result = save(&dir, &project, None);
    assert!(matches!(result, Err(StoreError::DuplicateId { .. })));

    let mut project = example();
    let mut place = project.locations[0].clone();
    place.id = "Saengersaal".into();
    project.locations.push(place);
    let result = save(&dir, &project, None);
    assert!(matches!(result, Err(StoreError::DuplicateId { .. })));
}

#[test]
fn save_script_writes_nothing_else() {
    let dir = test_dir("save-script");
    save(&dir, &example(), Some(SCRIPT)).unwrap();
    let foreign = dir.join("szenen/neu.json");
    fs::write(&foreign, "{}\n").unwrap();
    let project = fs::read(dir.join("projekt.toml")).unwrap();

    save_script(&dir, "Neu.\n").unwrap();

    assert!(foreign.exists());
    assert_eq!(fs::read(dir.join("projekt.toml")).unwrap(), project);
    assert_eq!(
        fs::read_to_string(dir.join("drehbuch.fountain")).unwrap(),
        "Neu.\n"
    );
}
