use serde::{Deserialize, Serialize};

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

/// Whether the inside of a closed shape is tinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fill {
    #[default]
    Outline,
    Filled,
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
}

/// Camera position at a moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraMark {
    pub at_s: f64,
    pub at: Point,
    pub facing_deg: f64,
    pub height_m: f64,
    #[serde(default)]
    pub tilt_deg: f64,
    #[serde(default)]
    pub hold_s: f64,
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
        };
        plan.figures = vec![FigureTrack {
            character: "e".into(),
            marks: vec![mark(0.0, 20.0), mark(5.0, 0.0)],
        }];

        assert_eq!(plan.duration_s(), 5.0);
    }
}
