//! The screenplay's setup as the project form shows it.

use klappe_model::{ScriptLanguage, ScriptSetup};
use serde::Serialize;

/// The `[script]` setup of the project as names, for the project form.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptRow {
    /// `de` or `en`; empty until the project has chosen: the surface's language decides then.
    pub language: &'static str,
    /// `local` or `international`.
    pub words: &'static str,
    pub paper: &'static str,
    pub contd_after_action: bool,
    /// `automatic` or `locked`.
    pub scene_numbering: &'static str,
    pub first_scene: u32,
}

impl From<&ScriptSetup> for ScriptRow {
    fn from(setup: &ScriptSetup) -> Self {
        Self {
            language: setup.language.map_or("", ScriptLanguage::name),
            words: setup.words.name(),
            paper: setup.paper.name(),
            contd_after_action: setup.contd_after_action,
            scene_numbering: setup.scene_numbering.name(),
            first_scene: setup.first_scene,
        }
    }
}
