//! Changes to the project itself: scenes, locations, characters (LH-01 to LH-03).

use std::collections::HashSet;

use klappe_model::{
    is_figure_size_m, is_object_height_m, normalize_deg, random_id, BookState, Character,
    FigureColor, FloorPlan, Kind, LibraryItem, Location, Numbering, Point, Project, Scene, SetPlan,
    Site, SunTime, Template, MAX_GROUP, MAX_SET_M, MIN_GROUP,
};

use crate::error::ServiceError;
use crate::field::Field;

mod copy;
mod library;
mod plans;
pub(crate) mod templates;
mod views;

pub(crate) use copy::{copy_character, free_short};
pub(crate) use plans::{add_plan, copy_plan, move_plan, remove_plan, rename_plan, seat_speakers};
pub use plans::{PlanShift, ShotsGo};
pub(crate) use views::{remove_view, save_view};

/// What the project view does. Ids are made from the name when something is added and never
/// change after.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectEdit {
    /// A scene with an empty `Plan A`.
    AddScene {
        number: String,
        heading: String,
        location: String,
    },
    /// Sets number, heading and location; the number is only checked when it changes, so a
    /// scene whose script is linked by its number keeps it while the heading changes.
    EditScene {
        id: String,
        number: String,
        heading: String,
        location: String,
        /// When it is shot (LH-52); `None` clears it.
        sun: Option<SunTime>,
    },
    RemoveScene {
        id: String,
    },
    /// `wall_height_m` is the set's room height (LH-90), `site` its place and north (LH-52).
    AddLocation {
        name: String,
        width_m: f64,
        depth_m: f64,
        wall_height_m: f64,
        site: Option<Site>,
    },
    /// Renames and resizes; the set's walls and objects stay where they are.
    EditLocation {
        id: String,
        name: String,
        width_m: f64,
        depth_m: f64,
        wall_height_m: f64,
        /// `None` clears the place.
        site: Option<Site>,
    },
    /// Refused while a scene is set there.
    RemoveLocation {
        id: String,
    },
    /// `group` is the head count of a group of extras (LH-23); `None` is a single person.
    /// `size_m` is the diameter of the body (LH-25); `None` is the standard.
    AddCharacter {
        name: String,
        short: String,
        color: FigureColor,
        group: Option<u32>,
        size_m: Option<f64>,
    },
    EditCharacter {
        id: String,
        name: String,
        short: String,
        color: FigureColor,
        group: Option<u32>,
        size_m: Option<f64>,
    },
    /// Refused while a plan uses the character.
    RemoveCharacter {
        id: String,
    },
    /// Files the object `object` of the set at `location` in the project's library under
    /// `name` (LH-12): its size and shape, a kind of its own.
    LibraryFromObject {
        location: String,
        object: String,
        name: String,
    },
    /// Takes the entries of a library file over (LH-12). An entry the project has already
    /// (same kind and same content) is skipped; one whose kind is taken by something else
    /// gets a kind of its own.
    ImportLibrary {
        items: Vec<LibraryItem>,
    },
    /// Puts the template `template` on the plan `plan` of the scene `scene`, its insertion
    /// point at `at` (LH-07).
    PlaceTemplate {
        scene: String,
        plan: String,
        template: String,
        at: Point,
    },
    /// Files the figures and cameras of the plan as a template of the project (LH-07).
    SaveTemplate {
        scene: String,
        plan: String,
        name: String,
    },
    /// Renames the project's own template `id` (LH-07). A built-in template is not the
    /// project's to change.
    EditTemplate {
        id: String,
        name: String,
    },
    /// Takes the project's own template `id` away (LH-07).
    RemoveTemplate {
        id: String,
    },
    /// Takes the templates of a template file over (LH-07). One the project has already (same
    /// name and content) is skipped; one whose id is taken by something else gets an id of its
    /// own.
    ImportTemplates {
        items: Vec<Template>,
    },
    /// Renames and regroups the project's own entry `kind` (LH-12); no group takes it out of
    /// any. Placed objects keep their kind. A built-in entry is not the project's to change.
    EditLibraryItem {
        kind: String,
        name: String,
        group: String,
    },
    /// Takes the project's own entry `kind` out of the library (LH-12); objects placed from
    /// it stay as they are.
    RemoveLibraryItem {
        kind: String,
    },
    /// Chooses how the project numbers new shots (LH-76). Numbers that exist stay until
    /// "Renumber".
    SetNumbering {
        numbering: Numbering,
    },
}

