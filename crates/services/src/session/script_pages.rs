use std::path::Path;

use super::export::write_file;
use super::Session;
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::script_pages::{lay_out, ScriptPages, ScriptPrint};
use klappe_model::ScriptSetup;

impl Session {
    /// Lays the screenplay out again and keeps the result (DN-02). Nothing else does: typing
    /// never lays out. A surface calls this when 5 s have passed without a key, and
    /// `export_script_pdf` and the printing call it themselves. `fallback` is the surface's
    /// language, used for the words (MORE) and (CONT'D) while the project has none.
    pub fn paginate(&mut self, fallback: Language) {
        let text = self.script_text().to_owned();

        self.paginate_text(&text, fallback);
    }

    /// Like `paginate` for `text` that is not (yet) the session's: the terminal editor holds
    /// its text apart until it is saved. The session's screenplay stays as it is.
    pub fn paginate_text(&mut self, text: &str, fallback: Language) {
        self.layout = Some(lay_out(text, &self.project.script, fallback));
    }

    /// The result of the last `paginate`: page breaks, scene lengths, total length. `None`
    /// before the first.
    pub fn script_pages(&self) -> Option<&ScriptPages> {
        self.layout.as_ref().map(|layout| &layout.view)
    }

    /// Remembers how the screenplay is paged and printed in the project, written with the next
    /// save. Not an undo step, as `set_paper`. The last layout is out of date until the next
    /// `paginate`.
    pub fn set_script_setup(&mut self, setup: ScriptSetup) {
        if self.project.script == setup {
            return;
        }

        self.project.script = setup;
        self.project_changed = true;
        self.touch();
    }

    /// Sets how the screenplay is paged and printed and writes only that into `projekt.toml`
    /// at once, leaving every other file as it is on disk (the TUI, which saves the book
    /// without the plans).
    pub fn keep_script_setup(&mut self, setup: ScriptSetup) -> Result<(), ServiceError> {
        klappe_store::save_script_setup(&self.dir, setup)?;
        self.project.script = setup;

        Ok(())
    }

    /// Writes the screenplay as PDF (DB-70) after laying it out again, so the pages are the
    /// ones in the text now.
    pub fn export_script_pdf(
        &mut self,
        path: &Path,
        print: &ScriptPrint,
    ) -> Result<(), ServiceError> {
        if self.script_text().trim().is_empty() {
            return Err(ServiceError::NoScript);
        }

        self.paginate(print.language);
        let Some(layout) = &self.layout else {
            return Err(ServiceError::NoScript);
        };
        let sheet = layout.sheet(&self.project.script, print, &self.project.title);

        write_file(path, klappe_export::script_pdf(&sheet))
    }
}
