//! What the select tool has picked and what the keys and a group drag do to it. Like the
//! tools (`tools.rs`), no geometry here.

use klappe_services::{PlanEdit, Point, Source};

/// What the keys do to the selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    Remove,
    /// Takes the bend out of the selected way (LH-21).
    Straighten,
    /// Clockwise for a positive angle.
    Turn {
        by_deg: f64,
    },
    /// Sets a copy of the selection apart from it (LH-08).
    Duplicate,
}

/// How far a copy sits from what it copies, in metres, to both sides: two grid cells, so it
/// is seen and still on the grid.
const COPY_OFFSET_M: f64 = 0.5;

/// What the select tool has picked, in the order it was picked: the canvas frames each.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selection(Vec<Source>);

/// What a pick does to the selection: replaces it, or (Shift held) adds to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Replace,
    Extend,
}

/// What a drag on empty ground pulls up: a box, or (Ctrl held) a freehand lasso (LH-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    Box,
    Lasso,
}

/// What a pick selects: a grip on a drawing's point stands for the drawing, which keeps its
/// grips, its card and its keys; a grip on a way is selected itself (it is the bend to
/// straighten).
fn whole(source: Source) -> Source {
    match source {
        Source::DrawingPoint { drawing, .. } => Source::Drawing(drawing),
        source => source,
    }
}

impl Selection {
    pub fn of(source: Option<Source>) -> Selection {
        Selection(source.map(whole).into_iter().collect())
    }

    pub fn contains(&self, source: &Source) -> bool {
        self.0.contains(source)
    }

    pub fn sources(&self) -> &[Source] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The one element picked; `None` for none or for several.
    pub fn only(&self) -> Option<&Source> {
        match self.0.as_slice() {
            [source] => Some(source),
            _ => None,
        }
    }

    /// A click on `picked` (`None`: empty ground). A plain click takes just that; with
    /// `Extend` it takes the element in, or out when it is in already, and a click on empty
    /// ground changes nothing.
    pub fn click(mut self, picked: Option<Source>, how: Pick) -> Selection {
        match (how, picked.map(whole)) {
            (Pick::Replace, picked) => Selection::of(picked),
            (Pick::Extend, None) => self,
            (Pick::Extend, Some(source)) => {
                match self.0.iter().position(|s| *s == source) {
                    Some(index) => {
                        self.0.remove(index);
                    }
                    None => self.0.push(source),
                }

                self.no_grips()
            }
        }
    }

    /// A box that took `found`: replaces the selection, or with `Extend` adds to it.
    pub fn boxed(mut self, found: Vec<Source>, how: Pick) -> Selection {
        if how == Pick::Replace {
            return Selection(found);
        }

        for source in found {
            if !self.contains(&source) {
                self.0.push(source);
            }
        }

        self.no_grips()
    }

    /// A grip shows only while its way alone is selected, so several hold none: it would be
    /// picked unseen and bend or straighten its way with the rest.
    fn no_grips(mut self) -> Selection {
        if self.0.len() > 1 {
            self.0
                .retain(|s| !matches!(s, Source::FigureVia { .. } | Source::CameraVia { .. }));
        }

        self
    }

    /// After a drag that grabbed `grabbed`: a member drags the whole selection along, else
    /// the grabbed element is all that is selected.
    pub fn grab(self, grabbed: Option<Source>) -> Selection {
        match grabbed {
            Some(source) if self.contains(&source) => self,
            grabbed => Selection::of(grabbed),
        }
    }

    /// The elements, `first` in front: the one the pointer holds snaps to the grid.
    fn leading(&self, first: &Source) -> Vec<Source> {
        let mut sources = vec![first.clone()];
        sources.extend(self.0.iter().filter(|s| *s != first).cloned());

        sources
    }

    /// Only the elements `shown` approves.
    pub fn keep(mut self, shown: impl Fn(&Source) -> bool) -> Selection {
        self.0.retain(shown);

        self
    }
}

/// What a drag that grabbed `grabbed` does when it holds a member of a selection of several:
/// the whole selection moves by the distance. `None` for a lone element, which `drag_edit`
/// handles, or for an element outside the selection.
pub fn group_drag_edit(
    selection: &Selection,
    grabbed: &Source,
    from: Point,
    to: Point,
) -> Option<PlanEdit> {
    if selection.len() < 2 || !selection.contains(grabbed) {
        return None;
    }

    Some(PlanEdit::ShiftGroup {
        sources: selection.leading(grabbed),
        dx_m: to.x_m - from.x_m,
        dy_m: to.y_m - from.y_m,
    })
}

