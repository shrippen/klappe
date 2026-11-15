//! Tests of `tools`.

use super::*;
use crate::catalog::Language;
use crate::mark::{aim_edit, hold_edit};
use crate::object::{object_height_edit, resize_edit};

/// Review of PR 27: a figure on its own path handle was grabbed on the first step and
/// the handle on the next, so the drag bent the path instead of moving the figure.
#[test]
fn a_drag_keeps_what_it_grabbed() {
    let figure = Source::Figure("burgvogt".into());
    let handle = Source::FigureVia {
        character: "burgvogt".into(),
        index: 1,
    };

    let (grip, first) = Grip::Free.take(|| Some(figure.clone()));
    let (_, next) = grip.take(|| Some(handle.clone()));

    assert_eq!(first, Some(figure.clone()));
    assert_eq!(next, Some(figure));
}

/// Review of PR 27: the grip stays held while dragging and is free after Done.
#[test]
fn a_grip_is_free_after_done() {
    let held = Grip::Held(Some(Source::Figure("burgvogt".into())));

    assert_eq!(held.clone().after(DragPhase::Moving), held);
    assert_eq!(held.after(DragPhase::Done), Grip::Free);
}

/// Review of PR 21: with the notes layer locked the tool still asked for a text and
/// dropped it on Enter.
#[test]
fn a_locked_notes_layer_asks_nothing() {
    assert!(matches!(
        note_open(Lock::Locked),
        Err(ServiceError::Locked(Layer::Notes))
    ));
    assert!(note_open(Lock::Unlocked).is_ok());
}

/// Review of PR 19: scale points snapped to the 25 cm grid gave a wrong scale
/// (2.8 m picked, 3.0 m used). Tools that place things still snap.
#[test]
fn scale_points_do_not_snap() {
    let snap = |p: Point| Point::new(p.x_m.round(), p.y_m.round());
    let at = Point::new(1.1, 1.0);

    assert_eq!(lands_at(&Tool::Scale, at, snap), at);
    assert_eq!(
        lands_at(&tool("camera", ""), at, snap),
        Point::new(1.0, 1.0)
    );
}

/// Review of PR 19: undoing the background left the picked points and the length
/// field; a valid length then failed as if it were invalid.
#[test]
fn scale_points_go_with_the_background() {
    let both = ScaleStep::Both(Point::new(0.0, 0.0), Point::new(1.0, 0.0));

    assert_eq!(both.under(Backdrop::Missing), ScaleStep::Idle);
    assert_eq!(both.under(Backdrop::Present), both);
}

/// Review of PR 13: a drag that ends in no edit (a wall snapped to length 0) or whose
/// preview fails must show the project again, not the last preview.
#[test]
fn a_drag_without_edit_restores_the_plan() {
    let shown = Preview::Shown;
    assert_eq!(drag_step(None, DragPhase::Moving, shown), DragStep::Restore);
    assert_eq!(drag_step(None, DragPhase::Done, shown), DragStep::Restore);

    let edit = PlanEdit::Remove {
        source: Source::Wall(0),
    };
    assert_eq!(
        drag_step(Some(edit.clone()), DragPhase::Moving, shown),
        DragStep::Preview(edit.clone())
    );
    assert_eq!(
        drag_step(Some(edit.clone()), DragPhase::Done, shown),
        DragStep::Commit(edit)
    );
}

/// Review of PR 13: a drag with a tool that places (camera, light …) has no edit and
/// never showed a preview; every mouse move still refreshed the whole plan.
#[test]
fn a_drag_without_preview_on_screen_redraws_nothing() {
    let hidden = Preview::Hidden;

    assert_eq!(
        drag_step(None, DragPhase::Moving, hidden),
        DragStep::Nothing
    );
    assert_eq!(drag_step(None, DragPhase::Done, hidden), DragStep::Nothing);
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn tool(name: &str, choice: &str) -> Tool {
    Tool::parse(name, choice).unwrap_or(Tool::Select)
}

#[test]
fn tools_are_named_as_the_tool_bar_spells_them() {
    assert_eq!(Tool::parse("select", ""), Some(Tool::Select));
    assert_eq!(Tool::parse("wall", ""), Some(Tool::Wall));
    assert_eq!(Tool::parse("camera", ""), Some(Tool::Camera(String::new())));
    assert_eq!(Tool::parse("camera", "a"), Some(Tool::Camera("a".into())));
    assert_eq!(
        Tool::parse("object", "table"),
        Some(Tool::Object("table".into()))
    );
    assert_eq!(
        Tool::parse("figure", "elisabeth"),
        Some(Tool::Figure("elisabeth".into()))
    );
    assert_eq!(Tool::parse("pencil", ""), None);
}

#[test]
fn every_lamp_kind_of_the_list_parses() {
    for kind in LightKind::ALL {
        assert_eq!(Tool::parse("light", kind.name()), Some(Tool::Light(kind)));
    }
    assert_eq!(Tool::parse("light", "laser"), None);
    assert_eq!(Tool::parse("light", ""), None);
}

/// The tool bar translates each kind with `lightKind.<name>`; the objects are named by the
/// library (services tests).
#[test]
fn every_lamp_kind_has_a_name_in_both_languages() {
    let keys = LightKind::ALL
        .iter()
        .map(|k| format!("lightKind.{}", k.name()));

    for key in keys {
        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key);
        }
    }
}

