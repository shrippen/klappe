use klappe_model::{Angle, Camera, Movement, Shot, ShotSize};

use crate::error::ServiceError;
use crate::field::Field;
use crate::number::{read_number, Range, Unit};

/// The fields of a shot the user edits directly (LH-71).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShotField {
    Number,
    Camera,
    Size,
    Angle,
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
            "angle" => Self::Angle,
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

    fn field(self) -> Field {
        match self {
            Self::Number => Field::Number,
            Self::Camera => Field::Camera,
            Self::Size => Field::Size,
            Self::Angle => Field::Angle,
            Self::Movement => Field::Movement,
            Self::Lens => Field::Lens,
            Self::Description => Field::Description,
            Self::Props => Field::Props,
            Self::Gear => Field::Gear,
            Self::Crew => Field::Crew,
            Self::Note => Field::Note,
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
        field: field.field(),
        value: value.to_owned(),
    };

    match field {
        ShotField::Number => shot.number = number_of(value).ok_or_else(bad)?,
        ShotField::Camera => shot.camera = camera_of(cameras, value).ok_or_else(bad)?,
        ShotField::Size => shot.size = size_of(value).ok_or_else(bad)?,
        ShotField::Angle => shot.angle = angle_of(value).ok_or_else(bad)?,
        ShotField::Movement => shot.movement = movement_of(value).ok_or_else(bad)?,
        ShotField::Lens => shot.lens_mm = lens_of(value)?,
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
fn lens_of(value: &str) -> Result<Option<f64>, ServiceError> {
    if value.trim().is_empty() {
        return Ok(None);
    }

    let range = Range::Within(MIN_LENS_MM, MAX_LENS_MM);

    read_number(Field::Lens, value, Unit::Millimetre, range).map(Some)
}

fn size_of(name: &str) -> Option<ShotSize> {
    ShotSize::ALL.into_iter().find(|s| s.name() == name)
}

fn angle_of(name: &str) -> Option<Angle> {
    Angle::ALL.into_iter().find(|a| a.name() == name)
}

fn movement_of(name: &str) -> Option<Movement> {
    Movement::ALL.into_iter().find(|m| m.name() == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lens_accepts_units_and_commas() {
        assert_eq!(lens_of("85").ok(), Some(Some(85.0)));
        assert_eq!(lens_of(" 85 mm ").ok(), Some(Some(85.0)));
        assert_eq!(lens_of("85,5").ok(), Some(Some(85.5)));
        assert_eq!(lens_of("").ok(), Some(None));
        assert_eq!(lens_of("-1").ok(), None);
        assert_eq!(lens_of("weit").ok(), None);
    }

    #[test]
    fn lens_has_a_range() {
        assert_eq!(lens_of("0,5").ok(), None);
        assert_eq!(lens_of("2001").ok(), None);
        assert_eq!(lens_of("inf").ok(), None);
        assert_eq!(lens_of("NaN").ok(), None);
        assert_eq!(lens_of("85 m").ok(), None);
    }

    #[test]
    fn names_round_trip() {
        for s in ShotSize::ALL {
            assert_eq!(size_of(s.name()), Some(s));
        }
        for a in Angle::ALL {
            assert_eq!(angle_of(a.name()), Some(a));
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
            ShotField::Angle,
            ShotField::Movement,
            ShotField::Lens,
            ShotField::Description,
            ShotField::Props,
            ShotField::Gear,
            ShotField::Crew,
            ShotField::Note,
        ];
        for f in fields {
            assert_eq!(ShotField::from_name(f.field().name()), Some(f));
        }
    }
}
