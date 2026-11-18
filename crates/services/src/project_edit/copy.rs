//! Copying a character (LH-08): the copy of a figure on the plan is a character of its own.

use klappe_model::Character;

use super::{slug, unique_id, CHARACTER_ID_FALLBACK, MAX_SHORT_CHARS};
use crate::error::ServiceError;

/// Numbers tried for a short code: `A2` … `A999`; more would not fit the label.
const SHORT_NUMBERS: std::ops::Range<usize> = 2..1000;

/// Adds a character that looks like `id` (colour, group, size): the name with the next free
/// number (`Anna` → `Anna 2`), a short code and an id of its own. Returns the new id.
pub(crate) fn copy_character(
    characters: &mut Vec<Character>,
    id: &str,
) -> Result<String, ServiceError> {
    let origin = characters
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or_else(|| ServiceError::NoCharacter(id.to_owned()))?;

    let name = numbered(characters, stem_of(&origin.name));
    let new_id = unique_id(
        &slug(&name, CHARACTER_ID_FALLBACK),
        characters.iter().map(|c| c.id.as_str()),
    );
    let short = free_short(characters, &origin.short);

    characters.push(Character {
        id: new_id.clone(),
        name,
        short,
        // The aliases name the origin; a cue must not name two characters.
        aliases: Vec::new(),
        ..origin
    });

    Ok(new_id)
}

/// `name` without a trailing ` 2` (space and number): `Anna 2` → `Anna`; `R2D2` stays.
fn stem_of(name: &str) -> &str {
    match name.rsplit_once(' ') {
        Some((head, tail))
            if !head.trim().is_empty()
                && !tail.is_empty()
                && tail.chars().all(|c| c.is_ascii_digit()) =>
        {
            head.trim_end()
        }
        _ => name,
    }
}

/// `stem 2`, `stem 3` … the first name no character answers to, as name or alias; compared as
/// the cues are. A copy always carries a number.
fn numbered(characters: &[Character], stem: &str) -> String {
    let taken = |name: &str| characters.iter().any(|c| c.is_called(name));

    (2..)
        .map(|n| format!("{stem} {n}"))
        .find(|name| !taken(name))
        .unwrap_or_default()
}

/// `text` without the digits a copy added: `A2` → `A`. A text that is
/// nothing but digits stays as it is.
fn without_number(text: &str) -> &str {
    let stem = text
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .trim_end();

    match stem.is_empty() {
        true => text,
        false => stem,
    }
}

/// `A2`, `A3` … the first short code of the stem of `short` that no character has.
pub(crate) fn free_short(characters: &[Character], short: &str) -> String {
    let stem = without_number(short);

    SHORT_NUMBERS
        .map(|n| {
            let number = n.to_string();
            let room = MAX_SHORT_CHARS.saturating_sub(number.len());

            format!("{}{number}", stem.chars().take(room).collect::<String>())
        })
        .find(|code| !characters.iter().any(|c| c.short == *code))
        .unwrap_or_else(|| stem.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::FigureColor;

    fn character(id: &str, name: &str, short: &str) -> Character {
        Character {
            id: id.into(),
            name: name.into(),
            short: short.into(),
            color: FigureColor::ALL[0],
            group: Some(4),
            size_m: Some(1.5),
            aliases: Vec::new(),
        }
    }

    #[test]
    fn a_copy_looks_like_its_original() {
        let mut all = vec![character("gaeste", "Gäste", "G")];

        let id = copy_character(&mut all, "gaeste").unwrap();

        let copy = all.last().unwrap();
        assert_eq!(copy.id, id);
        assert_eq!((copy.name.as_str(), copy.short.as_str()), ("Gäste 2", "G2"));
        assert_eq!(
            (copy.color, copy.group, copy.size_m),
            (all[0].color, Some(4), Some(1.5))
        );
    }

    #[test]
    fn a_copy_skips_a_name_another_figure_answers_to() {
        let mut bert = character("bert", "Bert", "B");
        bert.aliases = vec!["ANNA 2".into()];
        let mut all = vec![character("anna", "Anna", "A"), bert];

        copy_character(&mut all, "anna").unwrap();

        assert_eq!(all.last().unwrap().name, "Anna 3");
    }

    #[test]
    fn a_short_code_keeps_to_three_letters() {
        let mut all = vec![
            character("a", "Anna", "ANN"),
            character("b", "Anna 2", "AN2"),
        ];

        copy_character(&mut all, "a").unwrap();

        assert_eq!(all.last().unwrap().short, "AN3");
        assert_eq!(all.last().unwrap().name, "Anna 3");
    }

    #[test]
    fn a_copy_always_carries_a_number_not_in_use() {
        let mut all = vec![character("a", "Mann 1", "M1"), character("b", "R2D2", "R")];

        copy_character(&mut all, "a").unwrap();
        copy_character(&mut all, "b").unwrap();

        assert_eq!(all[2].name, "Mann 2");
        assert_eq!(all[3].name, "R2D2 2");
    }

    #[test]
    fn a_missing_character_has_no_copy() {
        let result = copy_character(&mut Vec::new(), "x");

        assert!(matches!(result, Err(ServiceError::NoCharacter(_))));
    }
}
