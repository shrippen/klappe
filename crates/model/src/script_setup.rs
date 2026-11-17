use serde::{Deserialize, Deserializer, Serialize};

/// The language of the project (DN-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    De,
    En,
}

impl ScriptLanguage {
    pub fn name(self) -> &'static str {
        match self {
            ScriptLanguage::De => "de",
            ScriptLanguage::En => "en",
        }
    }

    /// The language `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        [ScriptLanguage::De, ScriptLanguage::En]
            .into_iter()
            .find(|l| l.name() == name)
    }
}

/// The paper the screenplay is paged for (F-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum ScriptPaper {
    #[default]
    A4,
    Letter,
}

impl ScriptPaper {
    pub fn name(self) -> &'static str {
        match self {
            ScriptPaper::A4 => "a4",
            ScriptPaper::Letter => "letter",
        }
    }
}

impl From<String> for ScriptPaper {
    /// Any other text is A4, as for the plan's paper.
    fn from(name: String) -> Self {
        match name.as_str() {
            "letter" => ScriptPaper::Letter,
            _ => ScriptPaper::A4,
        }
    }
}

impl From<ScriptPaper> for String {
    fn from(paper: ScriptPaper) -> Self {
        paper.name().to_owned()
    }
}

/// Which words mark a dialogue broken over a page (F-04): `(MORE)` and `(CONT'D)` or
/// `(WEITER)` and `(FORTS.)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum ScriptWords {
    /// The words of the project's language.
    #[default]
    Local,
    /// `(MORE)` and `(CONT'D)`, also in a German project.
    International,
}

impl ScriptWords {
    pub fn name(self) -> &'static str {
        match self {
            ScriptWords::Local => "local",
            ScriptWords::International => "international",
        }
    }
}

impl From<String> for ScriptWords {
    /// Any other text is the words of the language.
    fn from(name: String) -> Self {
        match name.as_str() {
            "international" => ScriptWords::International,
            _ => ScriptWords::Local,
        }
    }
}

impl From<ScriptWords> for String {
    fn from(words: ScriptWords) -> Self {
        words.name().to_owned()
    }
}

/// A language that is not `de` or `en` counts as not chosen.
fn lenient_language<'de, D: Deserializer<'de>>(d: D) -> Result<Option<ScriptLanguage>, D::Error> {
    let name = Option::<String>::deserialize(d)?;

    Ok(name.and_then(|n| ScriptLanguage::from_name(&n)))
}

fn write_language<S: serde::Serializer>(
    language: &Option<ScriptLanguage>,
    s: S,
) -> Result<S::Ok, S::Error> {
    language.map(ScriptLanguage::name).serialize(s)
}

/// Who sets the scene numbers (DB-33): Klappe follows the book, or the numbers stay as they
/// are and only new scenes get one (12A).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum SceneNumbering {
    /// Sequential from the first number; changes with the book.
    #[default]
    Automatic,
    /// No number changes; a new scene is numbered between its neighbours, a struck one stays
    /// as "OMITTED".
    Locked,
}

impl SceneNumbering {
    pub fn name(self) -> &'static str {
        match self {
            SceneNumbering::Automatic => "automatic",
            SceneNumbering::Locked => "locked",
        }
    }
}

impl From<String> for SceneNumbering {
    /// Any other text is automatic.
    fn from(name: String) -> Self {
        match name.as_str() {
            "locked" => SceneNumbering::Locked,
            _ => SceneNumbering::Automatic,
        }
    }
}

impl From<SceneNumbering> for String {
    fn from(numbers: SceneNumbering) -> Self {
        numbers.name().to_owned()
    }
}

/// The number the first scene gets while numbers are automatic.
pub const DEFAULT_FIRST_SCENE: u32 = 1;

fn default_first_scene() -> u32 {
    DEFAULT_FIRST_SCENE
}

fn is_default_numbering(numbering: &SceneNumbering) -> bool {
    *numbering == SceneNumbering::default()
}

fn is_default_first(first: &u32) -> bool {
    *first == DEFAULT_FIRST_SCENE
}

/// How the screenplay is paged and printed; `[script]` in `projekt.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSetup {
    /// The project's language; `None` until chosen: the surface's language then decides.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "lenient_language",
        serialize_with = "write_language"
    )]
    pub language: Option<ScriptLanguage>,
    /// The words at a page break; the language's own unless chosen otherwise.
    #[serde(default, skip_serializing_if = "is_local")]
    pub words: ScriptWords,
    #[serde(default)]
    pub paper: ScriptPaper,
    /// `(CONT'D)` also when a character speaks again after action (DB-31).
    #[serde(default)]
    pub contd_after_action: bool,
    /// Who sets the scene numbers (DB-33).
    #[serde(default, skip_serializing_if = "is_default_numbering")]
    pub scene_numbering: SceneNumbering,
    /// The number of the first scene while they are automatic.
    #[serde(
        default = "default_first_scene",
        skip_serializing_if = "is_default_first"
    )]
    pub first_scene: u32,
}

impl Default for ScriptSetup {
    fn default() -> Self {
        Self {
            language: None,
            words: ScriptWords::default(),
            paper: ScriptPaper::default(),
            contd_after_action: false,
            scene_numbering: SceneNumbering::default(),
            first_scene: DEFAULT_FIRST_SCENE,
        }
    }
}

fn is_local(words: &ScriptWords) -> bool {
    *words == ScriptWords::Local
}
