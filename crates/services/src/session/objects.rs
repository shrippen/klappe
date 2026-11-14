use std::path::Path;

use klappe_model::CameraPresets;

use super::Session;
use crate::error::ServiceError;
use crate::library;
use crate::project_edit::ProjectEdit;
use crate::views::LibraryEntry;

impl Session {
    /// What the object tool offers: the built-in library, then the project's own objects.
    pub fn library(&self) -> Vec<LibraryEntry> {
        library::entries(&self.project.library)
    }

    /// What the camera card offers: sensor formats and lens sets (LH-32).
    pub fn camera_presets(&self) -> &'static CameraPresets {
        klappe_store::builtin_camera_presets()
    }

    /// Takes the entries of the library file `path` into the project; undoable. Returns how
    /// many it added: entries the project has already are not added again. A file with an
    /// unusable entry is refused as a whole and changes nothing.
    pub fn import_library(&mut self, path: &Path) -> Result<usize, ServiceError> {
        let items = klappe_store::load_library(path)?;
        let before = self.project.library.len();

        self.edit_project(ProjectEdit::ImportLibrary { items })?;

        Ok(self.project.library.len() - before)
    }

    /// Writes the project's own objects to the library file `path`, to pass on. Returns how
    /// many it wrote; none to write is an error.
    pub fn export_library(&self, path: &Path) -> Result<usize, ServiceError> {
        if self.project.library.is_empty() {
            return Err(ServiceError::EmptyLibrary);
        }

        klappe_store::save_library(path, &self.project.library)?;

        Ok(self.project.library.len())
    }
}
