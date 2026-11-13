use std::io;
use std::path::PathBuf;

/// Why a project folder could not be read or written. Every variant names the file.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("{}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{}: {message}", path.display())]
    Parse { path: PathBuf, message: String },

    #[error("{}: cannot encode: {message}", path.display())]
    Encode { path: PathBuf, message: String },

    #[error("{}: format version {found} is newer than the supported {supported}", path.display())]
    NewerFormat {
        path: PathBuf,
        found: u32,
        supported: u32,
    },

    #[error("{}: id {found:?} does not match the file name", path.display())]
    IdMismatch { path: PathBuf, found: String },

    #[error("id {id:?} is not usable as a file name")]
    BadId { id: String },

    #[error("id {id:?} is used twice")]
    DuplicateId { id: String },
}
