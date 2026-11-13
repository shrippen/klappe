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
    #[error("no project open")]
    NoProject,
}
