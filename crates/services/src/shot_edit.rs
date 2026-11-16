use std::collections::HashMap;

use klappe_model::{Angle, FloorPlan, LinkTarget, Movement, Shot, ShotSize};
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

/// The number and id for a new shot of the scene numbered `scene`: its number and one more
/// than the highest running number so far, so gaps stay and `12.4A` counts as 4. The id
/// is stable and independent of the number: a taken one (a shot renumbered since) moves on.
fn next_name(plan: &FloorPlan, scene: &str) -> (String, String) {
    let prefix = format!("{scene}.");
    let highest = plan
        .shots
        .iter()
        .filter_map(|s| s.number.strip_prefix(&prefix))
        .filter_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0);

    let number = highest + 1;

    let mut n = number;
    while plan.shots.iter().any(|s| s.id == format!("s{scene}-{n}")) {
        n += 1;
    }
    (format!("{scene}.{number}"), format!("s{scene}-{n}"))
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
pub(crate) fn add(plan: &mut FloorPlan, scene: &str, camera: &str, at_s: f64) -> String {
    let (number, id) = next_name(plan, scene);
    let (from_s, to_s) = span_at(plan, at_s);

    insert_by_start(
        plan,
        Shot {
            id: id.clone(),
            number,
            camera: camera.to_owned(),
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
        },
    );
    id
}

/// Adds a copy of shot `id` with a new number, right behind it; returns the copy's id.
/// Several shots of one setup start as copies (LH-70).
pub(crate) fn copy(plan: &mut FloorPlan, scene: &str, id: &str) -> Result<String, ServiceError> {
    let at = plan
        .shots
        .iter()
        .position(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
    let (number, new_id) = next_name(plan, scene);

    let mut shot = plan.shots[at].clone();
    shot.id = new_id.clone();
    shot.number = number;
    plan.shots.insert(at + 1, shot);
    Ok(new_id)
}

/// Splits a shot number into its base and version letters: `12.3B` is (`12.3`, `B`).
/// Letters are a version only after a digit and at most `MAX_VERSION_LETTERS` long;
/// `INSERT` is a base of its own (numbers are free text).
fn split_number(number: &str) -> (&str, &str) {
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
        .bytes()
        .fold(0, |n, b| n * LETTERS + usize::from(b - b'A') + 1)
}

/// Adds a copy of shot `id` as the next version of its number, behind the last version
/// of that number: a copy of `12.3` is `12.3A`, then `12.3B`; a copy of `12.3A` is `12.3B`.
/// Returns the copy's id.
pub(crate) fn version(plan: &mut FloorPlan, scene: &str, id: &str) -> Result<String, ServiceError> {
    let original = plan
        .shots
        .iter()
        .find(|s| s.id == id)
        .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
    let base = split_number(&original.number).0.to_owned();

    let in_base = |s: &Shot| split_number(&s.number).0 == base;
    let highest = plan
        .shots
        .iter()
        .filter(|s| in_base(s))
        .map(|s| version_index(split_number(&s.number).1))
        .max()
        .unwrap_or(0);
    let at = plan.shots.iter().rposition(in_base).map_or(0, |i| i + 1);

    let mut shot = original.clone();
    shot.id = next_name(plan, scene).1;
    shot.number = format!("{base}{}", letters(highest + 1));
    let new_id = shot.id.clone();
    plan.shots.insert(at, shot);
    Ok(new_id)
}

/// Numbers the shots `{scene}.1`, `{scene}.2` ... in cut order. Versions keep their letters
/// at the new number of their base: 12.1, 12.3, 12.3A, 12.7 become 12.1, 12.2, 12.2A, 12.3.
pub(crate) fn renumber(plan: &mut FloorPlan, scene: &str) {
    let mut new_base: HashMap<String, String> = HashMap::new();

    for shot in &mut plan.shots {
        let (base, letters) = split_number(&shot.number);
        let next = format!("{scene}.{}", new_base.len() + 1);
        let number = new_base.entry(base.to_owned()).or_insert(next);
        shot.number = format!("{number}{letters}");
    }
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
