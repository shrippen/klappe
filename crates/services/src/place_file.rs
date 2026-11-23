//! `orte.toml` against an answer (DB-51): the one entry an answer to a place name changed, kept
//! to do it again or take it back. The file can change under the session (the window beside
//! the TUI), so an answer, its undo and its redo change only their entry, in the file as it is
//! on disk.

use klappe_model::{same_place_name, Place};

use crate::figure_file::Way;
use crate::places::PlaceChoice;

/// The one entry of `orte.toml` an answer to a place name wrote.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PlaceDelta {
    Created(Place),
    Variant {
        id: String,
        name: String,
    },
    Renamed {
        id: String,
        old: String,
        new: String,
    },
    /// Changed in the list of scripted locations (D-21): name, variants, shooting location.
    Edited {
        before: Place,
        after: Place,
    },
    Removed(Place),
}

impl PlaceDelta {
    /// The entry that answering with `choice` wrote: `before` and `after` are the places
    /// without and with the answer.
    pub(crate) fn of(before: &[Place], choice: &PlaceChoice, after: &[Place]) -> Option<Self> {
        match choice {
            PlaceChoice::Create => after
                .iter()
                .find(|p| !before.iter().any(|b| b.id == p.id))
                .cloned()
                .map(Self::Created),
            PlaceChoice::Variant(id) => {
                let name = after.iter().find(|p| p.id == *id)?.variants.last()?.clone();

                Some(Self::Variant {
                    id: id.clone(),
                    name,
                })
            }
            PlaceChoice::Rename(id) => {
                let old = before.iter().find(|p| p.id == *id)?.name.clone();
                let new = after.iter().find(|p| p.id == *id)?.name.clone();

                Some(Self::Renamed {
                    id: id.clone(),
                    old,
                    new,
                })
            }
        }
    }

    /// Changes `places` by this delta, in `way`; an entry already (not) there stays as it is.
    /// A name another place is called by is not taken (a name belongs to one place).
    pub(crate) fn run(&self, places: &mut Vec<Place>, way: Way) {
        match (self, way) {
            (Self::Created(p), Way::Apply) => {
                let taken = places.iter().any(|o| o.id == p.id || o.is_called(&p.name));
                if !taken {
                    places.push(p.clone());
                }
            }
            (Self::Created(p), Way::Revert) => places.retain(|o| o.id != p.id),
            (Self::Variant { id, name }, Way::Apply) => {
                if places.iter().any(|o| o.is_called(name)) {
                    return;
                }
                if let Some(p) = places.iter_mut().find(|p| p.id == *id) {
                    p.variants.push(name.clone());
                }
            }
            (Self::Variant { id, name }, Way::Revert) => {
                if let Some(p) = places.iter_mut().find(|p| p.id == *id) {
                    p.variants.retain(|v| !same_place_name(v, name));
                }
            }
            (Self::Renamed { id, old, new }, way) => rename(places, id, (old, new), way),
            (Self::Edited { .. } | Self::Removed(_), way) => self.run_listed(places, way),
        }
    }

    /// `run` for a change in the list of scripted locations (D-21).
    fn run_listed(&self, places: &mut Vec<Place>, way: Way) {
        match (self, way) {
            (Self::Edited { after, .. }, Way::Apply) => replace(places, after),
            (Self::Edited { before, .. }, Way::Revert) => replace(places, before),
            (Self::Removed(p), Way::Apply) => places.retain(|o| o.id != p.id),
            (Self::Removed(p), Way::Revert) => {
                let taken = places.iter().any(|o| o.id == p.id || calls_alike(o, p));
                if !taken {
                    places.push(p.clone());
                }
            }
            _ => {}
        }
    }
}

/// Puts `place` in for the entry of its id, unless another place is called by one of its
/// names (a name belongs to one place); a gone entry stays gone.
fn replace(places: &mut [Place], place: &Place) {
    if places
        .iter()
        .any(|o| o.id != place.id && calls_alike(o, place))
    {
        return;
    }
    if let Some(entry) = places.iter_mut().find(|p| p.id == place.id) {
        *entry = place.clone();
    }
}

/// Whether `a` is called by a name or variant of `b`.
fn calls_alike(a: &Place, b: &Place) -> bool {
    std::iter::once(&b.name)
        .chain(&b.variants)
        .any(|n| a.is_called(n))
}

/// Applies or reverts the rename of place `id` from `old` to `new`: the name changes only if
/// the place still has the one the rename leaves, and no other place is called the one it takes.
fn rename(places: &mut [Place], id: &str, (old, new): (&str, &str), way: Way) {
    let (from, to) = match way {
        Way::Apply => (old, new),
        Way::Revert => (new, old),
    };
    if places.iter().any(|o| o.id != id && o.is_called(to)) {
        return;
    }
    let Some(place) = places.iter_mut().find(|p| p.id == id) else {
        return;
    };
    if same_place_name(&place.name, from) {
        place.name = to.to_owned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(id: &str, name: &str, variants: &[&str]) -> Place {
        Place {
            id: id.into(),
            name: name.into(),
            variants: variants.iter().map(|v| (*v).into()).collect(),
            location: None,
        }
    }

    #[test]
    fn each_delta_is_idempotent_both_ways() {
        let created = PlaceDelta::Created(place("b", "HOF", &[]));
        let variant = PlaceDelta::Variant {
            id: "a".into(),
            name: "SAAL".into(),
        };
        let renamed = PlaceDelta::Renamed {
            id: "a".into(),
            old: "SÄNGERSAAL".into(),
            new: "FESTSAAL".into(),
        };

        let edited = PlaceDelta::Edited {
            before: place("a", "SÄNGERSAAL", &[]),
            after: Place {
                location: Some("burg".into()),
                ..place("a", "FESTSAAL", &["SÄNGERSAAL"])
            },
        };
        let removed = PlaceDelta::Removed(place("a", "SÄNGERSAAL", &[]));

        for delta in [created, variant, renamed, edited, removed] {
            let start = vec![place("a", "SÄNGERSAAL", &[])];
            let mut places = start.clone();

            delta.run(&mut places, Way::Apply);
            let done = places.clone();
            assert_ne!(done, start);
            delta.run(&mut places, Way::Apply);
            assert_eq!(places, done);

            delta.run(&mut places, Way::Revert);
            assert_eq!(places, start);
            delta.run(&mut places, Way::Revert);
            assert_eq!(places, start);
        }
    }

    #[test]
    fn a_rename_is_not_taken_back_over_a_newer_name() {
        let delta = PlaceDelta::Renamed {
            id: "a".into(),
            old: "SAAL".into(),
            new: "FESTSAAL".into(),
        };
        let mut places = vec![place("a", "BALLSAAL", &[])];

        delta.run(&mut places, Way::Revert);

        assert_eq!(places[0].name, "BALLSAAL");
    }

    /// Review of PR 313: the revert put the old name over another place's.
    #[test]
    fn a_rename_is_not_taken_back_onto_a_name_another_place_has() {
        let delta = PlaceDelta::Renamed {
            id: "a".into(),
            old: "SÄNGERSAAL".into(),
            new: "FESTSAAL".into(),
        };
        let mut places = vec![
            place("a", "FESTSAAL", &[]),
            place("b", "HOF", &["SÄNGERSAAL"]),
        ];

        delta.run(&mut places, Way::Revert);

        assert_eq!(places[0].name, "FESTSAAL");
    }
}
