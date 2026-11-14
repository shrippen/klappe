//! All visible texts: the `tui.*` keys of the shared catalogue
//! (`crates/services/i18n`), as the app has them.

pub use klappe_services::Language;

/// Keys of the help lines, in the order they are shown.
const HELP_KEYS: [&str; 14] = [
    "tui.help.save",
    "tui.help.quit",
    "tui.help.undo",
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
    pub goto_title: String,
    pub goto_none: String,
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
            goto_title: t("tui.goto.title"),
            goto_none: t("tui.goto.none"),
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
                &t.goto_title,
                &t.goto_none,
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
            assert_eq!(t.help.len(), HELP_KEYS.len());
        }
    }
}
