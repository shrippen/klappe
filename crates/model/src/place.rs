//! Scripted locations (Spielorte, DB-51): the place a scene heading names, with a stable id.

use serde::{Deserialize, Serialize};

/// A place of the story: `SÄNGERSAAL`. Not a shooting location (`Location`): several scripted
/// locations may be shot at one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    /// Random and stable: the name may change, the id does not.
    pub id: String,
    pub name: String,
    /// Other spellings of the place in a heading, e.g. `SAAL` for `SÄNGERSAAL`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    /// The shooting location its scenes are shot at (`Location` id); `None` is "no shooting
    /// location yet". A scene may differ (`Scene::location`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Whether two names are the same place name: the case of a heading is style, and so is the
/// white space (`ß` and `SS` agree, as the capitals of `ß` are `SS`).
pub fn same_place_name(a: &str, b: &str) -> bool {
    fold(a) == fold(b)
}

fn fold(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();

    capitals(&words.join(" "))
}

/// Upper case turns `ß` into `SS`, not the capital `ẞ`: both become `SS`.
pub(crate) fn capitals(name: &str) -> String {
    name.to_uppercase().replace('ẞ', "SS")
}

impl Place {
    /// Whether the place name `name` of a heading is this place: its name or a variant.
    pub fn is_called(&self, name: &str) -> bool {
        same_place_name(&self.name, name) || self.variants.iter().any(|v| same_place_name(v, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sharp_s_agree_with_ss() {
        assert!(same_place_name("STRAẞE", "STRASSE"));
        assert!(same_place_name("Straße", "STRAẞE"));
        assert!(!same_place_name("STRASE", "STRASSE"));
    }
}
