//! `klappe-tui`: a terminal editor for the screenplay (Fountain) of a Klappe project
//! (planung/KONZEPT.md, E-09).
//!
//! ```text
//! main ─► terminal (crossterm events, raw mode) ─► app (keys → state) ─► editor (text, cursor, undo)
//!                          │                         │                     buffer, layout (wrap)
//!                          └────► ui (ratatui) ◄─────┘                     fountain (line kinds)
//!                                                    └─► klappe-services (Session)
//! ```

mod app;
mod buffer;
mod editor;
mod fountain;
mod i18n;
mod layout;
mod terminal;
mod ui;

pub use app::{App, Flow};
pub use i18n::Language;
pub use terminal::run;
pub use ui::draw;

/// The usage line.
pub fn usage(lang: Language) -> &'static str {
    i18n::tr(lang, i18n::Text::Usage)
}

pub fn unknown_language(lang: Language) -> &'static str {
    i18n::tr(lang, i18n::Text::UnknownLanguage)
}

pub fn open_failed(lang: Language) -> &'static str {
    i18n::tr(lang, i18n::Text::OpenFailed)
}

#[cfg(test)]
mod tests;
