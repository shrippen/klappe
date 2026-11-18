//! Between a script line and what it is linked to on the plan (DB-63).

use super::qobject;
use crate::selection::Selection;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{LinkKind, ScriptLine, Session, Source};

/// What `script_line` answers when no line is linked to the selection.
const NO_LINE: i32 = -1;

/// A link as the script panel's chip names it.
enum Link {
    Mark { character: String, index: usize },
    Shot(String),
}

impl Link {
    fn of(kind: &str, target: &str, index: i32) -> Option<Link> {
        match (kind, usize::try_from(index)) {
            ("mark", Ok(index)) => Some(Link::Mark {
                character: target.to_owned(),
                index,
            }),
            ("shot", _) => Some(Link::Shot(target.to_owned())),
            _ => None,
        }
    }

    /// Whether `line` has a chip for this link.
    fn is_on(&self, line: &ScriptLine) -> bool {
        line.chips.iter().any(|chip| match self {
            Link::Mark { character, index } => {
                chip.kind == LinkKind::Mark && chip.target == *character && chip.index == *index
            }
            Link::Shot(id) => chip.kind == LinkKind::Shot && chip.target == *id,
        })
    }
}

/// The mark selected alone, else the selected shot.
fn selected_link(s: &Session, selected: &Selection) -> Option<Link> {
    if let Some(Source::Mark { character, index }) = selected.only() {
        return Some(Link::Mark {
            character: character.clone(),
            index: *index,
        });
    }

    s.selected_shot().map(|id| Link::Shot(id.to_owned()))
}

impl qobject::Controller {
    pub(super) fn show_link(
        mut self: Pin<&mut Self>,
        kind: &QString,
        target: &QString,
        index: i32,
    ) -> bool {
        let Some(link) = Link::of(&kind.to_string(), &target.to_string(), index) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        // The mark is selected and the time goes to its arrival; a shot is selected as in
        // the shot list, and no mark stays selected beside it.
        let shown = match link {
            Link::Mark { character, index } => {
                let Some(mark) = s
                    .plan()
                    .and_then(|plan| plan.figure(&character))
                    .and_then(|figure| figure.marks.get(index))
                else {
                    return false;
                };

                let at_s = mark.at_s;
                s.set_time(at_s);
                Selection::of(Some(Source::Mark { character, index }))
            }
            Link::Shot(id) => {
                if s.select_shot(&id).is_err() {
                    return false;
                }

                Selection::default()
            }
        };

        self.as_mut().rust_mut().get_mut().selected = shown;
        self.refresh();
        true
    }

    pub(super) fn script_line(&self) -> i32 {
        let this = self.rust();
        let Some(s) = this.session.as_ref() else {
            return NO_LINE;
        };
        let Some(link) = selected_link(s, &this.selected) else {
            return NO_LINE;
        };

        s.script_lines()
            .iter()
            .position(|line| link.is_on(line))
            .and_then(|at| i32::try_from(at).ok())
            .unwrap_or(NO_LINE)
    }
}
