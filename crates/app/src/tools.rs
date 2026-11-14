//! What a drawing tool does with a click, a drag or a key: the tool, the gesture and the
//! choice made in the tool bar become one plan edit. No geometry beyond the two points a
//! gesture hands over (agent.md); the session snaps them and does the work.

use klappe_services::{
    read_number, Dash, DrawingShape, Field, FigureColor, Fill, Layer, LightKind, Lock, PlanEdit,
    Point, ProjectEdit, Range, ServiceError, Source, Unit,
};

/// A new camera looks to the right (0°); the user turns it afterwards.
const NEW_CAMERA_FACING_DEG: f64 = 0.0;

/// Decimals a measured length shows.
const MEASURE_DECIMALS: usize = 2;

/// Lamp kinds the tool bar offers, with the name QML and the catalogue (`lightKind.<name>`) use.
pub const LIGHT_KINDS: [(&str, LightKind); 6] = [
    ("fresnel", LightKind::Fresnel),
    ("led-panel", LightKind::LedPanel),
    ("tube", LightKind::Tube),
    ("balloon", LightKind::Balloon),
    ("reflector", LightKind::Reflector),
    ("flag", LightKind::Flag),
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
            color: FigureColor::from_name(color)?,
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
    /// Two clicks measure the distance between two points; the pointer shows it live. The
    /// measurement is no part of the plan.
    Measure,
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
            "measure" => Some(Tool::Measure),
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
    /// Sets a copy of the selection apart from it (LH-08).
    Duplicate,
}

/// How far a copy sits from what it copies, in metres, to both sides: two grid cells, so it
/// is seen and still on the grid.
const COPY_OFFSET_M: f64 = 0.5;

/// What the select tool has picked, in the order it was picked: the canvas frames each.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selection(Vec<Source>);

/// What a pick does to the selection: replaces it, or (Shift held) adds to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Replace,
    Extend,
}

impl Selection {
    pub fn of(source: Option<Source>) -> Selection {
        Selection(source.into_iter().collect())
    }

    pub fn contains(&self, source: &Source) -> bool {
        self.0.contains(source)
    }

    pub fn sources(&self) -> &[Source] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The one element picked; `None` for none or for several.
    pub fn only(&self) -> Option<&Source> {
        match self.0.as_slice() {
            [source] => Some(source),
            _ => None,
        }
    }

    /// A click on `picked` (`None`: empty ground). A plain click takes just that; with
    /// `Extend` it takes the element in, or out when it is in already, and a click on empty
    /// ground changes nothing.
    pub fn click(mut self, picked: Option<Source>, how: Pick) -> Selection {
        match (how, picked) {
            (Pick::Replace, picked) => Selection::of(picked),
            (Pick::Extend, None) => self,
            (Pick::Extend, Some(source)) => {
                match self.0.iter().position(|s| *s == source) {
                    Some(index) => {
                        self.0.remove(index);
                    }
                    None => self.0.push(source),
                }

                self.no_grips()
            }
        }
    }

    /// A box that took `found`: replaces the selection, or with `Extend` adds to it.
    pub fn boxed(mut self, found: Vec<Source>, how: Pick) -> Selection {
        if how == Pick::Replace {
            return Selection(found);
        }

        for source in found {
            if !self.contains(&source) {
                self.0.push(source);
            }
        }

        self.no_grips()
    }

    /// A grip shows only while its way alone is selected, so several hold none: it would be
    /// picked unseen and bend or straighten its way with the rest.
    fn no_grips(mut self) -> Selection {
        if self.0.len() > 1 {
            self.0
                .retain(|s| !matches!(s, Source::FigureVia { .. } | Source::CameraVia { .. }));
        }

        self
    }

    /// After a drag that grabbed `grabbed`: a member drags the whole selection along, else
    /// the grabbed element is all that is selected.
    pub fn grab(self, grabbed: Option<Source>) -> Selection {
        match grabbed {
            Some(source) if self.contains(&source) => self,
            grabbed => Selection::of(grabbed),
        }
    }

    /// The elements, `first` in front: the one the pointer holds snaps to the grid.
    fn leading(&self, first: &Source) -> Vec<Source> {
        let mut sources = vec![first.clone()];
        sources.extend(self.0.iter().filter(|s| *s != first).cloned());

        sources
    }

    /// Only the elements `shown` approves.
    pub fn keep(mut self, shown: impl Fn(&Source) -> bool) -> Selection {
        self.0.retain(shown);

        self
    }
}

/// What a drag that grabbed `grabbed` does when it holds a member of a selection of several:
/// the whole selection moves by the distance. `None` for a lone element, which `drag_edit`
/// handles, or for an element outside the selection.
pub fn group_drag_edit(
    selection: &Selection,
    grabbed: &Source,
    from: Point,
    to: Point,
) -> Option<PlanEdit> {
    if selection.len() < 2 || !selection.contains(grabbed) {
        return None;
    }

    Some(PlanEdit::ShiftGroup {
        sources: selection.leading(grabbed),
        dx_m: to.x_m - from.x_m,
        dy_m: to.y_m - from.y_m,
    })
}

/// The elements `after` shows that `before` did not: what a copy added.
pub fn added(before: &[Source], after: &[Source]) -> Selection {
    let new = after.iter().filter(|s| !before.contains(s)).cloned();

    Selection(new.collect())
}

