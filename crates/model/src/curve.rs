//! The shape of a way (LH-108).

use serde::{Deserialize, Serialize};

/// The shape of a way through its bends (LH-108).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Curve {
    /// A smooth (Catmull-Rom) curve through the bends.
    #[default]
    Smooth,
    /// Straight pieces from bend to bend, with corners: a rail, a corridor.
    Straight,
}

impl Curve {
    /// Every curve.
    pub const ALL: [Curve; 2] = [Curve::Smooth, Curve::Straight];

    /// The name the file format uses, e.g. `straight`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            Curve::Smooth => "smooth",
            Curve::Straight => "straight",
        }
    }

    /// The curve `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// For `skip_serializing_if`: a smooth way is not written.
pub(crate) fn is_smooth(curve: &Curve) -> bool {
    *curve == Curve::Smooth
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `name` is what the file holds and `from_name` reads it back.
    #[test]
    fn curve_names_are_the_file_names() {
        for curve in Curve::ALL {
            let json = serde_json::to_string(&curve).unwrap();
            assert_eq!(json, format!("\"{}\"", curve.name()));
            assert_eq!(Curve::from_name(curve.name()), Some(curve));
        }
        assert_eq!(Curve::from_name("wavy"), None);
    }
}
