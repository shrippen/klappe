//! All visible texts: the `tui.*` keys of the shared catalogue
//! (`crates/services/i18n`), as the app has them.

pub use klappe_services::Language;

/// Keys of the help lines, in the order they are shown.
const HELP_KEYS: [&str; 21] = [
    "tui.help.save",
    "tui.help.quit",
    "tui.help.undo",
    "tui.help.element",
    "tui.help.tab",
    "tui.help.suggest",
    "tui.help.suggestMove",
    "tui.help.find",
    "tui.help.findNext",
    "tui.help.title",
    "tui.help.focus",
    "tui.help.goto",
    "tui.help.list",
    "tui.help.jump",
    "tui.help.enter",
    "tui.help.arrows",
    "tui.help.home",
    "tui.help.page",
    "tui.help.word",
    "tui.help.wheel",
    "tui.help.help",
];

/// Keys of the title page form's labels, in the order of `TITLE_KEYS`.
const FIELD_KEYS: [&str; 6] = [
    "tui.field.title",
    "tui.field.credit",
    "tui.field.author",
    "tui.field.source",
    "tui.field.draftDate",
    "tui.field.contact",
];

/// The texts of one language, looked up once (a lookup parses the catalogue).
#[derive(Debug)]
pub(crate) struct Texts {
    lang: Language,
    pub scenes: String,
    pub no_scenes: String,
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
    /// The labels of the title form, in the order of its fields.
    pub fields: Vec<String>,
    pub backup_title: String,
    pub backup_found: String,
    pub backup_unreadable: String,
    pub backup_hint: String,
    pub backup_hint_unreadable: String,
    pub goto_title: String,
    pub goto_none: String,
    /// Marks the page breaks and lengths of an older text.
    pub stale: String,
    pub help: Vec<String>,
}

impl Texts {
    pub(crate) fn new(lang: Language) -> Self {
        let t = |key: &str| lang.text(key);

        Self {
            lang,
            scenes: t("tui.scenes"),
            no_scenes: t("tui.noScenes"),
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
            fields: FIELD_KEYS.iter().map(|k| lang.text(k)).collect(),
            backup_title: t("tui.backup.title"),
            backup_found: t("tui.backup.found"),
            backup_unreadable: t("tui.backup.unreadable"),
            backup_hint: t("tui.backup.hint"),
            backup_hint_unreadable: t("tui.backup.hintUnreadable"),
            goto_title: t("tui.goto.title"),
            goto_none: t("tui.goto.none"),
            stale: t("tui.stale"),
            help: HELP_KEYS.iter().map(|k| lang.text(k)).collect(),
        }
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

    /// "Page 3".
    pub(crate) fn page(&self, number: usize) -> String {
        self.lang
            .text("tui.page")
            .replace("{number}", &number.to_string())
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

    #[test]
    fn every_key_is_in_both_catalogues() {
        for lang in [Language::German, Language::English] {
            let t = Texts::new(lang);
            let all = [
                &t.scenes,
                &t.no_scenes,
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
                t.fields.iter().all(|s| !s.starts_with("tui.")),
                "{:?}",
                t.fields
            );
            assert_eq!(t.fields.len(), FIELD_KEYS.len());
            assert_eq!(t.help.len(), HELP_KEYS.len());
        }
    }
}
