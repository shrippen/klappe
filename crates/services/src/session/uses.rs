//! Where scenes and characters occur, and what renaming them does to the script (D-08).

use klappe_script::Element;

use super::Session;
use crate::script_refs::{self, ScriptChange};
use crate::views::{CastUse, SceneCard, SceneUse};

/// Headings end in the time of day behind a dash: `INT. SÄNGERSAAL – TAG`.
const TIME_SEPARATORS: [&str; 2] = [" – ", " - "];

impl Session {
    /// The scenes the character plays in, in scene order.
    pub fn character_uses(&self, id: &str) -> Vec<SceneUse> {
        let Some(character) = self.project.character(id) else {
            return Vec::new();
        };

        self.project
            .scenes
            .iter()
            .filter_map(|scene| {
                let cues = script_refs::cue_count(self.script_text(), scene, character);
                let figures = scene
                    .plans
                    .iter()
                    .filter(|p| p.figure(&character.id).is_some())
                    .count();
                if cues == 0 && figures == 0 {
                    return None;
                }

                Some(SceneUse {
                    id: scene.id.clone(),
                    number: scene.number.clone(),
                    heading: scene.heading.clone(),
                    cues,
                    figures,
                })
            })
            .collect()
    }

    /// The card of the scene `id`; `None` without such a scene.
    pub fn scene_card(&self, id: &str) -> Option<SceneCard> {
        let scene = self.project.scene(id)?;
        let cast = self
            .project
            .characters
            .iter()
            .filter_map(|c| {
                let cues = script_refs::cue_count(self.script_text(), scene, c);
                let planned = scene.plans.iter().any(|p| p.figure(&c.id).is_some());
                (cues > 0 || planned).then(|| CastUse {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    color: c.color.name(),
                    cues,
                })
            })
            .collect();

        Some(SceneCard {
            time: time_of(&scene.heading),
            plans: scene.plans.len(),
            shots: scene.plans.iter().map(|p| p.shots.len()).sum(),
            cast,
        })
    }

    /// What the new `number` and `heading` of the scene `id` would change in the script.
    pub fn scene_script_changes(&self, id: &str, number: &str, heading: &str) -> Vec<ScriptChange> {
        script_refs::scene_changes(&self.project, self.script_text(), id, number, heading)
    }

    /// What the new `name` of the character `id` would change in the script.
    pub fn character_script_changes(&self, id: &str, name: &str) -> Vec<ScriptChange> {
        script_refs::character_changes(&self.project, self.script_text(), id, name)
    }

    /// The id of the character a cue names.
    pub(super) fn cue_character(&self, element: &Element) -> String {
        self.project
            .characters
            .iter()
            .find(|c| script_refs::calls(element, c))
            .map(|c| c.id.clone())
            .unwrap_or_default()
    }
}

/// The text after the last dash of the heading; empty without a dash.
fn time_of(heading: &str) -> String {
    TIME_SEPARATORS
        .iter()
        .filter_map(|sep| heading.rsplit_once(sep))
        .map(|(_, time)| time.trim().to_owned())
        .next()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::time_of;

    #[test]
    fn the_time_follows_the_last_dash() {
        assert_eq!(time_of("INT. SÄNGERSAAL – TAG"), "TAG");
        assert_eq!(time_of("EXT. HOF - NACHT"), "NACHT");
        assert_eq!(time_of("EXT. HOF"), "");
    }
}
