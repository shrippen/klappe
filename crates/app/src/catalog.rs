//! German and English texts (agent.md: no texts in code). QML gets the whole catalogue as
//! JSON (`Texts.qml`); Rust reads single keys for what it prints.

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
}
