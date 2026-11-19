//! Tests of what the controller shows.

use super::*;
use crate::bridge::tests::example;
use crate::catalog::Language;
use klappe_services::{Lock, Visibility};

/// The object tool lists the library: sizes in metres, built in first.
#[test]
fn the_object_list_is_the_library() {
    let s = example();

    let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
    let objects = choices["object"].as_array().unwrap();

    assert_eq!(objects[0]["kind"], "table");
    assert_eq!(objects[0]["width_m"], 1.6);
    assert_eq!(objects[0]["depth_m"], 0.9);
    assert_eq!(objects[0]["shape"], "rect");
    assert_eq!(objects[0]["custom"], false);
    assert_eq!(objects[0]["height_m"], 0.75);
    // The group the list shows it under, and how many objects of the example use it.
    assert_eq!(objects[0]["group"], "furniture");
    assert_eq!(objects[0]["uses"], 1);
    assert!(objects
        .iter()
        .any(|o| o["kind"] == "horse" && o["shape"] == "ellipse"));
    assert_eq!(objects.len(), s.library().len());
}

/// The template list: built in first, with what each holds (LH-07).
#[test]
fn the_template_list_is_the_built_in_templates() {
    let s = example();

    let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
    let templates = choices["template"].as_array().unwrap();
    let ids: Vec<_> = templates
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();

    assert_eq!(ids, ["shot-reverse", "triangle", "long-take"]);
    assert_eq!(templates[0]["figures"], 2);
    assert_eq!(templates[0]["cameras"], 3);
    assert_eq!(templates[0]["custom"], false);
}

/// The camera tool lists the cameras of the plan, to add a mark to one of them.
#[test]
fn the_camera_list_is_the_plans_cameras() {
    let s = example();

    let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();

    assert_eq!(choices["cameras"][0], json!({ "id": "a", "label": "A" }));
    assert_eq!(
        choices["cameras"].as_array().unwrap().len(),
        s.plan().unwrap().cameras.len()
    );
}

/// The tool bar's lists, by the names the file and the catalogue use.
#[test]
fn the_tool_lists_are_the_file_names() {
    let s = example();

    let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();

    assert_eq!(
        choices["light"],
        json!([
            "fresnel-650",
            "fresnel-1k",
            "fresnel-2k",
            "led-panel",
            "tube",
            "balloon",
            "reflector",
            "floppy",
            "flag"
        ])
    );
    assert_eq!(choices["dash"], json!(["solid", "dashed", "dotted"]));
    assert_eq!(choices["fill"], json!(["outline", "filled"]));
    assert_eq!(
        choices["rig"],
        json!(["tripod", "dolly", "track", "crane", "handheld", "gimbal"])
    );
    assert_eq!(choices["size"][0], "extreme-wide");
    assert_eq!(
        choices["size"].as_array().unwrap().len(),
        ShotSize::ALL.len()
    );
    assert_eq!(choices["angle"][0], "eye-level");
    assert_eq!(choices["angle"].as_array().unwrap().len(), Angle::ALL.len());
    assert_eq!(choices["movement"][0], "static");
    assert_eq!(
        choices["movement"].as_array().unwrap().len(),
        Movement::ALL.len()
    );
    assert_eq!(
        choices["color"],
        json!(["red", "orange", "yellow", "green", "cyan", "blue", "purple"])
    );
}

/// QML keeps one switch per layer: `layersJson` and `locksJson` map the layer name to it.
#[test]
fn layers_and_locks_are_listed_by_name() {
    let mut s = example();
    s.set_layer(Layer::Notes, Visibility::Hidden);
    s.set_lock(Layer::Cameras, Lock::Locked);

    let shown: serde_json::Value = serde_json::from_str(&layers_json(&s)).unwrap();
    let locked: serde_json::Value = serde_json::from_str(&locks_json(&s)).unwrap();

    let names = [
        "set", "figures", "cameras", "lights", "notes", "drawings", "sketches",
    ];
    assert_eq!(shown.as_object().unwrap().len(), names.len());
    assert_eq!(locked.as_object().unwrap().len(), names.len());
    for name in names {
        assert_eq!(shown[name], name != "notes", "{name}");
        assert_eq!(locked[name], name == "cameras", "{name}");
    }
}

#[test]
fn shift_makes_a_pick_extend_the_selection() {
    let mut c = ControllerRust::default();
    assert_eq!(c.pick_mode(), Pick::Replace);

    c.extend_selection = true;

    assert_eq!(c.pick_mode(), Pick::Extend);
}

#[test]
fn ctrl_makes_a_drag_on_empty_ground_a_lasso() {
    let mut c = ControllerRust::default();
    assert_eq!(c.reach(), Reach::Box);

    c.lasso_selection = true;

    assert_eq!(c.reach(), Reach::Lasso);
}

/// A measurement shows length and direction; one point alone shows nothing to read.
#[test]
fn the_measurement_caption_has_length_and_angle() {
    let mut c = ControllerRust {
        language: Language::German,
        ..ControllerRust::default()
    };
    let (a, b) = (Point::new(0.0, 0.0), Point::new(3.0, 3.0));

    c.measure = MeasureStep::Start(a);
    assert_eq!(c.measure_view().unwrap().caption, "");

    c.measure = MeasureStep::Held(a, b);
    assert_eq!(c.measure_view().unwrap().caption, "4,24 m · 45°");
}