/// The label in a figure's circle has room for this many letters.
const MAX_SHORT_CHARS: usize = 3;

const LOCATION_ID_FALLBACK: &str = "drehort";
const CHARACTER_ID_FALLBACK: &str = "figur";
const FIRST_PLAN_ID: &str = "plan-a";
const FIRST_PLAN_NAME: &str = "Plan A";

pub(crate) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::AddScene {
            number,
            heading,
            location,
        } => add_scene(project, number, heading, location),
        ProjectEdit::EditScene {
            id,
            number,
            heading,
            location,
            sun,
        } => edit_scene(project, id, number, heading, location, *sun),
        ProjectEdit::RemoveScene { id } => remove_scene(project, id),
        ProjectEdit::SetNumbering { numbering } => {
            project.numbering = *numbering;
            Ok(())
        }
        ProjectEdit::AddLocation {
            name,
            width_m,
            depth_m,
            wall_height_m,
            site,
        } => add_location(project, name, (*width_m, *depth_m), *wall_height_m, *site),
        ProjectEdit::EditLocation {
            id,
            name,
            width_m,
            depth_m,
            wall_height_m,
            site,
        } => edit_location(
            project,
            id,
            name,
            (*width_m, *depth_m),
            *wall_height_m,
            *site,
        ),
        ProjectEdit::RemoveLocation { id } => remove_location(project, id),
        ProjectEdit::AddCharacter {
            name,
            short,
            color,
            group,
            size_m,
        } => add_character(project, name, short, *color, (*group, *size_m)),
        ProjectEdit::EditCharacter {
            id,
            name,
            short,
            color,
            group,
            size_m,
        } => edit_character(project, id, name, short, *color, (*group, *size_m)),
        ProjectEdit::RemoveCharacter { id } => remove_character(project, id),
        ProjectEdit::LibraryFromObject { .. } | ProjectEdit::ImportLibrary { .. } => {
            library::apply(project, edit)
        }
        ProjectEdit::PlaceTemplate { .. }
        | ProjectEdit::SaveTemplate { .. }
        | ProjectEdit::EditTemplate { .. }
        | ProjectEdit::RemoveTemplate { .. }
        | ProjectEdit::ImportTemplates { .. } => templates::apply(project, edit),
        ProjectEdit::EditLibraryItem { .. } | ProjectEdit::RemoveLibraryItem { .. } => {
            library::apply(project, edit)
        }
    }
}

fn add_scene(
    project: &mut Project,
    number: &str,
    heading: &str,
    location: &str,
) -> Result<(), ServiceError> {
    let heading = check_heading(heading)?;
    let number = free_number(project, number, None)?;
    check_location(project, location)?;

    // Appended: the register keeps the order of the project, not of the ids.
    project.scenes.push(Scene {
        id: free_scene_id(project, &[]),
        kind: Kind::Scene,
        number,
        heading,
        state: BookState::InBook,
        prints: Vec::new(),
        refused: Vec::new(),
        location: location.to_owned(),
        sun: None,
        plans: vec![FloorPlan::new(FIRST_PLAN_ID, FIRST_PLAN_NAME)],
    });
    Ok(())
}

fn edit_scene(
    project: &mut Project,
    id: &str,
    number: &str,
    heading: &str,
    location: &str,
    sun: Option<SunTime>,
) -> Result<(), ServiceError> {
    let heading = check_heading(heading)?;
    check_sun_time(sun)?;
    let current = project
        .scene(id)
        .ok_or_else(|| ServiceError::NoScene(id.to_owned()))?;
    let number = match number.trim() == current.number {
        true => current.number.clone(),
        false => free_number(project, number, Some(id))?,
    };
    check_location(project, location)?;

    let Some(scene) = project.scenes.iter_mut().find(|s| s.id == id) else {
        return Err(ServiceError::NoScene(id.to_owned()));
    };
    scene.number = number;
    scene.heading = heading;
    // The ways name objects of the old set; a same-named object of the new one is another.
    if scene.location != location {
        scene.plans.iter_mut().for_each(|p| p.object_tracks.clear());
    }
    scene.location = location.to_owned();
    scene.sun = sun;
    Ok(())
}

