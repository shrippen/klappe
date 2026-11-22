use std::collections::HashMap;
use std::path::Path;

use super::export::write_file;
use super::outside::PROJECT_FILE;
use super::Session;
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::script_pages::{lay_out, ScriptPages, ScriptPrint, Source};
use klappe_export::SheetPage;
use klappe_model::ScriptSetup;

/// Whether `paginate_if_changed` laid the screenplay out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Laid {
    /// It had not changed: the last layout holds.
    Kept,
    /// It had changed and is laid out anew.
    Anew,
}

impl Session {
    /// Lays the screenplay out again and keeps the result (DN-02). Nothing else does: typing
    /// never lays out. A surface calls this (or `paginate_if_changed`) when 5 s have passed
    /// without a key, and `export_script_pdf` and the printing call it themselves. `fallback` is the surface's
    /// language, used for the words (MORE) and (CONT'D) while the project has none.
    pub fn paginate(&mut self, fallback: Language) {
        let text = self.script_text().to_owned();

        self.paginate_text(&text, fallback);
        self.lengths = self.scene_lengths();
    }

    /// The scenes' lengths in the layout just made, by the line of each scene's heading.
    fn scene_lengths(&self) -> HashMap<String, String> {
        let Some(pages) = self.script_pages() else {
            return HashMap::new();
        };
        let spans: HashMap<usize, &str> = pages
            .scenes
            .iter()
            .map(|s| (s.line, s.length.as_str()))
            .collect();

        self.project
            .scenes
            .iter()
            .zip(self.heading_lines())
            .filter_map(|(scene, line)| Some((scene.id.clone(), spans.get(&line?)?.to_string())))
            .collect()
    }

    /// The length of scene `id` in the last `paginate`; empty before it or out of the book.
    pub(crate) fn scene_length(&self, id: &str) -> String {
        self.lengths.get(id).cloned().unwrap_or_default()
    }

    /// `paginate`, but only if text, paper or words changed since the last layout: the window
    /// calls it after 5 s without a key (DN-02) and need not show the same lengths again.
    pub fn paginate_if_changed(&mut self, fallback: Language) -> Laid {
        let source = Source::of(self.script_text(), &self.project.script, fallback);
        if self.layout.as_ref().is_some_and(|l| l.is_for(&source)) {
            return Laid::Kept;
        }

        self.paginate(fallback);
        Laid::Anew
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

    /// Sets "(CONT'D) after action" (DB-31) and writes only that into `projekt.toml` at once,
    /// leaving everything else as it is on disk (the TUI, which saves the book without the
    /// plans).
    pub fn keep_contd(&mut self, on: bool) -> Result<(), ServiceError> {
        let before = self.before(PROJECT_FILE)?;
        klappe_store::save_contd(&self.dir, on)?;
        self.project.script.contd_after_action = on;
        self.rebase(PROJECT_FILE, before)?;

        Ok(())
    }

    /// The screenplay's pages as `export_script_pdf` prints them, each text where it stands on
    /// the paper, laid out again from the text now: the print view of the window (D-15).
    pub fn print_pages(&mut self, print: &ScriptPrint) -> Vec<SheetPage> {
        self.paginate(print.language);
        let Some(layout) = &self.layout else {
            return Vec::new();
        };
        let sheet = layout.sheet(&self.project.script, print, &self.project.title);

        klappe_export::script_sheet_pages(&sheet)
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
