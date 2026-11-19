//! What makes up a view of the plan besides the project: which layers and tracks are hidden or
//! faded, and the sizes of the image files. The session holds one; a video job takes a frozen
//! copy, so both put the display list together the same way (`list`).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use klappe_model::{Location, Project, Scene};
use klappe_stellprobe::{add_sun, handles, render, set_at, DisplayList, Role, Source};
use klappe_store::ImageStamp;

use super::fade::{fades_of, Fade};
use super::layers::is_shown;
use super::{Layer, Visibility};

/// The pixel size of an image file and the stamp it was read at.
type StampedSize = (ImageStamp, (u32, u32));

/// Whether a known image size is trusted or checked against the file again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sizes {
    Checked,
    Frozen,
}

/// The view settings of a plan.
#[derive(Debug, Clone)]
pub(super) struct PlanView {
    hidden_layers: HashSet<Layer>,
    hidden_tracks: HashSet<String>,
    faded_layers: HashMap<Layer, Fade>,
    sizes: Sizes,
    /// Pixel size of the image files read so far: every redraw asks, the file header is read
    /// once. `bilder/` is the user's, so a file may change under its name: an entry counts
    /// only while the file's stamp (length, modification time) is the one it was read at.
    /// Unreadable files are not kept and are tried again.
    image_px: RefCell<HashMap<String, StampedSize>>,
}

impl PlanView {
    pub(super) fn new() -> Self {
        Self {
            hidden_layers: HashSet::new(),
            hidden_tracks: HashSet::new(),
            faded_layers: HashMap::new(),
            sizes: Sizes::Checked,
            image_px: RefCell::new(HashMap::new()),
        }
    }

    /// A copy for a job that runs over time: the image sizes it reads stay as read first, so
    /// a file replaced meanwhile cannot give a new size to the bytes the job read before.
    pub(super) fn frozen(&self) -> Self {
        Self {
            sizes: Sizes::Frozen,
            image_px: RefCell::new(HashMap::new()),
            ..self.clone()
        }
    }

    pub(super) fn set_layer(&mut self, layer: Layer, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_layers.remove(&layer),
            Visibility::Hidden => self.hidden_layers.insert(layer),
        };
    }

    pub(super) fn layer_hidden(&self, layer: Layer) -> bool {
        self.hidden_layers.contains(&layer)
    }

    pub(super) fn set_track(&mut self, id: &str, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_tracks.remove(id),
            Visibility::Hidden => self.hidden_tracks.insert(id.to_owned()),
        };
    }

    pub(super) fn track_hidden(&self, id: &str) -> bool {
        self.hidden_tracks.contains(id)
    }

    pub(super) fn set_fade(&mut self, layer: Layer, fade: Fade) {
        match fade {
            Fade::Full => self.faded_layers.remove(&layer),
            other => self.faded_layers.insert(layer, other),
        };
    }

    pub(super) fn fade(&self, layer: Layer) -> Fade {
        self.faded_layers.get(&layer).copied().unwrap_or_default()
    }

    /// The plan `plan` of scene `scene` of `project` (images read from `dir`) at `t_s`, without
    /// hidden layers and tracks, with the grips on the bends of the way `selected` belongs to
    /// (LH-21).
    pub(super) fn list(
        &self,
        dir: &Path,
        project: &Project,
        (scene, plan): (usize, usize),
        t_s: f64,
        selected: Option<&Source>,
    ) -> DisplayList {
        let scene = project.scenes.get(scene);
        let Some((scene, plan)) = scene.and_then(|s| Some((s, s.plans.get(plan)?))) else {
            return DisplayList::default();
        };
        let Some(location) = project.location(&scene.location) else {
            return DisplayList::default();
        };

        let mut list = render(project, &location.set, plan, t_s, &|file| {
            self.image_size(dir, file)
        });
        sun_on(&mut list, location, scene);
        if let Some(selected) = selected {
            let set = set_at(&location.set, plan, t_s);
            list.items.extend(handles(project, &set, plan, selected));
        }
        self.retain_shown(&mut list);

        list
    }

    /// The strength of each item of `list`, in its order, 1 for full.
    pub(super) fn fades(&self, list: &DisplayList) -> Vec<f64> {
        fades_of(list, &self.faded_layers)
    }

    fn shows(&self, role: Role, source: Option<&Source>) -> bool {
        is_shown(&self.hidden_layers, &self.hidden_tracks, role, source)
    }

    /// Leaves out of `list` what is hidden; a hidden lamp is no more in the legend than on the
    /// plan.
    fn retain_shown(&self, list: &mut DisplayList) {
        list.items
            .retain(|item| self.shows(item.role, item.source.as_ref()));
        let on_plan = |id: &str| {
            let source = Source::Light(id.to_owned());
            list.items
                .iter()
                .any(|i| i.source.as_ref() == Some(&source))
        };
        list.lamps.retain(|l| on_plan(&l.id));
    }

    /// Pixel size of an image of the project folder; remembered per file.
    fn image_size(&self, dir: &Path, file: &str) -> Option<(u32, u32)> {
        let mut known = self.image_px.borrow_mut();
        if self.sizes == Sizes::Frozen {
            if let Some((_, px)) = known.get(file) {
                return Some(*px);
            }
        }

        let stamp = klappe_store::image_stamp(dir, file).ok()?;
        if let Some((_, px)) = known.get(file).filter(|(seen, _)| *seen == stamp) {
            return Some(*px);
        }

        let px = klappe_store::image_size(dir, file).ok()?;
        known.insert(file.to_owned(), (stamp, px));

        Some(px)
    }
}

/// Adds the sun of `scene` at `location` to `list`, when both give place and time (LH-52).
fn sun_on(list: &mut DisplayList, location: &Location, scene: &Scene) {
    if let (Some(site), Some(time)) = (&location.site, &scene.sun) {
        add_sun(list, &location.set, site, time);
    }
}