/// Review of PR 23: area, scale and note drafts outlived a change of scene or project;
/// Enter then made an area from the old plan's points in the new one.
#[test]
fn drafts_go_with_the_plan() {
    let mut c = ControllerRust {
        area: AreaDraft::default().click(Point::new(1.0, 1.0)),
        scale: ScaleStep::Idle.click(Point::new(2.0, 2.0)),
        measure: MeasureStep::Idle.click(Point::new(4.0, 4.0)),
        note: NoteDraft::New(Point::new(3.0, 3.0)),
        selected: Selection::of(Some(Source::Wall(0))),
        ..ControllerRust::default()
    };

    c.drop_drafts();

    assert_eq!(c.measure, MeasureStep::Idle);
    assert!(c.area.is_empty());
    assert_eq!(c.scale, ScaleStep::Idle);
    assert_eq!(c.note, NoteDraft::Idle);
    assert!(c.selected.is_empty());
}

/// The start with a project shows what the first refresh would: every value from the
/// session, none of the tool or draft state.
#[test]
fn the_start_fills_every_value() {
    let mut c = ControllerRust {
        session: Some(example()),
        ..ControllerRust::default()
    };
    c.fill();

    assert!(c.loaded);
    assert_eq!(c.title.to_string(), "Die Wartburg-Szene");
    assert_eq!(
        c.plan_title.to_string(),
        "12 · INT. SÄNGERSAAL – TAG · Plan A"
    );
    assert_eq!(c.scene_id.to_string(), "m4djbtj6");
    assert_eq!(c.duration, 20.0);
    assert_eq!(c.time, 0.0);
    assert_eq!(c.selected_shot.to_string(), "");
    assert!(c.snapping_on);
    assert_eq!(c.grid_name.to_string(), "cm25");
    assert!(!c.has_background);
    assert_eq!(c.backup_state.to_string(), "none");
    assert_eq!(c.backup_message.to_string(), "");
    assert!(!c.can_undo && !c.can_redo && !c.dirty);
    assert_eq!(c.selected_key.to_string(), "");
    assert_eq!(c.selected_count, 0);
    assert_eq!((c.scale_points, c.area_points), (0, 0));
    assert!(!c.asks_note && !c.measuring);
    for json in [
        &c.plan_json,
        &c.tracks_json,
        &c.shots_json,
        &c.shots_shooting_json,
        &c.script_json,
        &c.layers_json,
        &c.locks_json,
        &c.choices_json,
        &c.project_json,
    ] {
        assert!(!json.to_string().is_empty());
    }
}

/// Review of PR 23, round 4: after a start with `--open` the view on screen was not
/// noted, so the first refresh dropped the selection or draft it was to show.
#[test]
fn the_first_refresh_keeps_its_draft() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut c = ControllerRust {
        session: Some(Session::open(&dir).unwrap()),
        ..ControllerRust::default()
    };
    c.fill();
    c.area = AreaDraft::default().click(Point::new(1.0, 1.0));

    let shown = shown_view(c.session.as_ref().unwrap());
    c.follow_view(shown);

    assert!(!c.area.is_empty());
}

/// LH-04: choosing, adding or copying a plan changes the plan on screen; a draft made on
/// the old plan must not be finished on the new one.
#[test]
fn drafts_go_when_another_plan_is_shown() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut session = Session::open(&dir).unwrap();
    let mut c = ControllerRust {
        shown: shown_view(&session),
        area: AreaDraft::default().click(Point::new(1.0, 1.0)),
        ..ControllerRust::default()
    };
    c.follow_view(shown_view(&session));
    assert!(!c.area.is_empty());

    session.copy_plan("Stand 1").unwrap();
    c.follow_view(shown_view(&session));

    assert!(c.area.is_empty());
    assert_eq!(c.shown, "m4djbtj6/stand-1");
}

/// Review of PR 23, round 3: undo, redo or deleting the scene on view changed the scene
/// without a click; the drafts stayed and Enter drew in the other scene. Here only the
/// comparison; `refresh` calls it with `shown_view`.
#[test]
fn drafts_go_when_the_view_changes() {
    let mut c = ControllerRust {
        shown: "m4djbtj6/a".into(),
        area: AreaDraft::default().click(Point::new(1.0, 1.0)),
        ..ControllerRust::default()
    };

    c.follow_view("m4djbtj6/a".into());
    assert!(!c.area.is_empty());

    c.follow_view("szene-013/a".into());
    assert!(c.area.is_empty());
    assert_eq!(c.shown, "szene-013/a");
}

/// Review of PR 47, round 2: undo across a plan, or removing the plan on view, showed
/// another plan while play went on. Every change of the view is a switch.
#[test]
fn undo_across_a_plan_is_a_switch() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut session = Session::open(&dir).unwrap();
    session.copy_plan("Stand 1").unwrap();
    let mut c = ControllerRust {
        shown: shown_view(&session),
        ..ControllerRust::default()
    };

    assert_eq!(c.follow_view(shown_view(&session)), View::Same);
    session.undo();
    assert_eq!(c.follow_view(shown_view(&session)), View::Switched);
}
