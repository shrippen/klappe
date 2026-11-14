//! What a drawing tool does with a click, a drag or a key: the tool, the gesture and the
//! choice made in the tool bar become one plan edit. No geometry beyond the two points a
//! gesture hands over (agent.md); the session snaps them and does the work.

use std::path::PathBuf;

use klappe_services::{
    Dash, DrawingShape, FigureColor, Fill, Layer, LightKind, Lock, PlanEdit, Point, ProjectEdit,
    ServiceError, Source,
};

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

/// Colours the drawing tools offer, in Kante's order of labels: the name QML and the
/// catalogue (`color.<name>`) use.
pub const DRAW_COLORS: [(&str, FigureColor); 7] = [
    ("red", FigureColor::Red),
    ("orange", FigureColor::Orange),
    ("yellow", FigureColor::Yellow),
    ("green", FigureColor::Green),
    ("cyan", FigureColor::Cyan),
    ("blue", FigureColor::Blue),
    ("purple", FigureColor::Purple),
];

/// Line styles of the drawing tools (`dash.<name>` in the catalogue).
pub const DRAW_DASHES: [(&str, Dash); 3] = [
    ("solid", Dash::Solid),
    ("dashed", Dash::Dashed),
    ("dotted", Dash::Dotted),
];

/// Fills of the drawing tools (`fill.<name>` in the catalogue).
pub const DRAW_FILLS: [(&str, Fill); 2] = [("outline", Fill::Outline), ("filled", Fill::Filled)];

/// How far from its first point a click still closes an area, in metres: the grid is 0.25 m,
/// so a click that snaps to a neighbour of the first point, diagonal ones (0.354 m) too,
/// closes it; the next ring (0.5 m) does not.
const AREA_CLOSE_M: f64 = 0.4;

/// An area has an inside from three points on.
const MIN_AREA_POINTS: usize = 3;

/// What the drawing tools draw with: picked in the tool bar, kept until changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawStyle {
    pub color: FigureColor,
    pub dash: Dash,
    pub fill: Fill,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            color: FigureColor::Red,
            dash: Dash::Solid,
            fill: Fill::Outline,
        }
    }
}

impl DrawStyle {
    /// The names the tool bar spells; `None` when one of them is unknown.
    pub fn parse(color: &str, dash: &str, fill: &str) -> Option<DrawStyle> {
        Some(DrawStyle {
            color: named(&DRAW_COLORS, color)?,
            dash: named(&DRAW_DASHES, dash)?,
            fill: named(&DRAW_FILLS, fill)?,
        })
    }
}

fn named<T: Copy>(list: &[(&str, T)], name: &str) -> Option<T> {
    list.iter()
        .find(|(n, _)| *n == name)
        .map(|(_, value)| *value)
}

/// The tool of the tool bar together with what was picked for it.
#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Select,
    Wall,
    /// Library kind of the object a click sets.
    Object(String),
    /// Id of the character whose next mark a click sets.
    Figure(String),
    Camera,
    Light(LightKind),
    /// Two clicks on a stretch of known length scale the background; a drag moves it.
    Scale,
    /// A click on empty ground pins a note (the text is asked next); a click on a note asks
    /// for its text again.
    Note,
    /// Drawing tools: a drag draws a line, an arrow, a rectangle or an ellipse between its
    /// two corners; clicks set the points of an area.
    Line,
    Arrow,
    Rect,
    Ellipse,
    Area,
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
            "note" => Some(Tool::Note),
            "line" => Some(Tool::Line),
            "arrow" => Some(Tool::Arrow),
            "rect" => Some(Tool::Rect),
            "ellipse" => Some(Tool::Ellipse),
            "area" => Some(Tool::Area),
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
    /// Takes the bend out of the selected way (LH-21).
    Straighten,
    /// Clockwise for a positive angle.
    Turn {
        by_deg: f64,
    },
}

