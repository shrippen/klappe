//! Locked versions of the screenplay (DB-40): a name, a day and the colour of the revision
//! they started.

use serde::{Deserialize, Serialize};

use crate::site::SunTime;

/// The colour of a revision, in the order the industry uses; the n-th locked version gets the
/// n-th colour, and the order starts again after the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RevisionColor {
    White,
    Blue,
    Pink,
    Yellow,
    Green,
    Goldenrod,
    Buff,
    Salmon,
    Cherry,
}

impl RevisionColor {
    pub const ALL: [RevisionColor; 9] = [
        RevisionColor::White,
        RevisionColor::Blue,
        RevisionColor::Pink,
        RevisionColor::Yellow,
        RevisionColor::Green,
        RevisionColor::Goldenrod,
        RevisionColor::Buff,
        RevisionColor::Salmon,
        RevisionColor::Cherry,
    ];

    /// The colour of the version that is the `n`-th locked, counted from 0.
    pub fn nth(n: usize) -> Self {
        Self::ALL[n % Self::ALL.len()]
    }

    /// The name the file format uses; also the end of the text key `revision_color.<name>`.
    pub fn name(self) -> &'static str {
        match self {
            RevisionColor::White => "white",
            RevisionColor::Blue => "blue",
            RevisionColor::Pink => "pink",
            RevisionColor::Yellow => "yellow",
            RevisionColor::Green => "green",
            RevisionColor::Goldenrod => "goldenrod",
            RevisionColor::Buff => "buff",
            RevisionColor::Salmon => "salmon",
            RevisionColor::Cherry => "cherry",
        }
    }
}

/// A locked version of the screenplay; its text is kept as it was when it was locked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Random, as every id of the project; names the file of the text.
    pub id: String,
    pub name: String,
    /// `2026-11-25`.
    pub date: String,
    pub color: RevisionColor,
}

/// Whether `text` is a day, `YYYY-MM-DD`, that exists.
pub fn is_version_date(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    let [year, month, day] = parts[..] else {
        return false;
    };
    let digits = |s: &str, len: usize| s.len() == len && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(year, 4) || !digits(month, 2) || !digits(day, 2) {
        return false;
    }

    let (Ok(year), Ok(month), Ok(day)) = (year.parse(), month.parse(), day.parse()) else {
        return false;
    };

    SunTime {
        year,
        month,
        day,
        hour: 0,
        minute: 0,
        utc_offset_min: None,
    }
    .is_valid()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_colours_start_again_after_the_last() {
        assert_eq!(RevisionColor::nth(0), RevisionColor::White);
        assert_eq!(RevisionColor::nth(8), RevisionColor::Cherry);
        assert_eq!(RevisionColor::nth(9), RevisionColor::White);
        assert_eq!(RevisionColor::nth(10), RevisionColor::Blue);
    }

    #[test]
    fn only_a_day_that_exists_is_a_date() {
        assert!(is_version_date("2026-11-25"));
        assert!(is_version_date("2024-02-29"));
        for bad in [
            "2026-02-29",
            "2026-13-01",
            "26-11-25",
            "2026-1-5",
            "",
            "heute",
        ] {
            assert!(!is_version_date(bad), "{bad}");
        }
    }
}