#[test]
fn a_click_sets_what_the_tool_stands_for() {
    let at = p(2.0, 3.0);

    assert_eq!(
        click_edit(&tool("object", "bed"), at),
        Some(PlanEdit::AddObject {
            kind: "bed".into(),
            at
        })
    );
    assert_eq!(
        click_edit(&tool("figure", "elisabeth"), at),
        Some(PlanEdit::AddMark {
            character: "elisabeth".into(),
            at
        })
    );
    assert_eq!(
        click_edit(&tool("camera", ""), at),
        Some(PlanEdit::AddCamera {
            at,
            facing_deg: 0.0
        })
    );
    assert_eq!(
        click_edit(&tool("camera", "a"), at),
        Some(PlanEdit::AddCameraMark {
            camera: "a".into(),
            at
        })
    );
    assert_eq!(
        click_edit(&tool("light", "tube"), at),
        Some(PlanEdit::AddLight {
            kind: LightKind::Tube,
            at
        })
    );
}

#[test]
fn a_click_with_select_or_wall_or_no_choice_edits_nothing() {
    let at = p(2.0, 3.0);

    assert_eq!(click_edit(&Tool::Select, at), None);
    assert_eq!(click_edit(&Tool::Wall, at), None);
    assert_eq!(click_edit(&tool("figure", ""), at), None);
    assert_eq!(click_edit(&tool("object", ""), at), None);
}

/// From (4, 5) to (6.5, 4): the grabbed thing moves 2.5 right and 1 up (y is down).
#[test]
fn select_shifts_by_the_distance_dragged() {
    let grabbed = Source::Camera("a".into());

    let edit = drag_edit(&Tool::Select, Some(&grabbed), p(4.0, 5.0), p(6.5, 4.0));

    assert_eq!(
        edit,
        Some(PlanEdit::Shift {
            source: grabbed,
            dx_m: 2.5,
            dy_m: -1.0
        })
    );
}

#[test]
fn select_on_empty_ground_does_nothing() {
    assert_eq!(
        drag_edit(&Tool::Select, None, p(0.0, 0.0), p(1.0, 1.0)),
        None
    );
}

#[test]
fn wall_draws_from_start_to_end_also_over_a_shape() {
    let over = Source::Object("tisch".into());
    let (from, to) = (p(1.0, 1.0), p(5.0, 1.0));

    let on_ground = drag_edit(&Tool::Wall, None, from, to);
    let on_shape = drag_edit(&Tool::Wall, Some(&over), from, to);

    assert_eq!(on_ground, Some(PlanEdit::AddWall { from, to }));
    assert_eq!(on_shape, on_ground);
}

#[test]
fn a_wall_of_no_length_is_not_drawn() {
    assert_eq!(drag_edit(&Tool::Wall, None, p(1.0, 1.0), p(1.0, 1.0)), None);
}

#[test]
fn placing_tools_do_not_drag() {
    let edit = drag_edit(&tool("camera", ""), None, p(0.0, 0.0), p(3.0, 0.0));

    assert_eq!(edit, None);
}

#[test]
fn scale_is_a_tool_and_a_click_edits_nothing() {
    assert_eq!(Tool::parse("scale", ""), Some(Tool::Scale));
    assert_eq!(click_edit(&Tool::Scale, p(1.0, 1.0)), None);
}

#[test]
fn scale_drags_the_background() {
    let edit = drag_edit(&Tool::Scale, None, p(1.0, 1.0), p(3.0, 0.5));

    assert_eq!(
        edit,
        Some(PlanEdit::MoveBackground {
            dx_m: 2.0,
            dy_m: -0.5
        })
    );
    assert_eq!(
        drag_edit(&Tool::Scale, None, p(1.0, 1.0), p(1.0, 1.0)),
        None
    );
}

#[test]
fn two_clicks_pick_two_points_and_a_third_starts_over() {
    let (a, b, c) = (p(1.0, 1.0), p(4.0, 1.0), p(0.0, 2.0));

    let first = ScaleStep::Idle.click(a);
    let both = first.click(b);

    assert_eq!(first, ScaleStep::First(a));
    assert_eq!(both, ScaleStep::Both(a, b));
    assert_eq!(both.points(), vec![a, b]);
    assert_eq!(both.click(c), ScaleStep::First(c));
    assert!(ScaleStep::Idle.points().is_empty());
}

#[test]
fn measure_is_a_tool_and_a_click_edits_nothing() {
    assert_eq!(Tool::parse("measure", ""), Some(Tool::Measure));
    assert_eq!(click_edit(&Tool::Measure, p(1.0, 1.0)), None);
    assert_eq!(
        drag_edit(&Tool::Measure, None, p(0.0, 0.0), p(2.0, 2.0)),
        None
    );
}

