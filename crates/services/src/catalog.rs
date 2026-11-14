//! German and English texts (agent.md: no texts in code). The app hands the whole catalogue
//! to QML as JSON (`Texts.qml`); Rust reads single keys for what it prints, so does the CLI
//! for its exports.

use klappe_export::{ShotColumns, ShotTexts, Texts};
use klappe_model::{Angle, Movement, ShotSize};
use serde_json::{Map, Value};

const DE: &str = include_str!("../i18n/de.json");
const EN: &str = include_str!("../i18n/en.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    German,
    English,
}

impl Language {
    /// `de` or `en`; else from the system (`LANG`), German only when it says so.
    pub fn pick(wanted: Option<&str>) -> Self {
        let system = std::env::var("LANG").unwrap_or_default();
        let code = wanted.unwrap_or(&system);
        if code.starts_with("de") {
            return Self::German;
        }
        Self::English
    }

    pub fn json(self) -> &'static str {
        match self {
            Self::German => DE,
            Self::English => EN,
        }
    }

    /// One text; the key itself if it is missing (visible, so a gap is noticed).
    pub fn text(self, key: &str) -> String {
        let map: Map<String, Value> = serde_json::from_str(self.json()).unwrap_or_default();
        map.get(key)
            .and_then(Value::as_str)
            .unwrap_or(key)
            .to_owned()
    }

    /// The printed texts of the exports (E-05: no texts in code).
    pub fn print_texts(self) -> Texts {
        let t = |k: &str| self.text(k);

        Texts {
            figure: t("legend.figure"),
            camera: t("legend.camera"),
            light: t("legend.light"),
            axis: t("legend.axis"),
            path: t("legend.path"),
            mark: t("legend.mark"),
            unit: t("unit.m"),
            decimal_separator: t("unit.decimal").chars().next().unwrap_or('.'),
            shots: self.shot_texts(),
        }
    }

    /// The shot list's words.
    pub fn shot_texts(self) -> ShotTexts {
        let t = |k: &str| self.text(k);

        ShotTexts {
            title: t("shots.title"),
            columns: ShotColumns {
                number: t("field.number"),
                camera: t("field.camera"),
                lens_mm: t("field.lens"),
                setup: t("field.setup"),
                size: t("field.size"),
                angle: t("field.perspective"),
                movement: t("field.movement"),
                from_s: t("field.from"),
                to_s: t("field.to"),
                duration_s: t("field.duration"),
                description: t("field.description"),
                props: t("field.props"),
                gear: t("field.gear"),
                crew: t("field.crew"),
                note: t("field.note"),
            },
            sizes: ShotSize::ALL
                .iter()
                .map(|s| (*s, t(&format!("size.{}", s.name()))))
                .collect(),
            angles: Angle::ALL
                .iter()
                .map(|a| (*a, t(&format!("angle.{}", a.name()))))
                .collect(),
            movements: Movement::ALL
                .iter()
                .map(|m| (*m, t(&format!("movement.{}", m.name()))))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(json: &str) -> Vec<String> {
        let map: Map<String, Value> = serde_json::from_str(json).unwrap_or_default();
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();
        keys
    }

    #[test]
    fn both_catalogues_have_the_same_keys() {
        assert!(!keys(DE).is_empty());
        assert_eq!(keys(DE), keys(EN));
    }

    #[test]
    fn picks_by_option_then_system() {
        assert_eq!(Language::pick(Some("de")), Language::German);
        assert_eq!(Language::pick(Some("en")), Language::English);
        assert_eq!(Language::German.text("legend.axis"), "Achse");
        assert_eq!(Language::English.text("nope"), "nope");
    }

    #[test]
    fn print_texts_follow_the_language() {
        assert_eq!(Language::German.print_texts().axis, "Achse");
        assert_eq!(Language::German.print_texts().decimal_separator, ',');
        assert_eq!(Language::English.print_texts().decimal_separator, '.');
    }
}
