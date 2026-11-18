//! All visible texts: the `tui.*` keys of the shared catalogue
//! (`crates/services/i18n`), as the app has them.

pub use klappe_services::Language;

/// Keys of the help lines, in the order they are shown.
const HELP_KEYS: [&str; 28] = [
    "tui.help.save",
    "tui.help.quit",
    "tui.help.undo",
    "tui.help.element",
    "tui.help.heading",
    "tui.help.transition",
    "tui.help.tab",
    "tui.help.suggest",
    "tui.help.suggestMove",
    "tui.help.find",
    "tui.help.findNext",
    "tui.help.title",
    "tui.help.book",
    "tui.help.pdf",
    "tui.help.focus",
    "tui.help.goto",
    "tui.help.list",
    "tui.help.jump",
    "tui.help.fold",
    "tui.help.move",
    "tui.help.filter",
    "tui.help.enter",
    "tui.help.arrows",
    "tui.help.home",
    "tui.help.page",
    "tui.help.word",
    "tui.help.wheel",
    "tui.help.help",
];

/// The title page form's fields (a `TITLE_KEYS` spelling) with the key of their label.
const FIELD_KEYS: [(&str, &str); 6] = [
    ("Title", "tui.field.title"),
    ("Credit", "tui.field.credit"),
    ("Author", "tui.field.author"),
    ("Source", "tui.field.source"),
    ("Draft date", "tui.field.draftDate"),
    ("Contact", "tui.field.contact"),
];

/// The texts of one language, looked up once (a lookup parses the catalogue).
#[derive(Debug)]
pub(crate) struct Texts {
    lang: Language,
    pub scenes: String,
    pub no_scenes: String,
    /// Label of the outline's filter.
    pub filter: String,
    pub hints: String,
    pub saved: String,
    pub save_failed: String,
    pub confirm_quit: String,
    pub help_title: String,
    pub modified: String,
    pub mode_text: String,
    pub mode_list: String,
    pub mode_goto: String,
    pub mode_help: String,
    pub mode_find: String,
    pub mode_title: String,
    pub mode_backup: String,
    pub mode_book: String,
    pub book_title: String,
    pub book_none: String,
    pub figure_title: String,
    pub book_hint: String,
    pub book_save_first: String,
    pub find_title: String,
    pub find_find: String,
    pub find_replace: String,
    pub find_speaker: String,
    pub find_case_on: String,
    pub find_case_off: String,
    pub find_none: String,
    pub find_hint: String,
    pub title_box: String,
    pub title_hint: String,
    /// The labels of the title form by field name (a `TITLE_KEYS` spelling).
    pub fields: Vec<(&'static str, String)>,
    pub backup_title: String,
    pub backup_found: String,
    pub backup_unreadable: String,
    pub backup_hint: String,
    pub backup_hint_unreadable: String,
    pub goto_title: String,
    pub goto_none: String,
    /// Marks the page breaks and lengths of an older text.
    pub stale: String,
    pub mode_pdf: String,
    pub pdf_title: String,
    pub pdf_hint: String,
    pub pdf_numbers: String,
    pub pdf_header: String,
    pub pdf_contd: String,
    pub pdf_on: String,
    pub pdf_off: String,
    pub pdf_save_first: String,
    pub help: Vec<String>,
}

impl Texts {
    pub(crate) fn new(lang: Language) -> Self {
        let t = |key: &str| lang.text(key);

        Self {
            lang,
            scenes: t("tui.scenes"),
            no_scenes: t("tui.noScenes"),
            filter: t("tui.filter"),
            hints: t("tui.hints"),
            saved: t("tui.saved"),
            save_failed: t("tui.saveFailed"),
            confirm_quit: t("tui.confirmQuit"),
            help_title: t("tui.helpTitle"),
            modified: t("tui.modified"),
            mode_text: t("tui.mode.text"),
            mode_list: t("tui.mode.list"),
            mode_goto: t("tui.mode.goto"),
            mode_help: t("tui.mode.help"),
            mode_find: t("tui.mode.find"),
            mode_title: t("tui.mode.title"),
            mode_backup: t("tui.mode.backup"),
            mode_book: t("tui.mode.book"),
            book_title: t("book.title"),
            book_none: t("book.none"),
            figure_title: t("book.figureTitle"),
            book_hint: t("tui.book.hint"),
            book_save_first: t("tui.book.saveFirst"),
            find_title: t("tui.find.title"),
            find_find: t("tui.find.find"),
            find_replace: t("tui.find.replace"),
            find_speaker: t("tui.find.speaker"),
            find_case_on: t("tui.find.caseOn"),
            find_case_off: t("tui.find.caseOff"),
            find_none: t("tui.find.none"),
            find_hint: t("tui.find.hint"),
            title_box: t("tui.titleBox"),
            title_hint: t("tui.titleHint"),
            fields: FIELD_KEYS
                .iter()
                .map(|(name, k)| (*name, lang.text(k)))
                .collect(),
            backup_title: t("tui.backup.title"),
            backup_found: t("tui.backup.found"),
            backup_unreadable: t("tui.backup.unreadable"),
            backup_hint: t("tui.backup.hint"),
            backup_hint_unreadable: t("tui.backup.hintUnreadable"),
            goto_title: t("tui.goto.title"),
            goto_none: t("tui.goto.none"),
            stale: t("tui.stale"),
            mode_pdf: t("tui.mode.pdf"),
            pdf_title: t("tui.pdf.title"),
            pdf_hint: t("tui.pdf.hint"),
            pdf_numbers: t("tui.pdf.numbers"),
            pdf_header: t("tui.pdf.header"),
            pdf_contd: t("tui.pdf.contd"),
            pdf_on: t("tui.pdf.on"),
            pdf_off: t("tui.pdf.off"),
            pdf_save_first: t("tui.pdf.saveFirst"),
            help: HELP_KEYS.iter().map(|k| lang.text(k)).collect(),
        }
    }

