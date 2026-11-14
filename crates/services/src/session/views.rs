//! What the session tells the surfaces about the project and the plan on view, ready to show:
//! the project lists, the tooltips and the set of the current scene.

use klappe_model::{CameraPresets, Location};
use klappe_stellprobe::horizontal_fov_deg;

use super::Session;
use crate::views::{CharacterRow, Hint, LocationRow, Overview, PlanRow, SceneRow};

/// Tenths of a degree: the angle of view in a tooltip is shown to one decimal.
const FOV_STEPS_PER_DEG: f64 = 10.0;

/// What a group's head count is for a single person.
const NO_GROUP: u32 = 0;

impl Session {
    /// The location of the scene on view.
    pub fn scene_location(&self) -> Option<&Location> {
        self.project().location(&self.scene()?.location)
    }

    /// Whether the scene's set has a background image.
    pub fn has_background(&self) -> bool {
        self.scene_location()
            .is_some_and(|l| l.set.background.is_some())
    }

    /// The text of the note `id` on the scene's set.
    pub fn note_text(&self, id: &str) -> Option<String> {
        let set = &self.scene_location()?.set;

        Some(set.notes.iter().find(|n| n.id == id)?.text.clone())
    }

    /// The lists of the project view.
    pub fn overview(&self) -> Overview {
        let project = self.project();
        let scenes = project.scenes.iter().map(|s| {
            let location = project.location(&s.location);

            SceneRow {
                id: s.id.clone(),
                number: s.number.clone(),
                heading: s.heading.clone(),
                location: s.location.clone(),
                location_name: location.map(|l| l.name.clone()).unwrap_or_default(),
            }
        });
        let locations = project.locations.iter().map(|l| LocationRow {
            id: l.id.clone(),
            name: l.name.clone(),
            width_m: l.set.width_m,
            depth_m: l.set.depth_m,
            scenes: project.scenes.iter().filter(|s| s.location == l.id).count(),
        });
        let characters = project.characters.iter().map(|c| CharacterRow {
            id: c.id.clone(),
            name: c.name.clone(),
            short: c.short.clone(),
            color: c.color.name(),
            group: c.group.unwrap_or(NO_GROUP),
        });
        let plans = self
            .scene()
            .map(|s| s.plans.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|p| PlanRow {
                id: p.id.clone(),
                name: p.name.clone(),
                shots: p.shots.len(),
            });

        Overview {
            scenes: scenes.collect(),
            locations: locations.collect(),
            characters: characters.collect(),
            plans: plans.collect(),
            plan: self.plan().map(|p| p.id.clone()).unwrap_or_default(),
        }
    }

    /// The tooltip of camera `id`; `None` without such a camera.
    pub fn camera_hint(&self, id: &str) -> Option<Hint> {
        let plan = self.plan()?;
        let c = plan.camera(id)?;
        let shots: Vec<&str> = plan
            .shots
            .iter()
            .filter(|shot| shot.camera == id)
            .map(|shot| shot.number.as_str())
            .collect();
        let follows = c
            .follows
            .as_deref()
            .and_then(|character| self.project().character(character))
            .map(|character| character.title())
            .unwrap_or_default();
        let lens_mm = self.lens_now_mm(id).unwrap_or(c.lens_mm);
        let fov_deg = horizontal_fov_deg(c.sensor, lens_mm);

        Some(Hint::Camera {
            label: c.label.clone(),
            lens_mm,
            own_lens_mm: c.lens_mm,
            sensor_mm: c.sensor.width_mm(),
            aspect: c.aspect,
            body: body_id(self.camera_presets(), c.sensor.width_mm()).to_owned(),
            fov_deg: (fov_deg * FOV_STEPS_PER_DEG).round() / FOV_STEPS_PER_DEG,
            height_m: c.marks.first().map(|m| m.height_m).unwrap_or(0.0),
            shots: shots.join(", "),
            follows,
        })
    }

    /// The tooltip of character `id`'s figure; `index` is the mark, 0-based, if any.
    pub fn figure_hint(&self, id: &str, index: Option<usize>) -> Hint {
        let (name, index, at_s) = self.figure_facts(id, index);

        Hint::Figure { name, index, at_s }
    }

    /// The tooltip of mark `index` (0-based) of character `id`.
    pub fn mark_hint(&self, id: &str, index: Option<usize>) -> Hint {
        let (name, index, at_s) = self.figure_facts(id, index);

        Hint::Mark { name, index, at_s }
    }

    /// The tooltip of light `id`; `None` without such a light.
    pub fn light_hint(&self, id: &str) -> Option<Hint> {
        let l = self.plan()?.lights.iter().find(|l| l.id == id)?;

        Some(Hint::Light {
            label: l.label.clone(),
            temp_k: l.color_temp_k,
            power_w: l.power_w,
        })
    }

    /// The tooltip of object `id` on the scene's set; `None` without such an object.
    pub fn object_hint(&self, id: &str) -> Option<Hint> {
        let set = &self.scene_location()?.set;
        let o = set.objects.iter().find(|o| o.id == id)?;

        Some(Hint::Object {
            label: o.label.clone(),
            library: o.kind.clone(),
            width_m: o.width_m,
            depth_m: o.depth_m,
            height_m: o.height_m,
        })
    }

    /// The tooltip of note `id`; empty text for an unknown note.
    pub fn note_hint(&self, id: &str) -> Hint {
        Hint::Note {
            text: self.note_text(id).unwrap_or_default(),
        }
    }

    /// Name, 1-based index and time of a figure's mark.
    fn figure_facts(&self, id: &str, index: Option<usize>) -> (String, Option<usize>, Option<f64>) {
        let name = self
            .project()
            .character(id)
            .map(|c| c.title())
            .unwrap_or_default();
        let at_s = index
            .and_then(|i| self.plan()?.figure(id)?.marks.get(i))
            .map(|m| m.at_s);

        (name, index.map(|i| i + 1), at_s)
    }
}

/// The id of the body whose sensor is `width_mm` wide, else empty: a sensor of another width
/// is a custom one.
fn body_id(presets: &CameraPresets, width_mm: f64) -> &str {
    let body = presets.bodies.iter().find(|b| b.width_mm == width_mm);

    body.map(|b| b.id.as_str()).unwrap_or_default()
}
