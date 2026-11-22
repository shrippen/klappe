//! The characters: `figuren.toml` (DB-50).

use std::path::{Path, PathBuf};

use klappe_model::{
    is_figure_size_m, Character, Project, MAX_FIGURE_M, MAX_GROUP, MIN_FIGURE_M, MIN_GROUP,
};
use serde::{Deserialize, Serialize};

use crate::files;
use crate::{StoreError, CHARACTERS_FILE};

/// `figuren.toml`.
#[derive(Serialize, Deserialize)]
pub(crate) struct Characters {
    /// Before the tables: TOML keys come ahead of `[[figur]]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) not_figures: Vec<String>,
    #[serde(default)]
    pub(crate) figur: Vec<Character>,
}

/// `figuren.toml` (characters and not-figures); empty when the file is missing.
pub(crate) fn load_characters(dir: &Path) -> Result<Characters, StoreError> {
    let path = dir.join(CHARACTERS_FILE);
    let characters: Characters = match files::exists(&path)? {
        true => files::read_toml(&path)?,
        false => Characters {
            not_figures: Vec::new(),
            figur: Vec::new(),
        },
    };
    check_characters(&characters.figur).map_err(|message| StoreError::Parse { path, message })?;

    Ok(characters)
}

/// `figuren.toml` of `project`, encoded.
pub(crate) fn encode_characters(
    dir: &Path,
    project: &Project,
) -> Result<(PathBuf, Vec<u8>), StoreError> {
    let characters = Characters {
        not_figures: project.not_figures.clone(),
        figur: project.characters.clone(),
    };
    let path = dir.join(CHARACTERS_FILE);
    let bytes = files::encode_toml(&path, &characters)?;

    Ok((path, bytes))
}

/// The characters and the not-figures of `figuren.toml` as it is on disk, for an answer that
/// changes only what it touches (DB-50).
pub fn load_figures(dir: &Path) -> Result<(Vec<Character>, Vec<String>), StoreError> {
    let characters = load_characters(dir)?;

    Ok((characters.figur, characters.not_figures))
}

/// Writes `figuren.toml` of `project` in `dir` and nothing else: the answer about a name of
/// the book (DB-50) is kept at once, like the register.
pub fn save_characters(dir: &Path, project: &Project) -> Result<(), StoreError> {
    let (path, bytes) = encode_characters(dir, project)?;
    files::write_atomic(&path, &bytes)
}

/// A group has between `MIN_GROUP` and `MAX_GROUP` people (LH-23); a size is between
/// `MIN_FIGURE_M` and `MAX_FIGURE_M` (LH-25).
fn check_characters(characters: &[Character]) -> Result<(), String> {
    for (index, character) in characters.iter().enumerate() {
        // A name or alias belongs to one figure: a cue must not name two.
        let rest = &characters[index + 1..];
        let names = std::iter::once(&character.name).chain(&character.aliases);
        for name in names {
            if let Some(other) = rest.iter().find(|o| o.is_called(name)) {
                return Err(format!(
                    "{}: {name} is also a name of {}",
                    character.id, other.id
                ));
            }
        }
        if let Some(count) = character.group {
            if !(MIN_GROUP..=MAX_GROUP).contains(&count) {
                return Err(format!(
                    "{}: group of {count} is not usable ({MIN_GROUP} to {MAX_GROUP})",
                    character.id
                ));
            }
        }
        if character.aliases.iter().any(|a| a.trim().is_empty()) {
            return Err(format!("{}: an alias is empty", character.id));
        }
        if let Some(size_m) = character.size_m {
            if !is_figure_size_m(size_m) {
                return Err(format!(
                    "{}: size of {size_m} m is not usable ({MIN_FIGURE_M} to {MAX_FIGURE_M})",
                    character.id
                ));
            }
        }
    }

    Ok(())
}
