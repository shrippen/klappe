use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("palette: {0}")]
    Palette(String),
    #[error("the plan has no size")]
    EmptyPlan,
    #[error("the page leaves no room for the plan")]
    PageTooSmall,
}
