//! Where a location lies and when a scene is shot, so that the sun can be placed (LH-52).

use serde::{Deserialize, Serialize};

const MAX_LATITUDE_DEG: f64 = 90.0;
const MAX_LONGITUDE_DEG: f64 = 180.0;
/// The calendar years the sun's position is calculated for with the accuracy the plan needs.
const FIRST_YEAR: i32 = 1900;
const LAST_YEAR: i32 = 2100;
const MINUTES_PER_HOUR: i16 = 60;
/// The widest offset a zone has (Kiribati, +14:00).
pub const MAX_UTC_OFFSET_MIN: i16 = 14 * MINUTES_PER_HOUR;
const FEBRUARY: u8 = 2;
const MONTHS: u8 = 12;

/// The place of a location on Earth and the way its plan lies (LH-52).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Site {
    /// North positive, in `[-90, 90]`.
    pub latitude_deg: f64,
    /// East positive, in `[-180, 180]`.
    pub longitude_deg: f64,
    /// How far north lies clockwise from the top of the plan: 0 is north at the top, 90 north
    /// to the right.
    #[serde(default)]
    pub north_deg: f64,
}

impl Site {
    /// Whether the place can exist; also excludes NaN.
    pub fn is_valid(&self) -> bool {
        self.latitude_deg.abs() <= MAX_LATITUDE_DEG
            && self.longitude_deg.abs() <= MAX_LONGITUDE_DEG
            && self.north_deg.is_finite()
    }
}

/// The wall-clock moment of a scene (LH-52): date and time as the crew's clocks show it, with
/// the offset from UTC of that clock.
///
/// The offset is stored, not worked out from the longitude: summer time and zone borders have
/// no rule a plan could know offline, and a call sheet says `14:30` by the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SunTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    /// Minutes the clock is ahead of UTC (`+120` for CEST); `None` takes the zone the
    /// location's longitude lies in, whole hours, without summer time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utc_offset_min: Option<i16>,
}

impl SunTime {
    /// Whether the date exists (the leap day included), the time is on a clock, and the offset
    /// is one a zone has.
    pub fn is_valid(&self) -> bool {
        let offset_ok = self
            .utc_offset_min
            .is_none_or(|o| o.abs() <= MAX_UTC_OFFSET_MIN);

        (FIRST_YEAR..=LAST_YEAR).contains(&self.year)
            && (1..=MONTHS).contains(&self.month)
            && (1..=days_in_month(self.year, self.month)).contains(&self.day)
            && self.hour < 24
            && self.minute < 60
            && offset_ok
    }

    /// `2026-06-21`.
    pub fn date_text(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// `14:30`.
    pub fn time_text(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// `+02:00`; empty while the zone follows the longitude.
    pub fn offset_text(&self) -> String {
        let Some(offset) = self.utc_offset_min else {
            return String::new();
        };
        let sign = if offset < 0 { '-' } else { '+' };
        let (hours, minutes) = (
            offset.abs() / MINUTES_PER_HOUR,
            offset.abs() % MINUTES_PER_HOUR,
        );

        format!("{sign}{hours:02}:{minutes:02}")
    }
}

/// How many days `month` of `year` has.
fn days_in_month(year: i32, month: u8) -> u8 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;

    match month {
        FEBRUARY if leap => 29,
        FEBRUARY => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(year: i32, month: u8, day: u8) -> SunTime {
        SunTime {
            year,
            month,
            day,
            hour: 12,
            minute: 0,
            utc_offset_min: None,
        }
    }

    #[test]
    fn only_a_date_that_exists_is_valid() {
        assert!(time(2024, 2, 29).is_valid());
        assert!(!time(2026, 2, 29).is_valid());
        assert!(!time(2100, 2, 29).is_valid());
        assert!(!time(2026, 4, 31).is_valid());
        assert!(!time(2026, 13, 1).is_valid());
        assert!(!time(2026, 6, 0).is_valid());
    }

    #[test]
    fn a_clock_and_an_offset_have_limits() {
        let mut t = time(2026, 6, 21);
        t.hour = 24;
        assert!(!t.is_valid());

        t.hour = 23;
        t.minute = 60;
        assert!(!t.is_valid());

        t.minute = 59;
        t.utc_offset_min = Some(15 * 60);
        assert!(!t.is_valid());

        t.utc_offset_min = Some(-5 * 60);
        assert!(t.is_valid());
    }

    #[test]
    fn the_texts_read_back_as_typed() {
        let mut t = time(2026, 6, 21);
        t.hour = 9;
        t.minute = 5;
        t.utc_offset_min = Some(-330);

        assert_eq!(t.date_text(), "2026-06-21");
        assert_eq!(t.time_text(), "09:05");
        assert_eq!(t.offset_text(), "-05:30");

        t.utc_offset_min = None;
        assert_eq!(t.offset_text(), "");
    }

    #[test]
    fn a_site_stays_on_the_globe() {
        let site = |latitude_deg, longitude_deg| Site {
            latitude_deg,
            longitude_deg,
            north_deg: 0.0,
        };

        assert!(site(52.5, 13.4).is_valid());
        assert!(!site(90.1, 0.0).is_valid());
        assert!(!site(0.0, -180.5).is_valid());
        assert!(!site(f64::NAN, 0.0).is_valid());
    }
}
