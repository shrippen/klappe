//! Joins the project's scripted locations to the scene headings of the book (DB-51, DATENMODELL
//! §4): which locations the headings name that no scripted location answers to, and what the
//! user can make of each. Nothing here becomes a scripted location on its own: a misspelling is
//! a location too.

use klappe_model::{random_id, Place, Project};
use klappe_script::{heading_place, is_omitted, Element, Script};

/// A place name of the headings no scripted location is called by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPlace {
    /// As the book spells it the first time.
    pub name: String,
    /// How many scene headings name it.
    pub headings: usize,
}

/// What the user makes of a place name of the headings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceChoice {
    /// A new scripted location of this name.
    Create,
    /// Another spelling of the scripted location with this id.
    Variant(String),
    /// The scripted location with this id is called so now (the book renamed it): its name
    /// changes, its id stays.
    Rename(String),
}

/// The location of every scene heading of the book, in book order.
fn heading_places(script: &Script) -> Vec<&str> {
    script
        .elements
        .iter()
        .filter_map(|e| match e {
            Element::SceneHeading { text, .. } if !is_omitted(text) => Some(heading_place(text)),
            _ => None,
        })
        .filter(|place| !place.is_empty())
        .collect()
}

/// The place names of the headings, each once, that no scripted location is called by, in book
/// order.
pub(crate) fn pending(project: &Project, script: &Script) -> Vec<PendingPlace> {
    let mut found: Vec<PendingPlace> = Vec::new();

    for place in heading_places(script) {
        if project.place_named(place).is_some() {
            continue;
        }
        let same = |p: &&mut PendingPlace| klappe_model::same_place_name(&p.name, place);
        match found.iter_mut().find(same) {
            Some(known) => known.headings += 1,
            None => found.push(PendingPlace {
                name: place.to_owned(),
                headings: 1,
            }),
        }
    }

    found
}

/// The ids of the scripted locations the headings do not name any more: a rename outside the
/// program leaves the old name without a heading.
pub(crate) fn unused(places: &[Place], script: &Script) -> Vec<String> {
    let named = heading_places(script);

    places
        .iter()
        .filter(|p| !named.iter().any(|n| p.is_called(n)))
        .map(|p| p.id.clone())
        .collect()
}

/// Settles the location `name` in `places` as `choice`; `false` when `choice` names no play
/// location or `name` already belongs to one (a name or variant belongs to one place).
pub(crate) fn decide(places: &mut Vec<Place>, name: &str, choice: &PlaceChoice) -> bool {
    let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() || places.iter().any(|p| p.is_called(&name)) {
        return false;
    }

    match choice {
        PlaceChoice::Create => {
            let id = free_id(places);
            places.push(Place {
                id,
                name,
                variants: Vec::new(),
            });

            true
        }
        PlaceChoice::Variant(id) => {
            let Some(place) = places.iter_mut().find(|p| p.id == *id) else {
                return false;
            };
            place.variants.push(name);

            true
        }
        PlaceChoice::Rename(id) => {
            let Some(place) = places.iter_mut().find(|p| p.id == *id) else {
                return false;
            };
            place.name = name;

            true
        }
    }
}

fn free_id(places: &[Place]) -> String {
    loop {
        let id = random_id();
        if !places.iter().any(|p| p.id == id) {
            return id;
        }
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
    fn a_name_of_another_place_is_refused() {
        let mut places = vec![place("a", "SÄNGERSAAL", &["SAAL"]), place("b", "HOF", &[])];

        for name in ["saal", "Sängersaal", " HOF "] {
            assert!(!decide(
                &mut places,
                name,
                &PlaceChoice::Variant("b".into())
            ));
            assert!(!decide(&mut places, name, &PlaceChoice::Create));
        }
        assert_eq!(places[1].variants, Vec::<String>::new());
        assert!(decide(
            &mut places,
            "GARTEN",
            &PlaceChoice::Variant("b".into())
        ));
    }

    #[test]
    fn sharp_s_and_ss_are_one_name() {
        let mut places = vec![place("a", "STRAßE", &[])];

        assert!(!decide(&mut places, "Strasse", &PlaceChoice::Create));
        assert!(!decide(&mut places, "STRASSE", &PlaceChoice::Create));
    }

    #[test]
    fn a_rename_keeps_the_id_and_the_variants() {
        let mut places = vec![place("a", "SÄNGERSAAL", &["SAAL"])];

        assert!(decide(
            &mut places,
            "FESTSAAL",
            &PlaceChoice::Rename("a".into())
        ));

        assert_eq!(places, [place("a", "FESTSAAL", &["SAAL"])]);
        assert!(!decide(
            &mut places,
            "X",
            &PlaceChoice::Rename("none".into())
        ));
    }
}
