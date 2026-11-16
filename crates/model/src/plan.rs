use serde::{Deserialize, Serialize};

use crate::curve::Curve;
use crate::geometry::Point;
use crate::project::FigureColor;
use crate::shot::Shot;

/// The plan of one scene: who stands and moves where, cameras, lights, shots (LH-04: a scene
/// may have variants, `Plan A` and `Plan B`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub figures: Vec<FigureTrack>,
    #[serde(default)]
    pub cameras: Vec<Camera>,
    #[serde(default)]
    pub lights: Vec<Light>,
    /// Shots in cut order (LH-72).
    #[serde(default)]
    pub shots: Vec<Shot>,
    #[serde(default)]
    pub links: Vec<ScriptLink>,
    /// The two characters the 180° line runs between (LH-40); `None` takes the first two
    /// figures.
    #[serde(default)]
    pub axis: Option<AxisPair>,
    /// Lines, arrows and shapes drawn on the plan by hand (LH-19).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drawings: Vec<Drawing>,
}

/// Ids of the two characters that span the axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxisPair {
    pub a: String,
    pub b: String,
}

impl FloorPlan {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            figures: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            shots: Vec::new(),
            links: Vec::new(),
            axis: None,
            drawings: Vec::new(),
        }
    }

    pub fn figure(&self, character: &str) -> Option<&FigureTrack> {
        self.figures.iter().find(|f| f.character == character)
    }

    pub fn camera(&self, id: &str) -> Option<&Camera> {
        self.cameras.iter().find(|c| c.id == id)
    }

    /// End of the last mark of any figure or camera, its pause included, or of the last
    /// shot: the length of the scene's blocking.
    pub fn duration_s(&self) -> f64 {
        let figures = self
            .figures
            .iter()
            .map(|f| track_end(f.marks.iter().map(|m| (m.at_s, m.hold_s))));
        let cameras = self
            .cameras
            .iter()
            .map(|c| track_end(c.marks.iter().map(|m| (m.at_s, m.hold_s))));
        let shots = self.shots.iter().map(|s| s.to_s);

        figures.chain(cameras).chain(shots).fold(0.0, f64::max)
    }
}

/// Last arrival plus its pause; earlier pauses end at the next mark anyway.
fn track_end(marks: impl Iterator<Item = (f64, f64)>) -> f64 {
    let mut end = 0.0_f64;
    let mut last_hold = 0.0;

    for (at_s, hold_s) in marks {
        end = end.max(at_s);
        last_hold = hold_s;
    }

    end + last_hold
}

/// A free drawing on the plan of a scene, e.g. an arrow for a move or a marked area (LH-19).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Drawing {
    /// Stable id within the plan, e.g. `d1`.
    pub id: String,
    pub shape: DrawingShape,
    /// A role, drawn in one of Kante's label colours.
    pub color: FigureColor,
    #[serde(default)]
    pub dash: Dash,
    /// Only shapes with an inside (rectangle, ellipse, area) are filled.
    #[serde(default)]
    pub fill: Fill,
    /// Clockwise about the middle of the box that holds the shape's points; a rectangle or
    /// ellipse turns with it, so it is no longer axis-parallel.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rotation_deg: f64,
}

pub(crate) fn is_zero(deg: &f64) -> bool {
    *deg == 0.0
}

impl Drawing {
    /// The points the shape is made of, before the turn: the two corners of a line, arrow,
    /// rectangle or ellipse, or the corners of an area.
    pub fn points(&self) -> Vec<Point> {
        match &self.shape {
            DrawingShape::Line { from, to }
            | DrawingShape::Arrow { from, to }
            | DrawingShape::Rect { from, to }
            | DrawingShape::Ellipse { from, to } => vec![*from, *to],
            DrawingShape::Area { points } => points.clone(),
        }
    }

    /// The middle of the box that holds `points`, which the drawing turns about; `None`
    /// for a shape without points.
    pub fn middle(&self) -> Option<Point> {
        let points = self.points();
        let first = *points.first()?;
        let (mut min, mut max) = (first, first);

        for p in points {
            min = Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m));
            max = Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m));
        }

        Some(min.lerp(max, 0.5))
    }

    /// `point` of the shape where it lies on the plan: turned by `rotation_deg`.
    pub fn placed(&self, point: Point) -> Point {
        match self.middle() {
            Some(middle) => point.turned(middle, self.rotation_deg),
            None => point,
        }
    }
}

