//! Opening and creating projects, the recent list, scene selection and the status line.

use super::q;
use super::qobject;
use super::view::{to_json, Shown};
use crate::fileurl;
use crate::manage;
use crate::tools::Preview;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use klappe_services::{book_report_text, error_text, load_notes_text};
use klappe_services::{ProjectEdit, ServiceError, Session};
use std::path::Path;

impl qobject::Controller {
    pub(super) fn load_recent(self: Pin<&mut Self>, stored: &QString) {
        let mut list = manage::parse_recent(&stored.to_string());
        // A project opened from the command line counts as opened last.
        if let Some(s) = self.rust().session.as_ref() {
            list = manage::push_recent(&list, &s.dir().to_string_lossy());
        }

        self.show_recent(&list);
    }

    pub(super) fn forget_recent(self: Pin<&mut Self>, path: &QString) {
        let list = manage::parse_recent(&self.rust().recent_store.to_string());
        let list = manage::forget_recent(&list, &path.to_string());

        self.show_recent(&list);
    }

    pub(super) fn rename_recent(mut self: Pin<&mut Self>, path: &QString, title: &QString) -> bool {
        let renamed =
            klappe_services::rename_project(Path::new(&path.to_string()), &title.to_string());
        if !self.as_mut().finish_folder(renamed) {
            return false;
        }

        // The card is read again: the tile shows the new title.
        let list = manage::parse_recent(&self.rust().recent_store.to_string());
        self.show_recent(&list);
        true
    }

    pub(super) fn archive_recent(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let archived = klappe_services::archive_project(Path::new(&path.to_string()));
        if !self.as_mut().finish_folder(archived.map(|_| ())) {
            return false;
        }

        self.forget_recent(path);
        true
    }

    pub(super) fn trash_recent(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let trashed = klappe_services::trash_project(Path::new(&path.to_string()));
        if !self.as_mut().finish_folder(trashed) {
            return false;
        }

        self.forget_recent(path);
        true
    }

    /// Says why a folder action failed; true when it did not. A success takes an earlier
    /// refusal's message away.
    fn finish_folder(mut self: Pin<&mut Self>, done: Result<(), ServiceError>) -> bool {
        let Err(e) = done else {
            self.clear_status();
            return true;
        };

        let language = self.rust().language;
        self.as_mut().say(error_text(language, &e));
        false
    }

    pub(super) fn create_project(self: Pin<&mut Self>, path: &QString, title: &QString) -> bool {
        let dir = fileurl::to_path(&path.to_string());
        let created = Session::create(&dir, &title.to_string(), self.rust().language);

        self.switch_to(created)
    }

    pub(super) fn open_project(self: Pin<&mut Self>, path: &QString) -> bool {
        let dir = fileurl::to_path(&path.to_string());
        let opened = Session::open(&dir);

        self.switch_to(opened)
    }

    pub(super) fn close_project(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let Some(old) = self.as_mut().session() else {
            return true;
        };
        let dir = old.dir().to_string_lossy().into_owned();
        if let Err(e) = old.autosave() {
            self.say(error_text(language, &e));
            return false;
        }

        {
            let this = self.as_mut().rust_mut().get_mut();
            this.session = None;
            this.drop_drafts();
        }

        // The start page stays, and no live page reads anything of the old project.
        self.as_mut().rust_mut().get_mut().shown = String::new();
        self.as_mut().rust_mut().get_mut().preview = Preview::Hidden;
        self.as_mut().set_loaded(false);
        self.as_mut().set_status(QString::default());
        self.as_mut().set_example_aside(QString::default());
        self.as_mut().rust_mut().get_mut().notice.say(String::new());
        self.as_mut().set_backup_state(QString::default());
        self.as_mut().set_backup_message(QString::default());
        self.as_mut().set_outside_files(QString::default());
        self.as_mut().show(Shown::default());
        self.remember(&dir);
        true
    }

    pub(super) fn open_example(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;

        // Without a data folder the copy would land in a temporary one and vanish unnoticed.
        let Some(data) = manage::data_dir() else {
            self.say(language.text("start.noDataDir"));
            return false;
        };

        let dir = manage::example_dir(language, &data);
        let aside = match klappe_services::ensure_example(language, &dir) {
            Ok(aside) => aside,
            Err(e) => {
                return self.switch_to(Err(e));
            }
        };
        if !self.as_mut().switch_to(Session::open(&dir)) {
            return false;
        }

        // After the switch, which clears the note of an earlier one. Not the status line: the
        // next edit would clear that.
        if let Some(aside) = aside {
            let path = aside.to_string_lossy();
            let note = language.text("start.exampleAside").replace("{path}", &path);
            self.as_mut().set_example_aside(q(&note));
        }
        true
    }

    pub(super) fn open_blank(self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;

        let Some(data) = manage::data_dir() else {
            self.say(language.text("start.noDataDirBlank"));
            return false;
        };

        let blank = manage::blank_project(language, &data);
        self.switch_to(blank)
    }

    pub(super) fn select_scene(mut self: Pin<&mut Self>, id: &QString) -> bool {
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if s.select_scene(&id.to_string()).is_err() {
            return false;
        }

        self.as_mut().rust_mut().get_mut().drop_drafts();
        self.as_mut().refresh();
        self.switched();
        true
    }

    pub(super) fn uses_of(&self, kind: &QString, id: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };

