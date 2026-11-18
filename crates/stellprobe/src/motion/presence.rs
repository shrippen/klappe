//! A figure that appears or leaves (LH-26) fades in or out instead of switching, and while it
//! fades it stands at its mark.
//!
//! ```text
//!  opacity
//!  1 |        ┌──────────┐
//!    |       /            \
//!  0 |──────/              \──────
//!         at_s          at_s + hold_s
//!         └ FADE_S ┘      └ FADE_S ┘
//!        (appears)       (leaves)
//! ```

use klappe_model::{Arrival, Departure, FigureTrack};

use super::{gone_on_arrival, place_at, Key, Placement};

/// How long a figure takes to fade in or out. Half a second is a few frames at film speed:
/// seen as a fade, but short enough that the action is not held up. The fade-out is cut short
/// where the next mark's time comes first.
const FADE_S: f64 = 0.5;

/// How strongly a figure is drawn at `t_s`, 0 to 1: it fades in at an appearing mark and out
/// after a leaving one (LH-26). 0 while it is not in the scene.
pub fn figure_opacity_at(track: &FigureTrack, t_s: f64) -> f64 {
    presence_at(track, t_s).map_or(0.0, |p| p.opacity)
}

/// A figure in the scene: where it stands and how strongly it is drawn, 0 (not yet) to 1.
pub(super) struct Presence {
    pub(super) place: Placement,
    pub(super) opacity: f64,
}

/// The figure of `track` at `t_s`; `None` while it is not in the scene, fading included.
pub(super) fn presence_at(track: &FigureTrack, t_s: f64) -> Option<Presence> {
    let keys: Vec<Key<'_>> = track.marks.iter().map(Key::from).collect();

    if let Some(place) = place_at(&keys, t_s) {
        return Some(Presence {
            place,
            opacity: fade_in(&keys, t_s),
        });
    }

    let (place, left_s) = fading_out(&keys, t_s)?;
    let out = 1.0 - (t_s - left_s) / FADE_S;

    Some(Presence {
        place,
        opacity: out * fade_in(&keys, t_s),
    })
}

/// How far the latest appearance the figure has reached has faded in; 1 for a figure that
/// was there from the start.
fn fade_in(keys: &[Key<'_>], t_s: f64) -> f64 {
    let appeared = keys
        .iter()
        .rev()
        .find(|k| k.at_s <= t_s && k.arrival == Arrival::Appear);

    appeared.map_or(1.0, |k| ((t_s - k.at_s) / FADE_S).clamp(0.0, 1.0))
}

/// Where the figure stands at `t_s` and the time it started to fade out, when it is fading
/// out then: after the pause at a mark it leaves or the next mark appears at, for `FADE_S`.
fn fading_out(keys: &[Key<'_>], t_s: f64) -> Option<(Placement, f64)> {
    let gone = gone_on_arrival(keys);

    for (i, key) in keys.iter().enumerate() {
        let next = keys.get(i + 1);
        let exits =
            key.departure == Departure::Leave || next.is_some_and(|n| n.arrival == Arrival::Appear);
        if gone[i] || !exits {
            continue;
        }

        let left_s = next.map_or(key.at_s + key.hold_s, |n| {
            (key.at_s + key.hold_s).min(n.at_s)
        });
        let until_s = next.map_or(f64::INFINITY, |n| n.at_s);
        if t_s > left_s && t_s < (left_s + FADE_S).min(until_s) {
            return Some((key.place, left_s));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use klappe_model::{Mark, Point, Pose};

    fn mark(at_s: f64, hold_s: f64, arrival: Arrival, departure: Departure) -> Mark {
        Mark {
            at_s,
            at: Point::new(at_s, 0.0),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s,
            via: Vec::new(),
            curve: Default::default(),
            arrival,
            departure,
        }
    }

    fn opacity(marks: Vec<Mark>, t_s: f64) -> Option<f64> {
        let track = FigureTrack {
            character: "e".into(),
            marks,
        };

        presence_at(&track, t_s).map(|p| p.opacity)
    }

    fn near(got: Option<f64>, want: f64) {
        let got = got.expect("figure is there");
        assert!((got - want).abs() < 1e-9, "{got} vs {want}");
    }

    /// Appears at 4 s: nothing before, rising for `FADE_S`, full after; a seek back is the same.
    #[test]
    fn an_appearing_figure_fades_in() {
        let marks = || {
            vec![
                mark(4.0, 0.0, Arrival::Appear, Departure::Stay),
                mark(20.0, 0.0, Arrival::Walk, Departure::Stay),
            ]
        };

        assert_eq!(opacity(marks(), 3.9), None);
        near(opacity(marks(), 4.0), 0.0);
        near(opacity(marks(), 4.0 + FADE_S / 2.0), 0.5);
        near(opacity(marks(), 4.0 + FADE_S), 1.0);
        near(opacity(marks(), 10.0), 1.0);
        assert_eq!(opacity(marks(), 3.0), None);
    }

    /// Leaves after a 2 s pause at 10 s: full during it, falling after, gone at the end, still
    /// at its mark while it fades.
    #[test]
    fn a_leaving_figure_fades_out_where_it_stands() {
        let marks = || {
            vec![
                mark(0.0, 0.0, Arrival::Walk, Departure::Stay),
                mark(10.0, 2.0, Arrival::Walk, Departure::Leave),
            ]
        };
        let track = FigureTrack {
            character: "e".into(),
            marks: marks(),
        };

        near(opacity(marks(), 5.0), 1.0);
        near(opacity(marks(), 12.0), 1.0);
        near(opacity(marks(), 12.0 + FADE_S / 4.0), 0.5 + 0.25);
        assert_eq!(opacity(marks(), 12.0 + FADE_S), None);
        assert_eq!(opacity(marks(), 99.0), None);
        assert_eq!(
            presence_at(&track, 12.2).map(|p| p.place.at),
            Some(Point::new(10.0, 0.0))
        );
    }

    /// A walker whose next mark appears fades out at its pause and fades in at the next mark.
    #[test]
    fn leaving_by_a_later_appearance_fades_both_ways() {
        let marks = || {
            vec![
                mark(0.0, 1.0, Arrival::Walk, Departure::Stay),
                mark(10.0, 0.0, Arrival::Appear, Departure::Stay),
            ]
        };

        near(opacity(marks(), 1.0), 1.0);
        near(opacity(marks(), 1.0 + FADE_S / 2.0), 0.5);
        assert_eq!(opacity(marks(), 5.0), None);
        near(opacity(marks(), 10.0), 0.0);
        near(opacity(marks(), 10.0 + FADE_S), 1.0);
    }

    /// The next mark's time ends a fade-out that has not run its course.
    #[test]
    fn a_fade_out_stops_at_the_next_mark() {
        let marks = vec![
            mark(0.0, 0.0, Arrival::Walk, Departure::Leave),
            mark(0.2, 0.0, Arrival::Appear, Departure::Stay),
        ];

        near(opacity(marks.clone(), 0.1), 1.0 - 0.1 / FADE_S);
        near(opacity(marks, 0.2), 0.0);
    }

    /// A figure that only walks is drawn in full all the time.
    #[test]
    fn a_walker_is_opaque() {
        let marks = || {
            vec![
                mark(0.0, 0.0, Arrival::Walk, Departure::Stay),
                mark(10.0, 0.0, Arrival::Walk, Departure::Stay),
            ]
        };

        for t_s in [0.0, 0.25, 5.0, 10.0, 30.0] {
            near(opacity(marks(), t_s), 1.0);
        }
    }
}
