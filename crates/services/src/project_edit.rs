//! Changes to the project itself: scenes, locations, characters (LH-01 to LH-03).

use std::collections::HashSet;

use klappe_model::{
    free_random_id, is_figure_size_m, same_character_name, BookState, Character, FigureColor,
    FloorPlan, IdCase, Kind, Numbering, Project, Scene, Site, SunTime, MAX_GROUP, MIN_GROUP,
};

use crate::error::ServiceError;
use crate::field::Field;

mod breakdown;
mod copy;
mod library;
mod locations;
mod names;
mod plans;
mod team;
pub(crate) mod templates;
mod views;

pub use breakdown::{BreakdownEdit, MarkTarget};
pub(crate) use copy::{copy_character, free_short};
pub use library::LibraryEdit;
use locations::{add_location, edit_location, remove_location};
pub use names::NameEdit;
pub(crate) use plans::{add_plan, copy_plan, move_plan, remove_plan, rename_plan, seat_speakers};
pub use plans::{PlanShift, ShotsGo};
pub(crate) use team::store_numbers;
pub use team::TeamEdit;
pub use templates::TemplateEdit;
pub(crate) use views::{remove_view, save_view};

/// What the project view does. Ids are made from the name when something is added and never
/// change after.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectEdit {
    /// A scene without a plan; the session gives it an empty `Plan A` on the set it is shot
    /// at. `location` is its own shooting location (DB-51); `None` follows its scripted
    /// location.
    AddScene {
        number: String,
        heading: String,
        location: Option<String>,
    },
    /// Sets number, heading and own shooting location; the number is only checked when it
    /// changes, so a scene whose script is linked by its number keeps it while the heading
    /// changes. Its plans stay on the sets they are drawn on.
    EditScene {
        id: String,
        number: String,
        heading: String,
        location: Option<String>,
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
        /// The other names the book calls the character by (DB-50); `None` keeps them.
        aliases: Option<Vec<String>>,
    },
    /// Refused while a plan uses the character.
    RemoveCharacter {
        id: String,
    },
    /// Chooses how the project numbers new shots (LH-76). Numbers that exist stay until
    /// "Renumber".
    SetNumbering {
        numbering: Numbering,
    },
    /// The other names of characters and the cues that are no character (DB-50).
    Names(NameEdit),
    /// Edits of the project's library (LH-12).
    Library(LibraryEdit),
    /// Edits of the project's templates (LH-07).
    Template(TemplateEdit),
    /// Edits of the breakdown: elements and scene sheets (DB-60, D-22).
    Breakdown(BreakdownEdit),
    /// Edits of the persons, the contacts and the casting (Besetzung/Team).
    Team(TeamEdit),
}

/// The label in a figure's circle has room for this many letters.
const MAX_SHORT_CHARS: usize = 3;

const CHARACTER_ID_FALLBACK: &str = "figur";
const FIRST_PLAN_ID: &str = "plan-a";
const FIRST_PLAN_NAME: &str = "Plan A";

/// The empty `Plan A` a new scene starts with, on the set of `location`.
pub(crate) fn first_plan(location: &str) -> FloorPlan {
    let mut plan = FloorPlan::new(FIRST_PLAN_ID, FIRST_PLAN_NAME);
    plan.location = location.to_owned();

    plan
}

pub(crate) fn apply(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    match edit {
        ProjectEdit::AddScene {
            number,
            heading,
            location,
        } => add_scene(project, number, heading, location.as_deref()),
        ProjectEdit::EditScene {
            id,
            number,
            heading,
            location,
            sun,
        } => edit_scene(project, id, number, heading, location.as_deref(), *sun),
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
        ProjectEdit::EditCharacter { .. } => edit_figure(project, edit),
        ProjectEdit::RemoveCharacter { id } => remove_character(project, id),
        ProjectEdit::Names(edit) => names::apply(project, edit),
        ProjectEdit::Library(edit) => library::apply(project, edit),
        ProjectEdit::Template(edit) => templates::apply(project, edit),
        ProjectEdit::Breakdown(edit) => breakdown::apply(project, edit),
        ProjectEdit::Team(edit) => team::apply(project, edit),
    }
}

