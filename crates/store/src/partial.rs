//! Writes of one file of a project folder and nothing else: what is on disk beside stays, as
//! another program (the TUI beside the window) may have saved it meanwhile.

use std::path::Path;

use klappe_model::{Scene, SceneNumbering};

use crate::{check_ids, files, read_header, register, StoreError, PROJECT_FILE, SCRIPT_FILE};

/// Sets "(CONT'D) after action" in `projekt.toml` of `dir` and nothing else: the rest of the
/// file stays as it is on disk, as another program may have saved it meanwhile (TUI beside
/// the window).
pub fn save_contd(dir: &Path, on: bool) -> Result<(), StoreError> {
    let mut header = read_header(dir)?;
    header.script.contd_after_action = on;

    let path = dir.join(PROJECT_FILE);
    let bytes = files::encode_toml(&path, &header)?;
    files::write_atomic(&path, &bytes)
}

/// Sets the scene numbering and the first scene number in `projekt.toml` of `dir` and
/// nothing else, as `save_contd` does.
pub fn save_scene_numbering(
    dir: &Path,
    numbering: SceneNumbering,
    first: u32,
) -> Result<(), StoreError> {
    let mut header = read_header(dir)?;
    header.script.scene_numbering = numbering;
    header.script.first_scene = first;

    let path = dir.join(PROJECT_FILE);
    let bytes = files::encode_toml(&path, &header)?;
    files::write_atomic(&path, &bytes)
}

/// Writes `drehbuch.fountain` in `dir` atomically and nothing else.
pub fn save_script(dir: &Path, script: &str) -> Result<(), StoreError> {
    files::write_atomic(&dir.join(SCRIPT_FILE), script.as_bytes())
}

/// Writes `szenen.toml` of `scenes` in `dir` and nothing else: the Stellprobe files stay, as
/// unsaved plans are for `save` to write (LH-05).
pub fn save_register(dir: &Path, scenes: &[Scene]) -> Result<(), StoreError> {
    check_ids(scenes.iter().map(|s| s.id.as_str()))?;

    let (path, bytes) = register::encode_register(dir, scenes)?;
    files::write_atomic(&path, &bytes)
}
