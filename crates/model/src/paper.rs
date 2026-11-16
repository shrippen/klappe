use serde::{Deserialize, Serialize};

/// The sheet sizes a plan can be printed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Paper {
    #[default]
    A4,
    A3,
    Letter,
}

impl Paper {
    /// Every size, in the order the surfaces offer them.
    pub const ALL: [Paper; 3] = [Paper::A4, Paper::A3, Paper::Letter];

    /// The name the file format and the command line use, e.g. `letter`.
    pub fn name(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::A3 => "a3",
            Paper::Letter => "letter",
        }
    }

    /// The name shown to the user, e.g. `Letter`.
    pub fn label(self) -> &'static str {
        match self {
            Paper::A4 => "A4",
            Paper::A3 => "A3",
            Paper::Letter => "Letter",
        }
    }

    /// The size `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Landscape,
    Portrait,
}

impl Orientation {
    /// Both, in the order the surfaces offer them.
    pub const ALL: [Orientation; 2] = [Orientation::Portrait, Orientation::Landscape];

    /// The name the file format and the command line use, e.g. `portrait`.
    pub fn name(self) -> &'static str {
        match self {
            Orientation::Landscape => "landscape",
            Orientation::Portrait => "portrait",
        }
    }

    /// The orientation `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|o| o.name() == name)
    }
}

/// A sheet size and how it lies: what a project prints on (A4 landscape if never chosen).
/// In files it is the one word `a4-portrait`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct PaperSetup {
    pub size: Paper,
    pub orientation: Orientation,
}

impl PaperSetup {
    /// `a4-portrait`: size and orientation as one word, the way surfaces hand it over.
    pub fn name(self) -> String {
        format!("{}-{}", self.size.name(), self.orientation.name())
    }

    /// The setup `name` names (`a4-portrait`). A part that is missing or unknown stays at its
    /// default, so `a3` is A3 landscape and anything else is A4 landscape.
    pub fn from_name(name: &str) -> Self {
        let (size, orientation) = name.split_once('-').unwrap_or((name, ""));

        Self {
            size: Paper::from_name(size).unwrap_or_default(),
            orientation: Orientation::from_name(orientation).unwrap_or_default(),
        }
    }
}

impl From<String> for PaperSetup {
    fn from(name: String) -> Self {
        Self::from_name(&name)
    }
}

impl From<PaperSetup> for String {
    fn from(setup: PaperSetup) -> Self {
        setup.name()
    }
}

impl From<Paper> for PaperSetup {
    /// A size alone lies landscape.
    fn from(size: Paper) -> Self {
        Self {
            size,
            orientation: Orientation::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for size in Paper::ALL {
            for orientation in Orientation::ALL {
                let setup = PaperSetup { size, orientation };

                assert_eq!(PaperSetup::from_name(&setup.name()), setup);
            }
        }
    }

    #[test]
    fn a_missing_or_unknown_part_is_the_default() {
        let a3 = PaperSetup::from(Paper::A3);

        assert_eq!(PaperSetup::from_name("a3"), a3);
        assert_eq!(PaperSetup::from_name("a3-sideways"), a3);
        assert_eq!(PaperSetup::from_name("tabloid-portrait").size, Paper::A4);
        assert_eq!(PaperSetup::from_name(""), PaperSetup::default());
    }
}