/// What a drawing is made of; points in metres.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum DrawingShape {
    Line {
        from: Point,
        to: Point,
    },
    /// A line with its head at `to`.
    Arrow {
        from: Point,
        to: Point,
    },
    /// Axis-parallel, between two opposite corners.
    Rect {
        from: Point,
        to: Point,
    },
    /// Axis-parallel, inscribed in the rectangle between the two corners.
    Ellipse {
        from: Point,
        to: Point,
    },
    /// A closed polygon; the last point joins the first.
    Area {
        points: Vec<Point>,
    },
}

/// How a line is drawn; the dash tells drawings apart where colour does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dash {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl Dash {
    /// Every dash, in the order the tool bar offers.
    pub const ALL: [Dash; 3] = [Dash::Solid, Dash::Dashed, Dash::Dotted];

    /// The name the file format uses, e.g. `dashed`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Dash::Solid => "solid",
            Dash::Dashed => "dashed",
            Dash::Dotted => "dotted",
        }
    }

    /// The dash `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.name() == name)
    }
}

/// Whether the inside of a closed shape is tinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fill {
    #[default]
    Outline,
    Filled,
}

impl Fill {
    /// Every fill, in the order the tool bar offers.
    pub const ALL: [Fill; 2] = [Fill::Outline, Fill::Filled];

    /// The name the file format uses, e.g. `filled`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Fill::Outline => "outline",
            Fill::Filled => "filled",
        }
    }

    /// The fill `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == name)
    }
}

/// A character's way through the scene: marks 1, 2, 3 … in time order (LH-21).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigureTrack {
    /// Id of the project's character.
    pub character: String,
    pub marks: Vec<Mark>,
}

/// Where a figure is at a moment: arrives at `at_s`, stays until it leaves for the next mark.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub at_s: f64,
    pub at: Point,
    pub facing_deg: f64,
    #[serde(default)]
    pub pose: Pose,
    /// Seconds the figure stays before it walks on (LH-63: pause at a mark).
    #[serde(default)]
    pub hold_s: f64,
    /// Bends of the way from the previous mark to this one: the smooth (Catmull-Rom) curve
    /// passes them in order; empty walks straight. Ignored on the first mark (LH-21).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<Point>,
    /// Whether the way through `via` is a smooth curve or straight pieces (LH-108).
    #[serde(default, skip_serializing_if = "crate::curve::is_smooth")]
    pub curve: Curve,
    /// How the figure comes to this mark (LH-26).
    #[serde(default, skip_serializing_if = "Arrival::is_walk")]
    pub arrival: Arrival,
    /// Whether the figure is still there after the pause at this mark (LH-26).
    #[serde(default, skip_serializing_if = "Departure::is_stay")]
    pub departure: Departure,
}

/// How a figure comes to a mark (LH-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Arrival {
    /// Walks from the mark before; on the first mark it is there from the start.
    #[default]
    Walk,
    /// Not there before `at_s`; shows up at the mark instead of walking to it.
    Appear,
}

impl Arrival {
    /// Every arrival.
    pub const ALL: [Arrival; 2] = [Arrival::Walk, Arrival::Appear];

    /// The name the file format uses, e.g. `appear`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Arrival::Walk => "walk",
            Arrival::Appear => "appear",
        }
    }

    /// The arrival `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }

    fn is_walk(&self) -> bool {
        *self == Arrival::Walk
    }
}

/// What a figure does after the pause at a mark (LH-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Departure {
    /// Stays in the scene.
    #[default]
    Stay,
    /// Gone after `at_s + hold_s`, until a later mark appears.
    Leave,
}

impl Departure {
    /// Every departure.
    pub const ALL: [Departure; 2] = [Departure::Stay, Departure::Leave];

    /// The name the file format uses, e.g. `leave`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Departure::Stay => "stay",
            Departure::Leave => "leave",
        }
    }

    /// The departure `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.name() == name)
    }

    fn is_stay(&self) -> bool {
        *self == Departure::Stay
    }
}

/// LH-22.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pose {
    #[default]
    Standing,
    Sitting,
    Lying,
}

impl Pose {
    /// Every pose.
    pub const ALL: [Pose; 3] = [Pose::Standing, Pose::Sitting, Pose::Lying];

    /// The name the file format uses, e.g. `sitting`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Pose::Standing => "standing",
            Pose::Sitting => "sitting",
            Pose::Lying => "lying",
        }
    }

    /// The pose `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// A camera body with its lens (LH-30 – LH-34).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub id: String,
    /// `A`, `B`, `C`.
    pub label: String,
    pub sensor: Sensor,
    pub lens_mm: f64,
    /// Width to height of the recorded frame, e.g. 2.39 (LH-38).
    pub aspect: f64,
    pub marks: Vec<CameraMark>,
    /// A figure the camera keeps its distance and angle to when that figure is moved (LH-37).
    #[serde(default)]
    pub follows: Option<String>,
    /// The colour the camera's shots are marked in (LH-111); `None` takes the one its place in
    /// the plan gives (`FloorPlan::camera_color`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<FigureColor>,
}

