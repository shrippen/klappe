use serde::{Deserialize, Serialize};

/// The language terms of the printed screenplay are in: `(MORE)` and `(CONT'D)` or
/// `(WEITER)` and `(FORTS.)` (F-04, DN-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLanguage {
    De,
    En,
}

/// The paper the screenplay is paged for (F-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptPaper {
    #[default]
    A4,
    Letter,
}

/// How the screenplay is paged and printed; `[script]` in `projekt.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScriptSetup {
    /// `None` until chosen: the surface's language then decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<ScriptLanguage>,
    #[serde(default)]
    pub paper: ScriptPaper,
    /// `(CONT'D)` also when a character speaks again after action (DB-31).
    #[serde(default)]
    pub contd_after_action: bool,
}