/// A click at `at`: the tools that put something there. Select only selects, which is no edit.
pub fn click_edit(tool: &Tool, at: Point) -> Option<PlanEdit> {
    match tool {
        Tool::Select
        | Tool::Wall
        | Tool::Scale
        | Tool::Measure
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

    /// The grip after a step: kept while moving, let go when the drag is done.
    pub fn after(self, phase: DragPhase) -> Grip {
        match phase {
            DragPhase::Moving => self,
            DragPhase::Done => Grip::Free,
        }
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

/// How far the measuring gesture has come. `Start` has the first point, `Live` follows the
/// pointer to a second one, `Held` is the finished measurement.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MeasureStep {
    #[default]
    Idle,
    Start(Point),
    Live(Point, Point),
    Held(Point, Point),
}

impl MeasureStep {
    /// A click at `at`: the first point, or the second. A second click on the first point
    /// changes nothing; a click on a held measurement starts a new one.
    pub fn click(self, at: Point) -> MeasureStep {
        match self {
            MeasureStep::Start(first) | MeasureStep::Live(first, _) if first == at => {
                MeasureStep::Start(first)
            }
            MeasureStep::Start(first) | MeasureStep::Live(first, _) => MeasureStep::Held(first, at),
            MeasureStep::Idle | MeasureStep::Held(..) => MeasureStep::Start(at),
        }
    }

    /// The pointer at `at`: it moves the open end of a measurement not yet held.
    pub fn hover(self, at: Point) -> MeasureStep {
        match self {
            MeasureStep::Start(first) | MeasureStep::Live(first, _) => MeasureStep::Live(first, at),
            MeasureStep::Idle | MeasureStep::Held(..) => self,
        }
    }

    /// The points to show: the first alone, or both with the line between them.
    pub fn points(self) -> Vec<Point> {
        match self {
            MeasureStep::Idle => Vec::new(),
            MeasureStep::Start(a) => vec![a],
            MeasureStep::Live(a, b) | MeasureStep::Held(a, b) => vec![a, b],
        }
    }

    /// The distance between the two points, once there are two.
    pub fn length_m(self) -> Option<f64> {
        match self {
            MeasureStep::Idle | MeasureStep::Start(_) => None,
            MeasureStep::Live(a, b) | MeasureStep::Held(a, b) => Some(a.distance_m(b)),
        }
    }
}

/// `length_m` as the caption of a measurement: two decimals at most, the catalogue's decimal
/// separator and unit ("4,25 m").
pub fn length_text(length_m: f64, decimal: &str, unit: &str) -> String {
    let fixed = format!("{length_m:.MEASURE_DECIMALS$}");
    let number = fixed.trim_end_matches('0').trim_end_matches('.');

    format!("{} {unit}", number.replace('.', decimal))
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

/// What a typed field makes of the selection: `None` when the selection does not fit the
/// field, an error (for the status line) when the text is no value.
pub type Typed = Option<Result<PlanEdit, ServiceError>>;

/// The edit a typed field made, if it made one.
#[cfg(test)]
pub(crate) fn edit_of(typed: Typed) -> Option<PlanEdit> {
    typed?.ok()
}

/// A length in metres as typed, more than zero.
fn length_m(field: Field, text: &str) -> Result<f64, ServiceError> {
    read_number(field, text, Unit::Metre, Range::Positive)
}

/// The size typed for the selected object, as an edit.
pub fn resize_edit(selected: Option<&Source>, width: &str, depth: &str) -> Typed {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let resized = length_m(Field::Width, width).and_then(|width_m| {
        Ok(PlanEdit::Resize {
            source: Source::Object(id.clone()),
            width_m,
            depth_m: length_m(Field::Depth, depth)?,
        })
    });

    Some(resized)
}

/// The height typed for the selected object, as an edit (LH-17): an empty text takes the
/// height off.
pub fn object_height_edit(selected: Option<&Source>, text: &str) -> Typed {
    let Some(Source::Object(id)) = selected else {
        return None;
    };
    let height_m = match text.trim().is_empty() {
        true => Ok(None),
        false => length_m(Field::Height, text).map(Some),
    };

    Some(height_m.map(|height_m| PlanEdit::SetObjectHeight {
        source: Source::Object(id.clone()),
        height_m,
    }))
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
pub fn scale_edit(step: ScaleStep, distance: &str) -> Typed {
    let ScaleStep::Both(from, to) = step else {
        return None;
    };
    let distance_m = length_m(Field::Distance, distance);

    Some(distance_m.map(|distance_m| PlanEdit::Calibrate {
        from,
        to,
        distance_m,
    }))
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

/// A key on the selection; `None` when nothing is selected. Several elements take one group
/// edit; a way is made straight for one element only.
pub fn key_edit(key: Key, selected: &Selection) -> Option<PlanEdit> {
    let sources = selected.sources().to_vec();
    if key == Key::Duplicate && !selected.is_empty() {
        return Some(PlanEdit::Duplicate {
            sources,
            dx_m: COPY_OFFSET_M,
            dy_m: COPY_OFFSET_M,
        });
    }

    let [source] = sources.as_slice() else {
        return group_key_edit(key, sources);
    };
    let source = source.clone();

    match key {
        Key::Remove => Some(PlanEdit::Remove { source }),
        Key::Straighten => Some(PlanEdit::Straighten { source }),
        Key::Turn { by_deg } => Some(PlanEdit::Spin { source, by_deg }),
        Key::Duplicate => None,
    }
}

/// `key_edit` for no element or several.
fn group_key_edit(key: Key, sources: Vec<Source>) -> Option<PlanEdit> {
    if sources.is_empty() {
        return None;
    }

    match key {
        Key::Remove => Some(PlanEdit::RemoveGroup { sources }),
        Key::Turn { by_deg } => Some(PlanEdit::SpinGroup { sources, by_deg }),
        Key::Straighten | Key::Duplicate => None,
    }
}

#[cfg(test)]
mod tests;