/// Camera position at a moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraMark {
    pub at_s: f64,
    pub at: Point,
    pub facing_deg: f64,
    /// Lens height above the floor (LH-34).
    pub height_m: f64,
    /// Up from the horizon: positive looks up, negative down, 0 is level (LH-34).
    #[serde(default)]
    pub tilt_deg: f64,
    #[serde(default)]
    pub hold_s: f64,
    /// Bends of the way from the previous mark to this one: the smooth (Catmull-Rom) curve
    /// passes them in order; empty walks straight. Ignored on the first mark (LH-21).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub via: Vec<Point>,
    /// Whether the way through `via` is a smooth curve or straight pieces (LH-108).
    #[serde(default, skip_serializing_if = "crate::curve::is_smooth")]
    pub curve: Curve,
    /// How the camera comes here from the previous mark (LH-33). Ignored on the first mark.
    #[serde(default, skip_serializing_if = "Rig::is_dolly")]
    pub rig: Rig,
    /// The lens at this mark; between two marks with a lens it runs linearly with time (a
    /// zoom). `None` states nothing (LH-33).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lens_mm: Option<f64>,
}

/// The gear that carries a camera from one mark to the next (LH-33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Rig {
    /// Stands still: pans and tilts, its place stays.
    Tripod,
    #[default]
    Dolly,
    /// A dolly on a rail.
    Track,
    Crane,
    Handheld,
    /// Steadicam or gimbal.
    Gimbal,
}

impl Rig {
    /// Every rig.
    pub const ALL: [Rig; 6] = [
        Rig::Tripod,
        Rig::Dolly,
        Rig::Track,
        Rig::Crane,
        Rig::Handheld,
        Rig::Gimbal,
    ];

    /// The name the file format uses, e.g. `track`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Rig::Tripod => "tripod",
            Rig::Dolly => "dolly",
            Rig::Track => "track",
            Rig::Crane => "crane",
            Rig::Handheld => "handheld",
            Rig::Gimbal => "gimbal",
        }
    }

    /// The rig `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.name() == name)
    }

    fn is_dolly(&self) -> bool {
        *self == Rig::Dolly
    }
}

/// Sensor formats with their active width; the frame's aspect crops the height (LH-32).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sensor {
    /// Super 35, 24.89 mm wide.
    Super35,
    /// Full frame, 36 mm wide.
    FullFrame,
    /// Micro Four Thirds, 17.3 mm wide.
    MicroFourThirds,
    /// Any other sensor, by its width.
    Custom { width_mm: f64 },
}

impl Sensor {
    /// The sensor of this width: a named format when the width is its own, else `Custom`.
    pub fn of_width_mm(width_mm: f64) -> Sensor {
        [Sensor::Super35, Sensor::FullFrame, Sensor::MicroFourThirds]
            .into_iter()
            .find(|sensor| sensor.width_mm() == width_mm)
            .unwrap_or(Sensor::Custom { width_mm })
    }

    pub fn width_mm(self) -> f64 {
        match self {
            Sensor::Super35 => 24.89,
            Sensor::FullFrame => 36.0,
            Sensor::MicroFourThirds => 17.3,
            Sensor::Custom { width_mm } => width_mm,
        }
    }
}

/// A lamp (LH-50, LH-51).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub id: String,
    pub label: String,
    pub kind: LightKind,
    pub at: Point,
    pub facing_deg: f64,
    pub beam_deg: f64,
    pub color_temp_k: f64,
    pub power_w: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightKind {
    Fresnel,
    LedPanel,
    Tube,
    Balloon,
    Reflector,
    Flag,
}

impl LightKind {
    /// Every lamp kind, in the order the tool bar offers.
    pub const ALL: [LightKind; 6] = [
        LightKind::Fresnel,
        LightKind::LedPanel,
        LightKind::Tube,
        LightKind::Balloon,
        LightKind::Reflector,
        LightKind::Flag,
    ];

