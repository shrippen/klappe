//! Between a script line and what it is linked to on the plan (DB-63).

use super::qobject;
use crate::selection::Selection;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{LinkKind, ScriptLine, Session, Source};

/// What `script_line` answers when no line is linked to the selection.
const NO_LINE: i32 = -1;

/// The index of a negative one from the surface: no mark has it.
const NO_INDEX: usize = usize::MAX;

/// A link as the script panel's chip names it; a shot's index is 0.
struct Link {
    kind: LinkKind,
    target: String,
    index: usize,
}

impl Link {
    /// Whether `line` has a chip for this link.
    fn is_on(&self, line: &ScriptLine) -> bool {
        line.chips.iter().any(|chip| {
            chip.kind == self.kind && chip.target == self.target && chip.index == self.index
        })
    }
}

/// The mark selected alone, else the selected shot.
fn selected_link(s: &Session, selected: &Selection) -> Option<Link> {
    if let Some(Source::Mark { character, index }) = selected.only() {
        return Some(Link {
            kind: LinkKind::Mark,
            target: character.clone(),
            index: *index,
        });
    }

    s.selected_shot().map(|id| Link {
        kind: LinkKind::Shot,
        target: id.to_owned(),
        index: 0,
    })
}

impl qobject::Controller {
    pub(super) fn show_link(
        mut self: Pin<&mut Self>,
        kind: &QString,
        target: &QString,
        index: i32,
    ) -> bool {
        let Some(kind) = LinkKind::parse(&kind.to_string()) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let index = usize::try_from(index).unwrap_or(NO_INDEX);
        let Ok(mark) = s.show_link(kind, &target.to_string(), index) else {
            return false;
        };

        // A shot is selected as in the shot list, and no mark stays selected beside it.
        let shown = Selection::of(mark);

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
