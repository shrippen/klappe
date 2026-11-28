//! A mark typed at the end of a cue or centered line (` ^`, `<`) is written as the mark.

use super::{read_in, written, Hide, Plan, Target};
use crate::parse::{CENTER_END, DUAL_MARK};
use crate::TextKind;

/// The mark that ends a line of `kind` (` ^`, `<`), which the editor hides.
fn end_mark(kind: TextKind) -> Option<char> {
    match kind {
        TextKind::Character => Some(DUAL_MARK),
        TextKind::Centered => Some(CENTER_END),
        _ => None,
    }
}

/// A mark typed at the end of the text (`BOB` and ` ^`) is not text any more once it is written:
/// the line is written without it, and the mark put behind, as the old line would have it.
pub(super) fn ended(target: &Target, plan: &Plan) -> Result<String, String> {
    let Some(mark) = end_mark(target.reads).filter(|_| target.hide.contains(&Hide::Forcing)) else {
        return Err(String::new());
    };
    let Some(cut) = plan.chars.len().checked_sub(1) else {
        return Err(String::new());
    };
    if plan.chars[cut] != mark || plan.typed.is_empty() || plan.typed.end != plan.chars.len() {
        return Err(String::new());
    }

    // The one blank in front of the mark goes with it.
    let blank = cut
        .checked_sub(1)
        .filter(|at| plan.chars[*at].is_whitespace());
    let keep = blank.unwrap_or(cut);
    let shorter = Plan {
        chars: plan.chars[..keep].to_vec(),
        marks: plan.marks[..keep].to_vec(),
        escaped: plan.escaped[..keep].to_vec(),
        notes: plan
            .notes
            .iter()
            .filter(|n| n.at <= keep)
            .cloned()
            .collect(),
        typed: plan.typed.start.min(keep)..keep,
        kept: plan.kept.min(keep),
    };
    let line = written(target, &shorter)?;
    let gap = blank.map_or(String::new(), |at| plan.chars[at].to_string());
    let full = format!("{line}{gap}{mark}");

    let read = read_in(&full, target.kind, target.above, target.hide);
    match read.kind == target.reads {
        true => Ok(full),
        false => Err(full),
    }
}
