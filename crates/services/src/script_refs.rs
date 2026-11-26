//! Where the screenplay text names a scene or a character, and what a rename changes there
//! (D-08): scene heading lines (`INT. SÄNGERSAAL – TAG #12#`) and the cues of dialogue
//! (`ELISABETH`). Action and dialogue text is not searched: a name in a sentence may be
//! meant differently.

use klappe_model::{same_character_name, Character, Project, Scene};
use klappe_script::{number_token, parse, parse_located, Element, Located, FORCE_CHARACTER};
use serde::Serialize;

const FORCE_HEADING: char = '.';

/// What a change replaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeKind {
    /// The text of a scene heading.
    Heading,
    /// The `#12#` of a scene heading.
    Number,
    /// The name above a dialogue.
    Cue,
    /// A cue that names no character any more (DB-50): the text stays, the book asks for it
    /// again. `to` is empty.
    Unknown,
}

/// A change of the text, grouped for the warning: `count` places go from `from` to `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScriptChange {
    pub kind: ChangeKind,
    pub from: String,
    pub to: String,
    pub count: usize,
}

/// One line of the text that becomes another.
struct LineChange {
    line: usize,
    kind: ChangeKind,
    from: String,
    to: String,
    new_line: String,
}

/// The changes the new `number` and `heading` of the scene `id` make to the text.
pub(crate) fn scene_changes(
    project: &Project,
    text: &str,
    id: &str,
    number: &str,
    heading: &str,
) -> Vec<ScriptChange> {
    group(&scene_lines(project, text, id, number, heading))
}

/// The changes the new `name` of the character `id` makes to the text.
pub(crate) fn character_changes(
    project: &Project,
    text: &str,
    id: &str,
    name: &str,
) -> Vec<ScriptChange> {
    group(&cue_lines(project, text, id, name))
}

/// The cues of `text` that `before` knows and `after` does not (DB-50), by name as first
/// written, in book order. A cue of the name `renamed` is left out: the rename carries it.
pub(crate) fn lost_cues(
    before: &Project,
    after: &Project,
    text: &str,
    renamed: Option<&str>,
) -> Vec<ScriptChange> {
    let mut out: Vec<ScriptChange> = Vec::new();

    for cue in cue_names(&parse(text)) {
        let carried = renamed.is_some_and(|r| same_character_name(r, &cue));
        if carried || !before.knows_cue(&cue) || after.knows_cue(&cue) {
            continue;
        }
        match out.iter_mut().find(|c| same_character_name(&c.from, &cue)) {
            Some(found) => found.count += 1,
            None => out.push(ScriptChange {
                kind: ChangeKind::Unknown,
                from: cue,
                to: String::new(),
                count: 1,
            }),
        }
    }

    out
}

/// The text with the new `number` and `heading` of the scene `id`; `None` when nothing changes.
pub(crate) fn scene_text(
    project: &Project,
    text: &str,
    id: &str,
    number: &str,
    heading: &str,
) -> Option<String> {
    rewrite(text, &scene_lines(project, text, id, number, heading))
}

/// The text with the new `name` of the character `id`; `None` when nothing changes.
pub(crate) fn character_text(
    project: &Project,
    text: &str,
    id: &str,
    name: &str,
) -> Option<String> {
    rewrite(text, &cue_lines(project, text, id, name))
}

/// How often the character's cue stands in the script of `scene`.
pub(crate) fn cue_count(text: &str, scene: &Scene, character: &Character) -> usize {
    let located = parse_located(text);
    let Some(range) = scene_range(&located, &scene.number) else {
        return 0;
    };

    located[range]
        .iter()
        .filter(|l| calls(&l.element, character))
        .count()
}

/// The ids of the project's characters whose cue stands in the script of `scene`, in the order
/// of their first cue. A cue that no character of the project has (by name) is left out.
pub(crate) fn speakers(text: &str, scene: &Scene, project: &Project) -> Vec<String> {
    let located = parse_located(text);
    let Some(range) = scene_range(&located, &scene.number) else {
        return Vec::new();
    };

    let mut ids: Vec<String> = Vec::new();
    for located in &located[range] {
        let named = project
            .characters
            .iter()
            .find(|c| calls(&located.element, c));
        if let Some(character) = named.filter(|c| !ids.contains(&c.id)) {
            ids.push(character.id.clone());
        }
    }

    ids
}

/// Whether a scene heading of the script carries `number` (the case of a number is style).
pub(crate) fn has_number(text: &str, number: &str) -> bool {
    let number = number.trim().to_lowercase();

    parse_located(text).iter().any(|l| match &l.element {
        Element::SceneHeading { number: n, .. } => {
            n.as_deref().map(str::to_lowercase).as_deref() == Some(number.as_str())
        }
        _ => false,
    })
}

