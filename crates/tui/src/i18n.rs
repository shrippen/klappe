//! All visible texts, German and English.

/// Language of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    De,
    En,
}

impl Language {
    /// `de` or `en`.
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_ascii_lowercase().as_str() {
            "de" => Some(Self::De),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    /// From a locale such as `de_DE.UTF-8`; English when it is anything else.
    pub fn from_locale(locale: &str) -> Self {
        let code: String = locale.chars().take(2).collect();
        Self::from_code(&code).unwrap_or(Self::En)
    }

    /// From the `LANG` environment variable.
    pub fn from_env() -> Self {
        Self::from_locale(&std::env::var("LANG").unwrap_or_default())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Text {
    Scenes,
    Script,
    NoScenes,
    Hints,
    Saved,
    SaveFailed,
    ConfirmQuit,
    HelpTitle,
    Usage,
    UnknownLanguage,
    OpenFailed,
}

pub(crate) fn tr(lang: Language, text: Text) -> &'static str {
    use Language::{De, En};
    match (text, lang) {
        (Text::Scenes, De) => "Szenen",
        (Text::Scenes, En) => "Scenes",
        (Text::Script, De) => "Drehbuch",
        (Text::Script, En) => "Screenplay",
        (Text::NoScenes, De) => "(keine Szenen)",
        (Text::NoScenes, En) => "(no scenes)",
        (Text::Hints, De) => "F1 Hilfe  ^S Speichern  ^Q Beenden",
        (Text::Hints, En) => "F1 Help  ^S Save  ^Q Quit",
        (Text::Saved, De) => "Gespeichert",
        (Text::Saved, En) => "Saved",
        (Text::SaveFailed, De) => "Speichern fehlgeschlagen",
        (Text::SaveFailed, En) => "Saving failed",
        (Text::ConfirmQuit, De) => "Ungespeicherte Änderungen. Trotzdem beenden? (j/n)",
        (Text::ConfirmQuit, En) => "Unsaved changes. Quit anyway? (y/n)",
        (Text::HelpTitle, De) => "Hilfe (beliebige Taste schließt)",
        (Text::HelpTitle, En) => "Help (any key closes)",
        (Text::Usage, De) => "Aufruf: klappe-tui [--language de|en] <Ordner>.klappe",
        (Text::Usage, En) => "Usage: klappe-tui [--language de|en] <folder>.klappe",
        (Text::UnknownLanguage, De) => "Unbekannte Sprache (de oder en)",
        (Text::UnknownLanguage, En) => "Unknown language (de or en)",
        (Text::OpenFailed, De) => "Projekt lässt sich nicht öffnen",
        (Text::OpenFailed, En) => "Cannot open the project",
    }
}

pub(crate) fn help(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::De => &[
            "Ctrl+S           Speichern",
            "Ctrl+Q           Beenden (fragt bei Änderungen)",
            "Ctrl+Z / Ctrl+Y  Rückgängig / Wiederholen",
            "Tab              Fokus: Szenenliste / Editor",
            "Ctrl+G           Zur gewählten Szene springen",
            "Enter (Liste)    Zur gewählten Szene springen",
            "Pfeile           Cursor bewegen",
            "Home / Ende      Zeilenanfang / -ende",
            "Bild auf / ab    Seitenweise",
            "Ctrl+Links/Rechts  Wortweise",
            "Mausrad          Scrollen",
            "F1               Diese Hilfe",
        ],
        Language::En => &[
            "Ctrl+S           Save",
            "Ctrl+Q           Quit (asks when modified)",
            "Ctrl+Z / Ctrl+Y  Undo / redo",
            "Tab              Focus: scene list / editor",
            "Ctrl+G           Jump to the selected scene",
            "Enter (list)     Jump to the selected scene",
            "Arrows           Move the cursor",
            "Home / End       Start / end of row",
            "PageUp / PageDn  Page by page",
            "Ctrl+Left/Right  By word",
            "Mouse wheel      Scroll",
            "F1               This help",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_picks_german_only_for_de() {
        assert_eq!(Language::from_locale("de_DE.UTF-8"), Language::De);
        assert_eq!(Language::from_locale("en_US.UTF-8"), Language::En);
        assert_eq!(Language::from_locale("C"), Language::En);
        assert_eq!(Language::from_locale(""), Language::En);
    }

    #[test]
    fn code_is_de_or_en() {
        assert_eq!(Language::from_code("DE"), Some(Language::De));
        assert_eq!(Language::from_code("fr"), None);
    }

    #[test]
    fn both_languages_list_the_same_keys() {
        assert_eq!(help(Language::De).len(), help(Language::En).len());
    }
}
