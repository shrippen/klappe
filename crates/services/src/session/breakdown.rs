//! The breakdown as the surfaces see it (DB-60, D-22): the element list and the sheet of the
//! scene on view. Rebuilt with the project lists on every playback tick, so each is one pass.

use std::collections::{HashMap, HashSet};

use klappe_model::{Character, Element, ElementCategory, Location, Scene};
use klappe_script::Element as Line;

use super::Session;
use crate::error::ServiceError;
use crate::field::Field;
use crate::project_edit::ProjectEdit;
use crate::script_refs;
use crate::views::{ElementRow, SheetCategory, SheetElement, SheetFigure, SheetView, Suggestion};

/// What a group's head count is for a single person.
const NO_GROUP: u32 = 0;

fn figure(character: &Character) -> SheetFigure {
    SheetFigure {
        id: character.id.clone(),
        name: character.name.clone(),
        color: character.color.name(),
        group: character.group.unwrap_or(NO_GROUP),
    }
}

impl Session {
    /// The elements with the numbers of the scenes that list them, built in one pass over
    /// the scenes, not once per element.
    pub(super) fn element_rows(&self) -> Vec<ElementRow> {
        let mut scenes: HashMap<&str, Vec<String>> = HashMap::new();
        for scene in &self.project.scenes {
            for id in &scene.sheet.elements {
                scenes.entry(id).or_default().push(scene.number.clone());
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

        let spoken = speaking.iter().filter_map(|id| self.project.character(id));
        let present = sheet
            .characters
            .iter()
            .filter(|id| !speaking.contains(id))
            .filter_map(|id| self.project.character(id));

        Some(SheetView {
            story_day: sheet.story_day.clone(),
            prestop_s: sheet.prestop_s,
            planned_min: sheet.planned_min,
            continuity: sheet.continuity.clone(),
            note: sheet.note.clone(),
            speaking: spoken.map(figure).collect(),
            present: present.map(figure).collect(),
            categories: self.sheet_categories(scene),
            suggestions: self.suggestions(scene, shot.get(self.scene).copied().flatten()),
        })
    }

    /// The elements of the scene by category, only the categories that have any. An id no
    /// element has is ignored.
    fn sheet_categories(&self, scene: &Scene) -> Vec<SheetCategory> {
        let listed: Vec<&Element> = scene
            .sheet
            .elements
            .iter()
            .filter_map(|id| self.project.elements.iter().find(|e| e.id == *id))
            .collect();

        ElementCategory::ALL
            .into_iter()
            .filter_map(|category| {
                let elements: Vec<SheetElement> = listed
                    .iter()
                    .filter(|e| e.category == category)
                    .map(|e| SheetElement {
                        id: e.id.clone(),
                        name: e.name.clone(),
                        status: e.status.name(),
                    })
                    .collect();

                (!elements.is_empty()).then(|| SheetCategory {
                    category: category.name(),
                    elements,
                })
            })
            .collect()
    }

    /// The objects of the set the scene is shot on that stand for an element the sheet neither
    /// lists nor turned down, one suggestion per element.
    fn suggestions(&self, scene: &Scene, at: Option<&Location>) -> Vec<Suggestion> {
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
                object: object.label.clone(),
            });
        }

        found
    }

    /// A character with a cue in the scene is in it already: adding it to the sheet is refused.
    pub(super) fn check_sheet_character(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::AddSceneCharacter { scene, character } = edit else {
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
}
