//! The open questions about the book as the surfaces show them (audit 17, finding 1): the
//! scenes first, then the place names of the headings (DB-51), each with title, question and
//! answer lines worded in the surface's language. A surface shows the lines and hands back the
//! position; it words nothing itself.
//!
//! ```text
//! Szene im Drehbuch?            Ort der Szenenköpfe
//! Eintrag 3 INT. KELLER — …     HOF, in 2 Szenenköpfen — …
//!   4  INT. KELLER – NACHT        Neuer Spielort HOF
//!      Nicht im Drehbuch          Schreibweise von SAAL
//! ```

use super::{Answer, PlaceAnswer, Session};
use crate::book::Choice;
use crate::catalog::Language;
use crate::error::ServiceError;
use crate::places::PlaceChoice;

/// What a question is about: an entry of the register (its id) or a place name of the headings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionKey {
    Scene(String),
    Place(String),
}

/// One answer line: the scene number (empty where there is none) and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerLine {
    pub number: String,
    pub text: String,
}

/// An open question, worded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookQuestion {
    pub key: QuestionKey,
    pub title: String,
    pub ask: String,
    pub answers: Vec<AnswerLine>,
}

impl Session {
    /// The open questions in the order they are asked, worded in `language`.
    pub fn book_questions(&self, language: Language) -> Vec<BookQuestion> {
        let scenes = self.pending_scenes().iter().map(|p| BookQuestion {
            key: QuestionKey::Scene(p.id.clone()),
            title: language.text("book.title"),
            ask: language
                .text("book.ask")
                .replace("{number}", &p.number)
                .replace("{heading}", &p.heading),
            answers: self.scene_lines(&p.id, language),
        });
        let places = self.pending_places().iter().map(|p| BookQuestion {
            key: QuestionKey::Place(p.name.clone()),
            title: language.text("book.placeTitle"),
            ask: language
                .text("book.placeAsk")
                .replace("{name}", &p.name)
                .replace("{count}", &p.headings.to_string()),
            answers: self.place_lines(&p.name, language),
        });

        scenes.chain(places).collect()
    }

    /// Answers question `key` with line `index` of its `answers`.
    pub fn answer_question(&mut self, key: &QuestionKey, index: usize) -> Result<(), ServiceError> {
        match key {
            QuestionKey::Scene(id) => self.answer_book(id, index),
            QuestionKey::Place(name) => self.answer_place(name, index),
        }
    }

    /// The scenes of the book entry `id` may be, then "not in the book".
    fn scene_lines(&self, id: &str, language: Language) -> Vec<AnswerLine> {
        let line = |a: Answer| match a.choice {
            Choice::NotInBook => AnswerLine {
                number: String::new(),
                text: language.text("book.none"),
            },
            Choice::Scene(_) => AnswerLine {
                number: a.number,
                text: a.heading,
            },
        };

        self.book_answers(id).into_iter().map(line).collect()
    }

    /// "New scripted location HOF", "Spelling of SAAL" or "SAAL is now called HOF".
    fn place_lines(&self, name: &str, language: Language) -> Vec<AnswerLine> {
        let line = |a: PlaceAnswer| {
            let key = match a.choice {
                PlaceChoice::Create => "book.placeNew",
                PlaceChoice::Variant(_) => "book.placeVariant",
                PlaceChoice::Rename(_) => "book.placeRename",
            };
            let text = language
                .text(key)
                .replace("{name}", name)
                .replace("{old}", &a.name)
                .replace("{new}", name);

            AnswerLine {
                number: String::new(),
                text,
            }
        };

        self.place_answers(name).into_iter().map(line).collect()
    }
}
