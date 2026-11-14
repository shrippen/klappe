//! Notes: asking for a note's text and committing it.

use super::q;
use super::qobject;
use crate::notice::error_text;
use crate::tools;
use crate::tools::NoteDraft;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Layer, Source};

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

    /// Starts asking for a note's text: empty for a new note, the note's own for an edit.
    /// False when the note to edit is gone.
    pub(super) fn ask_note(mut self: Pin<&mut Self>, draft: NoteDraft) -> bool {
        let language = self.rust().language;
        let lock = self.rust().session.as_ref().map(|s| s.lock(Layer::Notes));
        if let Some(Err(e)) = lock.map(tools::note_open) {
            self.as_mut().say(error_text(language, &e));
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