fn add_scene(
    project: &mut Project,
    number: &str,
    heading: &str,
    location: Option<&str>,
) -> Result<(), ServiceError> {
    let heading = check_heading(heading)?;
    let number = free_number(project, number, None)?;
    check_own_location(project, location)?;

    // Appended: the register keeps the order of the project, not of the ids.
    project.scenes.push(Scene {
        id: free_scene_id(project, &[]),
        kind: Kind::Scene,
        number,
        heading,
        state: BookState::InBook,
        prints: Vec::new(),
        refused: Vec::new(),
        location: location.map(str::to_owned),
        sun: None,
        plans: Vec::new(),
        sheet: Default::default(),
    });
    Ok(())
}

fn edit_scene(
    project: &mut Project,
    id: &str,
    number: &str,
    heading: &str,
    location: Option<&str>,
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
    check_own_location(project, location)?;

    let Some(scene) = project.scenes.iter_mut().find(|s| s.id == id) else {
        return Err(ServiceError::NoScene(id.to_owned()));
    };
    scene.number = number;
    scene.heading = heading;
    scene.location = location.map(str::to_owned);
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

/// Whether the name of a new character may be `base` itself, or always carries a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Numbered {
    /// `base` if it is free, else `base 2`, `base 3` … (a template's placeholder).
    WhenTaken,
    /// `base 2`, `base 3` … (a copy).
    Always,
}

/// The first name of `base`, `base 2`, `base 3` … (as `numbered` allows) no character answers to
/// and not in `marked`, the names marked no character (DB-50).
fn free_character_name(
    characters: &[Character],
    marked: &[String],
    base: &str,
    numbered: Numbered,
) -> String {
    let taken = |name: &str| {
        characters.iter().any(|c| c.is_called(name))
            || marked.iter().any(|m| same_character_name(m, name))
    };
    if numbered == Numbered::WhenTaken && !taken(base) {
        return base.to_owned();
    }

    (2..)
        .map(|n| format!("{base} {n}"))
        .find(|name| !taken(name))
        .unwrap_or_default()
}

/// Two characters of one name or alias (compared as cues are) would merge their cues; `own` is
/// the character being renamed, which may keep its name.
/// Refused: a name another character answers to, or one marked as no character (DB-50).
fn check_free_name(project: &Project, name: &str, own: Option<&str>) -> Result<(), ServiceError> {
    if !names::is_taken(project, name, own) {
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

/// `EditCharacter`: the character, then its other names (DB-50) when the edit gives them.
fn edit_figure(project: &mut Project, edit: &ProjectEdit) -> Result<(), ServiceError> {
    let ProjectEdit::EditCharacter {
        id,
        name,
        short,
        color,
        group,
        size_m,
        aliases,
    } = edit
    else {
        return Ok(());
    };

    edit_character(project, id, name, short, *color, (*group, *size_m))?;
    match aliases {
        Some(aliases) => names::set_aliases(project, id, aliases),
        None => Ok(()),
    }
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
    // Ids come from the name: a later character of that name must not inherit the scenes.
    for scene in &mut project.scenes {
        scene.sheet.characters.retain(|c| c != id);
    }
    project.team.castings.retain(|c| c.character != id);
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

/// A scene's own shooting location, if it has one, must exist.
fn check_own_location(project: &Project, id: Option<&str>) -> Result<(), ServiceError> {
    match id {
        Some(id) => check_location(project, id),
        None => Ok(()),
    }
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
    let taken = project
        .scenes
        .iter()
        .chain(placed.iter().map(|(_, s)| s))
        .map(|s| s.id.as_str());

    free_random_id(taken, IdCase::Ignored)
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
