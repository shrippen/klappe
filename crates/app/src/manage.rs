//! What the project view and the start page need besides the session: the lists as JSON for
//! QML, the recent projects, and the texts QML sends back parsed (LH-01 to LH-03).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use klappe_services::{
    read_number, CardState, Field, Language, ProjectCard, Range, ServiceError, Session, Unit,
};
use serde_json::{json, Value};

use crate::fileurl;

/// Projects the start page remembers.
pub const RECENT_MAX: usize = 8;

const STATE_LOADING: &str = "loading";
const STATE_MISSING: &str = "missing";
const CACHE_VAR: &str = "XDG_CACHE_HOME";
const HOME_VAR: &str = "HOME";
const CACHE_FALLBACK: &str = ".cache";
const DATA_FALLBACK: &str = ".local/share";
const CACHE_APP_DIR: &str = "klappe";
const DATA_VAR: &str = "XDG_DATA_HOME";
const DATA_APP_DIR: &str = "klappe";
const EXAMPLES_DIR: &str = "beispiel";
const HOME_SHORT: &str = "~";

/// `{ scenes, locations, characters, plans, plan }`: lists of plain objects for QML, `plans`
/// those of the scene on view and `plan` the id of the one shown. The colour is the model's
/// role name (`purple`), which Kante's `labelColor` takes as it is.
pub fn project_json(session: &Session) -> String {
    serde_json::to_string(&session.overview()).unwrap_or_default()
}

/// What the bridge and the form use for "a single person".
const NO_GROUP: u32 = 0;

/// The head count as the model has it: `None` for a single person.
pub(crate) fn group_of(count: u32) -> Option<u32> {
    match count {
        NO_GROUP => None,
        n => Some(n),
    }
}

/// Width and depth typed in the location form, `"8,5"` or `"8.5 m"`; the session sets the
/// range.
pub fn sizes(width: &str, depth: &str) -> Result<(f64, f64), ServiceError> {
    let width_m = read_number(Field::Width, width, Unit::Metre, Range::Any)?;
    let depth_m = read_number(Field::Depth, depth, Unit::Metre, Range::Any)?;

    Ok((width_m, depth_m))
}

/// The stored list (a JSON array of folders); anything else reads as empty.
pub fn parse_recent(stored: &str) -> Vec<String> {
    serde_json::from_str(stored).unwrap_or_default()
}

/// `path` first, once, and no more than `RECENT_MAX` in all.
pub fn push_recent(list: &[String], path: &str) -> Vec<String> {
    let mut out = vec![path.to_owned()];
    out.extend(list.iter().filter(|p| p.as_str() != path).cloned());
    out.truncate(RECENT_MAX);
    out
}

/// The list as it is stored.
pub fn recent_store(list: &[String]) -> String {
    serde_json::to_string(list).unwrap_or_default()
}

/// `[{ path, shortPath, name, state }]` for the start page before the cards are read: the
/// folder's name, `loading` while the folder is there and `missing` while it is not.
/// `shortPath` is the path with the home folder as `~` (the tile's tooltip).
pub fn recent_view(list: &[String]) -> String {
    let rows: Vec<_> = list
        .iter()
        .map(|p| {
            let state = if Path::new(p).is_dir() {
                STATE_LOADING
            } else {
                STATE_MISSING
            };
            json!({
                "path": p,
                "shortPath": short_path(p),
                "name": folder_name(p),
                "state": state,
            })
        })
        .collect();

    Value::Array(rows).to_string()
}

/// The same rows with what the cards found: `{ path, shortPath, name, state, scenes, modifiedS,
/// preview }`,
/// `state` being `ready`, `missing` or `unreadable`, `preview` a `file://` URL or empty. `cards`
/// are the first of `list` that are read; the others stay as `recent_view` shows them.
pub fn cards_view(list: &[String], cards: &[ProjectCard]) -> String {
    let Ok(Value::Array(mut rows)) = serde_json::from_str::<Value>(&recent_view(list)) else {
        return String::from("[]");
    };

    for (row, card) in rows.iter_mut().zip(cards) {
        *row = json!({
            "path": card.path.to_string_lossy(),
            "shortPath": short_path(&card.path.to_string_lossy()),
            "name": card.name,
            "state": card_state(card.state),
            "scenes": card.scenes,
            "modifiedS": card.modified_s,
            "preview": card.preview.as_deref().map(fileurl::from_path).unwrap_or_default(),
        });
    }

    Value::Array(rows).to_string()
}

/// `list` without `path`.
pub fn forget_recent(list: &[String], path: &str) -> Vec<String> {
    list.iter()
        .filter(|p| p.as_str() != path)
        .cloned()
        .collect()
}

/// `xdg` as the base if absolute, else `home` joined with `fallback` if absolute; `None` if
/// neither is. Empty and relative values count as unset, or the path would depend on the
/// working directory.
fn base_dir(xdg: Option<OsString>, home: Option<OsString>, fallback: &str) -> Option<PathBuf> {
    let absolute = |v: OsString| Some(PathBuf::from(v)).filter(|p| p.is_absolute());

    xdg.and_then(absolute)
        .or_else(|| home.and_then(absolute).map(|h| h.join(fallback)))
}

