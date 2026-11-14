//! The `klappe` section of the demo world as a project and its screenplay.
//!
//! ```text
//! world.json ──► localize(lang) ──► Section ──► Project   ──► store::save
//!                                      └──────► Fountain  ──┘ (drehbuch.fountain)
//! ```

use std::collections::HashMap;
use std::path::Path;

use klappe_model::{Character, FloorPlan, Location, Project, Scene};
use klappe_script::{Element, Script};
use serde::Deserialize;
use serde_json::Value;

use crate::error::DemoError;
use crate::lang::{localize, Lang};

/// The section of the world file that holds the example.
const SECTION: &str = "klappe";

#[derive(Deserialize)]
struct Section {
    title: String,
    #[serde(default)]
    title_page: Vec<(String, String)>,
    characters: Vec<Character>,
    locations: Vec<Location>,
    scenes: Vec<SceneSpec>,
}

#[derive(Deserialize)]
struct SceneSpec {
    id: String,
    number: String,
    heading: String,
    location: String,
    #[serde(default)]
    script: Vec<Line>,
    /// Plans as in the project format, except a link's `element` may be a script line's id.
    #[serde(default)]
    plans: Vec<Value>,
}

/// One block of the scene's script: an action, or a character's line.
#[derive(Deserialize)]
struct Line {
    id: Option<String>,
    action: Option<String>,
    say: Option<String>,
    paren: Option<String>,
    text: Option<String>,
}

/// Writes the example of one language to `dir`, replacing what is there.
pub fn write_example(world: &Value, lang: Lang, dir: &Path) -> Result<(), DemoError> {
    let section = world
        .get(SECTION)
        .ok_or_else(|| DemoError::World(format!("the world has no \"{SECTION}\" section")))?;
    let section: Section = serde_json::from_value(localize(section, lang)?)?;
    let (project, script) = build(section)?;

    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|source| DemoError::Io {
            path: dir.to_owned(),
            source,
        })?;
    }
    klappe_store::save(dir, &project, Some(&script))?;

    Ok(())
}

fn build(section: Section) -> Result<(Project, String), DemoError> {
    let mut project = Project::new(&section.title);
    let mut script = Script {
        title_page: section.title_page,
        elements: Vec::new(),
    };

    for spec in section.scenes {
        let (elements, ids) = elements_of(&spec, &section.characters)?;
        let mut plans = Vec::new();
        for plan in &spec.plans {
            plans.push(plan_of(plan, &ids)?);
        }

        script.elements.push(Element::SceneHeading {
            text: spec.heading.clone(),
            number: Some(spec.number.clone()),
        });
        script.elements.extend(elements);
        project.scenes.push(Scene {
            id: spec.id,
            number: spec.number,
            heading: spec.heading,
            location: spec.location,
            plans,
        });
    }
    project.characters = section.characters;
    project.locations = section.locations;

    Ok((project, klappe_script::write(&script)))
}

/// The scene's elements after its heading, and the index of the first element of each line
/// that has an id (what a script link names).
///
/// Example: an action, then a line with a parenthetical:
/// `[Action, Character, Parenthetical, Dialogue]`, ids `{"a": 0, "b": 1}`.
fn elements_of(
    spec: &SceneSpec,
    characters: &[Character],
) -> Result<(Vec<Element>, HashMap<String, usize>), DemoError> {
    let mut elements = Vec::new();
    let mut ids = HashMap::new();

    for line in &spec.script {
        if let Some(id) = &line.id {
            ids.insert(id.clone(), elements.len());
        }

        if let Some(action) = &line.action {
            elements.push(Element::Action(action.clone()));
            continue;
        }

        let who = line
            .say
            .as_deref()
            .ok_or_else(|| DemoError::World(format!("a line of {} says nothing", spec.id)))?;
        let character = characters
            .iter()
            .find(|c| c.id == who)
            .ok_or_else(|| DemoError::World(format!("no character {who}")))?;
        let text = line
            .text
            .clone()
            .ok_or_else(|| DemoError::World(format!("{who} has no text in {}", spec.id)))?;

        elements.push(Element::Character {
            name: character.name.to_uppercase(),
            extension: None,
            dual: false,
        });
        if let Some(paren) = &line.paren {
            elements.push(Element::Parenthetical(paren.clone()));
        }
        elements.push(Element::Dialogue(text));
    }

    Ok((elements, ids))
}

/// A plan, with each link's `element` given as an index.
fn plan_of(plan: &Value, ids: &HashMap<String, usize>) -> Result<FloorPlan, DemoError> {
    let mut plan = plan.clone();
    let links = plan.get_mut("links").and_then(Value::as_array_mut);

    for link in links.into_iter().flatten() {
        let Some(id) = link.get("element").and_then(Value::as_str) else {
            continue;
        };
        let index = ids
            .get(id)
            .ok_or_else(|| DemoError::World(format!("a link names the unknown line {id}")))?;
        link["element"] = Value::from(*index);
    }

    Ok(serde_json::from_value(plan)?)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn section() -> Value {
        json!({
            "title": {"de": "Titel", "en": "Title"},
            "characters": [{"id": "e", "name": "Elisabeth", "short": "E", "color": "purple"}],
            "locations": [],
            "scenes": [{
                "id": "s1", "number": "1",
                "heading": {"de": "INT. SAAL – TAG", "en": "INT. HALL – DAY"},
                "location": "saal",
                "script": [
                    {"id": "a", "action": {"de": "Sie tritt ein.", "en": "She enters."}},
                    {"id": "b", "say": "e", "paren": {"de": "leise", "en": "softly"},
                     "text": {"de": "Spät.", "en": "Late."}}
                ],
                "plans": [{"id": "p", "name": "Plan A",
                           "links": [{"element": "b", "target": {"kind": "shot", "id": "x"}}]}]
            }]
        })
    }

    #[test]
    fn a_link_names_the_first_element_of_its_line() {
        let section: Section =
            serde_json::from_value(localize(&section(), Lang::De).unwrap()).unwrap();

        let (project, script) = build(section).unwrap();

        // Action is 0, the cue of the next line is 1.
        assert_eq!(project.scenes[0].plans[0].links[0].element, 1);
        assert!(script.contains("INT. SAAL – TAG #1#"), "{script}");
        assert!(script.contains("ELISABETH\n(leise)\nSpät."), "{script}");
    }

    #[test]
    fn an_unknown_line_id_is_an_error() {
        let mut world = section();
        world["scenes"][0]["plans"][0]["links"][0]["element"] = json!("nope");
        let section: Section = serde_json::from_value(localize(&world, Lang::En).unwrap()).unwrap();

        let err = build(section).unwrap_err();

        assert!(err.to_string().contains("nope"), "{err}");
    }
}
