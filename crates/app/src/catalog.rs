//! The texts live in `klappe-services` (the CLI prints with them too); this checks that the
//! pages only use keys they have.

pub use klappe_services::Language;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};

    fn keys(json: &str) -> Vec<String> {
        let map: Map<String, Value> = serde_json::from_str(json).unwrap_or_default();
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Every `t("key"` literal in the pages exists in both catalogues.
    #[test]
    fn pages_use_only_known_keys() {
        let known = keys(Language::German.json());
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("qml");
        let mut checked = 0;

        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) != Some("qml") {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for part in source.split(".t(\"").skip(1) {
                let key = part.split('"').next().unwrap_or_default();
                // "tool." + name: only complete keys are checked.
                if key.ends_with('.') {
                    continue;
                }
                assert!(known.iter().any(|k| k == key), "{path:?}: {key}");
                checked += 1;
            }
        }
        assert!(checked > 20);
    }
}