    /// The name the file format uses, e.g. `led-panel`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            LightKind::Fresnel => "fresnel",
            LightKind::LedPanel => "led-panel",
            LightKind::Tube => "tube",
            LightKind::Balloon => "balloon",
            LightKind::Reflector => "reflector",
            LightKind::Flag => "flag",
        }
    }

    /// Whether the colour temperature can be set: LEDs yes, a tungsten lamp is what it is,
    /// bounce and flag make no light of their own.
    pub fn tunable_color(self) -> bool {
        matches!(self, LightKind::LedPanel | LightKind::Tube)
    }

    /// Whether the lamp makes light of its own: bounce and flag only shape it.
    pub fn gives_light(self) -> bool {
        !matches!(self, LightKind::Reflector | LightKind::Flag)
    }

    /// The lamp kind `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == name)
    }
}

/// Ties an element of the scene's script to the plan (design B: "when what happens").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptLink {
    /// Index of the element (action, dialogue …) within the scene in the script.
    pub element: usize,
    pub target: LinkTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum LinkTarget {
    /// Mark `index` (0-based) of a character's track.
    Mark {
        character: String,
        index: usize,
    },
    Shot {
        id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `name` is what the file holds and `from_name` reads it back.
    #[test]
    fn mark_names_are_the_file_names() {
        for pose in Pose::ALL {
            let json = serde_json::to_string(&pose).unwrap();
            assert_eq!(json, format!("\"{}\"", pose.name()));
            assert_eq!(Pose::from_name(pose.name()), Some(pose));
        }
        for arrival in Arrival::ALL {
            let json = serde_json::to_string(&arrival).unwrap();
            assert_eq!(json, format!("\"{}\"", arrival.name()));
            assert_eq!(Arrival::from_name(arrival.name()), Some(arrival));
        }
        for departure in Departure::ALL {
            let json = serde_json::to_string(&departure).unwrap();
            assert_eq!(json, format!("\"{}\"", departure.name()));
            assert_eq!(Departure::from_name(departure.name()), Some(departure));
        }
        assert_eq!(Pose::from_name("flying"), None);
        assert_eq!(Arrival::from_name("fly"), None);
        assert_eq!(Departure::from_name("vanish"), None);
    }

    /// Same for the drawing tools' dash and fill and for the lamp kinds.
    #[test]
    fn tool_names_are_the_file_names() {
        for dash in Dash::ALL {
            let json = serde_json::to_string(&dash).unwrap();
            assert_eq!(json, format!("\"{}\"", dash.name()));
            assert_eq!(Dash::from_name(dash.name()), Some(dash));
        }
        for fill in Fill::ALL {
            let json = serde_json::to_string(&fill).unwrap();
            assert_eq!(json, format!("\"{}\"", fill.name()));
            assert_eq!(Fill::from_name(fill.name()), Some(fill));
        }
        for kind in LightKind::ALL {
            let json = serde_json::to_string(&kind).unwrap();
            assert_eq!(json, format!("\"{}\"", kind.name()));
            assert_eq!(LightKind::from_name(kind.name()), Some(kind));
        }
        assert_eq!(Dash::from_name("wavy"), None);
        assert_eq!(Fill::from_name("hatched"), None);
        assert_eq!(LightKind::from_name("laser"), None);
    }

    /// A plan without drawings writes no `drawings` key; a file without it has none.
    #[test]
    fn drawings_are_optional_in_json() {
        let plan = FloorPlan::new("p", "Plan A");

        let json = serde_json::to_string(&plan).unwrap();
        let back: FloorPlan = serde_json::from_str(&json).unwrap();

        assert!(!json.contains("drawings"), "{json}");
        assert!(back.drawings.is_empty());
    }

    /// Dash and fill default to solid and outline; shapes are tagged by kind.
    #[test]
    fn a_drawing_reads_with_defaults() {
        let json = r#"{"id":"d1","shape":{"kind":"area","points":[
            {"x_m":0.0,"y_m":0.0},{"x_m":1.0,"y_m":0.0},{"x_m":1.0,"y_m":1.0}]},"color":"red"}"#;

        let drawing: Drawing = serde_json::from_str(json).unwrap();

        assert_eq!(drawing.dash, Dash::Solid);
        assert_eq!(drawing.fill, Fill::Outline);
        assert!(matches!(drawing.shape, DrawingShape::Area { ref points } if points.len() == 3));
    }

    /// A drawing that is not turned writes no `rotation_deg`; one that is keeps it.
    #[test]
    fn a_drawings_turn_is_written_only_when_there_is_one() {
        let mut drawing = Drawing {
            id: "d1".into(),
            shape: DrawingShape::Rect {
                from: Point::new(0.0, 0.0),
                to: Point::new(4.0, 2.0),
            },
            color: FigureColor::Red,
            dash: Dash::Solid,
            fill: Fill::Outline,
            rotation_deg: 0.0,
        };

        assert!(!serde_json::to_string(&drawing)
            .unwrap()
            .contains("rotation"));

        drawing.rotation_deg = 30.0;
        let back: Drawing =
            serde_json::from_str(&serde_json::to_string(&drawing).unwrap()).unwrap();

        assert_eq!(back, drawing);
        assert_eq!(drawing.middle(), Some(Point::new(2.0, 1.0)));
    }

    /// Review of PR 1: the pause at the last mark was cut off (LH-63).
    #[test]
    fn duration_holds_at_the_last_mark() {
        let mut plan = FloorPlan::new("p", "Plan A");
        let at = Point::new(0.0, 0.0);
        plan.figures = vec![FigureTrack {
            character: "e".into(),
            marks: vec![Mark {
                at_s: 10.0,
                at,
                facing_deg: 0.0,
                pose: Pose::Standing,
                hold_s: 5.0,
                via: Vec::new(),
                curve: Default::default(),
                arrival: Arrival::Walk,
                departure: Departure::Stay,
            }],
        }];

        assert_eq!(plan.duration_s(), 15.0);
    }

    /// Review round 2: a pause longer than the way to the next mark counted in full.
    #[test]
    fn pause_ends_at_the_next_mark() {
        let mut plan = FloorPlan::new("p", "Plan A");
        let at = Point::new(0.0, 0.0);
        let mark = |at_s, hold_s| Mark {
            at_s,
            at,
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s,
            via: Vec::new(),
            curve: Default::default(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        };
        plan.figures = vec![FigureTrack {
            character: "e".into(),
            marks: vec![mark(0.0, 20.0), mark(5.0, 0.0)],
        }];

        assert_eq!(plan.duration_s(), 5.0);
    }

    /// A mark with the default arrival and departure writes neither key; an old file has
    /// the defaults; the other values round-trip (LH-26).
    #[test]
    fn arrival_and_departure_are_optional_in_json() {
        let mut mark: Mark =
            serde_json::from_str(r#"{"at_s":0.0,"at":{"x_m":1.0,"y_m":2.0},"facing_deg":0.0}"#)
                .unwrap();

        assert_eq!(mark.arrival, Arrival::Walk);
        assert_eq!(mark.departure, Departure::Stay);
        let json = serde_json::to_string(&mark).unwrap();
        assert!(
            !json.contains("arrival") && !json.contains("departure"),
            "{json}"
        );

        mark.arrival = Arrival::Appear;
        mark.departure = Departure::Leave;
        let json = serde_json::to_string(&mark).unwrap();
        let back: Mark = serde_json::from_str(&json).unwrap();

        assert!(json.contains(r#""arrival":"appear""#), "{json}");
        assert!(json.contains(r#""departure":"leave""#), "{json}");
        assert_eq!(back, mark);
    }

    /// A camera mark with the default rig and no lens writes neither key; an old file has the
    /// defaults; the other values round-trip (LH-33).
    #[test]
    fn rig_and_lens_are_optional_in_json() {
        let mut mark: CameraMark = serde_json::from_str(
            r#"{"at_s":0.0,"at":{"x_m":1.0,"y_m":2.0},"facing_deg":0.0,"height_m":1.6}"#,
        )
        .unwrap();

        assert_eq!((mark.rig, mark.lens_mm), (Rig::Dolly, None));
        let json = serde_json::to_string(&mark).unwrap();
        assert!(!json.contains("rig") && !json.contains("lens_mm"), "{json}");

        mark.rig = Rig::Crane;
        mark.lens_mm = Some(50.0);
        let json = serde_json::to_string(&mark).unwrap();
        let back: CameraMark = serde_json::from_str(&json).unwrap();

        assert!(json.contains(r#""rig":"crane""#), "{json}");
        assert_eq!(back, mark);
    }

    /// `name` is what the file holds and `from_name` reads it back.
    #[test]
    fn rig_names_are_the_file_names() {
        for rig in Rig::ALL {
            let json = serde_json::to_string(&rig).unwrap();
            assert_eq!(json, format!("\"{}\"", rig.name()));
            assert_eq!(Rig::from_name(rig.name()), Some(rig));
        }
        assert_eq!(Rig::from_name("drone"), None);
    }

    /// A known width gets its named format, any other stays as typed.
    #[test]
    fn a_width_finds_its_sensor() {
        assert_eq!(Sensor::of_width_mm(24.89), Sensor::Super35);
        assert_eq!(Sensor::of_width_mm(36.0), Sensor::FullFrame);
        assert_eq!(Sensor::of_width_mm(17.3), Sensor::MicroFourThirds);
        assert_eq!(Sensor::of_width_mm(23.5), Sensor::Custom { width_mm: 23.5 });
    }
}
