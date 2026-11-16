//! Notes and images, which share the notes layer: asking for a note's text and committing
//! it; placing an image the dialog picked.

use super::q;
use super::qobject;
use crate::image;
use crate::tools;
use crate::tools::NoteDraft;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{Layer, Point, Source};

impl qobject::Controller {
    pub(super) fn commit_note(mut self: Pin<&mut Self>, text: &QString) -> bool {
        let draft = self.rust().note.clone();
        let Some(edit) = tools::note_edit(&draft, &text.to_string()) else {
            return false;
        };

        // The draft stays open when the edit is refused, so the typed text is not lost.
        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        self.as_mut().rust_mut().get_mut().note = NoteDraft::Idle;
        self.refresh();
        true
    }

    pub(super) fn cancel_note(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().note = NoteDraft::Idle;
        self.refresh();
    }

    pub(super) fn edit_selected_note(self: Pin<&mut Self>) -> bool {
        let Some(Source::Note(id)) = self.rust().selected.only().cloned() else {
            return false;
        };

        self.ask_note(NoteDraft::Edit(id))
    }

    /// Whether the notes layer takes something new; when it is locked, the status line says so.
    fn notes_open(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let lock = self.rust().session.as_ref().map(|s| s.lock(Layer::Notes));
        let Some(Err(e)) = lock.map(tools::note_open) else {
            return true;
        };

        self.as_mut().say(error_text(language, &e));
        false
    }

    /// A click of the image tool: true when the file dialog may open, false (with the reason in
    /// the status line) while the notes layer is locked.
    pub(super) fn ask_image(self: Pin<&mut Self>) -> bool {
        self.notes_open()
    }

    /// Places the image file `file` (a path or a `file://` URL) centred at (`x`, `y`), snapped.
    pub(super) fn add_image(mut self: Pin<&mut Self>, file: &QString, x: f64, y: f64) -> bool {
        let Some(at) = self
            .rust()
            .session
            .as_ref()
            .map(|s| s.snap(Point::new(x, y)))
        else {
            return false;
        };

        self.as_mut()
            .commit_edit(tools::image_edit(&file.to_string(), at))
    }

    /// Sets the opacity of the selected image to `percent`, as typed.
    pub(super) fn set_image_opacity(self: Pin<&mut Self>, percent: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = image::opacity_edit(selected, &percent.to_string());

        self.commit_typed(typed)
    }

    /// Swaps the file of the selected image for `path` (a path or a `file://` URL).
    pub(super) fn replace_image(self: Pin<&mut Self>, path: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = image::replace_edit(selected, &path.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    /// Starts asking for a note's text: empty for a new note, the note's own for an edit.
    /// False when the note to edit is gone.
    pub(super) fn ask_note(mut self: Pin<&mut Self>, draft: NoteDraft) -> bool {
        if !self.as_mut().notes_open() {
            return false;
        }

        let text = match &draft {
            NoteDraft::Edit(id) => {
                let text = self.rust().session.as_ref().and_then(|s| s.note_text(id));
                let Some(text) = text else {
                    return false;
                };
                text
            }
            _ => String::new(),
        };

        self.as_mut().rust_mut().get_mut().note = draft;
        self.as_mut().set_note_text(q(&text));
        self.refresh();
        true
    }
}
