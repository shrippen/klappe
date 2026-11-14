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
        ShotField::Number => shot.number = number_of(value).ok_or_else(bad)?,
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

/// A shot number is never empty.
fn number_of(value: &str) -> Option<String> {
    let number = value.trim();
    (!number.is_empty()).then(|| number.to_owned())
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

fn size_of(name: &str) -> Option<ShotSize> {
    ShotSize::ALL.into_iter().find(|s| s.name() == name)
}

fn movement_of(name: &str) -> Option<Movement> {
    Movement::ALL.into_iter().find(|m| m.name() == name)
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
        for s in ShotSize::ALL {
            assert_eq!(size_of(s.name()), Some(s));
        }
        for m in Movement::ALL {
            assert_eq!(movement_of(m.name()), Some(m));
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
