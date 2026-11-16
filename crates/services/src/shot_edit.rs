use std::collections::HashMap;

use klappe_model::{Angle, FloorPlan, LinkTarget, Movement, Numbering, Shot, ShotSize};
use klappe_stellprobe::is_setup_model;

use crate::error::ServiceError;
use crate::field::Field;

/// Length of a new shot when no mark follows the playhead soon.
const DEFAULT_SHOT_S: f64 = 5.0;

/// Letters of the alphabet a version suffix counts in.
const LETTERS: usize = 26;

/// Longest version suffix (`ZZZ`, the 18278th version); longer letters are part of the
/// number, so counting them cannot overflow.
const MAX_VERSION_LETTERS: usize = 3;

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

/// The highest running number among the numbers that start with `prefix`, 0 for none: the digits
/// right behind it count, so `12.4A` and `12A.4-B` count as 4.
fn highest_after(plan: &FloorPlan, prefix: &str) -> u32 {
    plan.shots
        .iter()
        .filter_map(|s| s.number.strip_prefix(prefix))
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0)
}

/// What a shot of `camera` is called in a number: its label, else its id.
fn camera_name(plan: &FloorPlan, camera: &str) -> String {
    plan.camera(camera)
        .map_or_else(|| camera.to_owned(), |c| c.label.clone())
}

/// The group part of a group number: `12.3` of `12.3B` and `12.3B-A`.
fn group_of<'a>(number: &'a str, scene: &str) -> Option<&'a str> {
    let rest = number.strip_prefix(scene)?.strip_prefix('.')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();

    (digits > 0).then(|| &number[..scene.len() + 1 + digits])
}

/// Whether the stretches of two shots share time.
fn overlap(a: &Shot, from_s: f64, to_s: f64) -> bool {
    a.from_s < to_s && from_s < a.to_s
}

/// The number for a new shot of `camera` over `from_s` to `to_s` (LH-76), by the scheme.
/// Scene: one more than the highest running number so far, so gaps stay. Camera: one more
/// than the highest of that camera. Group: the number of a group the shot overlaps in time
/// and whose cameras do not include this one, else a new group.
fn next_number(
    plan: &FloorPlan,
    numbering: Numbering,
    scene: &str,
    camera: &str,
    (from_s, to_s): (f64, f64),
) -> String {
    let label = camera_name(plan, camera);

    match numbering {
        Numbering::Scene => format!("{scene}.{}", highest_after(plan, &format!("{scene}.")) + 1),
        Numbering::Camera => {
            let prefix = format!("{scene}{label}.");

            format!("{prefix}{}", highest_after(plan, &prefix) + 1)
        }
        Numbering::Group => {
            let joined = plan
                .shots
                .iter()
                .filter(|s| overlap(s, from_s, to_s))
                .filter_map(|s| group_of(&s.number, scene))
                .find(|group| {
                    let taken = plan.shots.iter().any(|s| {
                        group_of(&s.number, scene) == Some(group)
                            && camera_name(plan, &s.camera) == label
                    });
                    !taken
                });
            let group = joined.map_or_else(
                || format!("{scene}.{}", highest_after(plan, &format!("{scene}.")) + 1),
                str::to_owned,
            );

            format!("{group}{label}")
        }
    }
}

/// The id for a new shot of the scene numbered `scene`. It is stable and independent of the
/// number: a taken one (a shot renumbered since) moves on.
fn next_id(plan: &FloorPlan, scene: &str) -> String {
    let mut n = highest_after(plan, &format!("{scene}.")) + 1;
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
        },
    );
    id
}

/// Adds a copy of shot `id` with a new number, right behind it; returns the copy's id.
/// Several shots of one setup start as copies (LH-70).
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
    plan.shots.insert(at + 1, shot);
    Ok(new_id)
}

/// Splits a shot number into its base and version: in the scene scheme `12.3B` is (`12.3`,
/// `B`), letters being a version only after a digit and at most `MAX_VERSION_LETTERS` long;
/// `INSERT` is a base of its own (numbers are free text). The other schemes use letters for
/// cameras, so their version is set off by a hyphen: `12A.3-B` is (`12A.3`, `-B`).
fn split_number(numbering: Numbering, number: &str) -> (&str, &str) {
    if numbering != Numbering::Scene {
        let Some((base, letters)) = number.rsplit_once(VERSION_SEPARATOR) else {
            return (number, "");
        };
        let valid = (1..=MAX_VERSION_LETTERS).contains(&letters.len())
            && letters.bytes().all(|b| b.is_ascii_uppercase());

        return if valid {
            (base, &number[base.len()..])
        } else {
            (number, "")
        };
    }

    let base = number.trim_end_matches(|c: char| c.is_ascii_uppercase());
    let letters = number.len() - base.len();
    let after_digit = base.ends_with(|c: char| c.is_ascii_digit());
    if !after_digit || letters > MAX_VERSION_LETTERS {
        return (number, "");
    }

    (base, &number[base.len()..])
}

