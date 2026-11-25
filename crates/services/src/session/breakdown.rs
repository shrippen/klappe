//! The breakdown as the surfaces see it (DB-60, D-22): the element list and the sheet of the
//! scene on view. Rebuilt with the project lists on every playback tick, so each is one pass.

use std::collections::{HashMap, HashSet};

use klappe_model::{Character, Element, ElementCategory, Location, Look, Scene};
use klappe_script::Element as Line;

use super::Session;
use crate::error::ServiceError;
use crate::field::Field;
use crate::project_edit::{BreakdownEdit, ProjectEdit};
use crate::scene_text::Shown;
use crate::script_refs;
use crate::sheet_time::{show_planned, show_prestop};
use crate::views::{
    ElementRow, LookElement, LookRow, SceneRef, SheetCategory, SheetElement, SheetFigure,
    SheetLookElement, SheetView, Suggestion,
};

/// What a group's head count is for a single person.
const NO_GROUP: u32 = 0;

fn figure(character: &Character, look: Option<&Look>) -> SheetFigure {
    SheetFigure {
        id: character.id.clone(),
        name: character.name.clone(),
        color: character.color.name(),
        group: character.group.unwrap_or(NO_GROUP),
        look: look.map(|l| l.id.clone()),
    }
}

impl Session {
    /// The elements with the numbers of the scenes that list them, built in one pass over
    /// the scenes, not once per element.
    pub(super) fn element_rows(&self) -> Vec<ElementRow> {
        let mut scenes: HashMap<&str, Vec<SceneRef>> = HashMap::new();
        for scene in &self.project.scenes {
            for id in &scene.sheet.elements {
                scenes.entry(id).or_default().push(SceneRef {
                    id: scene.id.clone(),
                    number: scene.number.clone(),
                });
            }
        }

        self.project
            .elements
            .iter()
            .map(|e| ElementRow {
                id: e.id.clone(),
                category: e.category.name(),
                name: e.name.clone(),
                description: e.description.clone(),
                status: e.status.name(),
                scenes: scenes.remove(e.id.as_str()).unwrap_or_default(),
            })
            .collect()
    }

    /// The looks of the characters the project has, in the order they were made; a look of a
    /// character that is gone is not offered. An element id no element has is left out.
    pub(super) fn look_rows(&self) -> Vec<LookRow> {
        let mut scenes: HashMap<&str, Vec<SceneRef>> = HashMap::new();
        for scene in &self.project.scenes {
            for (character, id) in &scene.sheet.looks {
                if self.scene_look(scene, character).is_none() {
                    continue;
                }
                scenes.entry(id).or_default().push(SceneRef {
                    id: scene.id.clone(),
                    number: scene.number.clone(),
                });
            }
        }
        let named = |l: &&Look| self.project.character(&l.character).is_some();

        self.project
            .looks
            .iter()
            .filter(named)
            .map(|l| LookRow {
                scenes: scenes.get(l.id.as_str()).cloned().unwrap_or_default(),
                id: l.id.clone(),
                character: l.character.clone(),
                name: l.name.clone(),
                elements: l
                    .elements
                    .iter()
                    .filter_map(|id| self.project.elements.iter().find(|e| e.id == *id))
                    .map(|e| LookElement {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        category: e.category.name(),
                    })
                    .collect(),
            })
            .collect()
    }

    /// The look `character` wears in `scene`; `None` for none, or a look that is gone or is
    /// another character's.
    pub(super) fn scene_look(&self, scene: &Scene, character: &str) -> Option<&Look> {
        let id = scene.sheet.looks.get(character)?;

        self.project
            .looks
            .iter()
            .find(|l| l.id == *id && l.character == character)
    }

    /// The ids of the characters with a cue in the scene at `entry`, in the order of their
    /// first cue; none when the entry is not a scene of the book.
    pub(super) fn speaking(&self, entry: usize) -> Vec<String> {
        let Some(book) = self.book_scene_of(entry) else {
            return Vec::new();
        };
        let scenes = self.script.scenes();
        let Some(scene) = scenes.get(book) else {
            return Vec::new();
        };

        let mut ids: Vec<String> = Vec::new();
        let cues = scene
            .elements
            .iter()
            .filter(|e| matches!(e, Line::Character { .. }));
        for cue in cues {
            let named = self
                .project
                .characters
                .iter()
                .find(|c| script_refs::calls(cue, c));
            if let Some(character) = named.filter(|c| !ids.contains(&c.id)) {
                ids.push(character.id.clone());
            }
        }

        ids
    }

    /// The sheet of the scene on view; `shot` holds where each scene is shot.
    pub(super) fn sheet_view(&self, shot: &[Option<&Location>]) -> Option<SheetView> {
        let scene = self.scene()?;
        let sheet = &scene.sheet;
        let speaking = self.speaking(self.scene);
        let lines = self.book_lines(self.scene);

        let spoken = speaking.iter().filter_map(|id| self.project.character(id));
        let present = sheet
            .characters
            .iter()
            .filter(|id| !speaking.contains(id))
            .filter_map(|id| self.project.character(id));

        Some(SheetView {
            story_day: sheet.story_day.clone(),
            prestop_text: show_prestop(sheet.prestop_s),
            planned_text: show_planned(sheet.planned_min),
            continuity: sheet.continuity.clone(),
            note: sheet.note.clone(),
            speaking: spoken
                .map(|c| figure(c, self.scene_look(scene, &c.id)))
                .collect(),
            present: present
                .map(|c| figure(c, self.scene_look(scene, &c.id)))
                .collect(),
            categories: self.sheet_categories(scene, &Self::lost_marks(scene, &lines)),
            suggestions: self.suggestions(scene, shot.get(self.scene).copied().flatten(), &lines),
            text: self.marked_text(scene, &lines),
        })
    }

