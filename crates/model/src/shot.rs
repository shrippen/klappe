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
    /// Short name next to the number (LH-71); empty for none.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub short_name: String,
    pub size: ShotSize,
    /// Camera angle: from above, below, tilted (LH-71).
    #[serde(default)]
    pub angle: Angle,
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
    /// Id of another shot of the plan: this shot is shot in that shot's setup (LH-77). `None`
    /// computes the setup from camera, place and lens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setup_of: Option<String>,
    /// The storyboard picture (LH-74): a file of the project's `bilder/`, e.g. `bilder/s12-3.png`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

impl Shot {
    pub fn duration_s(&self) -> f64 {
        (self.to_s - self.from_s).max(0.0)
    }
}

/// How the numbers of a project's shots are built (LH-76); the project chooses one. Each letter
/// has one meaning in a scheme: a camera letter is never also a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Numbering {
    /// Scene and running number, `12.3`; a version is a letter behind it, `12.3A`.
    #[default]
    Scene,
    /// Scene, camera letter, dot and the camera's running number, `12A.3`; a version is a
    /// hyphen and letters, `12A.3-B`.
    Camera,
    /// Scene, dot and the running number of a group of shots taken at the same time, then the
    /// camera letter, `12.3A`, `12.3B`; a version is a hyphen and letters, `12.3A-B`.
    Group,
}

impl Numbering {
    pub const ALL: [Numbering; 3] = [Numbering::Scene, Numbering::Camera, Numbering::Group];

    /// The name the file format uses, e.g. `group`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Numbering::Scene => "scene",
            Numbering::Camera => "camera",
            Numbering::Group => "group",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|n| n.name() == name)
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

impl ShotSize {
    /// Every size, widest first.
    pub const ALL: [ShotSize; 8] = [
        ShotSize::ExtremeWide,
        ShotSize::Wide,
        ShotSize::MediumWide,
        ShotSize::Cowboy,
        ShotSize::Medium,
        ShotSize::MediumClose,
        ShotSize::CloseUp,
        ShotSize::ExtremeCloseUp,
    ];

    /// The name the file format uses, e.g. `close-up`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            ShotSize::ExtremeWide => "extreme-wide",
            ShotSize::Wide => "wide",
            ShotSize::MediumWide => "medium-wide",
            ShotSize::Cowboy => "cowboy",
            ShotSize::Medium => "medium",
            ShotSize::MediumClose => "medium-close",
            ShotSize::CloseUp => "close-up",
            ShotSize::ExtremeCloseUp => "extreme-close-up",
        }
    }
}

/// Camera perspective as German film practice names it (NF-09), eye level first (LH-71).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Angle {
    /// Normalsicht.
    #[default]
    EyeLevel,
    /// Aufsicht.
    High,
    /// Untersicht.
    Low,
    /// Vogelperspektive.
    BirdsEye,
    /// Froschperspektive.
    WormsEye,
    /// Gekippte Kamera (Dutch Angle).
    Dutch,
}

impl Angle {
    pub const ALL: [Angle; 6] = [
        Angle::EyeLevel,
        Angle::High,
        Angle::Low,
        Angle::BirdsEye,
        Angle::WormsEye,
        Angle::Dutch,
    ];

    /// The name the file format uses, e.g. `birds-eye`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Angle::EyeLevel => "eye-level",
            Angle::High => "high",
            Angle::Low => "low",
            Angle::BirdsEye => "birds-eye",
            Angle::WormsEye => "worms-eye",
            Angle::Dutch => "dutch",
        }
    }
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

impl Movement {
    pub const ALL: [Movement; 8] = [
        Movement::Static,
        Movement::Pan,
        Movement::Tilt,
        Movement::Dolly,
        Movement::Crane,
        Movement::Handheld,
        Movement::Gimbal,
        Movement::Zoom,
    ];

    /// The name the file format uses, e.g. `handheld`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Movement::Static => "static",
            Movement::Pan => "pan",
            Movement::Tilt => "tilt",
            Movement::Dolly => "dolly",
            Movement::Crane => "crane",
            Movement::Handheld => "handheld",
            Movement::Gimbal => "gimbal",
            Movement::Zoom => "zoom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `name` is what the file holds, and `ALL` has no variant twice. (That `ALL` has every
    /// variant no test can show; `name`'s match lists them all for the compiler.)
    #[test]
    fn names_are_the_file_names() {
        for size in ShotSize::ALL {
            let json = serde_json::to_string(&size).unwrap();
            assert_eq!(json, format!("\"{}\"", size.name()));
        }
        for movement in Movement::ALL {
            let json = serde_json::to_string(&movement).unwrap();
            assert_eq!(json, format!("\"{}\"", movement.name()));
        }
        for angle in Angle::ALL {
            let json = serde_json::to_string(&angle).unwrap();
            assert_eq!(json, format!("\"{}\"", angle.name()));
        }

        let mut sizes: Vec<_> = ShotSize::ALL.iter().map(|s| s.name()).collect();
        let mut movements: Vec<_> = Movement::ALL.iter().map(|m| m.name()).collect();
        sizes.sort_unstable();
        sizes.dedup();
        movements.sort_unstable();
        movements.dedup();
        assert_eq!(sizes.len(), ShotSize::ALL.len());
        assert_eq!(movements.len(), Movement::ALL.len());

        let mut angles: Vec<_> = Angle::ALL.iter().map(|a| a.name()).collect();
        angles.sort_unstable();
        angles.dedup();
        assert_eq!(angles.len(), Angle::ALL.len());
    }

    /// Files from before the field have no `angle`: eye level.
    #[test]
    fn a_shot_without_angle_is_eye_level() {
        let json = r#"{"id":"s","number":"1","camera":"a","size":"wide","movement":"static",
            "from_s":0.0,"to_s":1.0}"#;
        let shot: Shot = serde_json::from_str(json).unwrap();

        assert_eq!(shot.angle, Angle::EyeLevel);
    }
}
