use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("palette: {0}")]
    Palette(String),
    #[error("the plan has no size")]
    EmptyPlan,
    #[error("the page leaves no room for the plan")]
    PageTooSmall,
    /// A background image the export cannot use; `file` as the display list names it.
    #[error("image {file}: {message}")]
    Image { file: String, message: String },
    /// A ZIP archive of 4 GiB or more, or of more than 65535 files: the format's plain limits.
    #[error("too big for a ZIP archive")]
    ZipTooBig,
    /// The program that writes the video is not installed (LH-84).
    #[error("{0} not found")]
    NoVideoProgram(String),
    /// The video program failed or could not be fed; its own message.
    #[error("video: {0}")]
    Video(String),
}
