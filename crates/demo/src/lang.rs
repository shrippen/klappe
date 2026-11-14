//! The two example languages and the `{de, en}` texts of the demo world.

use serde_json::{Map, Value};

use crate::error::DemoError;

#[derive(Debug, Clone, Copy)]
pub enum Lang {
    De,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::De, Lang::En];

    pub fn code(self) -> &'static str {
        match self {
            Lang::De => "de",
            Lang::En => "en",
        }
    }
}

/// Whether `map` is a text in several languages: keys `de` and `en` only.
fn is_text(map: &Map<String, Value>) -> bool {
    let all_langs = map.keys().all(|k| Lang::ALL.iter().any(|l| l.code() == k));

    !map.is_empty() && all_langs
}

/// Replaces every `{de, en}` object of `node` by its text in `lang`.
///
/// Example: `{"name": {"de": "Tisch", "en": "Table"}}` becomes `{"name": "Tisch"}` for `De`.
pub fn localize(node: &Value, lang: Lang) -> Result<Value, DemoError> {
    match node {
        Value::Object(map) if is_text(map) => {
            map.get(lang.code())
                .cloned()
                .ok_or_else(|| DemoError::NoText {
                    lang: lang.code(),
                    what: node.to_string(),
                })
        }
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| Ok((k.clone(), localize(v, lang)?)))
            .collect::<Result<Map<_, _>, DemoError>>()
            .map(Value::Object),
        Value::Array(items) => items
            .iter()
            .map(|v| localize(v, lang))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn picks_the_language_everywhere() {
        let node = json!({"a": [{"de": "Tisch", "en": "Table"}], "b": {"x_m": 1.0}});

        let de = localize(&node, Lang::De).unwrap();
        let en = localize(&node, Lang::En).unwrap();

        assert_eq!(de, json!({"a": ["Tisch"], "b": {"x_m": 1.0}}));
        assert_eq!(en, json!({"a": ["Table"], "b": {"x_m": 1.0}}));
    }

    #[test]
    fn a_missing_side_is_an_error() {
        let node = json!({"name": {"de": "Tisch"}});

        let err = localize(&node, Lang::En).unwrap_err();

        assert!(matches!(err, DemoError::NoText { lang: "en", .. }), "{err}");
    }
}