/// The elements `after` shows that `before` did not: what a copy added.
pub fn added(before: &[Source], after: &[Source]) -> Selection {
    let new = after.iter().filter(|s| !before.contains(s)).cloned();

    Selection(new.collect())
}

/// A key on the selection; `None` when nothing is selected. Several elements take one group
/// edit; a way is made straight for one element only.
pub fn key_edit(key: Key, selected: &Selection) -> Option<PlanEdit> {
    let sources = selected.sources().to_vec();
    if key == Key::Duplicate && !selected.is_empty() {
        return Some(PlanEdit::Duplicate {
            sources,
            dx_m: COPY_OFFSET_M,
            dy_m: COPY_OFFSET_M,
        });
    }

    let [source] = sources.as_slice() else {
        return group_key_edit(key, sources);
    };
    let source = source.clone();

    match key {
        Key::Remove => Some(PlanEdit::Remove { source }),
        Key::Straighten => Some(PlanEdit::Straighten { source }),
        Key::Turn { by_deg } => Some(PlanEdit::Spin { source, by_deg }),
        Key::Duplicate => None,
    }
}

/// `key_edit` for no element or several.
fn group_key_edit(key: Key, sources: Vec<Source>) -> Option<PlanEdit> {
    if sources.is_empty() {
        return None;
    }

    match key {
        Key::Remove => Some(PlanEdit::RemoveGroup { sources }),
        Key::Turn { by_deg } => Some(PlanEdit::SpinGroup { sources, by_deg }),
        Key::Straighten | Key::Duplicate => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_services::Bend;

    fn p(x_m: f64, y_m: f64) -> Point {
        Point::new(x_m, y_m)
    }

    #[test]
    fn keys_act_on_the_selection_only() {
        let selected = Source::Light("led-1".into());
        let one = Selection::of(Some(selected.clone()));

        assert_eq!(
            key_edit(Key::Remove, &one),
            Some(PlanEdit::Remove {
                source: selected.clone()
            })
        );
        assert_eq!(
            key_edit(Key::Turn { by_deg: -15.0 }, &one),
            Some(PlanEdit::Spin {
                source: selected,
                by_deg: -15.0
            })
        );
        assert_eq!(key_edit(Key::Remove, &Selection::default()), None);
        assert_eq!(
            key_edit(
                Key::Straighten,
                &Selection::of(Some(Source::Camera("a".into())))
            ),
            Some(PlanEdit::Straighten {
                source: Source::Camera("a".into())
            })
        );
    }

    fn two() -> Selection {
        Selection::of(Some(Source::Wall(0))).click(Some(Source::Wall(1)), Pick::Extend)
    }

    #[test]
    fn keys_on_several_elements_make_group_edits() {
        let sources = vec![Source::Wall(0), Source::Wall(1)];

        assert_eq!(
            key_edit(Key::Remove, &two()),
            Some(PlanEdit::RemoveGroup {
                sources: sources.clone()
            })
        );
        assert_eq!(
            key_edit(Key::Turn { by_deg: 15.0 }, &two()),
            Some(PlanEdit::SpinGroup {
                sources: sources.clone(),
                by_deg: 15.0
            })
        );
        assert_eq!(
            key_edit(Key::Duplicate, &two()),
            Some(PlanEdit::Duplicate {
                sources,
                dx_m: COPY_OFFSET_M,
                dy_m: COPY_OFFSET_M
            })
        );
        // One way at a time.
        assert_eq!(key_edit(Key::Straighten, &two()), None);
    }

    #[test]
    fn one_element_is_copied_too() {
        let one = Selection::of(Some(Source::Note("n1".into())));

        assert!(matches!(
            key_edit(Key::Duplicate, &one),
            Some(PlanEdit::Duplicate { .. })
        ));
        assert_eq!(key_edit(Key::Duplicate, &Selection::default()), None);
    }

    #[test]
    fn a_plain_click_takes_one_and_shift_adds_or_takes_out() {
        let (a, b) = (Source::Wall(0), Source::Wall(1));
        let one = Selection::of(Some(a.clone()));

        let replaced = one.clone().click(Some(b.clone()), Pick::Replace);
        let added = one.clone().click(Some(b.clone()), Pick::Extend);
        let toggled = added.clone().click(Some(a.clone()), Pick::Extend);

        assert_eq!(replaced.sources(), std::slice::from_ref(&b));
        assert_eq!(added.sources(), [a, b.clone()]);
        assert_eq!(toggled.sources(), [b]);
    }

    #[test]
    fn empty_ground_clears_unless_shift_is_held() {
        let two = two();

        assert!(two.clone().click(None, Pick::Replace).is_empty());
        assert_eq!(two.clone().click(None, Pick::Extend), two);
    }

    #[test]
    fn a_box_replaces_or_adds_without_repeats() {
        let found = vec![Source::Wall(1), Source::Wall(2)];
        let one = Selection::of(Some(Source::Wall(0)));

        let replaced = one.clone().boxed(found.clone(), Pick::Replace);
        let added = two().boxed(found, Pick::Extend);

        assert_eq!(replaced.sources(), [Source::Wall(1), Source::Wall(2)]);
        assert_eq!(
            added.sources(),
            [Source::Wall(0), Source::Wall(1), Source::Wall(2)]
        );
    }

    /// Review of PR 31: a grip shows only while its way is the one selected, so in a
    /// selection of several it would be picked unseen and bend or straighten its way.
    /// A grip on a drawing's point selects the drawing, so its card, its grips and its keys
    /// stay; a grip on a way selects itself.
    #[test]
    fn a_grip_on_a_drawing_selects_the_drawing() {
        let grip = Source::DrawingPoint {
            drawing: "d1".into(),
            index: 1,
        };
        let drawing = Source::Drawing("d1".into());

        assert_eq!(
            Selection::of(Some(grip.clone())),
            Selection::of(Some(drawing.clone()))
        );
        assert_eq!(
            Selection::of(None).click(Some(grip.clone()), Pick::Replace),
            Selection::of(Some(drawing.clone()))
        );
        assert_eq!(
            Selection::of(Some(drawing.clone())).grab(Some(grip)),
            Selection::of(Some(drawing))
        );
    }

    #[test]
    fn a_selection_of_several_holds_no_grip() {
        let grip = Source::FigureVia {
            character: "elisabeth".into(),
            index: 1,
            bend: Bend::New(0),
        };
        let clicked = Selection::of(Some(grip.clone())).click(Some(Source::Wall(0)), Pick::Extend);
        let boxed = Selection::of(Some(grip)).boxed(vec![Source::Wall(0)], Pick::Extend);

        assert_eq!(clicked.sources(), [Source::Wall(0)]);
        assert_eq!(boxed.sources(), [Source::Wall(0)]);
    }

    #[test]
    fn only_is_one_element_and_not_several() {
        assert_eq!(Selection::default().only(), None);
        assert_eq!(
            Selection::of(Some(Source::Wall(0))).only(),
            Some(&Source::Wall(0))
        );
        assert_eq!(two().only(), None);
    }

    #[test]
    fn grabbing_a_member_drags_all_and_another_only_itself() {
        let outside = Some(Source::Wall(5));

        assert_eq!(two().grab(Some(Source::Wall(1))), two());
        assert_eq!(two().grab(outside.clone()), Selection::of(outside));
    }

    #[test]
    fn a_group_drag_moves_all_with_the_grabbed_one_first() {
        let edit = group_drag_edit(&two(), &Source::Wall(1), p(1.0, 1.0), p(3.0, 0.5));

        assert_eq!(
            edit,
            Some(PlanEdit::ShiftGroup {
                sources: vec![Source::Wall(1), Source::Wall(0)],
                dx_m: 2.0,
                dy_m: -0.5
            })
        );
    }

    #[test]
    fn a_drag_of_one_or_of_a_stranger_is_no_group_drag() {
        let one = Selection::of(Some(Source::Wall(0)));

        assert_eq!(
            group_drag_edit(&one, &Source::Wall(0), p(0.0, 0.0), p(1.0, 0.0)),
            None
        );
        assert_eq!(
            group_drag_edit(&two(), &Source::Wall(7), p(0.0, 0.0), p(1.0, 0.0)),
            None
        );
    }

    #[test]
    fn a_copy_is_what_the_plan_shows_anew() {
        let before = [Source::Wall(0), Source::Object("o1".into())];
        let after = [
            Source::Wall(0),
            Source::Object("o1".into()),
            Source::Object("o2".into()),
        ];

        assert_eq!(
            added(&before, &after).sources(),
            [Source::Object("o2".into())]
        );
    }

    #[test]
    fn what_is_gone_leaves_the_selection() {
        let kept = two().keep(|s| *s == Source::Wall(1));

        assert_eq!(kept.sources(), [Source::Wall(1)]);
    }
}
