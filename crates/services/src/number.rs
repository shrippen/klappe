//! Numbers as typed: one reader for every field, so a unit, a decimal comma and a range
//! mean the same everywhere.

use crate::error::ServiceError;
use crate::field::Field;

/// The unit a field is in; typing it after the number is optional (`8 m` and `8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    None,
    Metre,
    Millimetre,
    Second,
    Degree,
}

impl Unit {
    fn suffix(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Metre => "m",
            Self::Millimetre => "mm",
            Self::Second => "s",
            Self::Degree => "°",
        }
    }
}

/// What a field accepts. Every range excludes NaN and infinity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Range {
    /// Any finite number, of either sign.
    Any,
    /// More than zero.
    Positive,
    /// Zero or more.
    NotNegative,
    /// Between the two, both included.
    Within(f64, f64),
}

impl Range {
    fn contains(self, number: f64) -> bool {
        match self {
            Self::Any => number.is_finite(),
            Self::Positive => number.is_finite() && number > 0.0,
            Self::NotNegative => number.is_finite() && number >= 0.0,
            Self::Within(min, max) => (min..=max).contains(&number),
        }
    }
}

/// `text` as a number of `unit` within `range`: a comma or a point as decimal separator,
/// the unit optional (`"8,5 m"`, `"8.5"`). Anything else is a `BadValue` for `field`.
pub fn read_number(
    field: Field,
    text: &str,
    unit: Unit,
    range: Range,
) -> Result<f64, ServiceError> {
    let typed = text.trim();
    let digits = typed.strip_suffix(unit.suffix()).unwrap_or(typed);
    let number: Option<f64> = digits.trim().replace(',', ".").parse().ok();

    number
        .filter(|n| range.contains(*n))
        .ok_or_else(|| ServiceError::BadValue {
            field,
            value: typed.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(text: &str, unit: Unit, range: Range) -> Option<f64> {
        read_number(Field::Width, text, unit, range).ok()
    }

    #[test]
    fn comma_point_and_space_are_the_same_number() {
        assert_eq!(read("8,5", Unit::None, Range::Any), Some(8.5));
        assert_eq!(read(" 8.5 ", Unit::None, Range::Any), Some(8.5));
        assert_eq!(read("-3", Unit::None, Range::Any), Some(-3.0));
    }

    #[test]
    fn the_own_unit_is_optional() {
        assert_eq!(read("85", Unit::Millimetre, Range::Any), Some(85.0));
        assert_eq!(read(" 85 mm ", Unit::Millimetre, Range::Any), Some(85.0));
        assert_eq!(read("85mm", Unit::Millimetre, Range::Any), Some(85.0));
        assert_eq!(read("8,5 m", Unit::Metre, Range::Any), Some(8.5));
        assert_eq!(read("2 s", Unit::Second, Range::Any), Some(2.0));
        assert_eq!(read("-10°", Unit::Degree, Range::Any), Some(-10.0));
    }

    #[test]
    fn another_unit_is_refused() {
        assert_eq!(read("85 m", Unit::Millimetre, Range::Any), None);
        assert_eq!(read("8 mm", Unit::Metre, Range::Any), None);
        assert_eq!(read("8 m", Unit::None, Range::Any), None);
        assert_eq!(read("8 mmm", Unit::Millimetre, Range::Any), None);
    }

    #[test]
    fn a_range_decides() {
        assert_eq!(read("0", Unit::None, Range::Positive), None);
        assert_eq!(read("0", Unit::None, Range::NotNegative), Some(0.0));
        assert_eq!(read("-1", Unit::None, Range::NotNegative), None);
        assert_eq!(read("1", Unit::None, Range::Within(1.0, 2.0)), Some(1.0));
        assert_eq!(read("2,1", Unit::None, Range::Within(1.0, 2.0)), None);
    }

    #[test]
    fn no_number_is_no_number() {
        for text in [
            "", " ", "abc", "1,234.5", "m", "NaN", "inf", "-inf", "1e999",
        ] {
            assert_eq!(read(text, Unit::Metre, Range::Any), None, "{text:?}");
        }
    }

    #[test]
    fn the_error_names_the_field_and_the_text() {
        let err = read_number(Field::Lens, " weit ", Unit::Millimetre, Range::Positive);

        assert!(matches!(
            err,
            Err(ServiceError::BadValue { field: Field::Lens, value }) if value == "weit"
        ));
    }
}
