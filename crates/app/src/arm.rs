//! What the next click on the plan does for the selection, once a key or the command palette
//! has armed it (LH-104, LH-105, LH-109): no tool is switched. Like the tools (`tools.rs`),
//! no geometry here.

use klappe_services::{PlanEdit, Point, Source};

/// What the next click does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Armed {
    #[default]
    Idle,
    /// Sets the next mark of the selected figure or camera there (a camera comes by dolly).
    NextMark,
    /// Sets a mark before the first one of the selected figure or camera there.
    MarkBefore,
    /// Sets the axis between the selected figure and the figure clicked.
    AxisTo,
}

impl Armed {
    /// The name QML uses; the end of the text key `arm.hint.<name>`.
    pub fn name(self) -> &'static str {
        match self {
            Armed::Idle => "",
            Armed::NextMark => "nextMark",
            Armed::MarkBefore => "markBefore",
            Armed::AxisTo => "axisTo",
        }
    }

    /// The state `name` names; `None` for the empty name and for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        [Armed::NextMark, Armed::MarkBefore, Armed::AxisTo]
            .into_iter()
            .find(|a| a.name() == name)
    }
}

/// The figure (its character) or camera a selection stands for, a grip on its way included.
enum Subject<'a> {
    Figure(&'a str),
    Camera(&'a str),
}

fn subject(selected: Option<&Source>) -> Option<Subject<'_>> {
    match selected? {
        Source::Figure(id)
        | Source::Mark { character: id, .. }
        | Source::FigureVia { character: id, .. } => Some(Subject::Figure(id)),
        Source::Camera(id) | Source::CameraVia { camera: id, .. } => Some(Subject::Camera(id)),
        _ => None,
    }
}

/// Whether `armed` has something to work on with this selection: the marks need a figure or
/// a camera, the axis a figure.
pub fn can_arm(armed: Armed, selected: Option<&Source>) -> bool {
    match (armed, subject(selected)) {
        (Armed::Idle, _) | (_, None) => false,
        (Armed::AxisTo, Some(subject)) => matches!(subject, Subject::Figure(_)),
        (_, Some(_)) => true,
    }
}

/// The edit a click at `at` makes; `None` for `AxisTo` (see `axis_pair`) and for a selection
/// the state does not fit.
pub fn click_edit(armed: Armed, selected: Option<&Source>, at: Point) -> Option<PlanEdit> {
    let subject = subject(selected)?;

    match (armed, subject) {
        (Armed::NextMark, Subject::Figure(character)) => Some(PlanEdit::AddMark {
            character: character.to_owned(),
            at,
        }),
        (Armed::NextMark, Subject::Camera(camera)) => Some(PlanEdit::AddCameraMark {
            camera: camera.to_owned(),
            at,
        }),
        (Armed::MarkBefore, Subject::Figure(character)) => Some(PlanEdit::AddMarkBefore {
            source: Source::Figure(character.to_owned()),
            at,
        }),
        (Armed::MarkBefore, Subject::Camera(camera)) => Some(PlanEdit::AddMarkBefore {
            source: Source::Camera(camera.to_owned()),
            at,
        }),
        _ => None,
    }
}

/// The two characters of the axis: the selected figure and the figure `clicked` stands for.
/// `None` when either is no figure; two the same are the session's to refuse.
pub fn axis_pair(selected: Option<&Source>, clicked: Option<&Source>) -> Option<(String, String)> {
    let (Subject::Figure(a), Subject::Figure(b)) = (subject(selected)?, subject(clicked)?) else {
        return None;
    };

    Some((a.to_owned(), b.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn figure(id: &str) -> Source {
        Source::Figure(id.into())
    }

    fn at() -> Point {
        Point::new(2.0, 3.0)
    }

    #[test]
    fn names_read_back() {
        for armed in [Armed::NextMark, Armed::MarkBefore, Armed::AxisTo] {
            assert_eq!(Armed::from_name(armed.name()), Some(armed));
        }
        assert_eq!(Armed::from_name(""), None);
        assert_eq!(Armed::Idle.name(), "");
    }

    /// A figure, its mark and a grip of its way all name the figure; a camera comes by dolly.
    #[test]
    fn a_click_sets_the_next_mark_of_the_selected_figure_or_camera() {
        let mark = Source::Mark {
            character: "e".into(),
            index: 1,
        };

        for selected in [figure("e"), mark] {
            assert_eq!(
                click_edit(Armed::NextMark, Some(&selected), at()),
                Some(PlanEdit::AddMark {
                    character: "e".into(),
                    at: at()
                })
            );
        }
        assert_eq!(
            click_edit(Armed::NextMark, Some(&Source::Camera("a".into())), at()),
            Some(PlanEdit::AddCameraMark {
                camera: "a".into(),
                at: at()
            })
        );
        assert_eq!(
            click_edit(Armed::NextMark, Some(&Source::Wall(0)), at()),
            None
        );
        assert_eq!(click_edit(Armed::NextMark, None, at()), None);
    }

    #[test]
    fn a_click_sets_a_mark_before_the_first() {
        assert_eq!(
            click_edit(Armed::MarkBefore, Some(&figure("e")), at()),
            Some(PlanEdit::AddMarkBefore {
                source: figure("e"),
                at: at()
            })
        );
        assert_eq!(
            click_edit(Armed::MarkBefore, Some(&Source::Camera("a".into())), at()),
            Some(PlanEdit::AddMarkBefore {
                source: Source::Camera("a".into()),
                at: at()
            })
        );
        assert_eq!(click_edit(Armed::AxisTo, Some(&figure("e")), at()), None);
    }

    #[test]
    fn the_axis_runs_between_two_figures() {
        let mark = Source::Mark {
            character: "b".into(),
            index: 0,
        };

        assert_eq!(
            axis_pair(Some(&figure("a")), Some(&mark)),
            Some(("a".into(), "b".into()))
        );
        assert_eq!(axis_pair(Some(&figure("a")), Some(&Source::Wall(0))), None);
        assert_eq!(axis_pair(Some(&figure("a")), None), None);
        assert_eq!(
            axis_pair(Some(&Source::Camera("a".into())), Some(&figure("b"))),
            None
        );
    }

    #[test]
    fn what_can_be_armed() {
        let camera = Source::Camera("a".into());

        assert!(can_arm(Armed::NextMark, Some(&camera)));
        assert!(can_arm(Armed::MarkBefore, Some(&figure("e"))));
        assert!(can_arm(Armed::AxisTo, Some(&figure("e"))));
        assert!(!can_arm(Armed::AxisTo, Some(&camera)));
        assert!(!can_arm(Armed::NextMark, None));
        assert!(!can_arm(Armed::Idle, Some(&figure("e"))));
    }
}
