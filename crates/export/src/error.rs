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
}
