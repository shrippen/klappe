//! What an answer does to a file, kept to take it back: undo and redo of an answer change
//! only the entry the answer touched, in the file as it is on disk (another program may have
//! changed the rest). The answers about the figures (`figure_file`) use it; the answers
//! about the play locations are meant to.

/// Which way a step goes on the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Way {
    /// The answer, done (again).
    Apply,
    /// The answer, taken back.
    Revert,
}

/// One answer's change to the content of a file.
pub(crate) trait DiskStep {
    type File;

    /// Changes `file` by this step, in `way`; an entry already (not) there stays as it is.
    fn run(&self, file: &mut Self::File, way: Way);
}
