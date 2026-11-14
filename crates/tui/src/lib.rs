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
mod links;
mod terminal;
mod theme;
mod ui;

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

pub fn open_failed(lang: Language) -> String {
    lang.text("tui.openFailed")
}

#[cfg(test)]
mod tests;
