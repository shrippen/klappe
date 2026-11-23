//! Edits of the project's shooting locations: add, change, remove.

use klappe_model::{
    is_object_height_m, normalize_deg, Location, Project, SetPlan, Site, MAX_SET_M,
};

use crate::error::ServiceError;
use crate::field::Field;

use super::{name_of, slug, unique_id};

const LOCATION_ID_FALLBACK: &str = "drehort";

pub(super) fn add_location(
    project: &mut Project,
    name: &str,
    (width_m, depth_m): (f64, f64),
    wall_height_m: f64,
    site: Option<Site>,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    check_size(width_m, depth_m)?;
    check_room_height(wall_height_m)?;
    let site = check_site(site)?;

    let base = slug(&name, LOCATION_ID_FALLBACK);
    let id = unique_id(&base, project.locations.iter().map(|l| l.id.as_str()));
    project.locations.push(Location {
        id,
        name,
        site,
        set: SetPlan {
            width_m,
            depth_m,
            wall_height_m,
            ..SetPlan::default()
        },
    });
    project.locations.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}

pub(super) fn edit_location(
    project: &mut Project,
    id: &str,
    name: &str,
    (width_m, depth_m): (f64, f64),
    wall_height_m: f64,
    site: Option<Site>,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    check_size(width_m, depth_m)?;
    check_room_height(wall_height_m)?;
    let site = check_site(site)?;

    let Some(location) = project.locations.iter_mut().find(|l| l.id == id) else {
        return Err(ServiceError::NoLocation(id.to_owned()));
    };
    location.name = name;
    location.set.width_m = width_m;
    location.set.depth_m = depth_m;
    location.set.wall_height_m = wall_height_m;
    location.site = site;
    Ok(())
}

pub(super) fn remove_location(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    if project.location(id).is_none() {
        return Err(ServiceError::NoLocation(id.to_owned()));
    }

    let scenes = project
        .scenes
        .iter()
        .filter(|s| s.location.as_deref() == Some(id) || s.plans.iter().any(|p| p.location == id))
        .count();
    if scenes > 0 {
        return Err(ServiceError::LocationInUse {
            id: id.to_owned(),
            scenes,
        });
    }

    project.locations.retain(|l| l.id != id);
    Ok(())
}

/// Both sides in `(0, MAX_SET_M]`; also excludes NaN.
pub(super) fn check_size(width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
    let fits = |side: f64| side > 0.0 && side <= MAX_SET_M;

    for (field, side) in [(Field::Width, width_m), (Field::Depth, depth_m)] {
        if !fits(side) {
            return Err(ServiceError::BadValue {
                field,
                value: side.to_string(),
            });
        }
    }
    Ok(())
}

/// The room height: a number, not under the least height, not absurd.
pub(super) fn check_room_height(wall_height_m: f64) -> Result<(), ServiceError> {
    if is_object_height_m(wall_height_m) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::RoomHeight,
        value: wall_height_m.to_string(),
    })
}

/// Folding into the circle adds float residue (370.1 gives 10.100000000000023); the form shows
/// the stored value, so it is rounded to 1e-9°, far below what a plan shows (a power of ten
/// as a divisor, so the result is the nearest double of the decimal).
const NORTH_STEPS_PER_DEG: f64 = 1e9;

fn fold_north(deg: f64) -> f64 {
    let folded = (normalize_deg(deg) * NORTH_STEPS_PER_DEG).round() / NORTH_STEPS_PER_DEG;

    // Rounding up to 360 is the top again.
    normalize_deg(folded)
}

/// The place is on the globe; north is turned into `[0, 360)`.
pub(super) fn check_site(site: Option<Site>) -> Result<Option<Site>, ServiceError> {
    let Some(site) = site else {
        return Ok(None);
    };
    if site.is_valid() {
        return Ok(Some(Site {
            north_deg: fold_north(site.north_deg),
            ..site
        }));
    }

    Err(ServiceError::BadValue {
        field: Field::Latitude,
        value: format!("{}, {}", site.latitude_deg, site.longitude_deg),
    })
}
