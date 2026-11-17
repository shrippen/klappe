use std::collections::{HashMap, HashSet};

use klappe_model::{Angle, FloorPlan, LinkTarget, Movement, Numbering, Shot, ShotSize, VersionOf};
use klappe_stellprobe::is_setup_model;

use crate::error::ServiceError;
use crate::field::Field;

/// Length of a new shot when no mark follows the playhead soon.
const DEFAULT_SHOT_S: f64 = 5.0;

/// Letters of the alphabet a version suffix counts in.
const LETTERS: u32 = 26;

/// Sets a version off from the camera letter in the camera and group schemes.
const VERSION_SEPARATOR: char = '-';

/// A new shot ends at the next mark only if that is at least this far away.
const MIN_SHOT_S: f64 = 0.5;

/// The stretch a new shot at `at_s` covers: up to the next mark of any figure or camera,
/// else `DEFAULT_SHOT_S`. Example: marks at 6 s and 9 s, playhead at 6 s: 6 to 9 s.
fn span_at(plan: &FloorPlan, at_s: f64) -> (f64, f64) {
    let figures = plan
        .figures
        .iter()
        .flat_map(|f| f.marks.iter().map(|m| m.at_s));
    let cameras = plan
        .cameras
        .iter()
        .flat_map(|c| c.marks.iter().map(|m| m.at_s));
    let next_s = figures
        .chain(cameras)
        .filter(|&mark_s| mark_s >= at_s + MIN_SHOT_S)
        .fold(f64::INFINITY, f64::min);

    if next_s.is_finite() {
        return (at_s, next_s);
    }
    (at_s, at_s + DEFAULT_SHOT_S)
}

/// What a shot of `camera` is called in a number: its label, else its id.
fn camera_name(plan: &FloorPlan, camera: &str) -> String {
    plan.camera(camera)
        .map_or_else(|| camera.to_owned(), |c| c.label.clone())
}

/// The base shot `shot` is a version of, if it names one that is a base itself.
fn base_of<'a>(plan: &'a FloorPlan, shot: &Shot) -> Option<&'a Shot> {
    let of = shot.version_of.as_ref()?;

    plan.shots
        .iter()
        .find(|s| s.id == of.shot && s.version_of.is_none())
}

/// A shot that is no version, or whose base is gone, counts as a base.
fn is_base(plan: &FloorPlan, shot: &Shot) -> bool {
    base_of(plan, shot).is_none()
}

/// Whether the stretches of two shots share time.
fn overlap(a: &Shot, from_s: f64, to_s: f64) -> bool {
    a.from_s < to_s && from_s < a.to_s
}

fn taken(plan: &FloorPlan, number: &str) -> bool {
    plan.shots.iter().any(|s| s.number == number)
}

/// The group of a base shot numbered by the group scheme: its number without the camera
/// label, e.g. `12.3` of `12.3B`. `None` for a number that does not end in the label.
fn group_key<'a>(plan: &FloorPlan, shot: &'a Shot) -> Option<&'a str> {
    shot.number
        .strip_suffix(camera_name(plan, &shot.camera).as_str())
}

/// The number for a new base shot of `camera` over `from_s` to `to_s` (LH-76), by the scheme,
/// made from the shots' data and never from reading digits or letters out of other numbers.
/// The count of base shots (of the camera, or of the groups) gives the running number, and
/// goes on while that number is taken. Group: the group of a shot that overlaps in time and
/// has no shot of this camera, else a new group.
fn next_number(
    plan: &FloorPlan,
    numbering: Numbering,
    scene: &str,
    camera: &str,
    (from_s, to_s): (f64, f64),
) -> String {
    let label = camera_name(plan, camera);
    let bases: Vec<&Shot> = plan.shots.iter().filter(|s| is_base(plan, s)).collect();

    match numbering {
        Numbering::Scene => {
            let first = bases.len() + 1;

            (first..)
                .map(|n| format!("{scene}.{n}"))
                .find(|number| !taken(plan, number))
                .unwrap_or_default()
        }
        Numbering::Camera => {
            let own = bases.iter().filter(|s| s.camera == camera).count();

            (own + 1..)
                .map(|n| format!("{scene}{label}.{n}"))
                .find(|number| !taken(plan, number))
                .unwrap_or_default()
        }
        Numbering::Group => {
            let groups: HashSet<&str> = bases.iter().filter_map(|s| group_key(plan, s)).collect();
            let joined = bases
                .iter()
                .filter(|s| overlap(s, from_s, to_s))
                .filter_map(|s| group_key(plan, s))
                .find(|key| {
                    !bases.iter().any(|s| {
                        group_key(plan, s) == Some(*key) && camera_name(plan, &s.camera) == label
                    })
                });

            match joined {
                Some(key) => format!("{key}{label}"),
                None => (groups.len() + 1..)
                    .map(|n| format!("{scene}.{n}"))
                    .find(|key| {
                        !groups.contains(key.as_str()) && !taken(plan, &format!("{key}{label}"))
                    })
                    .map(|key| format!("{key}{label}"))
                    .unwrap_or_default(),
            }
        }
    }
}