fn remove_scene(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    let before = project.scenes.len();
    project.scenes.retain(|s| s.id != id);
    if project.scenes.len() == before {
        return Err(ServiceError::NoScene(id.to_owned()));
    }
    Ok(())
}

fn add_location(
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

fn edit_location(
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

fn remove_location(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    if project.location(id).is_none() {
        return Err(ServiceError::NoLocation(id.to_owned()));
    }

    let scenes = project.scenes.iter().filter(|s| s.location == id).count();
    if scenes > 0 {
        return Err(ServiceError::LocationInUse {
            id: id.to_owned(),
            scenes,
        });
    }

    project.locations.retain(|l| l.id != id);
    Ok(())
}

/// Two characters of one name or alias (compared as cues are) would merge their cues; `own` is
/// the character being renamed, which may keep its name.
fn check_free_name(project: &Project, name: &str, own: Option<&str>) -> Result<(), ServiceError> {
    let taken = project
        .characters
        .iter()
        .filter(|c| Some(c.id.as_str()) != own)
        .any(|c| c.is_called(name));
    if !taken {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Name,
        value: name.to_owned(),
    })
}

fn add_character(
    project: &mut Project,
    name: &str,
    short: &str,
    color: FigureColor,
    (group, size_m): (Option<u32>, Option<f64>),
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let short = short_of(short)?;
    check_group(group)?;
    check_figure_size(size_m)?;
    check_free_name(project, &name, None)?;

    let base = slug(&name, CHARACTER_ID_FALLBACK);
    let id = unique_id(&base, project.characters.iter().map(|c| c.id.as_str()));
    project.characters.push(Character {
        id,
        name,
        short,
        color,
        group,
        size_m,
        aliases: Vec::new(),
    });
    Ok(())
}

fn edit_character(
    project: &mut Project,
    id: &str,
    name: &str,
    short: &str,
    color: FigureColor,
    (group, size_m): (Option<u32>, Option<f64>),
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let short = short_of(short)?;
    check_group(group)?;
    check_figure_size(size_m)?;

    check_free_name(project, &name, Some(id))?;

    let Some(character) = project.characters.iter_mut().find(|c| c.id == id) else {
        return Err(ServiceError::NoCharacter(id.to_owned()));
    };
    character.name = name;
    character.short = short;
    character.color = color;
    character.group = group;
    character.size_m = size_m;
    Ok(())
}

fn remove_character(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    if project.character(id).is_none() {
        return Err(ServiceError::NoCharacter(id.to_owned()));
    }

    let plans = project
        .scenes
        .iter()
        .flat_map(|s| &s.plans)
        .filter(|p| uses(p, id))
        .count();
    if plans > 0 {
        return Err(ServiceError::CharacterInUse {
            id: id.to_owned(),
            plans,
        });
    }

    project.characters.retain(|c| c.id != id);
    Ok(())
}

/// Whether the plan has a track for the character or draws the axis through them.
fn uses(plan: &FloorPlan, character: &str) -> bool {
    let axis = plan
        .axis
        .as_ref()
        .is_some_and(|a| a.a == character || a.b == character);

    axis || plan.figure(character).is_some()
}

/// A heading is one line: the register and the script both hold it so.
fn check_heading(heading: &str) -> Result<String, ServiceError> {
    let heading = heading.trim();
    if !heading.contains(['\n', '\r']) {
        return Ok(heading.to_owned());
    }

    Err(ServiceError::BadValue {
        field: Field::Heading,
        value: heading.to_owned(),
    })
}

/// The trimmed number, if it is not empty and no other scene (not `own`) has it. `12a` and
/// `12A` count as one.
fn free_number(project: &Project, number: &str, own: Option<&str>) -> Result<String, ServiceError> {
    let number = number.trim();
    let bad = || ServiceError::BadValue {
        field: Field::Number,
        value: number.to_owned(),
    };
    if number.is_empty() {
        return Err(bad());
    }

    let taken = project
        .scenes
        .iter()
        .filter(|s| Some(s.id.as_str()) != own)
        .any(|s| s.number.to_lowercase() == number.to_lowercase());
    if taken {
        return Err(ServiceError::NumberTaken(number.to_owned()));
    }

    Ok(number.to_owned())
}

fn check_location(project: &Project, id: &str) -> Result<(), ServiceError> {
    match project.location(id) {
        Some(_) => Ok(()),
        None => Err(ServiceError::NoLocation(id.to_owned())),
    }
}

fn name_of(name: &str) -> Result<String, ServiceError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Name,
            value: String::new(),
        });
    }
    Ok(name.to_owned())
}