        q(&manage::uses_json(s, &kind.to_string(), &id.to_string()))
    }

    pub(super) fn scene_changes(
        &self,
        id: &QString,
        number: &QString,
        heading: &QString,
    ) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        let changes =
            s.scene_script_changes(&id.to_string(), &number.to_string(), &heading.to_string());

        q(&to_json(&changes))
    }

    pub(super) fn character_changes(
        &self,
        id: &QString,
        name: &QString,
        aliases: &QString,
    ) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        // Names the form cannot send are refused on apply; the preview keeps them.
        let aliases = super::script::aliases_of(aliases).ok();
        let changes =
            s.character_script_changes(&id.to_string(), &name.to_string(), aliases.as_deref());

        q(&to_json(&changes))
    }

    pub(super) fn not_figure_changes(&self, name: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };

        q(&to_json(&s.not_figure_changes(&name.to_string())))
    }

    pub(super) fn clear_status(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().notice.say(String::new());
        self.set_status(QString::default());
    }

    /// Applies `edit` (or says why its input was refused) to the project; an error goes to
    /// the status line.
    pub(super) fn commit_project(
        mut self: Pin<&mut Self>,
        edit: Result<ProjectEdit, ServiceError>,
    ) -> bool {
        let result = match (edit, self.as_mut().session()) {
            (Ok(edit), Some(s)) => s.edit_project(edit),
            (Err(e), _) => Err(e),
            (_, None) => return false,
        };

        self.report(Some(result))
    }

    /// Tells what became of a change that makes something (`None`: no project open); the id
    /// of what it made, empty when nothing was.
    pub(super) fn made(
        self: Pin<&mut Self>,
        result: Option<Result<String, ServiceError>>,
    ) -> QString {
        let id = match &result {
            Some(Ok(id)) => id.clone(),
            _ => String::new(),
        };
        self.report(result.map(|r| r.map(drop)));

        q(&id)
    }

    /// Tells what became of a change (`None`: no project open): an error goes to the status
    /// line, a change makes an old message stale; the view is redrawn either way.
    pub(super) fn report(
        mut self: Pin<&mut Self>,
        result: Option<Result<(), ServiceError>>,
    ) -> bool {
        let Some(result) = result else {
            return false;
        };
        let language = self.rust().language;

        let ok = result.is_ok();
        match result {
            Ok(()) => self.as_mut().changed(),
            Err(e) => self.as_mut().say(error_text(language, &e)),
        }
        self.refresh();
        ok
    }

    /// Makes `next` the open project: the one on screen goes to its backup first, so a
    /// switch loses nothing. A refusal leaves the open project as it is.
    fn switch_to(mut self: Pin<&mut Self>, next: Result<Session, ServiceError>) -> bool {
        let language = self.rust().language;
        let mut session = match next {
            Ok(session) => session,
            Err(e) => {
                self.say(error_text(language, &e));
                return false;
            }
        };

        if let Some(old) = self.as_mut().session() {
            if let Err(e) = old.autosave() {
                self.say(error_text(language, &e));
                return false;
            }
        }

        let dir = session.dir().to_string_lossy().into_owned();
        let report = session.book_report();
        let notes = [
            load_notes_text(language, session.load_notes()),
            book_report_text(language, &report),
        ];
        let notes = notes.join(" ").trim().to_owned();
        // The scene list shows lengths at once, also when the editor's text stays the same and
        // so does not lay out (DN-02: opening is no typing).
        session.paginate(language);
        {
            let this = self.as_mut().rust_mut().get_mut();
            this.session = Some(session);
            this.drop_drafts();
        }
        self.as_mut().switched();
        self.as_mut().set_example_aside(QString::default());
        self.as_mut().set_loaded(true);
        self.as_mut().changed();
        self.as_mut().refresh();
        self.as_mut().show_backup();
        // On the status line, which the first edit clears.
        if !notes.is_empty() {
            self.as_mut().say(notes);
        }
        self.remember(&dir);
        true
    }

    /// Puts the folder first in the list of recent projects.
    fn remember(self: Pin<&mut Self>, dir: &str) {
        let list = manage::parse_recent(&self.rust().recent_store.to_string());
        let list = manage::push_recent(&list, dir);

        self.show_recent(&list);
    }

    /// Shows `list` at once (names only), stores it, and reads the cards on a thread of
    /// their own: a preview takes a rendering and must not hold up the window. Each finished
    /// card reaches `recentJson` through the Qt thread, unless the list has changed since.
    fn show_recent(mut self: Pin<&mut Self>, list: &[String]) {
        let store = manage::recent_store(list);
        self.as_mut().set_recent_json(q(&manage::recent_view(list)));
        self.as_mut().set_recent_store(q(&store));

        let qt = self.qt_thread();
        let language = self.rust().language;
        let list = list.to_vec();
        std::thread::spawn(move || {
            let cache = manage::cache_dir().unwrap_or_else(std::env::temp_dir);
            let mut cards = Vec::new();
            for path in &list {
                cards.push(klappe_services::project_card(
                    Path::new(path),
                    &cache,
                    language,
                ));
                let json = manage::cards_view(&list, &cards);
                let store = store.clone();
                // Fails only when the window is gone; then nobody waits for the cards.
                let _ = qt.queue(move |mut this| {
                    if this.recent_store.to_string() == store {
                        this.as_mut().set_recent_json(q(&json));
                    }
                });
            }
        });
    }
}
