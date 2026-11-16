//! What the session tells the surfaces about the project and the plan on view, ready to show:
//! the project lists, the tooltips and the set of the current scene.

use klappe_model::{CameraPresets, DrawingShape, Location, Orientation, Paper};
use klappe_stellprobe::{
    aim_at, axis_pair, horizontal_fov_deg, object_extent, opening_span, picture, wall_top, Eye,
    Picture, Source,
};

use super::Session;
use crate::library;
use crate::views::{
    AxisChoice, CharacterRow, Hint, LocationRow, Overview, PaperRow, PlanRow, SceneRow,
};

/// Tenths of a degree: the angle of view in a tooltip is shown to one decimal.
const FOV_STEPS_PER_DEG: f64 = 10.0;

/// What a group's head count is for a single person.
const NO_GROUP: u32 = 0;

impl Session {
    /// The axis pair of the plan on view; `None` with fewer than two figures and no choice.
    pub fn axis_choice(&self) -> Option<AxisChoice> {
        let plan = self.plan()?;
        let (a, b) = axis_pair(plan)?;

        Some(AxisChoice {
            a: a.to_owned(),
            b: b.to_owned(),
            own: plan.axis.is_some(),
        })
    }

    /// The location of the scene on view.
    pub(crate) fn scene_location(&self) -> Option<&Location> {
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
            wall_height_m: l.set.wall_height_m,
            scenes: project.scenes.iter().filter(|s| s.location == l.id).count(),
        });
        let characters = project.characters.iter().map(|c| CharacterRow {
            id: c.id.clone(),
            name: c.name.clone(),
            short: c.short.clone(),
            color: c.color.name(),
            group: c.group.unwrap_or(NO_GROUP),
            size_m: c.size_m,
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
            paper: project.paper.name(),
            papers: Paper::ALL
                .map(|p| PaperRow {
                    id: p.name(),
                    label: p.label(),
                })
                .into(),
            orientations: Orientation::ALL.map(Orientation::name).into(),
        }
    }

    /// What a camera sees at the playhead as pixels, `width_px` wide (LH-36, LH-99): that of
    /// the selected shot, else `fallback` (the camera picked on the plan). Covering is right
    /// per pixel, roles instead of colours. `None` without a camera, or one without marks.
    pub fn camera_picture(&self, fallback: Option<&str>, width_px: usize) -> Option<Picture> {
        let plan = self.plan()?;
        let of_shot = self
            .selected_shot()
            .and_then(|id| plan.shots.iter().find(|shot| shot.id == id))
            .map(|shot| shot.camera.as_str());
        let location = self.project.location(&self.scene()?.location)?;
        let eye = Eye::Shot {
            camera: of_shot.or(fallback)?.to_owned(),
            t_s: self.time_s,
        };

        let forms = library::forms_of(&self.project.library, &location.set);

        picture(&self.project, &location.set, plan, &forms, &eye, width_px)
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
        // On the way the rig of the mark the camera runs to, else that of the mark it is at.
        let travelling = c
            .marks
            .windows(2)
            .find(|pair| (pair[0].at_s + pair[0].hold_s..pair[1].at_s).contains(&self.time_s))
            .map(|pair| pair[1].rig);
        let rig = travelling
            .or_else(|| {
                self.mark_info(&Source::Camera(id.to_owned()))
                    .filter(|info| info.has_way)
                    .and_then(|info| info.rig)
            })
            .map(|rig| rig.name().to_owned())
            .unwrap_or_default();

        Some(Hint::Camera {
            label: c.label.clone(),
            lens_mm,
            own_lens_mm: c.lens_mm,
            sensor_mm: c.sensor.width_mm(),
            aspect: c.aspect,
            body: body_id(self.camera_presets(), c.sensor.width_mm()).to_owned(),
            fov_deg: (fov_deg * FOV_STEPS_PER_DEG).round() / FOV_STEPS_PER_DEG,
            rig,
            height_m: aim_at(c, self.time_s).map_or(0.0, |(height_m, _)| height_m),
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
            light_kind: l.kind.name().to_owned(),
            facing_deg: l.facing_deg,
            beam_deg: l.beam_deg,
            tunable: l.kind.tunable_color(),
            gives_light: l.kind.gives_light(),
            temp_k: l.color_temp_k,
            power_w: l.power_w,
        })
    }

    /// How many objects of the scene's set have no height yet (LH-93).
    pub fn missing_heights(&self) -> usize {
        let Some(location) = self.scene_location() else {
            return 0;
        };

        location
            .set
            .objects
            .iter()
            .filter(|o| o.height_m.is_none())
            .count()
    }

    /// The next object without a height after object `after`, in the set's order and
    /// round again (LH-93); the first one without `after` or when it is gone. `None` when
    /// all have a height. The object `after` itself comes last.
    pub fn next_missing_height(&self, after: Option<&str>) -> Option<String> {
        let objects = &self.scene_location()?.set.objects;
        let count = objects.len();
        let last = after
            .and_then(|id| objects.iter().position(|o| o.id == id))
            .unwrap_or(count.checked_sub(1)?);

        (1..=count)
            .map(|step| &objects[(last + step) % count])
            .find(|o| o.height_m.is_none())
            .map(|o| o.id.clone())
    }

    /// How tall `source` stands now, in metres: an object's extent (what it is shown as without
    /// a height of its own), a wall's top (LH-95). `None` for any other source.
    pub fn height_of(&self, source: &Source) -> Option<f64> {
        let set = &self.scene_location()?.set;

        match source {
            Source::Object(id) => {
                let index = set.objects.iter().position(|o| o.id == *id)?;
                let (base_m, top_m) = object_extent(set, index);

                Some(top_m - base_m)
            }
            Source::Wall(index) => Some(wall_top(set, set.walls.get(*index)?)),
            _ => None,
        }
    }

    /// The tooltip of object `id` on the scene's set; `None` without such an object.
    pub fn object_hint(&self, id: &str) -> Option<Hint> {
        let set = &self.scene_location()?.set;
        let index = set.objects.iter().position(|o| o.id == id)?;
        let o = &set.objects[index];

        Some(Hint::Object {
            label: o.label.clone(),
            library: o.kind.clone(),
            width_m: o.width_m,
            depth_m: o.depth_m,
            height_m: o.height_m,
            base_m: object_extent(set, index).0,
            base_default: o.base_m.is_none(),
        })
    }

    /// The card of the set's wall `index` (LH-90); `None` without such a wall.
    pub fn wall_hint(&self, index: usize) -> Option<Hint> {
        let set = &self.scene_location()?.set;
        let wall = set.walls.get(index)?;

        Some(Hint::Wall {
            height_m: wall_top(set, wall),
            height_default: wall.height_m.is_none(),
        })
    }

    /// The tooltip and card of opening `index` of the set's wall `wall` (LH-10); `None`
    /// without such an opening.
    pub fn opening_hint(&self, wall: usize, index: usize) -> Option<Hint> {
        let set = &self.scene_location()?.set;
        let o = set.walls.get(wall)?.openings.get(index)?;

        let (sill_m, head_m) = opening_span(o);

        Some(Hint::Opening {
            opening: o.kind,
            width_m: o.width_m,
            sill_m,
            head_m,
            sill_default: o.sill_m.is_none(),
            head_default: o.head_m.is_none(),
            side: o.side,
            hinge: o.hinge,
        })
    }

    /// What the style card of drawing `id` shows; `None` without such a drawing.
    pub fn drawing_hint(&self, id: &str) -> Option<Hint> {
        let d = self.plan()?.drawings.iter().find(|d| d.id == id)?;

        Some(Hint::Drawing {
            color: d.color,
            dash: d.dash,
            fill: d.fill,
            closed: !matches!(
                d.shape,
                DrawingShape::Line { .. } | DrawingShape::Arrow { .. }
            ),
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
