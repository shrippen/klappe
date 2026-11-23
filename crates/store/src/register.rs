//! The register of what is shot: `szenen.toml` (id, kind, number, heading, state, prints, in
//! the order of the project) and, per entry, `szenen/<id>.json` (location, plans).

use std::collections::HashMap;
use std::path::Path;

use klappe_model::{BookState, FloorPlan, Kind, Print, Scene, SceneKey, SunTime};
use serde::{Deserialize, Serialize};

use crate::files::{self, Dir, Kind as FileKind};
use crate::{LoadNote, StoreError, SCENES_DIR, SCENES_FILE};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Register {
    #[serde(default)]
    szene: Vec<Entry>,
}

/// One `[[szene]]` of the register.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    #[serde(default)]
    art: Kind,
    number: String,
    heading: String,
    #[serde(default, skip_serializing_if = "is_in_book")]
    state: BookState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    prints: Vec<Print>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    refused: Vec<SceneKey>,
}

fn is_in_book(state: &BookState) -> bool {
    *state == BookState::InBook
}

/// `szenen/<id>.json`: what Stellprobe keeps of an entry.
#[derive(Serialize, Deserialize)]
struct Stage {
    id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sun: Option<SunTime>,
    #[serde(default)]
    plans: Vec<FloorPlan>,
}

/// The register in project order, each entry joined with its Stellprobe file. An entry
/// without a file has no location and no plans. A file without an entry (a crash between the
/// writes, a Git merge) is not lost: it comes last, not in the book, with no number or
/// heading, and is reported (DATENMODELL §2.5).
pub(crate) fn load(dir: &Path) -> Result<(Vec<Scene>, Vec<LoadNote>), StoreError> {
    let path = dir.join(SCENES_FILE);
    let register = read(&path)?;
    crate::check_ids(register.szene.iter().map(|e| e.id.as_str()))?;

    let mut stages: HashMap<String, Stage> =
        crate::load_all::<Stage>(&dir.join(SCENES_DIR), FileKind::Json, |s| &s.id)?
            .into_iter()
            .map(|s| (s.id.clone(), s))
            .collect();

    let mut scenes: Vec<Scene> = register
        .szene
        .into_iter()
        .map(|entry| {
            let stage = stages.remove(&entry.id);
            join(entry, stage)
        })
        .collect();

    let mut orphans: Vec<Stage> = stages.into_values().collect();
    orphans.sort_by(|a, b| a.id.cmp(&b.id));
    let mut notes = Vec::new();
    for stage in orphans {
        notes.push(LoadNote::OrphanScene {
            id: stage.id.clone(),
        });
        scenes.push(join(
            Entry {
                id: stage.id.clone(),
                art: Kind::Scene,
                number: String::new(),
                heading: String::new(),
                state: BookState::NotInBook,
                prints: Vec::new(),
                refused: Vec::new(),
            },
            Some(stage),
        ));
    }

    crate::sun::check_suns(&dir.join(SCENES_DIR), &scenes)?;

    Ok((scenes, notes))
}

fn read(path: &Path) -> Result<Register, StoreError> {
    let Some(register): Option<Register> = files::read_toml_merged(path)? else {
        return Ok(Register { szene: Vec::new() });
    };
    let broken = register
        .szene
        .iter()
        .find(|e| e.heading.contains(['\n', '\r']));
    if let Some(entry) = broken {
        return Err(StoreError::Parse {
            path: path.to_path_buf(),
            message: format!("{}: the heading has a line break", entry.id),
        });
    }

    Ok(register)
}

fn join(entry: Entry, stage: Option<Stage>) -> Scene {
    let (location, sun, plans) =
        stage.map_or_else(Default::default, |s| (s.location, s.sun, s.plans));

    Scene {
        id: entry.id,
        kind: entry.art,
        number: entry.number,
        heading: entry.heading,
        state: entry.state,
        prints: entry.prints,
        refused: entry.refused,
        location,
        sun,
        sheet: Default::default(),
        plans,
    }
}

/// The content of `szenen.toml` with its path: the register alone, without the Stellprobe files.
pub(crate) fn encode_register(
    dir: &Path,
    scenes: &[Scene],
) -> Result<(std::path::PathBuf, Vec<u8>), StoreError> {
    let register = Register {
        szene: scenes
            .iter()
            .map(|s| Entry {
                id: s.id.clone(),
                art: s.kind,
                number: s.number.clone(),
                heading: s.heading.clone(),
                state: s.state,
                prints: s.prints.clone(),
                refused: s.refused.clone(),
            })
            .collect(),
    };
    let path = dir.join(SCENES_FILE);
    let bytes = files::encode_toml(&path, &register)?;

    Ok((path, bytes))
}

/// The content of every `szenen/<id>.json` and, last, of `szenen.toml`, with their paths. In
/// this order a crash leaves a file the register lacks (loads, reported), never an entry whose
/// plans were lost.
pub(crate) fn encode(
    dir: &Path,
    scenes: &[Scene],
) -> Result<Vec<(std::path::PathBuf, Vec<u8>)>, StoreError> {
    let mut writes = Vec::new();

    let stages = Dir::new(dir.join(SCENES_DIR), FileKind::Json);
    for scene in scenes {
        let stage = Stage {
            id: scene.id.clone(),
            location: scene.location.clone(),
            sun: scene.sun,
            plans: scene.plans.clone(),
        };
        writes.push(stages.encode(&scene.id, &stage)?);
    }
    writes.push(encode_register(dir, scenes)?);

    Ok(writes)
}