fn short_of(short: &str) -> Result<String, ServiceError> {
    let short = short.trim();
    if short.is_empty() || short.chars().count() > MAX_SHORT_CHARS {
        return Err(ServiceError::BadValue {
            field: Field::Short,
            value: short.to_owned(),
        });
    }
    Ok(short.to_owned())
}

fn check_group(group: Option<u32>) -> Result<(), ServiceError> {
    let Some(count) = group else {
        return Ok(());
    };
    if !(MIN_GROUP..=MAX_GROUP).contains(&count) {
        return Err(ServiceError::BadValue {
            field: Field::Group,
            value: count.to_string(),
        });
    }
    Ok(())
}

fn check_figure_size(size_m: Option<f64>) -> Result<(), ServiceError> {
    let Some(size_m) = size_m else {
        return Ok(());
    };
    if !is_figure_size_m(size_m) {
        return Err(ServiceError::BadValue {
            field: Field::Size,
            value: size_m.to_string(),
        });
    }
    Ok(())
}

/// Both sides in `(0, MAX_SET_M]`; also excludes NaN.
fn check_size(width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
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
fn check_room_height(wall_height_m: f64) -> Result<(), ServiceError> {
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
fn check_site(site: Option<Site>) -> Result<Option<Site>, ServiceError> {
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

/// The date exists and the time is on a clock.
fn check_sun_time(sun: Option<SunTime>) -> Result<(), ServiceError> {
    let Some(time) = sun else {
        return Ok(());
    };
    if time.is_valid() {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Date,
        value: format!("{} {}", time.date_text(), time.time_text()),
    })
}

/// A random id that no entry has, kept or yet to be placed (ids are file names, which may
/// not differ by case only).
pub(crate) fn free_scene_id(project: &Project, placed: &[(usize, Scene)]) -> String {
    loop {
        let id = random_id();
        let taken = project
            .scenes
            .iter()
            .chain(placed.iter().map(|(_, s)| s))
            .any(|s| s.id.eq_ignore_ascii_case(&id));
        if !taken {
            return id;
        }
    }
}

/// Lower-case ASCII with `-` between words (`Sängersaal` → `saengersaal`); `fallback` when
/// nothing is left.
fn slug(name: &str, fallback: &str) -> String {
    let mut out = String::new();

    for c in name.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' => out.push_str("ae"),
            'ö' => out.push_str("oe"),
            'ü' => out.push_str("ue"),
            'ß' => out.push_str("ss"),
            c if c.is_ascii_alphanumeric() => out.push(c),
            _ if !out.ends_with('-') => out.push('-'),
            _ => {}
        }
    }

    let out = out.trim_matches('-');
    match out.is_empty() {
        true => fallback.to_owned(),
        false => out.to_owned(),
    }
}

/// `base`, or `base-2`, `base-3` … until no id of `existing` is the same and the id can name
/// a file. `Hof` and `hof` are the same: the store refuses both in one project.
fn unique_id<'a>(base: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let taken: HashSet<String> = existing.map(str::to_lowercase).collect();
    let mut id = base.to_owned();
    let mut n = 1;

    while taken.contains(&id.to_lowercase()) || !klappe_store::is_safe_id(&id) {
        n += 1;
        id = format!("{base}-{n}");
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_ascii_words() {
        assert_eq!(slug("Sängersaal", "x"), "saengersaal");
        assert_eq!(slug("  Burg – Hof! ", "x"), "burg-hof");
        assert_eq!(slug("§§", "x"), "x");
    }

    #[test]
    fn a_device_name_gets_a_suffix() {
        assert_eq!(unique_id("con", [].into_iter()), "con-2");
    }
}
