//! The templates: the project's own in `vorlagen.toml` and the built-in ones (LH-07).

use std::path::Path;
use std::sync::OnceLock;

use klappe_model::{
    is_aspect, is_camera_height_m, is_camera_lens_mm, is_hold_s, is_sensor_width_mm, is_tilt_deg,
    CameraMark, Mark, Point, Template, MAX_SET_M,
};
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

/// Reads the project's `vorlagen.toml`; no file means no own templates. A file with a
/// template that cannot be used is refused as a whole.
pub(crate) fn load_templates(path: &Path) -> Result<Vec<Template>, StoreError> {
    if !files::exists(path)? {
        return Ok(Vec::new());
    }

    let file: TemplateFile = files::read_toml(path)?;
    check_own(&file.vorlage).map_err(|message| StoreError::Parse {
        path: path.to_path_buf(),
        message,
    })?;

    Ok(file.vorlage)
}

/// The file content of `templates`; refuses templates that could not be read back.
pub(crate) fn encode(path: &Path, templates: &[Template]) -> Result<Vec<u8>, StoreError> {
    check_own(templates).map_err(|message| StoreError::Encode {
        path: path.to_path_buf(),
        message,
    })?;
    let file = TemplateFile {
        vorlage: templates.to_vec(),
    };

    files::encode_toml(path, &file)
}

/// A place within the set's bounds, as the editor takes them.
fn is_point(point: &Point) -> bool {
    point.x_m.abs() <= MAX_SET_M && point.y_m.abs() <= MAX_SET_M
}

/// Whether every number of a figure's mark is one the editor takes: `nan`, `inf` and `1e307`
/// are valid TOML but cannot be saved again or drawn.
fn is_mark(mark: &Mark) -> bool {
    mark.at_s.is_finite()
        && mark.facing_deg.is_finite()
        && is_hold_s(mark.hold_s)
        && is_point(&mark.at)
        && mark.via.as_ref().is_none_or(is_point)
}

/// Like `is_mark`; the lens height from the floor up to a crane's, the tilt within straight down
/// and up, a lens set at the mark within the lens bounds.
fn is_camera_mark(mark: &CameraMark) -> bool {
    mark.at_s.is_finite()
        && mark.facing_deg.is_finite()
        && is_tilt_deg(mark.tilt_deg)
        && is_hold_s(mark.hold_s)
        && is_camera_height_m(mark.height_m)
        && is_point(&mark.at)
        && mark.via.as_ref().is_none_or(is_point)
        && mark.lens_mm.is_none_or(is_camera_lens_mm)
}

/// The project's own templates: usable ones, each with a name to list it by.
fn check_own(templates: &[Template]) -> Result<(), String> {
    check_templates(templates)?;

    match templates.iter().find(|t| t.label.trim().is_empty()) {
        Some(template) => Err(format!("template {:?}: label is empty", template.id)),
        None => Ok(()),
    }
}

/// Whether `template` could be kept as an own one: what the project file takes.
pub fn check_template(template: &Template) -> Result<(), String> {
    check_own(std::slice::from_ref(template))
}

/// Every template has an id of its own and something to place; figures and cameras are usable.
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

        check_parts(template).map_err(|part| format!("template {id:?}: {part}"))?;
    }

    Ok(())
}

/// Figures have a slot of their own and marks, all with finite numbers; cameras have marks, a lens and a frame, and
/// follow a slot of the template.
fn check_parts(template: &Template) -> Result<(), String> {
    for (at, figure) in template.figures.iter().enumerate() {
        let twice = template.figures[..at].iter().any(|f| f.slot == figure.slot);
        if figure.slot.trim().is_empty()
            || twice
            || figure.marks.is_empty()
            || !figure.marks.iter().all(is_mark)
        {
            return Err(format!("figure {} is not usable", at + 1));
        }
    }

    for camera in &template.cameras {
        let slot_known = |slot: &String| template.figures.iter().any(|f| f.slot == *slot);
        let usable = !camera.marks.is_empty()
            && camera.marks.iter().all(is_camera_mark)
            && is_camera_lens_mm(camera.lens_mm)
            && is_aspect(camera.aspect)
            && is_sensor_width_mm(camera.sensor.width_mm())
            && camera.follows.as_ref().is_none_or(slot_known);
        if !usable {
            return Err(format!("camera {:?} is not usable", camera.label));
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
