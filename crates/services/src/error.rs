use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Store(#[from] klappe_store::StoreError),
    #[error("no scene {0}")]
    NoScene(String),
    #[error("there is no backup")]
    NoBackup,
    #[error("no shot {0}")]
    NoShot(String),
    /// What an edit names is no longer in the plan.
    #[error("the element is gone")]
    Gone,
    /// The scene's location is missing, so its set cannot change.
    #[error("no location {0}")]
    NoLocation(String),
    #[error("every camera letter is taken")]
    NoFreeCamera,
    /// By label and shot number, as the user knows them.
    #[error("camera {camera} is used by shot {shot}")]
    CameraInUse { camera: String, shot: String },
    #[error("bad value for {field}: {value}")]
    BadValue { field: &'static str, value: String },
    #[error(transparent)]
    Export(#[from] klappe_export::ExportError),
    #[error("cannot write {path}: {source}")]
    Write {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}