/// Whether the element is a cue the character answers to: its name or an alias (DB-50).
pub(crate) fn calls(element: &Element, character: &Character) -> bool {
    match element {
        Element::Character { name, .. } => character.is_called(name),
        _ => false,
    }
}

/// The name of every cue of the book, one per cue, in book order. The extension (`V.O.`,
/// `CONT'D`) and the dual mark are no part of the name: the parser has split them off.
pub(crate) fn cue_names(script: &klappe_script::Script) -> Vec<String> {
    script
        .elements
        .iter()
        .filter_map(|e| match e {
            Element::Character { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// Whether the element is a cue of `name` (the case of a cue is style).
pub(crate) fn names(element: &Element, name: &str) -> bool {
    let Element::Character { name: cue, .. } = element else {
        return false;
    };

    same_character_name(cue, name)
}

/// The elements after the heading numbered `number`, up to the next heading.
fn scene_range(located: &[Located], number: &str) -> Option<std::ops::Range<usize>> {
    let is_head = |l: &Located| matches!(l.element, Element::SceneHeading { .. });
    let start = located.iter().position(|l| match &l.element {
        Element::SceneHeading { number: n, .. } => n.as_deref() == Some(number),
        _ => false,
    })?;
    let end = located[start + 1..]
        .iter()
        .position(is_head)
        .map_or(located.len(), |i| start + 1 + i);

    Some(start + 1..end)
}

fn scene_lines(
    project: &Project,
    text: &str,
    id: &str,
    number: &str,
    heading: &str,
) -> Vec<LineChange> {
    let Some(scene) = project.scene(id) else {
        return Vec::new();
    };
    let (number, heading) = (number.trim(), heading.trim());
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let head = parse_located(text).into_iter().find(|l| match &l.element {
        Element::SceneHeading { number: n, .. } => n.as_deref() == Some(scene.number.as_str()),
        _ => false,
    });
    let Some((Element::SceneHeading { text: old, .. }, line)) =
        head.and_then(|h| Some((h.element, *h.lines.first()?)))
    else {
        return Vec::new();
    };
    let Some(raw) = lines.get(line) else {
        return Vec::new();
    };

    let mut new_line = raw.to_string();
    let mut changes = Vec::new();
    // A heading the user wrote on their own is left alone.
    if heading != scene.heading && old.trim() == scene.heading {
        if let Some(next) = replace_heading(&new_line, &old, heading) {
            new_line = next;
            changes.push((ChangeKind::Heading, old.clone(), heading.to_owned()));
        }
    }
    if number != scene.number {
        let (from, to) = (number_token(&scene.number), number_token(number));
        if let Some(next) = replace_once(&new_line, &from, &to) {
            new_line = next;
            changes.push((ChangeKind::Number, from, to));
        }
    }

    // A heading that no longer reads as one is forced with a dot.
    if !changes.is_empty() && !reads_as_heading(&new_line) {
        new_line.insert(0, FORCE_HEADING);
    }

    changes
        .into_iter()
        .map(|(kind, from, to)| LineChange {
            line,
            kind,
            from,
            to,
            new_line: new_line.clone(),
        })
        .collect()
}

/// Whether the line, with a dialogue under it, reads as a cue of `name`, not as a heading or
/// a cue of another name (`Anna (jung)` is `Anna` with an extension).
fn reads_as_cue(line: &str, name: &str) -> bool {
    let probe = format!("{}\nx\n", line.trim_end());

    matches!(
        parse(&probe).elements.first(),
        Some(Element::Character { name: n, .. }) if same_character_name(n, name)
    )
}

/// Whether `name` can be a cue at all, forced with `@` if need be: `Anna (jung)` cannot, it
/// reads as `Anna` with the extension `jung` in any spelling.
pub(crate) fn can_be_cue(name: &str) -> bool {
    reads_as_cue(&format!("{FORCE_CHARACTER}{name}"), name)
}

fn reads_as_heading(line: &str) -> bool {
    let probe = format!("{}\n\n", line.trim_end());

    matches!(
        parse(&probe).elements.first(),
        Some(Element::SceneHeading { .. })
    )
}

fn cue_lines(project: &Project, text: &str, id: &str, name: &str) -> Vec<LineChange> {
    let Some(character) = project.character(id) else {
        return Vec::new();
    };
    let name = name.trim();
    if name == character.name || name.is_empty() {
        return Vec::new();
    }
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut changes = Vec::new();

    for located in parse_located(text) {
        let Element::Character { name: cue, .. } = &located.element else {
            continue;
        };
        let Some(line) = located.lines.first().copied() else {
            continue;
        };
        if !names(&located.element, &character.name) {
            continue;
        }

        // A cue in capitals stays in capitals.
        let to = match *cue == cue.to_uppercase() {
            true => name.to_uppercase(),
            false => name.to_owned(),
        };
        // Only the case of the name changed: the cue stays.
        if to == *cue {
            continue;
        }
        let Some(mut new_line) = lines.get(line).and_then(|raw| replace_once(raw, cue, &to)) else {
            continue;
        };
        // `Ext. Agentin` would read as a heading: forced with `@`, as a heading with `.`.
        if !reads_as_cue(&new_line, &to) {
            let indent = new_line.len() - new_line.trim_start().len();
            new_line.insert(indent, FORCE_CHARACTER);
        }
        changes.push(LineChange {
            line,
            kind: ChangeKind::Cue,
            from: cue.clone(),
            to,
            new_line,
        });
    }

    changes
}

/// `raw` with the heading `old` replaced. An empty one (`.#12#`) has no text to replace: the
/// new heading goes in front of the number, and the dot that forced the empty heading goes.
fn replace_heading(raw: &str, old: &str, heading: &str) -> Option<String> {
    if !old.trim().is_empty() {
        return replace_once(raw, old, heading);
    }

    Some(format!(
        "{heading} {}",
        raw.trim_start_matches(FORCE_HEADING)
    ))
}

/// `raw` with the first `from` replaced; `None` if it is not there (a boneyard may hide it).
fn replace_once(raw: &str, from: &str, to: &str) -> Option<String> {
    raw.contains(from).then(|| raw.replacen(from, to, 1))
}

/// Counts equal changes once: `3 × ANNA → ANNE`.
fn group(changes: &[LineChange]) -> Vec<ScriptChange> {
    let mut out: Vec<ScriptChange> = Vec::new();

    for change in changes {
        let same = |c: &&mut ScriptChange| {
            c.kind == change.kind && c.from == change.from && c.to == change.to
        };
        match out.iter_mut().find(same) {
            Some(found) => found.count += 1,
            None => out.push(ScriptChange {
                kind: change.kind,
                from: change.from.clone(),
                to: change.to.clone(),
                count: 1,
            }),
        }
    }

    out
}

/// The text with the lines replaced; the last change of a line wins (both of a heading's
/// changes carry the line as it ends up).
fn rewrite(text: &str, changes: &[LineChange]) -> Option<String> {
    if changes.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_owned).collect();

    for change in changes {
        lines[change.line] = change.new_line.clone();
    }

    Some(lines.concat())
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{FigureColor, Location};

    const SCRIPT: &str = "INT. SAAL – TAG #12#\n\nElisabeth tritt ein.\n\nELISABETH\nGuten Tag.\n\nBURGVOGT\nHerrin.\n\nELISABETH (V.O.)\nWeiter.\n\nINT. HOF – NACHT #13#\n\nELISABETH\nHier.\n";

    fn project() -> Project {
        let mut p = Project::new("Film");
        p.locations.push(Location {
            id: "saal".into(),
            name: "Saal".into(),
            set: Default::default(),
            site: None,
        });
        for (id, name) in [("elisabeth", "Elisabeth"), ("burgvogt", "Burgvogt")] {
            p.characters.push(Character {
                id: id.into(),
                name: name.into(),
                short: "X".into(),
                color: FigureColor::Red,
                group: None,
                size_m: None,
                aliases: Vec::new(),
            });
        }
        for (id, number, heading) in [
            ("m4djbtj6", "12", "INT. SAAL – TAG"),
            ("szene-013", "13", "INT. HOF – NACHT"),
        ] {
            p.scenes.push(Scene {
                id: id.into(),
                number: number.into(),
                heading: heading.into(),
                location: Some("saal".into()),
                plans: Vec::new(),
                ..Default::default()
            });
        }
        p
    }

    /// DB-50: a cue with an alias of the character is one of its speakers and cues.
    #[test]
    fn an_alias_cue_counts_for_the_character() {
        let mut p = project();
        let text = "INT. SAAL – TAG #12#\n\nLIESL (V.O.)\nHallo.\n\n";
        let scene = p.scenes[0].clone();
        assert!(speakers(text, &scene, &p).is_empty());

        p.characters[0].aliases.push("liesl".into());

        assert_eq!(speakers(text, &scene, &p), ["elisabeth"]);
        assert_eq!(cue_count(text, &scene, &p.characters[0]), 1);
    }

    /// Every cue of the character changes, in all scenes, in its own case.
    #[test]
    fn a_rename_changes_every_cue() {
        let p = project();

        let changes = character_changes(&p, SCRIPT, "elisabeth", "Elise");
        assert_eq!(
            changes,
            vec![ScriptChange {
                kind: ChangeKind::Cue,
                from: "ELISABETH".into(),
                to: "ELISE".into(),
                count: 3,
            },]
        );

        let text = character_text(&p, SCRIPT, "elisabeth", "Elise").unwrap();
        assert_eq!(text.matches("ELISE\n").count(), 2);
        assert!(text.contains("ELISE (V.O.)\n"));
        assert!(text.contains("Elisabeth tritt ein."));
        assert!(text.contains("BURGVOGT\n"));
    }

    /// Only the case of the name differs: the cues stay as they are.
    #[test]
    fn a_name_in_another_case_changes_nothing() {
        let p = project();

        assert!(character_changes(&p, SCRIPT, "elisabeth", "ELISABETH").is_empty());
        assert_eq!(character_text(&p, SCRIPT, "elisabeth", "ELISABETH"), None);
    }

    /// An empty heading (`.#12#`) is replaced as a whole.
    #[test]
    fn an_empty_old_heading_is_replaced() {
        let mut p = project();
        p.scenes[0].heading = String::new();
        let script = ".#12#\n\nA.\n";

        let text = scene_text(&p, script, "m4djbtj6", "12", "INT. NEU").unwrap();
        assert!(text.starts_with("INT. NEU #12#\n"), "{text:?}");
    }

    #[test]
    fn the_same_name_changes_nothing() {
        let p = project();

        assert!(character_changes(&p, SCRIPT, "elisabeth", " Elisabeth ").is_empty());
        assert_eq!(character_text(&p, SCRIPT, "elisabeth", "Elisabeth"), None);
        assert_eq!(character_text(&p, SCRIPT, "nobody", "X"), None);
    }

    #[test]
    fn a_new_heading_and_number_change_one_line() {
        let p = project();

        let changes = scene_changes(&p, SCRIPT, "m4djbtj6", "12A", "INT. SAAL – NACHT");
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].kind, ChangeKind::Heading);
        assert_eq!(changes[0].to, "INT. SAAL – NACHT");
        assert_eq!(changes[1].kind, ChangeKind::Number);
        assert_eq!(
            (changes[1].from.as_str(), changes[1].to.as_str()),
            ("#12#", "#12A#")
        );

        let text = scene_text(&p, SCRIPT, "m4djbtj6", "12A", "INT. SAAL – NACHT").unwrap();
        assert!(text.starts_with("INT. SAAL – NACHT #12A#\n"));
        assert!(text.contains("INT. HOF – NACHT #13#\n"));
    }

    /// The location alone, or the same heading, leaves the text as it is.
    #[test]
    fn an_unchanged_heading_changes_nothing() {
        let p = project();

        assert_eq!(
            scene_text(&p, SCRIPT, "m4djbtj6", "12", "INT. SAAL – TAG"),
            None
        );
    }

    /// A heading the user wrote differently in the script keeps its text; the number follows.
    #[test]
    fn an_own_script_heading_keeps_its_text() {
        let p = project();
        let script = SCRIPT.replace("INT. SAAL – TAG", "INT. GROSSER SAAL – TAG");

        let text = scene_text(&p, &script, "m4djbtj6", "14", "INT. SAAL – NACHT").unwrap();
        assert!(text.starts_with("INT. GROSSER SAAL – TAG #14#\n"));
    }

    /// A heading that would read as action gets the dot that forces it.
    #[test]
    fn a_heading_without_a_prefix_is_forced() {
        let p = project();

        let text = scene_text(&p, SCRIPT, "m4djbtj6", "12", "Der Saal").unwrap();
        assert!(text.starts_with(".Der Saal #12#\n"));
        assert!(matches!(
            parse(&text).elements.first(),
            Some(Element::SceneHeading { .. })
        ));
    }

    #[test]
    fn cues_are_counted_per_scene() {
        let p = project();
        let elisabeth = p.character("elisabeth").unwrap();

        assert_eq!(
            cue_count(SCRIPT, p.scene("m4djbtj6").unwrap(), elisabeth),
            2
        );
        assert_eq!(
            cue_count(SCRIPT, p.scene("szene-013").unwrap(), elisabeth),
            1
        );
        assert_eq!(cue_count("", p.scene("szene-013").unwrap(), elisabeth), 0);
    }
}
