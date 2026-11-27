//! What stands in front of and behind the text of a written line, and which of the choices to
//! try for a kind: the marks that force a kind or end a line.

use super::Target;
use crate::inline::UNDERLINE;
use crate::parse::{
    CENTER_END, DUAL_MARK, FORCE_ACTION, FORCE_CHARACTER, FORCE_HEADING, FORCE_TRANSITION, LYRICS,
    NUMBER_MARK,
};
use crate::TextKind;

/// Whether a forcing character in front of the line was there before or is new.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Lead {
    Kept,
    Added,
}

/// What stands behind the text: what ended the line (` ^`, `<`, the number of a heading) as it
/// was, taken away, closed with `<`, or an empty emphasis (`__`) that keeps `@(V.O.)` and
/// `(V.O.)(V.O.)` from reading as a parenthetical.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum End {
    Kept,
    Dropped,
    Closed,
    Guarded,
}

/// Whether the part behind the text of a line of `kind` holds what ends it: the mark of a cue
/// or a centered line, the number of a heading.
fn ends_with_mark(kind: TextKind, tail: &str) -> bool {
    let mark = match kind {
        TextKind::Character => DUAL_MARK,
        TextKind::Centered => CENTER_END,
        TextKind::Heading => NUMBER_MARK,
        _ => return false,
    };

    tail.trim_start().starts_with(mark)
}

/// What may stand in front of the text and behind it, in the order to try: as it was, without
/// its forcing character, without what ends the line, with the forcing character of the
/// target kind, and the guard of a cue or speech.
pub(super) fn leads(target: &Target, tail: &str) -> Vec<(String, Lead, End)> {
    let prefix = &target.old[..target.split.start];
    let mark = prefix.trim_start();
    let bare = &prefix[..prefix.len() - mark.len()];
    let ends = ends_with_mark(target.kind, tail);
    let mut leads = vec![(prefix.to_owned(), Lead::Kept, End::Kept)];

    if !mark.is_empty() {
        leads.push((bare.to_owned(), Lead::Kept, End::Kept));
    }
    if ends {
        leads.push((bare.to_owned(), Lead::Kept, End::Dropped));
    }
    if let Some(force) = forcing_char(target.reads) {
        leads.push((format!("{bare}{force}"), Lead::Added, End::Kept));
        if ends {
            leads.push((format!("{bare}{force}"), Lead::Added, End::Dropped));
        }
    }
    if target.reads == TextKind::Centered {
        leads.push((
            format!("{bare}{FORCE_TRANSITION}"),
            Lead::Added,
            End::Closed,
        ));
    }
    if matches!(target.reads, TextKind::Character | TextKind::Dialogue) {
        leads.push((prefix.to_owned(), Lead::Kept, End::Guarded));
    }
    // A name that is no cue by itself (`(Anna.)`) needs both.
    if let (TextKind::Character, Some(force)) = (target.reads, forcing_char(target.reads)) {
        leads.push((format!("{bare}{force}"), Lead::Added, End::Guarded));
    }

    leads
}

/// The part behind the text for `end`.
pub(super) fn behind(tail: &str, end: End, kind: TextKind) -> String {
    match end {
        End::Kept => tail.to_owned(),
        End::Dropped if ends_with_mark(kind, tail) => String::new(),
        End::Dropped => tail.to_owned(),
        End::Closed => CENTER_END.to_string(),
        End::Guarded => format!("{UNDERLINE}{UNDERLINE}{tail}"),
    }
}

/// The character that forces a line to be of `kind`, where Fountain has one.
fn forcing_char(kind: TextKind) -> Option<char> {
    match kind {
        TextKind::Action => Some(FORCE_ACTION),
        TextKind::Character => Some(FORCE_CHARACTER),
        TextKind::Heading => Some(FORCE_HEADING),
        TextKind::Transition => Some(FORCE_TRANSITION),
        TextKind::Lyrics => Some(LYRICS),
        _ => None,
    }
}
