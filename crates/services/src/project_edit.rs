//! Changes to the project itself: scenes, locations, characters (LH-01 to LH-03).

use klappe_model::{Character, FigureColor, FloorPlan, Location, Project, Scene, SetPlan};

use crate::error::ServiceError;

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
    /// Sets number and heading; the number is only checked when it changes, so a scene
    /// whose script is linked by its number keeps it while the heading changes.
    EditScene {
        id: String,
        number: String,
        heading: String,
    },
    /// Sets the location of a scene.
    MoveScene {
        id: String,
        location: String,
    },
    RemoveScene {
        id: String,
    },
    AddLocation {
        name: String,
        width_m: f64,
        depth_m: f64,
    },
    /// Renames and resizes; the set's walls and objects stay where they are.
    EditLocation {
        id: String,
        name: String,
        width_m: f64,
        depth_m: f64,
    },
    /// Refused while a scene is set there.
    RemoveLocation {
        id: String,
    },
    AddCharacter {
        name: String,
        short: String,
        color: FigureColor,
    },
    EditCharacter {
        id: String,
        name: String,
        short: String,
        color: FigureColor,
    },
    /// Refused while a plan uses the character.
    RemoveCharacter {
        id: String,
    },
}

/// Largest side of a set: a stadium fits; 1e300 m would not draw.
const MAX_SET_M: f64 = 1000.0;

/// The label in a figure's circle has room for this many letters.
const MAX_SHORT_CHARS: usize = 3;

/// Scene ids pad the number's digits so that file order is scene order (`szene-012`).
const SCENE_ID_DIGITS: usize = 3;

const SCENE_ID_PREFIX: &str = "szene";
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
        } => edit_scene(project, id, number, heading),
        ProjectEdit::MoveScene { id, location } => move_scene(project, id, location),
        ProjectEdit::RemoveScene { id } => remove_scene(project, id),
        ProjectEdit::AddLocation {
            name,
            width_m,
            depth_m,
        } => add_location(project, name, *width_m, *depth_m),
        ProjectEdit::EditLocation {
            id,
            name,
            width_m,
            depth_m,
        } => edit_location(project, id, name, *width_m, *depth_m),
        ProjectEdit::RemoveLocation { id } => remove_location(project, id),
        ProjectEdit::AddCharacter { name, short, color } => {
            add_character(project, name, short, *color)
        }
        ProjectEdit::EditCharacter {
            id,
            name,
            short,
            color,
        } => edit_character(project, id, name, short, *color),
        ProjectEdit::RemoveCharacter { id } => remove_character(project, id),
    }
}

fn add_scene(
    project: &mut Project,
    number: &str,
    heading: &str,
    location: &str,
) -> Result<(), ServiceError> {
    let number = free_number(project, number, None)?;
    check_location(project, location)?;

    let id = unique_id(&scene_base(&number), |id| project.scene(id).is_some());
    project.scenes.push(Scene {
        id,
        number,
        heading: heading.trim().to_owned(),
        location: location.to_owned(),
        plans: vec![FloorPlan::new(FIRST_PLAN_ID, FIRST_PLAN_NAME)],
    });
    project.scenes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(())
}

fn edit_scene(
    project: &mut Project,
    id: &str,
    number: &str,
    heading: &str,
) -> Result<(), ServiceError> {
    let current = project
        .scene(id)
        .ok_or_else(|| ServiceError::NoScene(id.to_owned()))?;
    let number = match number.trim() == current.number {
        true => current.number.clone(),
        false => free_number(project, number, Some(id))?,
    };

    let Some(scene) = project.scenes.iter_mut().find(|s| s.id == id) else {
        return Err(ServiceError::NoScene(id.to_owned()));
    };
    scene.number = number;
    scene.heading = heading.trim().to_owned();
    Ok(())
}