    /// The label of the title form's field `name`; the name itself if there is none.
    pub(crate) fn field_label<'a>(&'a self, name: &'a str) -> &'a str {
        self.fields
            .iter()
            .find(|(field, _)| *field == name)
            .map_or(name, |(_, label)| label.as_str())
    }

    /// "Scene 2/5".
    pub(crate) fn scene_of(&self, current: usize, total: usize) -> String {
        self.lang
            .text("tui.sceneOf")
            .replace("{current}", &current.to_string())
            .replace("{total}", &total.to_string())
    }

    /// "2/5" of the matches of a search.
    pub(crate) fn match_of(&self, current: usize, total: usize) -> String {
        self.lang
            .text("tui.find.count")
            .replace("{current}", &current.to_string())
            .replace("{total}", &total.to_string())
    }

    /// "3 replaced".
    pub(crate) fn replaced(&self, count: usize) -> String {
        self.lang
            .text("tui.replaced")
            .replace("{count}", &count.to_string())
    }

    /// "Entry 1 INT. A - TAG - which scene of the book is this?".
    pub(crate) fn book_ask(&self, number: &str, heading: &str) -> String {
        self.lang
            .text("book.ask")
            .replace("{number}", number)
            .replace("{heading}", heading)
    }

    /// "ANNA, 3 cue(s) in the book - which figure is this?".
    pub(crate) fn figure_ask(&self, name: &str, cues: usize) -> String {
        self.lang
            .text("book.figureAsk")
            .replace("{name}", name)
            .replace("{count}", &cues.to_string())
    }

    /// A line of the figure question: "New figure ANNA", "Is Elisabeth" or "Not a figure".
    pub(crate) fn figure_line(&self, answer: &klappe_services::FigureAnswer, cue: &str) -> String {
        let (key, name) = match answer.choice {
            klappe_services::FigureChoice::Create => ("book.figureNew", cue),
            klappe_services::FigureChoice::Assign(_) => ("book.figureIs", answer.name.as_str()),
            klappe_services::FigureChoice::NotFigure => ("book.figureNone", ""),
        };

        self.lang.text(key).replace("{name}", name)
    }

