use thiserror::Error;

#[derive(Debug, Error)]
pub enum DemoError {
    #[error("{path}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("world file: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Store(#[from] klappe_store::StoreError),
    /// A text of the world that has no `{de, en}` side for the language.
    #[error("no {lang} text for {what}")]
    NoText { lang: &'static str, what: String },
    /// The world breaks a rule of the example (an id that is not there).
    #[error("{0}")]
    World(String),
}
