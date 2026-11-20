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
mod history;
mod i18n;
mod layout;
mod links;
mod terminal;
mod theme;
mod ui;
mod ui_list;
mod ui_overlay;

pub use app::{App, Flow};
pub use i18n::Language;
pub use terminal::run;
pub use ui::draw;

/// The usage line.
pub fn usage(lang: Language) -> String {
    lang.text("tui.usage")
}

pub fn unknown_language(lang: Language) -> String {
    lang.text("tui.unknownLanguage")
}

/// Why the project did not open, in the terminal's language.
pub fn open_error(lang: Language, error: &klappe_services::ServiceError) -> String {
    // The service's text says what failed and why; a lead-in would say it twice.
    klappe_services::error_text(lang, error)
}

#[cfg(test)]
mod tests;
