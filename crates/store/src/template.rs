//! The templates: the project's own in `vorlagen.toml` and the built-in ones (LH-07).

use std::path::Path;
use std::sync::OnceLock;

use klappe_model::Template;
use serde::{Deserialize, Serialize};

use crate::files;
use crate::StoreError;

const BUILTIN: &str = include_str!("../template/standard.toml");

/// A template file: one `[[vorlage]]` per template. Other keys or no `vorlage` at all mean the
/// file is no template file.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateFile {
    vorlage: Vec<Template>,
}

/// The templates that ship with Klappe. Embedded and tested to parse; an unreadable file would
/// only leave the list empty.
pub fn builtin_templates() -> &'static [Template] {
    static ITEMS: OnceLock<Vec<Template>> = OnceLock::new();

    ITEMS.get_or_init(|| {
        toml::from_str::<TemplateFile>(BUILTIN)
            .map(|file| file.vorlage)
            .unwrap_or_default()
    })
}

/// Reads the project's `vorlagen.toml`. A file with a template that cannot be used is refused
/// as a whole.
pub(crate) fn load_templates(path: &Path) -> Result<Vec<Template>, StoreError> {
    let file: TemplateFile = files::read_toml(path)?;
    check_templates(&file.vorlage).map_err(|message| StoreError::Parse {
        path: path.to_path_buf(),
        message,
    })?;

    Ok(file.vorlage)
}

/// The file content of `templates`; refuses templates that could not be read back.
pub(crate) fn encode(path: &Path, templates: &[Template]) -> Result<Vec<u8>, StoreError> {
    check_templates(templates).map_err(|message| StoreError::Encode {
        path: path.to_path_buf(),
        message,
    })?;
    let file = TemplateFile {
        vorlage: templates.to_vec(),
    };

    files::encode_toml(path, &file)
}

fn is_length(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Every template has an id of its own and something to place; figures have a slot of their
/// own and marks; cameras have marks, a lens and a frame, and follow a slot of the template.
fn check_templates(templates: &[Template]) -> Result<(), String> {
    for (index, template) in templates.iter().enumerate() {
        let id = &template.id;
        if id.trim().is_empty() {
            return Err(format!("template {}: id is empty", index + 1));
        }
        if templates[..index].iter().any(|other| other.id == *id) {
            return Err(format!("template {id:?} is there twice"));
        }
        if template.figures.is_empty() && template.cameras.is_empty() {
            return Err(format!("template {id:?} holds nothing"));
        }

        for (at, figure) in template.figures.iter().enumerate() {
            let twice = template.figures[..at].iter().any(|f| f.slot == figure.slot);
            if figure.slot.trim().is_empty() || twice || figure.marks.is_empty() {
                return Err(format!("template {id:?}: figure {} is not usable", at + 1));
            }
        }

        for camera in &template.cameras {
            let slot_known = |slot: &String| template.figures.iter().any(|f| f.slot == *slot);
            let usable = !camera.marks.is_empty()
                && is_length(camera.lens_mm)
                && is_length(camera.aspect)
                && is_length(camera.sensor.width_mm())
                && camera.follows.as_ref().is_none_or(slot_known);
            if !usable {
                return Err(format!(
                    "template {id:?}: camera {:?} is not usable",
                    camera.label
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shipped file parses and holds the set-ups LH-07 names.
    #[test]
    fn the_builtin_templates_load() {
        let templates = builtin_templates();
        let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();

        assert_eq!(ids, ["shot-reverse", "triangle", "long-take"]);
        assert_eq!(check_templates(templates), Ok(()));

        let counts = |id: &str| {
            let t = templates
                .iter()
                .find(|t| t.id == id)
                .unwrap_or(&templates[0]);
            (t.figures.len(), t.cameras.len())
        };
        assert_eq!(counts("shot-reverse"), (2, 3));
        assert_eq!(counts("triangle"), (2, 3));
        assert_eq!(counts("long-take"), (1, 1));

        let take = &templates[2];
        assert_eq!(take.cameras[0].follows.as_deref(), Some("A"));
        assert!(take.cameras[0].marks.len() > 2);
    }

    #[test]
    fn unusable_templates_are_refused() {
        let mut good = builtin_templates()[0].clone();
        assert_eq!(check_templates(std::slice::from_ref(&good)), Ok(()));

        good.id = " ".into();
        assert!(check_templates(&[good.clone()]).is_err());

        let mut empty = builtin_templates()[0].clone();
        empty.figures.clear();
        empty.cameras.clear();
        assert!(check_templates(&[empty]).is_err());

        let mut twice = builtin_templates()[0].clone();
        twice.figures[1].slot = twice.figures[0].slot.clone();
        assert!(check_templates(&[twice]).is_err());

        let mut lost = builtin_templates()[0].clone();
        lost.cameras[0].follows = Some("Z".into());
        assert!(check_templates(&[lost]).is_err());

        let mut no_marks = builtin_templates()[0].clone();
        no_marks.cameras[0].marks.clear();
        assert!(check_templates(&[no_marks]).is_err());

        let first = builtin_templates()[0].clone();
        assert!(check_templates(&[first.clone(), first]).is_err());
    }
}