    /// The elements of the scene by category, only the categories that have any, its own or
    /// those of the looks worn in it; `lost` are those whose marks the text no longer has.
    fn sheet_categories(&self, scene: &Scene, lost: &HashSet<String>) -> Vec<SheetCategory> {
        let worn = self.look_elements(scene);
        let listed = self.by_category(scene);
        let mut rows: Vec<SheetCategory> = Vec::new();

        for category in ElementCategory::ALL {
            let own: Vec<SheetElement> = listed
                .iter()
                .filter(|(c, _)| *c == category)
                .flat_map(|(_, elements)| elements)
                .map(|e| SheetElement {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    status: e.status.name(),
                    lost: lost.contains(&e.id),
                })
                .collect();
            let from_looks: Vec<SheetLookElement> = worn
                .iter()
                .filter(|(c, _)| *c == category)
                .map(|(_, e)| e.clone())
                .collect();
            if own.is_empty() && from_looks.is_empty() {
                continue;
            }

            rows.push(SheetCategory {
                category: category.name(),
                elements: own,
                from_looks,
            });
        }

        rows
    }

    /// The elements of the looks the scene's characters wear, with their category.
    fn look_elements(&self, scene: &Scene) -> Vec<(ElementCategory, SheetLookElement)> {
        let mut worn = Vec::new();
        let entry = self.project.scenes.iter().position(|s| s.id == scene.id);
        let Some(entry) = entry else {
            return worn;
        };

        for character in self.scene_figures(entry, scene) {
            let Some(look) = self.scene_look(scene, &character.id) else {
                continue;
            };
            let found = look
                .elements
                .iter()
                .filter_map(|id| self.project.elements.iter().find(|e| e.id == *id));
            for e in found {
                let shown = SheetLookElement {
                    id: e.id.clone(),
                    name: e.name.clone(),
                    status: e.status.name(),
                    figure: character.name.clone(),
                };
                worn.push((e.category, shown));
            }
        }

        worn
    }

    /// The elements of a scene's sheet, by category in the sheet's order; an id no element
    /// has is ignored.
    pub(super) fn by_category(&self, scene: &Scene) -> Vec<(ElementCategory, Vec<&Element>)> {
        self.group_by_category(&scene.sheet.elements)
    }

    /// The elements of these ids by category in the sheet's order, only the categories that
    /// have any; an id no element has is ignored.
    pub(super) fn group_by_category(
        &self,
        ids: &[String],
    ) -> Vec<(ElementCategory, Vec<&Element>)> {
        let listed: Vec<&Element> = ids
            .iter()
            .filter_map(|id| self.project.elements.iter().find(|e| e.id == *id))
            .collect();

        ElementCategory::ALL
            .into_iter()
            .map(|category| {
                let elements: Vec<&Element> = listed
                    .iter()
                    .filter(|e| e.category == category)
                    .copied()
                    .collect();

                (category, elements)
            })
            .filter(|(_, elements)| !elements.is_empty())
            .collect()
    }

    /// The elements the sheet neither lists nor turned down that the set the scene is shot on
    /// offers through an object, then those named in its text (D-23); one suggestion each.
    fn suggestions(
        &self,
        scene: &Scene,
        at: Option<&Location>,
        lines: &[Shown],
    ) -> Vec<Suggestion> {
        let mut found = self.set_offers(scene, at);
        let named = self.named_in_text(scene, lines, &found);
        found.extend(named);

        found
    }

    /// The objects of the set that stand for an element the sheet neither lists nor turned
    /// down, one suggestion per element.
    fn set_offers(&self, scene: &Scene, at: Option<&Location>) -> Vec<Suggestion> {
        let Some(location) = at else {
            return Vec::new();
        };
        let sheet = &scene.sheet;
        let mut seen: HashSet<&str> = HashSet::new();
        let mut found = Vec::new();

        for object in &location.set.objects {
            let Some(id) = object.element.as_deref() else {
                continue;
            };
            let known = self.project.elements.iter().find(|e| e.id == id);
            let Some(element) = known else {
                continue;
            };
            let taken = sheet
                .elements
                .iter()
                .chain(&sheet.excluded)
                .any(|e| e == id);
            if taken || !seen.insert(id) {
                continue;
            }

            found.push(Suggestion {
                element: element.id.clone(),
                name: element.name.clone(),
                category: element.category.name(),
                object: Some(object.label.clone()),
            });
        }

        found
    }

    /// A character with a cue in the scene is in it already: adding it to the sheet is refused.
    pub(super) fn check_sheet_character(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter { scene, character }) = edit
        else {
            return Ok(());
        };
        let Some(entry) = self.project.scenes.iter().position(|s| s.id == *scene) else {
            return Ok(());
        };
        if !self.speaking(entry).contains(character) {
            return Ok(());
        }

        Err(ServiceError::BadValue {
            field: Field::Character,
            value: character.clone(),
        })
    }

    /// A character not in the scene wears no look there: `SetSceneLook` is refused for it.
    /// In the scene is: with a cue in the book, or named by the sheet.
    pub(super) fn check_scene_look(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::Breakdown(BreakdownEdit::SetSceneLook {
            scene,
            character,
            look: Some(_),
        }) = edit
        else {
            return Ok(());
        };
        let Some(entry) = self.project.scenes.iter().position(|s| s.id == *scene) else {
            return Ok(());
        };
        let named = self.project.scenes[entry]
            .sheet
            .characters
            .contains(character);
        if named || self.speaking(entry).contains(character) {
            return Ok(());
        }

        Err(ServiceError::BadValue {
            field: Field::Character,
            value: character.clone(),
        })
    }
}