/// Where the app keeps what it can make again (previews): `$XDG_CACHE_HOME/klappe`, else
/// `~/.cache/klappe`.
pub fn cache_dir() -> Option<PathBuf> {
    let base = base_dir(
        std::env::var_os(CACHE_VAR),
        std::env::var_os(HOME_VAR),
        CACHE_FALLBACK,
    )?;

    Some(base.join(CACHE_APP_DIR))
}

/// Where the app keeps what the user may change (the example copies): `$XDG_DATA_HOME/klappe`,
/// else `~/.local/share/klappe`.
pub fn data_dir() -> Option<PathBuf> {
    let base = base_dir(
        std::env::var_os(DATA_VAR),
        std::env::var_os(HOME_VAR),
        DATA_FALLBACK,
    )?;

    Some(base.join(DATA_APP_DIR))
}

/// The user's copy of the example in `language`, below `data`: `beispiel/beispiel-de.klappe`.
pub fn example_dir(language: Language, data: &Path) -> PathBuf {
    let name = match language {
        Language::German => "beispiel-de.klappe",
        Language::English => "beispiel-en.klappe",
    };

    data.join(EXAMPLES_DIR).join(name)
}

/// `path` with `home` at its start as `~`; unchanged elsewhere.
fn tilde(path: &str, home: &Path) -> String {
    match Path::new(path).strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => HOME_SHORT.to_owned(),
        Ok(rest) => format!("{HOME_SHORT}/{}", rest.to_string_lossy()),
        Err(_) => path.to_owned(),
    }
}

fn short_path(path: &str) -> String {
    match std::env::var_os(HOME_VAR) {
        Some(home) if !home.is_empty() => tilde(path, Path::new(&home)),
        _ => path.to_owned(),
    }
}

fn card_state(state: CardState) -> &'static str {
    match state {
        CardState::Ready => "ready",
        CardState::Missing => STATE_MISSING,
        CardState::Unreadable => "unreadable",
    }
}

