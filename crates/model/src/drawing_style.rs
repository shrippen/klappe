use serde::{Deserialize, Serialize};

/// How a line is drawn; the dash tells drawings apart where colour does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dash {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl Dash {
    /// Every dash, in the order the tool bar offers.
    pub const ALL: [Dash; 3] = [Dash::Solid, Dash::Dashed, Dash::Dotted];

    /// The name the file format uses, e.g. `dashed`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Dash::Solid => "solid",
            Dash::Dashed => "dashed",
            Dash::Dotted => "dotted",
        }
    }

    /// The dash `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.name() == name)
    }
}

/// Whether the inside of a closed shape is tinted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fill {
    #[default]
    Outline,
    Filled,
}

impl Fill {
    /// Every fill, in the order the tool bar offers.
    pub const ALL: [Fill; 2] = [Fill::Outline, Fill::Filled];

    /// The name the file format uses, e.g. `filled`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Fill::Outline => "outline",
            Fill::Filled => "filled",
        }
    }

    /// The fill `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == name)
    }
}
