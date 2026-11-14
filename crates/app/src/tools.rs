//! What a drawing tool does with a click, a drag or a key: the tool, the gesture and the
//! choice made in the tool bar become one plan edit. No geometry beyond the two points a
//! gesture hands over (agent.md); the session snaps them and does the work.

use std::path::PathBuf;

use klappe_services::{LightKind, PlanEdit, Point, Source};

const FILE_URL: &str = "file://";

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
    /// Two clicks on a stretch of known length scale the background; a drag moves it.
    Scale,
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
            "scale" => Some(Tool::Scale),
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
        Tool::Select | Tool::Wall | Tool::Scale => None,
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
/// from start to end wherever it began; Scale moves the background by the distance dragged.
/// The points of a wall are expected snapped already.
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
        Tool::Scale if from == to => None,
        Tool::Scale => Some(PlanEdit::MoveBackground {
            dx_m: to.x_m - from.x_m,
            dy_m: to.y_m - from.y_m,
        }),
        _ => None,
    }
}

/// How far the scale gesture has come: the points picked so far.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ScaleStep {
    #[default]
    Idle,
    First(Point),
    Both(Point, Point),
}

impl ScaleStep {
    /// A click at `at`: the first or the second point. A second click on the first point
    /// changes nothing (no stretch); a click after both points starts over.
    pub fn click(self, at: Point) -> ScaleStep {
        match self {
            ScaleStep::First(first) if first == at => self,
            ScaleStep::First(first) => ScaleStep::Both(first, at),
            ScaleStep::Idle | ScaleStep::Both(..) => ScaleStep::First(at),
        }
    }

    /// The points picked, for the preview on the plan.
    pub fn points(self) -> Vec<Point> {
        match self {
            ScaleStep::Idle => Vec::new(),
            ScaleStep::First(a) => vec![a],
            ScaleStep::Both(a, b) => vec![a, b],
        }
    }
}

/// A length in metres as typed: a comma or a point as decimal separator, more than zero.
pub fn parse_metres(text: &str) -> Option<f64> {
    let metres: f64 = text.trim().replace(',', ".").parse().ok()?;

    (metres.is_finite() && metres > 0.0).then_some(metres)
}

/// Both points picked and a length typed: the scale edit. `None` until then.
pub fn scale_edit(step: ScaleStep, distance: &str) -> Option<PlanEdit> {
    let ScaleStep::Both(from, to) = step else {
        return None;
    };

    Some(PlanEdit::Calibrate {
        from,
        to,
        distance_m: parse_metres(distance)?,
    })
}

/// `file://` URL or plain path as the file dialog hands it over, as a path. Percent-escapes
/// in a URL are decoded.
pub fn local_path(url: &str) -> PathBuf {
    let Some(rest) = url.strip_prefix(FILE_URL) else {
        return PathBuf::from(url);
    };

    let mut bytes = Vec::with_capacity(rest.len());
    let raw = rest.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        let escaped = (raw[i] == b'%')
            .then(|| raw.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| std::str::from_utf8(hex).ok())
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                bytes.push(byte);
                i += 3;
            }
            None => {
                bytes.push(raw[i]);
                i += 1;
            }
        }
    }

    let path = String::from_utf8_lossy(&bytes).into_owned();
    // `file:///C:/x`: no slash before a drive letter.
    match path.as_bytes() {
        [b'/', drive, b':', ..] if drive.is_ascii_alphabetic() => PathBuf::from(&path[1..]),
        _ => PathBuf::from(path),
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
    fn a_second_click_on_the_first_point_is_no_stretch() {
        let a = p(1.0, 1.0);

        assert_eq!(ScaleStep::First(a).click(a), ScaleStep::First(a));
    }

    #[test]
    fn metres_take_comma_or_point() {
        assert_eq!(parse_metres("4,5"), Some(4.5));
        assert_eq!(parse_metres(" 4.5 "), Some(4.5));
        assert_eq!(parse_metres("12"), Some(12.0));
        assert_eq!(parse_metres("0"), None);
        assert_eq!(parse_metres("-3"), None);
        assert_eq!(parse_metres("1,234.5"), None);
        assert_eq!(parse_metres("abc"), None);
        assert_eq!(parse_metres(""), None);
        assert_eq!(parse_metres("inf"), None);
    }

    #[test]
    fn the_scale_edit_needs_both_points_and_a_length() {
        let (a, b) = (p(1.0, 1.0), p(4.0, 1.0));

        assert_eq!(
            scale_edit(ScaleStep::Both(a, b), "3,5"),
            Some(PlanEdit::Calibrate {
                from: a,
                to: b,
                distance_m: 3.5
            })
        );
        assert_eq!(scale_edit(ScaleStep::Both(a, b), "x"), None);
        assert_eq!(scale_edit(ScaleStep::First(a), "3"), None);
        assert_eq!(scale_edit(ScaleStep::Idle, "3"), None);
    }

    #[test]
    fn the_dialog_url_becomes_a_path() {
        assert_eq!(
            local_path("file:///home/a%20b/Plan%C3%A4.png"),
            PathBuf::from("/home/a b/Planä.png")
        );
        assert_eq!(local_path("/home/x/p.png"), PathBuf::from("/home/x/p.png"));
        assert_eq!(
            local_path("file:///C:/x/p.png"),
            PathBuf::from("C:/x/p.png")
        );
        assert_eq!(local_path("file:///a%zz"), PathBuf::from("/a%zz"));
    }
}