    /// "Page 3".
    pub(crate) fn page(&self, number: usize) -> String {
        self.lang
            .text("tui.page")
            .replace("{number}", &number.to_string())
    }

    /// "2 min": the estimated running time, half a minute up (2.5 → 3); a book shorter than
    /// half a minute shows 1, not 0.
    pub(crate) fn minutes(&self, minutes: f64) -> String {
        let shown = match minutes > 0.0 {
            true => minutes.round().max(1.0),
            false => 0.0,
        };

        self.lang
            .text("tui.minutes")
            .replace("{count}", &format!("{shown:.0}"))
    }

    /// "PDF written: drehbuch.pdf".
    pub(crate) fn pdf_done(&self, file: &str) -> String {
        self.lang.text("tui.pdf.done").replace("{file}", file)
    }

    /// "Shots: 3".
    pub(crate) fn shots(&self, count: usize) -> String {
        self.lang
            .text("tui.shots")
            .replace("{count}", &count.to_string())
    }

    /// "Characters: 2".
    pub(crate) fn figures(&self, count: usize) -> String {
        self.lang
            .text("tui.figures")
            .replace("{count}", &count.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::TITLE_KEYS;

    #[test]
    fn every_key_is_in_both_catalogues() {
        for lang in [Language::German, Language::English] {
            let t = Texts::new(lang);
            let all = [
                &t.scenes,
                &t.no_scenes,
                &t.filter,
                &t.hints,
                &t.saved,
                &t.save_failed,
                &t.confirm_quit,
                &t.help_title,
                &t.modified,
                &t.mode_text,
                &t.mode_list,
                &t.mode_goto,
                &t.mode_help,
                &t.mode_find,
                &t.mode_title,
                &t.mode_backup,
                &t.mode_book,
                &t.book_title,
                &t.book_none,
                &t.book_hint,
                &t.book_save_first,
                &t.book_ask("1", "INT. A - TAG"),
                &t.find_title,
                &t.find_find,
                &t.find_replace,
                &t.find_speaker,
                &t.find_case_on,
                &t.find_case_off,
                &t.find_none,
                &t.find_hint,
                &t.title_box,
                &t.title_hint,
                &t.backup_title,
                &t.backup_found,
                &t.backup_unreadable,
                &t.backup_hint,
                &t.backup_hint_unreadable,
                &t.match_of(1, 2),
                &t.replaced(1),
                &t.goto_title,
                &t.goto_none,
                &t.stale,
                &t.mode_pdf,
                &t.pdf_title,
                &t.pdf_hint,
                &t.pdf_numbers,
                &t.pdf_header,
                &t.pdf_contd,
                &t.pdf_on,
                &t.pdf_off,
                &t.pdf_save_first,
                &t.pdf_done("a.pdf"),
                &t.page(3),
                &t.scene_of(1, 2),
                &t.shots(1),
                &t.figures(1),
            ];

            assert!(all.iter().all(|s| !s.starts_with("tui.")), "{all:?}");
            assert!(
                t.help.iter().all(|s| !s.starts_with("tui.")),
                "{:?}",
                t.help
            );
            assert!(
                t.fields.iter().all(|(_, s)| !s.starts_with("tui.")),
                "{:?}",
                t.fields
            );
            assert_eq!(t.fields.len(), FIELD_KEYS.len());
            for key in TITLE_KEYS {
                assert!(t.fields.iter().any(|(name, _)| *name == key), "{key}");
            }
            assert_eq!(t.help.len(), HELP_KEYS.len());
        }
    }

    #[test]
    fn minutes_round_half_up_and_never_show_zero() {
        let t = Texts::new(Language::English);

        assert_eq!(t.minutes(2.5), "3 min");
        assert_eq!(t.minutes(3.5), "4 min");
        assert_eq!(t.minutes(0.375), "1 min");
        assert_eq!(t.minutes(0.0), "0 min");
    }
}
