//! What a drawing tool does with a click, a drag or a key: the tool, the gesture and the
//! choice made in the tool bar become one plan edit. No geometry beyond the two points a
//! gesture hands over (agent.md); the session snaps them and does the work.

use klappe_services::{LightKind, PlanEdit, Point, Source};

/// A new camera looks to the right (0°); the user turns it afterwards.
const NEW_CAMERA_FACING_DEG: f64 = 0.0;

/// Lamp kinds the tool bar offers, with the name QML and the catalogue (`lightKind.<name>`) use.
pub const LIGHT_KINDS: [(&str, LightKind); 6] = [
    ("fresnel", LightKind::Fresnel),
    ("led-panel", LightKind::LedPanel),
    ("tube", LightKind::Tube),
    ("balloon", LightKind::Balloon),
    ("reflector", LightKind::Reflector),
    ("flag", LightKind::Flag),
];

/// Object kinds the tool bar offers (`objectKind.<name>` in the catalogue).
pub const OBJECT_KINDS: [&str; 7] = [
    "table",
    "chair",
    "bed",
    "door",
    "window",
    "pillar",
    "fireplace",
];

/// The tool of the tool bar together with what was picked for it.
#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Select,
    Wall,
    /// Kind of the object a click sets.
    Object(String),
    /// Id of the character whose next mark a click sets.
    Figure(String),
    Camera,
    Light(LightKind),
}

impl Tool {
    /// `name` as the tool bar spells it (`select`, `wall` …); `choice` is the value picked in
    /// the tool's list (object kind, character id, lamp kind). `None` for an unknown name or
    /// a choice the tool cannot use.
    pub fn parse(name: &str, choice: &str) -> Option<Tool> {
        match name {
            "select" => Some(Tool::Select),
            "wall" => Some(Tool::Wall),
            "object" => Some(Tool::Object(choice.to_owned())),
            "figure" => Some(Tool::Figure(choice.to_owned())),
            "camera" => Some(Tool::Camera),
            "light" => LIGHT_KINDS
                .iter()
                .find(|(n, _)| *n == choice)
                .map(|(_, kind)| Tool::Light(*kind)),
            _ => None,
        }
    }
}

/// What the keys do to the selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Remove,
    /// Clockwise for a positive angle.
    Turn {
        by_deg: f64,
    },
}

/// A click at `at`: the tools that put something there. Select only selects, which is no edit.
pub fn click_edit(tool: &Tool, at: Point) -> Option<PlanEdit> {
    match tool {
        Tool::Select | Tool::Wall => None,
        Tool::Object(kind) if kind.is_empty() => None,
        Tool::Object(kind) => Some(PlanEdit::AddObject {
            kind: kind.clone(),
            at,
        }),
        Tool::Figure(character) if character.is_empty() => None,
        Tool::Figure(character) => Some(PlanEdit::AddMark {
            character: character.clone(),
            at,
        }),
        Tool::Camera => Some(PlanEdit::AddCamera {
            at,
            facing_deg: NEW_CAMERA_FACING_DEG,
        }),
        Tool::Light(kind) => Some(PlanEdit::AddLight { kind: *kind, at }),
    }
}

/// A drag from `from` to `to` that began on `grabbed` (`None`: empty ground). Select moves what
/// it grabbed by the distance dragged, so the grab point stays under the pointer; Wall draws
/// from start to end wherever it began. The points of a wall are expected snapped already.
pub fn drag_edit(
    tool: &Tool,
    grabbed: Option<&Source>,
    from: Point,
    to: Point,
) -> Option<PlanEdit> {
    match tool {
        Tool::Select => grabbed.map(|source| PlanEdit::Shift {
            source: source.clone(),
            dx_m: to.x_m - from.x_m,
            dy_m: to.y_m - from.y_m,
        }),
        Tool::Wall if from == to => None,
        Tool::Wall => Some(PlanEdit::AddWall { from, to }),
        _ => None,
    }
}

/// Where a drag stands: still moving, or let go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    Moving,
    Done,
}

/// What one step of a drag shows.
#[derive(Debug, Clone, PartialEq)]
pub enum DragStep {
    /// The plan as the edit would leave it; the project stays.
    Preview(PlanEdit),
    /// The edit applied to the project.
    Commit(PlanEdit),
    /// The project as it is: no edit, so no earlier preview may stay on screen.
    Restore,
}

pub fn drag_step(edit: Option<PlanEdit>, phase: DragPhase) -> DragStep {
    match (edit, phase) {
        (None, _) => DragStep::Restore,
        (Some(edit), DragPhase::Moving) => DragStep::Preview(edit),
        (Some(edit), DragPhase::Done) => DragStep::Commit(edit),
    }
}

/// A key on the selection; `None` when nothing is selected.
pub fn key_edit(key: Key, selected: Option<&Source>) -> Option<PlanEdit> {
    let source = selected?.clone();

    match key {
        Key::Remove => Some(PlanEdit::Remove { source }),
        Key::Turn { by_deg } => Some(PlanEdit::Spin { source, by_deg }),
    }
}

#[cfg(test)]
mod tests {

    /// Review of PR 13: a drag that ends in no edit (a wall snapped to length 0) or whose
    /// preview fails must show the project again, not the last preview.
    #[test]
    fn a_drag_without_edit_restores_the_plan() {
        assert_eq!(drag_step(None, DragPhase::Moving), DragStep::Restore);
        assert_eq!(drag_step(None, DragPhase::Done), DragStep::Restore);

        let edit = PlanEdit::Remove {
            source: Source::Wall(0),
        };
        assert_eq!(
            drag_step(Some(edit.clone()), DragPhase::Moving),
            DragStep::Preview(edit.clone())
        );
        assert_eq!(
            drag_step(Some(edit.clone()), DragPhase::Done),
            DragStep::Commit(edit)
        );
    }

    use super::*;
    use crate::catalog::Language;

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
        assert_eq!(Tool::parse("camera", ""), Some(Tool::Camera));
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
        for (name, kind) in LIGHT_KINDS {
            assert_eq!(Tool::parse("light", name), Some(Tool::Light(kind)));
        }
        assert_eq!(Tool::parse("light", "laser"), None);
        assert_eq!(Tool::parse("light", ""), None);
    }

    /// The tool bar translates each kind with `objectKind.<name>` and `lightKind.<name>`.
    #[test]
    fn every_kind_has_a_name_in_both_languages() {
        let keys = OBJECT_KINDS
            .iter()
            .map(|k| format!("objectKind.{k}"))
            .chain(LIGHT_KINDS.iter().map(|(k, _)| format!("lightKind.{k}")));

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
            click_edit(&Tool::Camera, at),
            Some(PlanEdit::AddCamera {
                at,
                facing_deg: 0.0
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
        let edit = drag_edit(&Tool::Camera, None, p(0.0, 0.0), p(3.0, 0.0));

        assert_eq!(edit, None);
    }

    #[test]
    fn keys_act_on_the_selection_only() {
        let selected = Source::Light("led-1".into());

        assert_eq!(
            key_edit(Key::Remove, Some(&selected)),
            Some(PlanEdit::Remove {
                source: selected.clone()
            })
        );
        assert_eq!(
            key_edit(Key::Turn { by_deg: -15.0 }, Some(&selected)),
            Some(PlanEdit::Spin {
                source: selected,
                by_deg: -15.0
            })
        );
        assert_eq!(key_edit(Key::Remove, None), None);
    }
}
