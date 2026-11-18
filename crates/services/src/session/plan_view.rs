//! What makes up a view of the plan besides the project: which layers and tracks are hidden or
//! faded, and the sizes of the image files. The session holds one; a video job takes a copy, so
//! both put the display list together the same way (`list`).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use klappe_model::{Location, Project, Scene};
use klappe_stellprobe::{add_sun, handles, render, set_at, DisplayList, Role, Source};
use klappe_store::ImageStamp;

use super::fade::{fades_of, Fade};
use super::layers::is_shown;
use super::Layer;

/// The pixel size of an image file and the stamp it was read at.
type StampedSize = (ImageStamp, (u32, u32));

/// The view settings of a plan and the project folder its images are read from.
#[derive(Debug, Clone)]
pub(super) struct PlanView {
    pub(super) dir: PathBuf,
    pub(super) hidden_layers: HashSet<Layer>,
    pub(super) hidden_tracks: HashSet<String>,
    pub(super) faded_layers: HashMap<Layer, Fade>,
    /// Pixel size of the image files read so far: every redraw asks, the file header is read
    /// once. `bilder/` is the user's, so a file may change under its name: an entry counts
    /// only while the file's stamp (length, modification time) is the one it was read at.
    /// Unreadable files are not kept and are tried again.
    image_px: RefCell<HashMap<String, StampedSize>>,
}

impl PlanView {
    pub(super) fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            hidden_layers: HashSet::new(),
            hidden_tracks: HashSet::new(),
            faded_layers: HashMap::new(),
            image_px: RefCell::new(HashMap::new()),
        }
    }

    /// The plan `plan` of scene `scene` of `project` at `t_s`, without hidden layers and
    /// tracks, with the grips on the bends of the way `selected` belongs to (LH-21).
    pub(super) fn list(
        &self,
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
            self.image_size(file)
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
    fn image_size(&self, file: &str) -> Option<(u32, u32)> {
        let stamp = klappe_store::image_stamp(&self.dir, file).ok()?;
        let mut known = self.image_px.borrow_mut();
        if let Some((_, px)) = known.get(file).filter(|(seen, _)| *seen == stamp) {
            return Some(*px);
        }

        let px = klappe_store::image_size(&self.dir, file).ok()?;
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