/// Version letters for the n-th version (1-based): A to Z, then AA, AB, ...
fn letters(mut n: usize) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % LETTERS) as u8);
        n /= LETTERS;
    }
    out.reverse();
    String::from_utf8_lossy(&out).into_owned()
}

/// The n of version letters (`A` is 1, `AA` is 27); 0 for none.
fn version_index(letters: &str) -> usize {
    letters
        .trim_start_matches(VERSION_SEPARATOR)
        .bytes()
        .fold(0, |n, b| n * LETTERS + usize::from(b - b'A') + 1)
}

/// Adds a copy of shot `id` as the next version of its number, behind the last version
/// of that number: a copy of `12.3` is `12.3A`, then `12.3B`; a copy of `12.3A` is `12.3B`
/// (`12A.3-A` in the camera and group schemes). A number that does not end in a digit
/// (`INSERT`) has no versions in the scene scheme. Returns the copy's id.
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
    let base = split_number(numbering, &original.number).0.to_owned();

    let in_base = |s: &Shot| split_number(numbering, &s.number).0 == base;
    let highest = plan
        .shots
        .iter()
        .filter(|s| in_base(s))
        .map(|s| version_index(split_number(numbering, &s.number).1))
        .max()
        .unwrap_or(0);
    let at = plan.shots.iter().rposition(in_base).map_or(0, |i| i + 1);

    // Letters that would not count as a version again (`INSERTA`, `12.3AAAA`) are refused:
    // the next version would repeat the number.
    let suffix = letters(highest + 1);
    let digit_base = base.ends_with(|c: char| c.is_ascii_digit());
    let counts =
        (numbering != Numbering::Scene || digit_base) && suffix.len() <= MAX_VERSION_LETTERS;
    if !counts {
        return Err(ServiceError::NoVersion(original.number.clone()));
    }

    let separator = match numbering {
        Numbering::Scene => String::new(),
        Numbering::Camera | Numbering::Group => VERSION_SEPARATOR.to_string(),
    };
    let mut shot = original.clone();
    shot.id = next_id(plan, scene);
    shot.number = format!("{base}{separator}{suffix}");
    let new_id = shot.id.clone();
    plan.shots.insert(at, shot);
    Ok(new_id)
}

/// Numbers the shots again in cut order, versions keeping their tail at the new number of
/// their base (LH-76). Scene: `12.1`, `12.2` ... (12.1, 12.3, 12.3A, 12.7 become 12.1, 12.2,
/// 12.2A, 12.3). Camera: each camera counts its own, `12A.1`, `12B.1`, `12A.2`. Group: shots
/// that overlap in time with other cameras share a number, `12.1A`, `12.1B`, `12.2A`.
pub(crate) fn renumber(plan: &mut FloorPlan, numbering: Numbering, scene: &str) {
    let mut new_base: HashMap<String, String> = HashMap::new();
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut groups: Vec<RunGroup> = Vec::new();

    for at in 0..plan.shots.len() {
        let shot = &plan.shots[at];
        let (base, tail) = split_number(numbering, &shot.number);
        let (base, tail) = (base.to_owned(), tail.to_owned());
        let label = camera_name(plan, &shot.camera);

        let number = match new_base.get(&base) {
            Some(number) => number.clone(),
            None => {
                let number = match numbering {
                    Numbering::Scene => format!("{scene}.{}", new_base.len() + 1),
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
                new_base.insert(base, number.clone());
                number
            }
        };
        plan.shots[at].number = format!("{number}{tail}");
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
    plan.links
        .retain(|l| !matches!(&l.target, LinkTarget::Shot { id: linked } if linked == id));

    let neighbour = plan
        .shots
        .get(at)
        .or(at.checked_sub(1).and_then(|i| plan.shots.get(i)));
    Ok(neighbour.map(|s| s.id.clone()))
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
