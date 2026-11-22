//! Joins the project's characters to the cues of the book (DB-50, DATENMODELL §4): which
//! names the book has that no character answers to, and what the user can make of each.
//! Nothing here becomes a character on its own: `ALLE` and `STIMME` are cues too.

use klappe_model::{same_character_name, Character, FigureColor, Project};
use klappe_script::Script;

use crate::project_edit::{self, ProjectEdit};
use crate::script_refs;

/// Longest short code the figure label takes.
const SHORT_MAX: usize = 3;

/// Forces a cue in the book (`@anna`).
const CUE_MARK: char = '@';
const SHARP_S: char = 'ß';

/// A cue of the book no character is called by, and which is no figure by the user's word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingFigure {
    /// As the book spells it the first time.
    pub name: String,
    /// How often the book cues it.
    pub cues: usize,
}

/// What the user makes of a cue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FigureChoice {
    /// A new character of this name, with a short code and a colour of its own.
    Create,
    /// The character with this id is meant: the cue becomes one of its aliases.
    Assign(String),
    /// Not a character (`ALLE`, `STIMME`): not asked again.
    NotFigure,
}

/// The cues of the book, each once, that nothing in the project answers to, in book order.
pub(crate) fn pending(project: &Project, script: &Script) -> Vec<PendingFigure> {
    let mut found: Vec<PendingFigure> = Vec::new();

    for cue in script_refs::cue_names(script) {
        if project.knows_cue(&cue) {
            continue;
        }
        let same = |p: &&mut PendingFigure| same_character_name(&p.name, &cue);
        match found.iter_mut().find(same) {
            Some(known) => known.cues += 1,
            None => found.push(PendingFigure { name: cue, cues: 1 }),
        }
    }

    found
}

/// The ids of the characters to offer for a cue: those the book does not cue any more first
/// (a rename outside the program leaves its old name without a cue), then the others.
pub(crate) fn candidates(project: &Project, script: &Script) -> Vec<String> {
    let cues = script_refs::cue_names(script);
    let cued = |c: &&Character| cues.iter().any(|cue| c.is_called(cue));
    let (with, without): (Vec<_>, Vec<_>) = project.characters.iter().partition(cued);

    without
        .iter()
        .chain(with.iter())
        .map(|c| c.id.clone())
        .collect()
}

/// Settles the cue `name` in `project` as `choice`; `false` when `choice` names no character.
pub(crate) fn decide(project: &mut Project, name: &str, choice: &FigureChoice) -> bool {
    let name = name.trim();

    match choice {
        FigureChoice::NotFigure => {
            project.not_figures.push(name.to_owned());

            true
        }
        FigureChoice::Assign(id) => {
            // A cue answers to one figure only.
            if project
                .characters
                .iter()
                .any(|c| c.id != *id && c.is_called(name))
            {
                return false;
            }
            let Some(character) = project.characters.iter_mut().find(|c| c.id == *id) else {
                return false;
            };
            character.aliases.push(name.to_owned());

            true
        }
        FigureChoice::Create => create(project, name),
    }
}

fn create(project: &mut Project, cue: &str) -> bool {
    let name = display_name(cue);
    let edit = ProjectEdit::AddCharacter {
        short: short_for(&project.characters, &name),
        color: next_color(&project.characters),
        name,
        group: None,
        size_m: None,
    };

    project_edit::apply(project, &edit).is_ok()
}

/// A cue in capitals or all in small letters reads as a name in capitals and small letters:
/// `MARIE` → `Marie`, `STRAUß` → `Strauß`. One that mixes them already (`@McBride`) stays.
fn display_name(cue: &str) -> String {
    let cue = cue.trim_start_matches(CUE_MARK);
    // `ß` has no uppercase form in everyday use, so it fits a name in capitals.
    let small = cue.chars().any(|c| c.is_lowercase() && c != SHARP_S);
    let capital = cue.chars().any(char::is_uppercase);
    if small && capital {
        return cue.to_owned();
    }

    cue.split(' ')
        .map(|word| {
            let mut letters = word.chars();
            let first = letters.next().into_iter().flat_map(char::to_uppercase);

            first
                .chain(letters.flat_map(char::to_lowercase))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The first letters of the name no character has as its short code: `M`, then `MA`, `MAR`;
/// at last the next free number after the first letter.
fn short_for(characters: &[Character], name: &str) -> String {
    let letters: Vec<char> = name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect();
    let free = |code: &String| !characters.iter().any(|c| c.short == *code);

    (1..=SHORT_MAX)
        .map(|n| letters.iter().take(n).collect::<String>())
        .find(free)
        .unwrap_or_else(|| {
            project_edit::free_short(characters, &name.chars().take(1).collect::<String>())
        })
}

/// The first colour no character has; when all are taken, round again.
fn next_color(characters: &[Character]) -> FigureColor {
    let free = FigureColor::ALL
        .iter()
        .find(|color| !characters.iter().any(|c| c.color == **color));

    match free {
        Some(color) => *color,
        None => FigureColor::ALL[characters.len() % FigureColor::ALL.len()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(id: &str, name: &str, aliases: &[&str]) -> Character {
        Character {
            id: id.into(),
            name: name.into(),
            short: id.to_uppercase(),
            color: FigureColor::Red,
            group: None,
            size_m: None,
            aliases: aliases.iter().map(|a| (*a).into()).collect(),
        }
    }

    #[test]
    fn a_name_reads_in_capital_and_small_letters() {
        assert_eq!(display_name("MARIE"), "Marie");
        assert_eq!(display_name("STRAUß"), "Strauß");
        assert_eq!(display_name("@anna"), "Anna");
        assert_eq!(display_name("McBride"), "McBride");
    }

    #[test]
    fn a_cue_another_figure_answers_to_is_no_alias() {
        let mut project = Project::new("Test");
        project.characters = vec![figure("a", "Anna", &["ANNI"]), figure("b", "Bert", &[])];

        assert!(!decide(
            &mut project,
            "anna",
            &FigureChoice::Assign("b".into())
        ));
        assert!(!decide(
            &mut project,
            "Anni",
            &FigureChoice::Assign("b".into())
        ));
        assert!(project.characters[1].aliases.is_empty());
        assert!(decide(
            &mut project,
            "Berti",
            &FigureChoice::Assign("b".into())
        ));
    }
}
