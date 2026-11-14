//! What the start page shows of a project: name, scenes, last change and a preview of the
//! first plan (D-07), the preview kept in a cache folder.
//!
//! ```text
//! project folder ──► newest file time ──► cache/previews/<hash>-<time>.png there ? reuse
//!                                         else Session::export_preview ──► cache
//! ```

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Language, Session};

const PREVIEW_DIR: &str = "previews";
const PREVIEW_EXT: &str = "png";

// FNV-1a, 64 bit: names the preview after the project's path.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    Ready,
    /// The folder is gone.
    Missing,
    /// The folder is there, the project does not load.
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCard {
    pub path: PathBuf,
    /// The project's title, else the folder's name.
    pub name: String,
    pub scenes: usize,
    /// Newest file in the folder, seconds since 1970; 0 if unknown.
    pub modified_s: u64,
    /// A PNG of the first scene's plan; none without a plan.
    pub preview: Option<PathBuf>,
    pub state: CardState,
}

/// The card of the project in `dir`; makes or renews its preview in `cache`.
pub fn project_card(dir: &Path, cache: &Path, language: Language) -> ProjectCard {
    let name = folder_name(dir);
    let mut card = ProjectCard {
        path: dir.to_owned(),
        name,
        scenes: 0,
        modified_s: 0,
        preview: None,
        state: CardState::Missing,
    };
    if !dir.is_dir() {
        return card;
    }

    let newest = klappe_store::project_mtime(dir);
    card.modified_s = newest.and_then(seconds).unwrap_or(0);
    let Ok(session) = Session::open(dir) else {
        card.state = CardState::Unreadable;
        return card;
    };

    card.state = CardState::Ready;
    card.scenes = session.project().scenes.len();
    if !session.project().title.is_empty() {
        card.name = session.project().title.clone();
    }
    if session.plan().is_some() {
        card.preview = preview_for(&session, cache, card.modified_s, language);
    }

    card
}

/// The cached preview of this state of the project, else a fresh one. The file is named after
/// the project's path and its last change, so an edited project never meets an old picture.
fn preview_for(
    session: &Session,
    cache: &Path,
    modified_s: u64,
    language: Language,
) -> Option<PathBuf> {
    let path = preview_path(cache, session.dir(), modified_s);
    if path.is_file() {
        return Some(path);
    }

    remove_old_previews(&path);
    // The export writes aside and renames, so a cut-off write never passes for a preview.
    session.export_preview(&path, language.print_texts()).ok()?;

    Some(path)
}

/// `<hash>-<modified_s>.png`
fn preview_path(cache: &Path, dir: &Path, modified_s: u64) -> PathBuf {
    let mut hash = FNV_OFFSET;
    for byte in dir.to_string_lossy().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
    }

    cache
        .join(PREVIEW_DIR)
        .join(format!("{hash:016x}-{modified_s}.{PREVIEW_EXT}"))
}

/// Removes the earlier previews of the same project (same hash before the dash).
fn remove_old_previews(keep: &Path) {
    let Some(folder) = keep.parent() else {
        return;
    };
    let Some(name) = keep.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return;
    };
    let Some((hash, _)) = name.split_once('-') else {
        return;
    };

    klappe_store::remove_prefixed(folder, &format!("{hash}-"));
}

fn seconds(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs())
}

fn folder_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.to_string_lossy().into_owned())
}