/// A click at `at`: the tools that put something there. Select only selects, which is no edit.
pub fn click_edit(tool: &Tool, at: Point) -> Option<PlanEdit> {
    match tool {
        Tool::Select
        | Tool::Wall
        | Tool::Scale
        | Tool::Note
        | Tool::Line
        | Tool::Arrow
        | Tool::Rect
        | Tool::Ellipse
        | Tool::Area => None,
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

/// A drag with a drawing tool from `from` to `to` (both snapped already): the drawing it
/// makes. `None` for the other tools, for a drag that goes nowhere, and for a rectangle or
/// ellipse without width or height.
pub fn draw_edit(tool: &Tool, style: DrawStyle, from: Point, to: Point) -> Option<PlanEdit> {
    let boxed = from.x_m != to.x_m && from.y_m != to.y_m;
    let shape = match tool {
        Tool::Line if from != to => DrawingShape::Line { from, to },
        Tool::Arrow if from != to => DrawingShape::Arrow { from, to },
        Tool::Rect if boxed => DrawingShape::Rect { from, to },
        Tool::Ellipse if boxed => DrawingShape::Ellipse { from, to },
        _ => return None,
    };

    Some(add_drawing(shape, style))
}

fn add_drawing(shape: DrawingShape, style: DrawStyle) -> PlanEdit {
    PlanEdit::AddDrawing {
        shape,
        color: style.color,
        dash: style.dash,
        fill: style.fill,
    }
}

/// Where a click lands on the plan: `snap` puts it on the grid. The scale tool keeps the
/// exact point: a stretch measured on the grid gives a wrong scale.
pub fn lands_at(tool: &Tool, at: Point, snap: impl Fn(Point) -> Point) -> Point {
    if *tool == Tool::Scale {
        return at;
    }

    snap(at)
}

/// Whether the plan on screen has a background image to scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backdrop {
    Missing,
    Present,
}

/// What a drag holds: picked once when it starts, kept until it ends. Picking again on every
/// step found other things as the selection changed (a figure's path handle on the figure).
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Grip {
    #[default]
    Free,
    Held(Option<Source>),
}

impl Grip {
    /// The element this drag moves: `pick`'s answer on the first step, the same after.
    pub fn take(self, pick: impl FnOnce() -> Option<Source>) -> (Grip, Option<Source>) {
        let grabbed = match self {
            Grip::Held(held) => held,
            Grip::Free => pick(),
        };

        (Grip::Held(grabbed.clone()), grabbed)
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

    /// The points still valid once the plan shows `backdrop` (undo can take the image away).
    pub fn under(self, backdrop: Backdrop) -> ScaleStep {
        match backdrop {
            Backdrop::Missing => ScaleStep::Idle,
            Backdrop::Present => self,
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

/// The points of an area picked so far; the area is made on closing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AreaDraft(Vec<Point>);

impl AreaDraft {
    /// The draft with `at` as its next point. A click on the point before changes nothing
    /// (a double click is no new corner).
    pub fn click(mut self, at: Point) -> AreaDraft {
        if self.0.last() != Some(&at) {
            self.0.push(at);
        }

        self
    }

    /// Whether a click at `at` closes the area: it has three points and `at` is on the first.
    pub fn closes(&self, at: Point) -> bool {
        let Some(first) = self.0.first() else {
            return false;
        };

        self.0.len() >= MIN_AREA_POINTS && first.distance_m(at) <= AREA_CLOSE_M
    }

    /// The points picked, for the preview on the plan.
    pub fn points(&self) -> &[Point] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// The area closed from its draft; `None` below three points.
pub fn area_edit(draft: &AreaDraft, style: DrawStyle) -> Option<PlanEdit> {
    if draft.0.len() < MIN_AREA_POINTS {
        return None;
    }

    Some(add_drawing(
        DrawingShape::Area {
            points: draft.0.clone(),
        },
        style,
    ))
}

/// Whether the area tool may take points: not while the drawings layer is locked, or the
/// points would be lost on the refused edit.
pub fn drawing_open(lock: Lock) -> Result<(), ServiceError> {
    match lock {
        Lock::Locked => Err(ServiceError::Locked(Layer::Drawings)),
        Lock::Unlocked => Ok(()),
    }
}

/// A note whose text is being asked: a new one at a place, or an existing one by id.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NoteDraft {
    #[default]
    Idle,
    New(Point),
    Edit(String),
}

impl NoteDraft {
    /// A click of the note tool at `at`: on a note it edits that one, else a new one goes
    /// there.
    pub fn click(at: Point, picked: Option<&Source>) -> NoteDraft {
        match picked {
            Some(Source::Note(id)) => NoteDraft::Edit(id.clone()),
            _ => NoteDraft::New(at),
        }
    }
}

/// Whether the note tool may ask for a text: not while the notes layer is locked, or the
/// typed text would be lost on the refused edit.
pub fn note_open(lock: Lock) -> Result<(), ServiceError> {
    match lock {
        Lock::Locked => Err(ServiceError::Locked(Layer::Notes)),
        Lock::Unlocked => Ok(()),
    }
}

/// The text typed for the draft as an edit; `None` without a draft or for a text of only
/// blanks.
pub fn note_edit(draft: &NoteDraft, text: &str) -> Option<PlanEdit> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    match draft {
        NoteDraft::Idle => None,
        NoteDraft::New(at) => Some(PlanEdit::AddNote {
            at: *at,
            text: text.to_owned(),
        }),
        NoteDraft::Edit(id) => Some(PlanEdit::EditNote {
            id: id.clone(),
            text: text.to_owned(),
        }),
    }
}

/// A length in metres as typed: a comma or a point as decimal separator, more than zero.
pub fn parse_metres(text: &str) -> Option<f64> {
    let metres: f64 = text.trim().replace(',', ".").parse().ok()?;

    (metres.is_finite() && metres > 0.0).then_some(metres)
}

/// The size typed for the selected object, as an edit. `None` when something other than an
/// object is selected or a side is not a length.
pub fn resize_edit(selected: Option<&Source>, width: &str, depth: &str) -> Option<PlanEdit> {
    let Some(Source::Object(id)) = selected else {
        return None;
    };

    Some(PlanEdit::Resize {
        source: Source::Object(id.clone()),
        width_m: parse_metres(width)?,
        depth_m: parse_metres(depth)?,
    })
}

/// Files the selected object in the library of the project under `name`. `None` when
/// something other than an object is selected, there is no scene to take the set from, or the
/// name is blank.
pub fn library_edit(
    selected: Option<&Source>,
    location: Option<&str>,
    name: &str,
) -> Option<ProjectEdit> {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    Some(ProjectEdit::LibraryFromObject {
        location: location?.to_owned(),
        object: id.clone(),
        name: name.to_owned(),
    })
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
    /// No edit and no preview on screen: nothing to redraw.
    Nothing,
}

/// Whether the plan on screen shows a preview instead of the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Hidden,
    Shown,
}

/// What a drag step does, given what the screen shows: a drag without edit (tools that do not
/// drag) must not redraw the plan on every mouse move.
pub fn drag_step(edit: Option<PlanEdit>, phase: DragPhase, shown: Preview) -> DragStep {
    match (edit, phase) {
        (None, _) if shown == Preview::Hidden => DragStep::Nothing,
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
        Key::Straighten => Some(PlanEdit::Straighten { source }),
        Key::Turn { by_deg } => Some(PlanEdit::Spin { source, by_deg }),
    }
}

#[cfg(test)]
mod tests {

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
        assert_eq!(lands_at(&Tool::Camera, at, snap), Point::new(1.0, 1.0));
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

    /// The tool bar translates each kind with `lightKind.<name>`; the objects are named by the
    /// library (services tests).
    #[test]
    fn every_lamp_kind_has_a_name_in_both_languages() {
        let keys = LIGHT_KINDS.iter().map(|(k, _)| format!("lightKind.{k}"));

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

    #[test]
    fn a_size_typed_for_the_selected_object_is_a_resize() {
        let object = Source::Object("o1".into());

        assert_eq!(
            resize_edit(Some(&object), "2,5", " 0.8 "),
            Some(PlanEdit::Resize {
                source: object.clone(),
                width_m: 2.5,
                depth_m: 0.8
            })
        );
        assert_eq!(resize_edit(Some(&object), "2,5", "0"), None);
        assert_eq!(resize_edit(Some(&object), "", "1"), None);
        assert_eq!(resize_edit(Some(&object), "1", "breit"), None);
        assert_eq!(resize_edit(Some(&Source::Wall(0)), "1", "1"), None);
        assert_eq!(resize_edit(None, "1", "1"), None);
    }

    #[test]
    fn the_selected_object_is_filed_under_a_trimmed_name() {
        let object = Source::Object("o1".into());

        assert_eq!(
            library_edit(Some(&object), Some("saal"), "  Thron "),
            Some(ProjectEdit::LibraryFromObject {
                location: "saal".into(),
                object: "o1".into(),
                name: "Thron".into()
            })
        );
        assert_eq!(library_edit(Some(&object), Some("saal"), "  "), None);
        assert_eq!(library_edit(Some(&object), None, "Thron"), None);
        assert_eq!(
            library_edit(Some(&Source::Wall(0)), Some("saal"), "T"),
            None
        );
        assert_eq!(library_edit(None, Some("saal"), "T"), None);
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
        assert_eq!(
            key_edit(Key::Straighten, Some(&Source::Camera("a".into()))),
            Some(PlanEdit::Straighten {
                source: Source::Camera("a".into())
            })
        );
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
    fn note_is_a_tool_and_a_click_edits_nothing() {
        assert_eq!(Tool::parse("note", ""), Some(Tool::Note));
        assert_eq!(click_edit(&Tool::Note, p(1.0, 1.0)), None);
    }

    #[test]
    fn a_click_on_a_note_edits_it_else_a_new_one_goes_there() {
        let at = p(2.0, 3.0);
        let note = Source::Note("n1".into());
        let wall = Source::Wall(0);

        assert_eq!(
            NoteDraft::click(at, Some(&note)),
            NoteDraft::Edit("n1".into())
        );
        assert_eq!(NoteDraft::click(at, Some(&wall)), NoteDraft::New(at));
        assert_eq!(NoteDraft::click(at, None), NoteDraft::New(at));
    }

    #[test]
    fn the_typed_text_makes_the_note_edit() {
        let at = p(2.0, 3.0);

        assert_eq!(
            note_edit(&NoteDraft::New(at), " Licht an "),
            Some(PlanEdit::AddNote {
                at,
                text: "Licht an".into()
            })
        );
        assert_eq!(
            note_edit(&NoteDraft::Edit("n1".into()), "Licht aus"),
            Some(PlanEdit::EditNote {
                id: "n1".into(),
                text: "Licht aus".into()
            })
        );
        assert_eq!(note_edit(&NoteDraft::New(at), "  "), None);
        assert_eq!(note_edit(&NoteDraft::Idle, "x"), None);
    }

    fn blue_dotted_filled() -> DrawStyle {
        DrawStyle {
            color: FigureColor::Blue,
            dash: Dash::Dotted,
            fill: Fill::Filled,
        }
    }

    #[test]
    fn the_drawing_tools_are_named_and_a_click_edits_nothing() {
        for name in ["line", "arrow", "rect", "ellipse", "area"] {
            let tool = Tool::parse(name, "");

            assert!(tool.is_some(), "{name}");
            assert_eq!(click_edit(&tool.unwrap_or(Tool::Select), p(1.0, 1.0)), None);
        }
    }

    #[test]
    fn a_drag_draws_the_shape_of_the_tool_in_the_style_picked() {
        let (from, to) = (p(1.0, 1.0), p(4.0, 3.0));
        let style = blue_dotted_filled();
        let drawn = |shape| PlanEdit::AddDrawing {
            shape,
            color: FigureColor::Blue,
            dash: Dash::Dotted,
            fill: Fill::Filled,
        };

        let edit = |tool| draw_edit(&tool, style, from, to);

        assert_eq!(
            edit(Tool::Line),
            Some(drawn(DrawingShape::Line { from, to }))
        );
        assert_eq!(
            edit(Tool::Arrow),
            Some(drawn(DrawingShape::Arrow { from, to }))
        );
        assert_eq!(
            edit(Tool::Rect),
            Some(drawn(DrawingShape::Rect { from, to }))
        );
        assert_eq!(
            edit(Tool::Ellipse),
            Some(drawn(DrawingShape::Ellipse { from, to }))
        );
        assert_eq!(edit(Tool::Wall), None);
        assert_eq!(edit(Tool::Area), None);
    }

    /// A line needs a length; a rectangle and an ellipse a width and a height.
    #[test]
    fn a_drag_that_makes_nothing_draws_nothing() {
        let style = DrawStyle::default();
        let at = p(2.0, 2.0);

        assert_eq!(draw_edit(&Tool::Line, style, at, at), None);
        assert_eq!(draw_edit(&Tool::Arrow, style, at, at), None);
        assert_eq!(draw_edit(&Tool::Rect, style, at, p(5.0, 2.0)), None);
        assert_eq!(draw_edit(&Tool::Ellipse, style, at, p(2.0, 5.0)), None);
        assert!(draw_edit(&Tool::Line, style, at, p(5.0, 2.0)).is_some());
    }

    #[test]
    fn the_style_is_read_from_the_names_of_the_tool_bar() {
        assert_eq!(
            DrawStyle::parse("blue", "dotted", "filled"),
            Some(blue_dotted_filled())
        );
        assert_eq!(DrawStyle::parse("pink", "solid", "outline"), None);
        assert_eq!(DrawStyle::parse("red", "wavy", "outline"), None);
        assert_eq!(DrawStyle::parse("red", "solid", "half"), None);
        assert_eq!(
            DrawStyle::parse("red", "solid", "outline"),
            Some(DrawStyle::default())
        );
    }

    /// The tool bar translates each name with `color.<name>`, `dash.<name>`, `fill.<name>`.
    #[test]
    fn every_style_name_has_a_text_in_both_languages() {
        let keys = DRAW_COLORS
            .iter()
            .map(|(n, _)| format!("color.{n}"))
            .chain(DRAW_DASHES.iter().map(|(n, _)| format!("dash.{n}")))
            .chain(DRAW_FILLS.iter().map(|(n, _)| format!("fill.{n}")));

        for key in keys {
            for language in [Language::German, Language::English] {
                assert_ne!(language.text(&key), key);
            }
        }
    }

    /// Review of PR 24: with the 0.25 m grid the diagonal neighbour of the first point is
    /// 0.354 m away and added a corner instead of closing the area.
    #[test]
    fn a_click_on_a_diagonal_grid_neighbour_closes() {
        let draft = AreaDraft::default()
            .click(Point::new(0.0, 0.0))
            .click(Point::new(2.0, 0.0))
            .click(Point::new(2.0, 2.0));

        assert!(draft.closes(Point::new(0.25, 0.25)));
        assert!(!draft.closes(Point::new(0.5, 0.5)));
    }

    #[test]
    fn clicks_collect_the_points_of_an_area() {
        let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));

        let draft = AreaDraft::default().click(a).click(b).click(b).click(c);

        assert_eq!(draft.points(), [a, b, c]);
        assert!(!draft.is_empty());
    }

    /// Two points make no area: a click on the first point adds a point. With three, a click
    /// within 0.4 m of the first (a grid neighbour) closes; further away it adds.
    #[test]
    fn a_click_on_the_first_point_closes_from_three_points_on() {
        let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));
        let two = AreaDraft::default().click(a).click(b);
        let three = two.clone().click(c);

        assert!(!AreaDraft::default().closes(a));
        assert!(!two.closes(a));
        assert!(three.closes(a));
        assert!(three.closes(p(0.25, 0.0)));
        assert!(!three.closes(p(0.5, 0.0)));
    }

    #[test]
    fn an_area_needs_three_points_and_takes_the_style() {
        let (a, b, c) = (p(0.0, 0.0), p(4.0, 0.0), p(4.0, 3.0));
        let two = AreaDraft::default().click(a).click(b);
        let three = two.clone().click(c);

        assert_eq!(area_edit(&two, DrawStyle::default()), None);
        assert_eq!(
            area_edit(&three, blue_dotted_filled()),
            Some(PlanEdit::AddDrawing {
                shape: DrawingShape::Area {
                    points: vec![a, b, c]
                },
                color: FigureColor::Blue,
                dash: Dash::Dotted,
                fill: Fill::Filled,
            })
        );
    }

    /// Like the notes: with the drawings layer locked the tool took points and dropped the
    /// area on closing.
    #[test]
    fn a_locked_drawings_layer_takes_no_points() {
        assert!(matches!(
            drawing_open(Lock::Locked),
            Err(ServiceError::Locked(Layer::Drawings))
        ));
        assert!(drawing_open(Lock::Unlocked).is_ok());
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