#[test]
fn measure_clicks_set_two_points_and_a_third_starts_over() {
    let (a, b, c) = (p(1.0, 1.0), p(4.0, 5.0), p(0.0, 2.0));

    let first = MeasureStep::Idle.click(a);
    let held = first.click(b);

    assert_eq!(first, MeasureStep::Start(a));
    assert_eq!(held, MeasureStep::Held(a, b));
    assert_eq!(held.click(c), MeasureStep::Start(c));
}

#[test]
fn measure_follows_the_pointer_until_held() {
    let (a, b, c) = (p(0.0, 0.0), p(3.0, 4.0), p(6.0, 8.0));

    let live = MeasureStep::Start(a).hover(b);

    assert_eq!(live, MeasureStep::Live(a, b));
    assert_eq!(live.hover(c), MeasureStep::Live(a, c));
    assert_eq!(live.click(c), MeasureStep::Held(a, c));
    assert_eq!(MeasureStep::Held(a, b).hover(c), MeasureStep::Held(a, b));
    assert_eq!(MeasureStep::Idle.hover(c), MeasureStep::Idle);
}

#[test]
fn a_second_measure_click_on_the_first_point_is_no_line() {
    let a = p(1.0, 1.0);

    assert_eq!(MeasureStep::Start(a).click(a), MeasureStep::Start(a));
    assert_eq!(
        MeasureStep::Live(a, p(2.0, 2.0)).click(a),
        MeasureStep::Start(a)
    );
}

#[test]
fn measure_clicks_land_on_the_grid_when_snapping() {
    let snap = |at: Point| p(at.x_m.round(), at.y_m.round());

    assert_eq!(lands_at(&Tool::Measure, p(1.2, 2.7), snap), p(1.0, 3.0));
}

#[test]
fn measure_length_is_the_distance_of_its_points() {
    let (a, b) = (p(0.0, 0.0), p(3.0, 4.0));

    assert_eq!(MeasureStep::Idle.length_m(), None);
    assert_eq!(MeasureStep::Start(a).length_m(), None);
    assert_eq!(MeasureStep::Live(a, b).length_m(), Some(5.0));
    assert_eq!(MeasureStep::Held(a, b).length_m(), Some(5.0));
    assert!(MeasureStep::Idle.points().is_empty());
    assert_eq!(MeasureStep::Held(a, b).points(), vec![a, b]);
}

#[test]
fn lengths_read_with_catalogue_separator_and_unit() {
    assert_eq!(length_text(4.25, ",", "m"), "4,25 m");
    assert_eq!(length_text(4.0, ",", "m"), "4 m");
    assert_eq!(length_text(1.5, ".", "m"), "1.5 m");
    assert_eq!(length_text(0.004, ".", "m"), "0 m");
}

#[test]
fn a_second_click_on_the_first_point_is_no_stretch() {
    let a = p(1.0, 1.0);

    assert_eq!(ScaleStep::First(a).click(a), ScaleStep::First(a));
}

#[test]
fn metres_take_comma_or_point() {
    assert_eq!(length_m(Field::Width, "4,5").ok(), Some(4.5));
    assert_eq!(length_m(Field::Width, " 4.5 ").ok(), Some(4.5));
    assert_eq!(length_m(Field::Width, "12").ok(), Some(12.0));
    assert_eq!(length_m(Field::Width, "0").ok(), None);
    assert_eq!(length_m(Field::Width, "-3").ok(), None);
    assert_eq!(length_m(Field::Width, "1,234.5").ok(), None);
    assert_eq!(length_m(Field::Width, "abc").ok(), None);
    assert_eq!(length_m(Field::Width, "").ok(), None);
    assert_eq!(length_m(Field::Width, "inf").ok(), None);
}

/// Audit, Fund 2: a field takes its own unit, as the lens in the shot list always did.
#[test]
fn typed_units_are_taken_by_the_plan_tools() {
    let object = Source::Object("o".into());
    let camera = Source::Camera("a".into());
    let (a, b) = (p(1.0, 1.0), p(4.0, 1.0));

    assert!(edit_of(resize_edit(Some(&object), "2 m", "1,5 m")).is_some());
    assert!(edit_of(object_height_edit(Some(&object), "1 m")).is_some());
    assert!(edit_of(scale_edit(ScaleStep::Both(a, b), "3 m")).is_some());
    assert!(edit_of(hold_edit(Some(&camera), "2 s")).is_some());
    assert!(edit_of(aim_edit(Some(&camera), "1 m", "10°")).is_some());
    assert!(edit_of(resize_edit(Some(&object), "2 s", "1")).is_none());
}

#[test]
fn the_scale_edit_needs_both_points_and_a_length() {
    let (a, b) = (p(1.0, 1.0), p(4.0, 1.0));

    assert_eq!(
        edit_of(scale_edit(ScaleStep::Both(a, b), "3,5")),
        Some(PlanEdit::Calibrate {
            from: a,
            to: b,
            distance_m: 3.5
        })
    );
    assert_eq!(edit_of(scale_edit(ScaleStep::Both(a, b), "x")), None);
    assert_eq!(edit_of(scale_edit(ScaleStep::First(a), "3")), None);
    assert_eq!(edit_of(scale_edit(ScaleStep::Idle, "3")), None);
}

mod notes_and_shapes;
