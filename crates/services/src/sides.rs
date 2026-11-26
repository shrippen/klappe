//! A character's sides (DB-62): the text of the scenes they speak in, whole, to be laid out
//! as a screenplay of its own.
//!
//! ```text
//! book ─► scenes with a cue of the figure ─► text (headings numbered as in the book)
//!      ─► lay_out ─► PDF headed with the figure's name
//! ```

use klappe_model::{same_character_name, Project};
use klappe_script::{numbered_head, parse_located, Element, Located};

use crate::error::ServiceError;
use crate::script_refs::names;

/// The sides of one figure.
pub(crate) struct Sides {
    /// The name the pages are headed with: the character's, else the cue as the book has it.
    pub(crate) name: String,
    pub(crate) text: String,
}

/// The sides of `figure`, a character's name or alias, else a cue of the book that no
/// character answers to.
pub(crate) fn sides(text: &str, project: &Project, figure: &str) -> Result<Sides, ServiceError> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let located = parse_located(&text);
    let (name, cues) = figure_cues(&located, project, figure)
        .ok_or_else(|| ServiceError::NoCharacter(figure.to_owned()))?;

    let lines: Vec<&str> = text.split('\n').collect();
    let chunks = scene_chunks(&located, lines.len());
    let mut out = Vec::new();
    for chunk in &chunks {
        let spoken = located[chunk.elements.clone()]
            .iter()
            .any(|l| cues.iter().any(|cue| names(&l.element, cue)));
        if !spoken {
            continue;
        }

        out.extend(chunk_lines(&lines, chunk));
    }

    if out.is_empty() {
        return Err(ServiceError::NoDialogue(name));
    }

    Ok(Sides {
        name,
        text: out.join("\n"),
    })
}

/// The name to show and the cue names the figure answers to; `None` if the project has no
/// such character and the book no such cue.
fn figure_cues(
    located: &[Located],
    project: &Project,
    figure: &str,
) -> Option<(String, Vec<String>)> {
    if let Some(character) = project.characters.iter().find(|c| c.is_called(figure)) {
        let mut cues = vec![character.name.clone()];
        cues.extend(character.aliases.iter().cloned());

        return Some((character.name.clone(), cues));
    }

    located.iter().find_map(|l| match &l.element {
        Element::Character { name, .. } if same_character_name(name, figure) => {
            Some((name.clone(), vec![name.clone()]))
        }
        _ => None,
    })
}

/// A scene of the book: its elements and the text lines it takes. What stands before the
/// first heading is a unit of its own, or the whole book if it has no heading.
struct Chunk {
    elements: std::ops::Range<usize>,
    lines: std::ops::Range<usize>,
    /// The place in the book of a heading that has no number in the text.
    unnumbered: Option<usize>,
}

/// The units in book order.
fn scene_chunks(located: &[Located], line_count: usize) -> Vec<Chunk> {
    let heads: Vec<usize> = (0..located.len())
        .filter(|&i| matches!(located[i].element, Element::SceneHeading { .. }))
        .collect();
    let first = heads.first().copied().unwrap_or(located.len());
    let mut chunks = Vec::new();

    if first > 0 {
        let end = located.get(first).map_or(line_count, first_line);

        chunks.push(Chunk {
            elements: 0..first,
            lines: first_line(&located[0])..end,
            unnumbered: None,
        });
    }

    for (n, &head) in heads.iter().enumerate() {
        let next = heads.get(n + 1).copied();
        let end = next.map_or(line_count, |i| first_line(&located[i]));
        let numbered = matches!(
            &located[head].element,
            Element::SceneHeading {
                number: Some(_),
                ..
            }
        );

        chunks.push(Chunk {
            elements: head + 1..next.unwrap_or(located.len()),
            lines: first_line(&located[head])..end,
            unnumbered: (!numbered).then_some(n + 1),
        });
    }

    chunks
}

fn first_line(located: &Located) -> usize {
    located.lines.first().copied().unwrap_or_default()
}

/// The lines of a unit; a heading without a number gets its place in the book, as the full
/// PDF prints it, so the sides say which scene is meant.
fn chunk_lines(lines: &[&str], chunk: &Chunk) -> Vec<String> {
    let mut out: Vec<String> = lines[chunk.lines.clone()]
        .iter()
        .map(|l| (*l).to_owned())
        .collect();
    if let (Some(place), Some(head)) = (chunk.unnumbered, out.first_mut()) {
        *head = numbered_head(head, &place.to_string());
    }

    out
}
