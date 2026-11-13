use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Store(#[from] klappe_store::StoreError),
    #[error("no scene {0}")]
    NoScene(String),
    #[error("no shot {0}")]
    NoShot(String),
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
