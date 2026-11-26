//! Locked versions of the screenplay (DB-40, DB-41): what a name, a day and the revision's
//! texts are made of. The session's part is `session/versions.rs`.

use std::time::{SystemTime, UNIX_EPOCH};

use klappe_model::{RevisionColor, ScriptLanguage, ScriptSetup, Version};
use klappe_script::{SceneDiff, ScenePlace, SceneStatus};

use crate::catalog::Language;
use crate::error::ServiceError;

const SECONDS_PER_DAY: u64 = 86_400;
/// Days from 0000-03-01 to 1970-01-01, the shift of `civil_from_days`.
const DAYS_TO_UNIX_EPOCH: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;

/// The name as kept: without the blanks at its ends; empty or already used by a version
/// (case aside) is refused.
pub(crate) fn check_name<'a>(name: &'a str, versions: &[Version]) -> Result<&'a str, ServiceError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ServiceError::EmptyVersionName);
    }

    let lower = name.to_lowercase();
    if versions.iter().any(|v| v.name.to_lowercase() == lower) {
        return Err(ServiceError::VersionNameTaken(name.to_owned()));
    }

    Ok(name)
}

/// The colour of the next version: the one after the last version's, white for the first.
pub(crate) fn next_color(versions: &[Version]) -> RevisionColor {
    versions
        .last()
        .map_or(RevisionColor::White, |v| v.color.next())
}

/// Today as `YYYY-MM-DD`, in UTC.
pub(crate) fn today() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    date_text((seconds / SECONDS_PER_DAY) as i64)
}

/// The day that is `days` after 1970-01-01, as `YYYY-MM-DD` (H. Hinnant's `civil_from_days`).
fn date_text(days: i64) -> String {
    let shifted = days + DAYS_TO_UNIX_EPOCH;
    let era = shifted.div_euclid(DAYS_PER_ERA);
    let day_of_era = shifted.rem_euclid(DAYS_PER_ERA);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);

    format!("{year:04}-{month:02}-{day:02}")
}

/// The language the revision's words are in: the project's, else the surface's.
pub(crate) fn text_language(setup: &ScriptSetup, fallback: Language) -> Language {
    match setup.language {
        Some(ScriptLanguage::De) => Language::German,
        Some(ScriptLanguage::En) => Language::English,
        None => fallback,
    }
}

/// The colour's name in `language`: `Blau`.
pub fn color_text(language: Language, color: RevisionColor) -> String {
    language.text(&format!("revision_color.{}", color.name()))
}

/// What became of a scene, in words: `changed, moved`; `moved` for one that only moved;
/// `unchanged` when nothing did.
pub fn scene_status_text(language: Language, scene: &SceneDiff) -> String {
    let mut parts = Vec::new();
    match scene.status {
        SceneStatus::New => parts.push("compare.new"),
        SceneStatus::Removed => parts.push("compare.removed"),
        SceneStatus::Changed => parts.push("compare.changed"),
        SceneStatus::Same => {}
    }
    if scene.place == ScenePlace::Moved {
        parts.push("compare.moved");
    }
    if parts.is_empty() {
        parts.push("compare.same");
    }

    let words: Vec<_> = parts.into_iter().map(|key| language.text(key)).collect();

    words.join(", ")
}

/// The heading of a scene as shown; `(ohne Überschrift)` for a scene without one.
pub fn scene_heading_text(language: Language, heading: Option<&str>) -> String {
    match heading {
        Some(h) if !h.is_empty() => h.to_owned(),
        _ => language.text("version.noHeading"),
    }
}

/// The line above the pages of a revision: `Revision Blau · 2026-11-25`, the colour of the
/// next version and today.
pub(crate) fn header_text(language: Language, color: RevisionColor, date: &str) -> String {
    language
        .text("revision.header")
        .replace("{color}", &color_text(language, color))
        .replace("{date}", date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(name: &str) -> Version {
        Version {
            id: "a".into(),
            name: name.into(),
            date: "2026-11-25".into(),
            color: RevisionColor::White,
            pages: Vec::new(),
        }
    }

    #[test]
    fn the_day_count_gives_the_date() {
        assert_eq!(date_text(0), "1970-01-01");
        assert_eq!(date_text(11_017), "2000-03-01");
        assert_eq!(date_text(19_782), "2024-02-29");
        assert_eq!(date_text(20_625), "2026-06-21");
        assert_eq!(date_text(-1), "1969-12-31");
    }

    #[test]
    fn today_is_a_date() {
        assert!(klappe_model::is_version_date(&today()));
    }

    #[test]
    fn a_name_is_trimmed_and_neither_empty_nor_used() {
        let used = [version("Drehfassung")];

        assert_eq!(check_name("  Nachdreh ", &used).unwrap(), "Nachdreh");
        assert!(matches!(
            check_name("   ", &used),
            Err(ServiceError::EmptyVersionName)
        ));
        assert!(matches!(
            check_name("drehfassung", &used),
            Err(ServiceError::VersionNameTaken(_))
        ));
    }

    fn scene(status: SceneStatus, place: ScenePlace, heading: Option<&str>) -> SceneDiff {
        SceneDiff {
            number: Some("3".into()),
            heading: heading.map(Into::into),
            old_number: None,
            old_heading: None,
            status,
            place,
            lines: Vec::new(),
        }
    }

    /// Owner decision 26.11.: one wording for window and command line.
    #[test]
    fn a_scene_status_names_what_became_of_it() {
        let text = |s, p| scene_status_text(Language::German, &scene(s, p, None));

        assert_eq!(text(SceneStatus::Same, ScenePlace::Kept), "unverändert");
        assert_eq!(text(SceneStatus::Same, ScenePlace::Moved), "verschoben");
        assert_eq!(
            text(SceneStatus::Changed, ScenePlace::Moved),
            "geändert, verschoben"
        );
        assert_eq!(text(SceneStatus::New, ScenePlace::Kept), "neu");
    }

    #[test]
    fn a_scene_without_heading_says_so() {
        assert_eq!(
            scene_heading_text(Language::German, None),
            "(ohne Überschrift)"
        );
        assert_eq!(
            scene_heading_text(Language::German, Some("")),
            "(ohne Überschrift)"
        );
        assert_eq!(
            scene_heading_text(Language::German, Some("INT. HOF")),
            "INT. HOF"
        );
    }

    #[test]
    fn the_header_names_the_colour_and_the_day() {
        let text = header_text(Language::German, RevisionColor::Blue, "2026-11-25");

        assert_eq!(text, "Revision Blau · 2026-11-25");
    }
}
