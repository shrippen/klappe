//! Where a scene is shot (DB-51): at its own shooting location when it has one, else at the
//! one of its scripted location (the place its heading names).
//!
//!   heading "INT. WACHRAUM – NACHT" ─► place WACHRAUM ─► place.location ─┐
//!                                       scene.location (if set) ─────────┴─► where it is shot

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
    let id = match &scene.location {
        Some(own) => own.as_str(),
        None => place_of(places, scene)?.location.as_deref()?,
    };

    project.location(id)
}
