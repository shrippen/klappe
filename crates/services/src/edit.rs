use klappe_model::{Camera, Movement, Shot, ShotSize};

use crate::error::ServiceError;

/// The fields of a shot the user edits directly (LH-71).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotField {
    Number,
    Camera,
    Size,
    Movement,
    Lens,
    Description,
    Props,
    Gear,
    Crew,
    Note,
}

impl ShotField {
    /// By its name as the GUI sends it, e.g. `"lens"`.
    pub fn from_name(name: &str) -> Option<Self> {
        let field = match name {
            "number" => Self::Number,
            "camera" => Self::Camera,
            "size" => Self::Size,
            "movement" => Self::Movement,
            "lens" => Self::Lens,
            "description" => Self::Description,
            "props" => Self::Props,
            "gear" => Self::Gear,
            "crew" => Self::Crew,
            "note" => Self::Note,
            _ => return None,
        };
        Some(field)
    }

    fn name(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Camera => "camera",
            Self::Size => "size",
            Self::Movement => "movement",
            Self::Lens => "lens",
            Self::Description => "description",
            Self::Props => "props",
            Self::Gear => "gear",
            Self::Crew => "crew",
            Self::Note => "note",
        }
    }
}

/// Focal lengths accepted; outside, the angle of view is absurd (1e-300 mm is ~180 degrees)
/// and `inf` would be saved as `null`. Also excludes NaN.
const MIN_LENS_MM: f64 = 1.0;
const MAX_LENS_MM: f64 = 2000.0;

/// Sets one field of `shot` from text. Camera: an id of `cameras`. Lens: millimetres,
/// empty = the camera's lens.
pub(crate) fn apply(
    shot: &mut Shot,
    cameras: &[Camera],
    field: ShotField,
    value: &str,
) -> Result<(), ServiceError> {
    let bad = || ServiceError::BadValue {
        field: field.name(),
        value: value.to_owned(),
    };

    match field {
        ShotField::Number => shot.number = value.trim().to_owned(),
        ShotField::Camera => shot.camera = camera_of(cameras, value).ok_or_else(bad)?,
        ShotField::Size => shot.size = size_of(value).ok_or_else(bad)?,
        ShotField::Movement => shot.movement = movement_of(value).ok_or_else(bad)?,
        ShotField::Lens => shot.lens_mm = lens_of(value).ok_or_else(bad)?,
        ShotField::Description => shot.description = value.to_owned(),
        ShotField::Props => shot.props = value.to_owned(),
        ShotField::Gear => shot.gear = value.to_owned(),
        ShotField::Crew => shot.crew = value.to_owned(),
        ShotField::Note => shot.note = value.to_owned(),
    }

    Ok(())
}

fn camera_of(cameras: &[Camera], value: &str) -> Option<String> {
    let id = value.trim();
    cameras.iter().find(|c| c.id == id).map(|c| c.id.clone())
}

/// `"85"`, `"85 mm"` or `"85,5"`; empty means "use the camera's lens".
fn lens_of(value: &str) -> Option<Option<f64>> {
    let digits = value.trim().trim_end_matches("mm").trim().replace(',', ".");
    if digits.is_empty() {
        return Some(None);
    }

    let mm: f64 = digits.parse().ok()?;
    (MIN_LENS_MM..=MAX_LENS_MM)
        .contains(&mm)
        .then_some(Some(mm))
}

/// Names as the model serializes them, e.g. `close-up`.
pub(crate) fn size_name(size: ShotSize) -> &'static str {
    match size {
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

const SIZES: [ShotSize; 8] = [
    ShotSize::ExtremeWide,
    ShotSize::Wide,
    ShotSize::MediumWide,
    ShotSize::Cowboy,
    ShotSize::Medium,
    ShotSize::MediumClose,
    ShotSize::CloseUp,
    ShotSize::ExtremeCloseUp,
];

fn size_of(name: &str) -> Option<ShotSize> {
    SIZES.into_iter().find(|s| size_name(*s) == name)
}

pub(crate) fn movement_name(movement: Movement) -> &'static str {
    match movement {
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

const MOVEMENTS: [Movement; 8] = [
    Movement::Static,
    Movement::Pan,
    Movement::Tilt,
    Movement::Dolly,
    Movement::Crane,
    Movement::Handheld,
    Movement::Gimbal,
    Movement::Zoom,
];

fn movement_of(name: &str) -> Option<Movement> {
    MOVEMENTS.into_iter().find(|m| movement_name(*m) == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_accepts_units_and_commas() {
        assert_eq!(lens_of("85"), Some(Some(85.0)));
        assert_eq!(lens_of(" 85 mm "), Some(Some(85.0)));
        assert_eq!(lens_of("85,5"), Some(Some(85.5)));
        assert_eq!(lens_of(""), Some(None));
        assert_eq!(lens_of("-1"), None);
        assert_eq!(lens_of("weit"), None);
    }

    #[test]
    fn names_round_trip() {
        for s in SIZES {
            assert_eq!(size_of(size_name(s)), Some(s));
        }
        for m in MOVEMENTS {
            assert_eq!(movement_of(movement_name(m)), Some(m));
        }
    }

    #[test]
    fn field_names_round_trip() {
        let fields = [
            ShotField::Number,
            ShotField::Camera,
            ShotField::Size,
            ShotField::Movement,
            ShotField::Lens,
            ShotField::Description,
            ShotField::Props,
            ShotField::Gear,
            ShotField::Crew,
            ShotField::Note,
        ];
        for f in fields {
            assert_eq!(ShotField::from_name(f.name()), Some(f));
        }
    }
}
