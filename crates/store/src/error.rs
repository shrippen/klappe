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

    #[error("{}: format version {found} is older than the supported {supported}; older projects are not migrated", path.display())]
    OlderFormat {
        path: PathBuf,
        found: u32,
        supported: u32,
    },

    /// A file holds the markers Git leaves for a merge it could not do.
    #[error("{}:{line}: unresolved Git merge conflict", path.display())]
    Conflict { path: PathBuf, line: usize },

    #[error("{}: id {found:?} does not match the file name", path.display())]
    IdMismatch { path: PathBuf, found: String },

    #[error("id {id:?} is not usable as a file name")]
    BadId { id: String },

    /// Not a PNG or JPEG, or one whose header cannot be read.
    #[error("{}: {message}", path.display())]
    BadImage { path: PathBuf, message: String },

    /// An image file of a type the project does not take (not `.png`, `.jpg`, `.jpeg`).
    #[error("{}: only .png, .jpg and .jpeg images are supported", path.display())]
    ImageType { path: PathBuf },

    /// The system trash refused the folder.
    #[error("{}: cannot move to the trash: {message}", path.display())]
    Trash { path: PathBuf, message: String },

    #[error("id {id:?} is used twice")]
    DuplicateId { id: String },
}