/// The id for a new shot. It is stable and independent of the number: a taken one moves on.
fn next_id(plan: &FloorPlan, scene: &str) -> String {
    let mut n = plan.shots.len() + 1;
    while plan.shots.iter().any(|s| s.id == format!("s{scene}-{n}")) {
        n += 1;
    }

    format!("s{scene}-{n}")
}

/// Puts `shot` in cut order: after the last shot that starts no later, else first.
fn insert_by_start(plan: &mut FloorPlan, shot: Shot) {
    let at = plan
        .shots
        .iter()
        .rposition(|s| s.from_s <= shot.from_s)
        .map_or(0, |i| i + 1);
    plan.shots.insert(at, shot);
}

/// Adds a shot of `camera` at `at_s`; returns its id.
pub(crate) fn add(
    plan: &mut FloorPlan,
    numbering: Numbering,
    scene: &str,
    camera: &str,
    at_s: f64,
) -> String {
    let id = next_id(plan, scene);
    let (from_s, to_s) = span_at(plan, at_s);
    let number = next_number(plan, numbering, scene, camera, (from_s, to_s));

    insert_by_start(
        plan,
        Shot {
            id: id.clone(),
            number,
            camera: camera.to_owned(),
            short_name: String::new(),
            size: ShotSize::Medium,
            angle: Angle::EyeLevel,
            movement: Movement::Static,
            lens_mm: None,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
            setup_of: None,
            image: None,
            version_of: None,
        },
    );
    id
}

/// Adds a copy of shot `id` as a base shot with a new number, right behind it; returns the
/// copy's id. Several shots of one setup start as copies (LH-70).
pub(crate) fn copy(
    plan: &mut FloorPlan,
    numbering: Numbering,
    scene: &str,
    id: &str,
) -> Result<String, ServiceError> {
    let at = plan
        .shots
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
    let original = &plan.shots[at];
    let span = (original.from_s, original.to_s);
    let number = next_number(plan, numbering, scene, &original.camera, span);
    let new_id = next_id(plan, scene);

    let mut shot = original.clone();
    shot.id = new_id.clone();
    shot.number = number;
    shot.version_of = None;
    plan.shots.insert(at + 1, shot);
    Ok(new_id)
}

/// Version letters for the n-th version (1-based): A to Z, then AA, AB, ...
fn letters(mut n: u32) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % LETTERS) as u8);
        n /= LETTERS;
    }
    out.reverse();
    String::from_utf8_lossy(&out).into_owned()
}

/// What a version adds to the number of its base: letters in the scene scheme (`12.3A`), a
/// hyphen and letters where letters name cameras (`12A.3-B`).
fn suffix(numbering: Numbering, index: u32) -> String {
    match numbering {
        Numbering::Scene => letters(index),
        Numbering::Camera | Numbering::Group => format!("{VERSION_SEPARATOR}{}", letters(index)),
    }
}

/// Adds a copy of shot `id` as the next version of its base, behind the last version: a copy of
/// `12.3` is `12.3A`, then `12.3B`; a copy of `12.3A` is `12.3B` (`12A.3-A` in the camera and
/// group schemes). The version refers to its base and keeps its camera. Returns the copy's id.
pub(crate) fn version(
    plan: &mut FloorPlan,
    numbering: Numbering,
    scene: &str,
    id: &str,
) -> Result<String, ServiceError> {
    let original = plan
        .shots
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
    let base = base_of(plan, original).unwrap_or(original);
    let (base_id, base_number) = (base.id.clone(), base.number.clone());

    let of_base =
        |s: &Shot| s.id == base_id || s.version_of.as_ref().is_some_and(|v| v.shot == base_id);
    let index = plan
        .shots
        .iter()
        .filter_map(|s| s.version_of.as_ref())
        .filter(|v| v.shot == base_id)
        .map(|v| v.index)
        .max()
        .unwrap_or(0)
        + 1;
    let at = plan.shots.iter().rposition(of_base).map_or(0, |i| i + 1);

    let mut shot = original.clone();
    shot.id = next_id(plan, scene);
    shot.number = format!("{base_number}{}", suffix(numbering, index));
    shot.version_of = Some(VersionOf {
        shot: base_id,
        index,
    });
    let new_id = shot.id.clone();
    plan.shots.insert(at, shot);
    Ok(new_id)
}