fn folder_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::{FigureColor, ProjectEdit};

    fn os(s: &str) -> Option<OsString> {
        Some(OsString::from(s))
    }

    #[test]
    fn an_empty_or_relative_home_is_no_base() {
        assert_eq!(base_dir(None, os(""), DATA_FALLBACK), None);
        assert_eq!(base_dir(None, os("home"), DATA_FALLBACK), None);
        assert_eq!(base_dir(None, None, DATA_FALLBACK), None);
    }

    #[test]
    fn an_empty_or_relative_xdg_falls_back_to_home() {
        let want = Some(PathBuf::from("/h/.cache"));

        assert_eq!(base_dir(os(""), os("/h"), CACHE_FALLBACK), want);
        assert_eq!(base_dir(os("rel"), os("/h"), CACHE_FALLBACK), want);
        assert_eq!(
            base_dir(os("/x"), os("/h"), CACHE_FALLBACK),
            Some(PathBuf::from("/x"))
        );
    }

    fn list(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    #[test]
    fn the_newest_project_comes_first_once() {
        let recent = push_recent(&list(&["/a", "/b", "/c"]), "/b");

        assert_eq!(recent, list(&["/b", "/a", "/c"]));
    }

    #[test]
    fn only_eight_are_kept() {
        let mut recent = Vec::new();
        for n in 0..RECENT_MAX + 3 {
            recent = push_recent(&recent, &format!("/p{n}"));
        }

        assert_eq!(recent.len(), RECENT_MAX);
        assert_eq!(recent[0], format!("/p{}", RECENT_MAX + 2));
        assert!(!recent.contains(&"/p0".to_owned()));
    }

    #[test]
    fn the_store_round_trips_and_bad_text_is_empty() {
        let recent = list(&["/a", "/b c"]);

        assert_eq!(parse_recent(&recent_store(&recent)), recent);
        assert!(parse_recent("").is_empty());
        assert!(parse_recent("{\"a\":1}").is_empty());
    }

    #[test]
    fn the_view_keeps_folders_that_are_gone_as_missing() {
        let here = env!("CARGO_MANIFEST_DIR");
        let view: Value = serde_json::from_str(&recent_view(&list(&[here, "/gibt/es/nicht"])))
            .unwrap_or_default();

        assert_eq!(view.as_array().map(Vec::len), Some(2));
        assert_eq!(view[0]["name"], "app");
        assert_eq!(view[0]["path"], here);
        assert_eq!(view[0]["state"], STATE_LOADING);
        assert_eq!(view[1]["name"], "nicht");
        assert_eq!(view[1]["state"], STATE_MISSING);
    }

    #[test]
    fn the_cards_replace_the_rows_they_cover() {
        let here = env!("CARGO_MANIFEST_DIR");
        let card = ProjectCard {
            path: PathBuf::from(here),
            name: "Film".into(),
            scenes: 3,
            modified_s: 7,
            preview: Some(PathBuf::from("/cache/a.png")),
            state: CardState::Ready,
        };
        let view: Value =
            serde_json::from_str(&cards_view(&list(&[here, "/gibt/es/nicht"]), &[card]))
                .unwrap_or_default();

        assert_eq!(view[0]["name"], "Film");
        assert_eq!(view[0]["state"], "ready");
        assert_eq!(view[0]["scenes"], 3);
        assert_eq!(view[0]["modifiedS"], 7);
        assert_eq!(view[0]["preview"], "file:///cache/a.png");
        assert_eq!(view[1]["state"], STATE_MISSING);
    }

    #[test]
    fn the_home_folder_is_a_tilde() {
        let home = Path::new("/home/anna");

        assert_eq!(tilde("/home/anna/film/a.klappe", home), "~/film/a.klappe");
        assert_eq!(tilde("/home/anna", home), "~");
        assert_eq!(tilde("/home/annabel/a", home), "/home/annabel/a");
        assert_eq!(tilde("/srv/film", home), "/srv/film");
    }

    #[test]
    fn the_rows_carry_the_short_path() {
        let view: Value =
            serde_json::from_str(&recent_view(&list(&["/gibt/es/nicht"]))).unwrap_or_default();

        assert_eq!(view[0]["shortPath"], "/gibt/es/nicht");
    }

    #[test]
    fn the_example_copy_lives_in_the_data_folder_by_language() {
        let data = Path::new("/data/klappe");

        assert_eq!(
            example_dir(Language::German, data),
            Path::new("/data/klappe/beispiel/beispiel-de.klappe")
        );
        assert_eq!(
            example_dir(Language::English, data),
            Path::new("/data/klappe/beispiel/beispiel-en.klappe")
        );
    }

    #[test]
    fn a_forgotten_project_leaves_the_list() {
        let rest = forget_recent(&list(&["/a", "/b", "/c"]), "/b");

        assert_eq!(rest, list(&["/a", "/c"]));
    }

    #[test]
    fn lengths_take_commas_and_units() {
        assert_eq!(sizes("8,5", " 12 m ").ok(), Some((8.5, 12.0)));
        assert!(matches!(
            sizes("8", "breit"),
            Err(ServiceError::BadValue {
                field: Field::Depth,
                ..
            })
        ));
    }

    /// Audit, Fund 2: no range here (the session sets it), but never NaN or infinity.
    #[test]
    fn lengths_have_no_range_here() {
        assert_eq!(sizes("0", "-3").ok(), Some((0.0, -3.0)));
        assert!(sizes("", "1").is_err());
        assert!(sizes("inf", "1").is_err());
        assert!(sizes("8 s", "1").is_err());
    }

    #[test]
    fn colours_go_by_their_role_name() {
        assert_eq!(FigureColor::from_name("purple"), Some(FigureColor::Purple));
        assert_eq!(FigureColor::from_name("magenta"), None);
        assert_eq!(FigureColor::Cyan.name(), "cyan");
    }

    #[test]
    fn the_plans_are_those_of_the_scene_on_view() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut session = Session::open(&dir).unwrap();
        session.copy_plan("Stand 1").unwrap();

        let value: Value = serde_json::from_str(&project_json(&session)).unwrap();

        assert_eq!(
            value["plans"],
            json!([
                { "id": "plan-a", "name": "Plan A", "shots": 5 },
                { "id": "stand-1", "name": "Stand 1", "shots": 5 },
            ])
        );
        assert_eq!(value["plan"], "stand-1");
    }

    #[test]
    fn the_project_lists_carry_what_the_view_shows() {
        // Next to the test binary, in the target folder.
        let dir = std::env::current_exe()
            .map(|e| e.with_file_name("manage-json"))
            .unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        let mut session = Session::create(&dir, "Film").unwrap();
        session
            .edit_project(ProjectEdit::AddLocation {
                name: "Halle".to_owned(),
                width_m: 8.0,
                depth_m: 5.0,
            })
            .unwrap();
        session
            .edit_project(ProjectEdit::AddScene {
                number: "1".to_owned(),
                heading: "INT. HALLE".to_owned(),
                location: "halle".to_owned(),
            })
            .unwrap();
        session
            .edit_project(ProjectEdit::AddCharacter {
                name: "Anna".to_owned(),
                short: "A".to_owned(),
                color: FigureColor::Red,
                group: Some(12),
            })
            .unwrap();

        let value: Value = serde_json::from_str(&project_json(&session)).unwrap();

        assert_eq!(
            value["plans"][0],
            json!({ "id": "plan-a", "name": "Plan A", "shots": 0 })
        );
        assert_eq!(value["plan"], "plan-a");
        assert_eq!(value["scenes"][0]["locationName"], "Halle");
        assert_eq!(value["scenes"][0]["number"], "1");
        assert_eq!(value["locations"][0]["scenes"], 1);
        assert_eq!(value["locations"][0]["width"], 8.0);
        assert_eq!(value["characters"][0]["color"], "red");
        assert_eq!(value["characters"][0]["group"], 12);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
