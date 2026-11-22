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
        }
    }
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

        for delta in [created, variant, renamed] {
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
