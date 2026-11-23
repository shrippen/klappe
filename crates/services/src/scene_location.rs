//! Where a scene is shot (DB-51): at its own shooting location when it has one, else at the
//! one of its scripted location (the place its heading names).
//!
//!   heading "INT. WACHRAUM – NACHT" ─► place WACHRAUM ─► place.location ─┐
//!                                       scene.location (if set) ─────────┴─► where it is shot

use std::collections::HashMap;

use klappe_model::{Kind, Location, Place, Project, Scene};
use klappe_script::heading_place;

/// The scripted location the heading of `scene` names; `None` for a content item or a place
/// name no scripted location is called by (yet).
pub(crate) fn place_of<'a>(places: &'a [Place], scene: &Scene) -> Option<&'a Place> {
    if scene.kind != Kind::Scene {
        return None;
    }
    let name = heading_place(&scene.heading);

    places.iter().find(|p| p.is_called(name))
}

/// The shooting location `scene` is shot at; `None` when neither the scene nor its scripted
/// location has one, or it is gone from the project.
pub(crate) fn location_of<'a>(
    project: &'a Project,
    places: &[Place],
    scene: &Scene,
) -> Option<&'a Location> {
    shot_at(project, scene, place_of(places, scene))
}

/// Where `scene` is shot, its scripted location `place` known.
pub(crate) fn shot_at<'a>(
    project: &'a Project,
    scene: &Scene,
    place: Option<&Place>,
) -> Option<&'a Location> {
    let id = match &scene.location {
        Some(own) => own.as_str(),
        None => place?.location.as_deref()?,
    };

    project.location(id)
}

/// The scripted location of each of `scenes`, in order. Each place name of the headings is
/// looked up once: the project lists are built on every playback tick.
pub(crate) fn places_of<'a>(places: &'a [Place], scenes: &[Scene]) -> Vec<Option<&'a Place>> {
    let mut known: HashMap<&str, Option<&'a Place>> = HashMap::new();

    scenes
        .iter()
        .map(|s| {
            if s.kind != Kind::Scene {
                return None;
            }
            let name = heading_place(&s.heading);
            *known
                .entry(name)
                .or_insert_with(|| places.iter().find(|p| p.is_called(name)))
        })
        .collect()
}

/// How many scenes use location `id`: shot there (`shot`, per scene) or with a plan on its
/// set. Such a location cannot go.
pub(crate) fn scenes_using(project: &Project, shot: &[Option<&Location>], id: &str) -> usize {
    project
        .scenes
        .iter()
        .zip(shot)
        .filter(|(s, at)| {
            at.is_some_and(|l| l.id == id) || s.plans.iter().any(|p| p.location == id)
        })
        .count()
}
