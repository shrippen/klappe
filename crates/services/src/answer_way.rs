//! Which way an answer's delta goes on its file: shared by `figure_file.rs` and `place_file.rs`.

/// Which way a delta goes on the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Way {
    /// The answer, done (again).
    Apply,
    /// The answer, taken back.
    Revert,
}
