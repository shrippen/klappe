//! What the project view and the start page need besides the session: the lists as JSON for
//! QML, the recent projects, and the texts QML sends back parsed (LH-01 to LH-03).

use std::path::{Path, PathBuf};

use klappe_services::{CardState, Field, Language, ProjectCard, ServiceError, Session};
use serde_json::{json, Value};

/// Projects the start page remembers.
pub const RECENT_MAX: usize = 8;

const STATE_LOADING: &str = "loading";
const STATE_MISSING: &str = "missing";
const CACHE_VAR: &str = "XDG_CACHE_HOME";
const HOME_VAR: &str = "HOME";
const CACHE_APP_DIR: &str = "klappe";
const DATA_VAR: &str = "XDG_DATA_HOME";
const DATA_APP_DIR: &str = "klappe";
const EXAMPLES_DIR: &str = "beispiel";
const HOME_SHORT: &str = "~";

/// `{ scenes, locations, characters, plans, plan }`: lists of plain objects for QML, `plans`
/// those of the scene on view and `plan` the id of the one shown. The colour is the model's
/// role name (`purple`), which Kante's `labelColor` takes as it is.
pub fn project_json(session: &Session) -> String {
    let project = session.project();
    let scenes: Vec<_> = project
        .scenes
        .iter()
        .map(|s| {
            let location = project.location(&s.location).map(|l| l.name.as_str());
            json!({
                "id": s.id,
                "number": s.number,
                "heading": s.heading,
                "location": s.location,
                "locationName": location.unwrap_or_default(),
            })
        })
        .collect();

    let locations: Vec<_> = project
        .locations
        .iter()
        .map(|l| {
            let scenes = project.scenes.iter().filter(|s| s.location == l.id).count();
            json!({
                "id": l.id,
                "name": l.name,
                "width": l.set.width_m,
                "depth": l.set.depth_m,
                "scenes": scenes,
            })
        })
        .collect();

    let characters: Vec<_> = project
        .characters
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "short": c.short,
                "color": c.color.name(),
                "group": c.group.unwrap_or(NO_GROUP),
            })
        })
        .collect();

    // The variants of the scene on view (LH-04): what the plan chooser lists.
    let plans: Vec<_> = session
        .scene()
        .map(|s| s.plans.as_slice())
        .unwrap_or_default()
        .iter()
        .map(|p| json!({ "id": p.id, "name": p.name, "shots": p.shots.len() }))
        .collect();
    let plan = session.plan().map(|p| p.id.as_str()).unwrap_or_default();

    json!({
        "scenes": scenes,
        "locations": locations,
        "characters": characters,
        "plans": plans,
        "plan": plan,
    })
    .to_string()
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

/// A length typed by the user, `"8,5"` or `"8.5 m"`.
pub fn metres(field: Field, text: &str) -> Result<f64, ServiceError> {
    let digits = text.trim().trim_end_matches('m').trim().replace(',', ".");

    digits.parse().map_err(|_| ServiceError::BadValue {
        field,
        value: text.trim().to_owned(),
    })
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
/// `state` being `ready`, `missing` or `unreadable`, `preview` a file path or empty. `cards`
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
            "preview": card.preview.as_ref().map(|p| p.to_string_lossy()).unwrap_or_default(),
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

/// Where the app keeps what it can make again (previews): `$XDG_CACHE_HOME/klappe`, else
/// `~/.cache/klappe`.
pub fn cache_dir() -> Option<PathBuf> {
    let base = match std::env::var_os(CACHE_VAR) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os(HOME_VAR)?).join(".cache"),
    };

    Some(base.join(CACHE_APP_DIR))
}

/// Where the app keeps what the user may change (the example copies): `$XDG_DATA_HOME/klappe`,
/// else `~/.local/share/klappe`.
pub fn data_dir() -> Option<PathBuf> {
    let base = match std::env::var_os(DATA_VAR) {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os(HOME_VAR)?).join(".local/share"),
    };

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
        assert_eq!(view[0]["preview"], "/cache/a.png");
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
        assert_eq!(metres(Field::Width, "8,5").ok(), Some(8.5));
        assert_eq!(metres(Field::Width, " 12 m ").ok(), Some(12.0));
        assert!(matches!(
            metres(Field::Depth, "breit"),
            Err(ServiceError::BadValue {
                field: Field::Depth,
                ..
            })
        ));
    }

    /// Characterisation: the form reads any finite-or-not number; the session sets the range.
    #[test]
    fn lengths_have_no_range_here() {
        assert_eq!(metres(Field::Width, "0").ok(), Some(0.0));
        assert_eq!(metres(Field::Width, "-3").ok(), Some(-3.0));
        assert!(metres(Field::Width, "").is_err());
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