fn move_scene(project: &mut Project, id: &str, location: &str) -> Result<(), ServiceError> {
    check_location(project, location)?;

    let Some(scene) = project.scenes.iter_mut().find(|s| s.id == id) else {
        return Err(ServiceError::NoScene(id.to_owned()));
    };
    scene.location = location.to_owned();
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
    width_m: f64,
    depth_m: f64,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    check_size(width_m, depth_m)?;

    let base = slug(&name, LOCATION_ID_FALLBACK);
    let id = unique_id(&base, |id| project.location(id).is_some());
    project.locations.push(Location {
        id,
        name,
        set: SetPlan {
            width_m,
            depth_m,
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
    width_m: f64,
    depth_m: f64,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    check_size(width_m, depth_m)?;

    let Some(location) = project.locations.iter_mut().find(|l| l.id == id) else {
        return Err(ServiceError::NoLocation(id.to_owned()));
    };
    location.name = name;
    location.set.width_m = width_m;
    location.set.depth_m = depth_m;
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

fn add_character(
    project: &mut Project,
    name: &str,
    short: &str,
    color: FigureColor,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let short = short_of(short)?;

    let base = slug(&name, CHARACTER_ID_FALLBACK);
    let id = unique_id(&base, |id| project.character(id).is_some());
    project.characters.push(Character {
        id,
        name,
        short,
        color,
    });
    Ok(())
}

fn edit_character(
    project: &mut Project,
    id: &str,
    name: &str,
    short: &str,
    color: FigureColor,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let short = short_of(short)?;

    let Some(character) = project.characters.iter_mut().find(|c| c.id == id) else {
        return Err(ServiceError::NoCharacter(id.to_owned()));
    };
    character.name = name;
    character.short = short;
    character.color = color;
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

/// The trimmed number, if it is not empty and no other scene (not `own`) has it. `12a` and
/// `12A` count as one.
fn free_number(project: &Project, number: &str, own: Option<&str>) -> Result<String, ServiceError> {
    let number = number.trim();
    let bad = || ServiceError::BadValue {
        field: "number",
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
            field: "name",
            value: String::new(),
        });
    }
    Ok(name.to_owned())
}

fn short_of(short: &str) -> Result<String, ServiceError> {
    let short = short.trim();
    if short.is_empty() || short.chars().count() > MAX_SHORT_CHARS {
        return Err(ServiceError::BadValue {
            field: "short",
            value: short.to_owned(),
        });
    }
    Ok(short.to_owned())
}

/// Both sides in `(0, MAX_SET_M]`; also excludes NaN.
fn check_size(width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
    let fits = |side: f64| side > 0.0 && side <= MAX_SET_M;

    for (field, side) in [("width", width_m), ("depth", depth_m)] {
        if !fits(side) {
            return Err(ServiceError::BadValue {
                field,
                value: side.to_string(),
            });
        }
    }
    Ok(())
}

/// `szene-012` for `12`, `szene-012a` for `12A`: the digits are padded so that ids sort
/// like numbers.
fn scene_base(number: &str) -> String {
    let digits = number.chars().take_while(char::is_ascii_digit).count();
    let (head, tail) = number.split_at(digits);
    let rest = slug(tail, "");

    let number = match head.is_empty() {
        true => rest,
        false => format!("{head:0>SCENE_ID_DIGITS$}{rest}"),
    };
    format!("{SCENE_ID_PREFIX}-{}", slug(&number, "0"))
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

/// `base`, or `base-2`, `base-3` … until `taken` says no and the id can name a file.
fn unique_id(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let mut id = base.to_owned();
    let mut n = 1;

    while taken(&id) || !klappe_store::is_safe_id(&id) {
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
    fn scene_ids_sort_like_numbers() {
        assert_eq!(scene_base("12"), "szene-012");
        assert_eq!(scene_base("2"), "szene-002");
        assert_eq!(scene_base("12A"), "szene-012a");
        assert_eq!(scene_base("A"), "szene-a");
        assert_eq!(scene_base("1234"), "szene-1234");
    }

    #[test]
    fn a_device_name_gets_a_suffix() {
        assert_eq!(unique_id("con", |_| false), "con-2");
    }
}