/// Numbers the shots again in cut order from their data and the scheme (LH-76), never from
/// their old numbers: base shots first, then each version as its base plus its suffix. Scene:
/// `12.1`, `12.2` ... Camera: each camera counts its own, `12A.1`, `12B.1`, `12A.2`. Group:
/// base shots of different cameras that overlap in time share a number, `12.1A`, `12.1B`,
/// `12.2A`. A version of a base that is gone counts as a base.
pub(crate) fn renumber(plan: &mut FloorPlan, numbering: Numbering, scene: &str) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<RunGroup> = Vec::new();
    let mut base_numbers: HashMap<String, String> = HashMap::new();
    let mut count = 0;

    for at in 0..plan.shots.len() {
        let shot = &plan.shots[at];
        if !is_base(plan, shot) {
            continue;
        }

        count += 1;
        let label = camera_name(plan, &shot.camera);
        let number = match numbering {
            Numbering::Scene => format!("{scene}.{count}"),
            Numbering::Camera => {
                let n = counts.entry(label.clone()).or_insert(0);
                *n += 1;
                format!("{scene}{label}.{n}")
            }
            Numbering::Group => {
                let group = join_group(&mut groups, shot, &label);
                format!("{scene}.{}{label}", group + 1)
            }
        };
        base_numbers.insert(shot.id.clone(), number);
    }

    let versions: Vec<(usize, String)> = plan
        .shots
        .iter()
        .enumerate()
        .filter_map(|(at, shot)| {
            let of = shot.version_of.as_ref()?;
            let base = base_numbers
                .get(&of.shot)
                .filter(|_| !is_base(plan, shot))?;

            Some((at, format!("{base}{}", suffix(numbering, of.index))))
        })
        .collect();

    for shot in &mut plan.shots {
        if let Some(number) = base_numbers.remove(&shot.id) {
            shot.number = number;
        }
    }
    for (at, number) in versions {
        plan.shots[at].number = number;
    }
}

/// A group of shots taken at the same time, as `renumber` builds it: the stretch it covers
/// and the cameras in it.
struct RunGroup {
    from_s: f64,
    to_s: f64,
    cameras: Vec<String>,
}

/// Puts `shot` into the last group it overlaps and whose cameras lack its own, else into a
/// new one; returns the group's index.
fn join_group(groups: &mut Vec<RunGroup>, shot: &Shot, label: &str) -> usize {
    let joined = groups.iter().rposition(|g| {
        shot.from_s < g.to_s && g.from_s < shot.to_s && !g.cameras.iter().any(|c| c == label)
    });
    let Some(at) = joined else {
        groups.push(RunGroup {
            from_s: shot.from_s,
            to_s: shot.to_s,
            cameras: vec![label.to_owned()],
        });
        return groups.len() - 1;
    };

    let group = &mut groups[at];
    group.from_s = group.from_s.min(shot.from_s);
    group.to_s = group.to_s.max(shot.to_s);
    group.cameras.push(label.to_owned());
    at
}

/// Removes shot `id` and the script links to it; returns the id of the shot that takes
/// its place in the list (the next, else the previous), if any.
pub(crate) fn remove(plan: &mut FloorPlan, id: &str) -> Result<Option<String>, ServiceError> {
    let at = plan
        .shots
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

    plan.shots.remove(at);
    promote_version(plan, id);
    plan.links
        .retain(|l| !matches!(&l.target, LinkTarget::Shot { id: linked } if linked == id));

    let neighbour = plan
        .shots
        .get(at)
        .or(at.checked_sub(1).and_then(|i| plan.shots.get(i)));
    Ok(neighbour.map(|s| s.id.clone()))
}

/// The first version of the deleted base `id` becomes the base of the others, so no version
/// hangs on a shot that is gone; the versions keep their order, counted from 1 again.
fn promote_version(plan: &mut FloorPlan, id: &str) {
    let versions: Vec<usize> = (0..plan.shots.len())
        .filter(|&at| {
            plan.shots[at]
                .version_of
                .as_ref()
                .is_some_and(|v| v.shot == id)
        })
        .collect();
    let Some((&first, rest)) = versions.split_first() else {
        return;
    };

    plan.shots[first].version_of = None;
    let base = plan.shots[first].id.clone();
    for (rank, &at) in rest.iter().enumerate() {
        plan.shots[at].version_of = Some(VersionOf {
            shot: base.clone(),
            index: rank as u32 + 1,
        });
    }
}

/// A version has the camera of its base: its camera is not edited alone.
pub(crate) fn check_camera(plan: &FloorPlan, id: &str) -> Result<(), ServiceError> {
    let is_version = plan
        .shots
        .iter()
        .find(|s| s.id == id)
        .is_some_and(|s| !is_base(plan, s));
    if !is_version {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Camera,
        value: id.to_owned(),
    })
}

/// Gives the versions of shot `id` the camera of `id`, after its camera changed.
pub(crate) fn follow_camera(plan: &mut FloorPlan, id: &str) {
    let Some(camera) = plan
        .shots
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.camera.clone())
    else {
        return;
    };

    for shot in &mut plan.shots {
        if shot.version_of.as_ref().is_some_and(|v| v.shot == id) {
            shot.camera = camera.clone();
        }
    }
}

/// Refuses a setup target for shot `id` that is no model (`is_setup_model`). Empty
/// (computed) is fine.
pub(crate) fn check_setup(plan: &FloorPlan, id: &str, target: &str) -> Result<(), ServiceError> {
    let target = target.trim();
    if target.is_empty() || is_setup_model(plan, id, target) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Setup,
        value: target.to_owned(),
    })
}
