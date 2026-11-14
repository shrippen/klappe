use serde::ser::Error;
use serde::{Deserialize, Serialize, Serializer};

/// Refuses NaN and infinity: JSON would write them as `null`, which reads back as `None`.
fn finite_or_none<S: Serializer>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) if !v.is_finite() => Err(S::Error::custom(format!("non-finite number: {v}"))),
        _ => value.serialize(serializer),
    }
}

/// One shot: a camera's take of a stretch of the scene (LH-70, LH-71).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shot {
    pub id: String,
    /// Shot number as written on the slate, e.g. `12.3` or `12.4A` (LH-76).
    pub number: String,
    pub camera: String,
    pub size: ShotSize,
    pub movement: Movement,
    /// Lens for this shot; `None` uses the camera's.
    #[serde(default, serialize_with = "finite_or_none")]
    pub lens_mm: Option<f64>,
    /// The stretch of the blocking this shot covers.
    pub from_s: f64,
    pub to_s: f64,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub props: String,
    #[serde(default)]
    pub gear: String,
    #[serde(default)]
    pub crew: String,
    #[serde(default)]
    pub note: String,
}

impl Shot {
    pub fn duration_s(&self) -> f64 {
        (self.to_s - self.from_s).max(0.0)
    }
}

/// Shot sizes as German film practice names them (NF-09), widest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShotSize {
    ExtremeWide,
    Wide,
    MediumWide,
    Cowboy,
    Medium,
    MediumClose,
    CloseUp,
    ExtremeCloseUp,
}

/// LH-33.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Movement {
    Static,
    Pan,
    Tilt,
    Dolly,
    Crane,
    Handheld,
    Gimbal,
    Zoom,
}
