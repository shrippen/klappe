//! What the project view and the start page need besides the session: the lists as JSON for
//! QML, the recent projects, and the texts QML sends back parsed (LH-01 to LH-03).

use std::path::Path;

use klappe_services::{Field, FigureColor, ServiceError, Session};
use serde_json::{json, Value};

/// Projects the start page remembers.
pub const RECENT_MAX: usize = 8;

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
                "color": color_name(c.color),
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

/// The role as the model writes it in `figuren.toml`.
fn color_name(color: FigureColor) -> String {
    match serde_json::to_value(color) {
        Ok(Value::String(name)) => name,
        _ => String::new(),
    }
}

/// A colour by the name `project_json` gives it; `None` for any other text.
pub fn color_of(name: &str) -> Option<FigureColor> {
    serde_json::from_value(Value::String(name.to_owned())).ok()
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

/// `[{ path, name }]` of the folders that still exist, for the start page.
pub fn recent_view(list: &[String]) -> String {
    let rows: Vec<_> = list
        .iter()
        .filter(|p| Path::new(p).is_dir())
        .map(|p| json!({ "path": p, "name": folder_name(p) }))
        .collect();

    Value::Array(rows).to_string()
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
    use klappe_services::ProjectEdit;

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
    fn the_view_drops_folders_that_are_gone() {
        let here = env!("CARGO_MANIFEST_DIR");
        let view: Value = serde_json::from_str(&recent_view(&list(&[here, "/gibt/es/nicht"])))
            .unwrap_or_default();

        let rows = view.as_array().map(Vec::len);
        assert_eq!(rows, Some(1));
        assert_eq!(view[0]["name"], "app");
        assert_eq!(view[0]["path"], here);
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

    #[test]
    fn colours_go_by_their_role_name() {
        assert_eq!(color_of("purple"), Some(FigureColor::Purple));
        assert_eq!(color_of("magenta"), None);
        assert_eq!(color_name(FigureColor::Cyan), "cyan");
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
