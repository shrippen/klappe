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
/// seen as a fade, but short enough that the action is not held up. A fade-out that the next mark's
/// time would cut short starts earlier instead.
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
    let fading = fading_out(&keys, t_s);
    let place = place_at(&keys, t_s).or(fading.map(|(place, _)| place))?;
    let out = fading.map_or(1.0, |(_, out)| out);

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

/// The mark the figure fades out at `t_s` and how much is left of it, 1 to 0, when it is
/// fading out then. The fade follows the pause at a mark it leaves or the next mark appears at
/// and always lasts `FADE_S`, ending at the next mark's time at the latest; where that leaves
/// less than `FADE_S` after the pause, it starts earlier, never as a jump:
///
/// ```text
///  pause ends          next mark              pause ends   next mark
///      |--- FADE_S ---|                   |-- FADE_S --|       |
///                                  starts earlier: |-- FADE_S -|
/// ```
fn fading_out(keys: &[Key<'_>], t_s: f64) -> Option<(Placement, f64)> {
    let gone = gone_on_arrival(keys);

    for (i, key) in keys.iter().enumerate() {
        let next = keys.get(i + 1);
        let exits =
            key.departure == Departure::Leave || next.is_some_and(|n| n.arrival == Arrival::Appear);
        if gone[i] || !exits {
            continue;
        }

        let ends_s = (key.at_s + key.hold_s + FADE_S).min(next.map_or(f64::INFINITY, |n| n.at_s));
        let starts_s = ends_s - FADE_S;
        if t_s > starts_s && t_s < ends_s {
            return Some((key.place, (ends_s - t_s) / FADE_S));
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

    /// The largest step of the opacity between two moments a millisecond apart, over `0..to_s`.
    fn largest_step(marks: &[Mark], to_s: f64) -> f64 {
        let mut worst = 0.0_f64;
        let mut t_s = 0.0;
        while t_s < to_s {
            let now = opacity(marks.to_vec(), t_s).unwrap_or(0.0);
            let then = opacity(marks.to_vec(), t_s + 0.001).unwrap_or(0.0);
            worst = worst.max((now - then).abs());
            t_s += 0.001;
        }

        worst
    }

    /// A fade-out never jumps, however near the next mark is: it lasts `FADE_S` and ends at the
    /// next mark, starting earlier than the pause's end when it has to.
    #[test]
    fn a_fade_out_is_never_a_jump() {
        let cases = [
            // Leaves at 10 s after 2 s, an appearance at 12 s.
            vec![
                mark(10.0, 2.0, Arrival::Walk, Departure::Leave),
                mark(12.0, 0.0, Arrival::Appear, Departure::Stay),
            ],
            // Two marks at the same time.
            vec![
                mark(10.0, 0.0, Arrival::Walk, Departure::Leave),
                mark(10.0, 0.0, Arrival::Appear, Departure::Stay),
            ],
            // The next mark comes 0.2 s after the pause.
            vec![
                mark(8.0, 2.0, Arrival::Walk, Departure::Leave),
                mark(10.2, 0.0, Arrival::Walk, Departure::Stay),
            ],
            // The old case: 0.2 s between the marks.
            vec![
                mark(10.0, 0.0, Arrival::Walk, Departure::Leave),
                mark(10.2, 0.0, Arrival::Appear, Departure::Stay),
            ],
        ];

        for marks in cases {
            let step = largest_step(&marks, 13.0);
            assert!(step < 0.01, "step {step} in {marks:?}");
        }
    }

    /// With room for `FADE_S` after the pause the fade is as before; with less it ends at the
    /// next mark and starts `FADE_S` before.
    #[test]
    fn a_short_room_moves_the_fade_forward() {
        let marks = vec![
            mark(10.0, 2.0, Arrival::Walk, Departure::Leave),
            mark(12.2, 0.0, Arrival::Appear, Departure::Stay),
        ];

        near(opacity(marks.clone(), 11.7), 1.0);
        near(opacity(marks.clone(), 11.95), 0.5);
        near(opacity(marks.clone(), 12.2), 0.0);

        let same = vec![
            mark(10.0, 0.0, Arrival::Walk, Departure::Leave),
            mark(10.0, 0.0, Arrival::Appear, Departure::Stay),
        ];
        near(opacity(same, 9.75), 0.5);
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
