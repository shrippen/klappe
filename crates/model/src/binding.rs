//! What a bound note names (LH-118), and the text the project file keeps for it.
//!
//! ```text
//!  object:<id>                  an object of the set
//!  figure:<character>           a figure of a plan
//!  mark:<character>:<index>     a mark of a figure
//! ```

/// The element a note is a label of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Object(String),
    Figure(String),
    Mark { character: String, index: usize },
}

impl Binding {
    /// Reads the text of `Note::bound`; `None` for an unknown kind or an empty id.
    pub fn parse(text: &str) -> Option<Self> {
        let (kind, rest) = text.split_once(':')?;

        match kind {
            "object" if !rest.is_empty() => Some(Self::Object(rest.to_owned())),
            "figure" if !rest.is_empty() => Some(Self::Figure(rest.to_owned())),
            "mark" => {
                let (character, index) = rest.rsplit_once(':')?;

                if character.is_empty() {
                    return None;
                }

                Some(Self::Mark {
                    character: character.to_owned(),
                    index: index.parse().ok()?,
                })
            }
            _ => None,
        }
    }

    /// The text of `Note::bound`.
    pub fn key(&self) -> String {
        match self {
            Self::Object(id) => format!("object:{id}"),
            Self::Figure(character) => format!("figure:{character}"),
            Self::Mark { character, index } => format!("mark:{character}:{index}"),
        }
    }

    /// Whether a label of a plan may have it; an object's label is a note of the set.
    pub fn is_plan(&self) -> bool {
        !matches!(self, Self::Object(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_reads_back() {
        let all = [
            Binding::Object("tisch".into()),
            Binding::Figure("elisabeth".into()),
            Binding::Mark {
                character: "elisabeth".into(),
                index: 2,
            },
            Binding::Mark {
                character: "a:b".into(),
                index: 0,
            },
        ];

        for binding in all {
            assert_eq!(Binding::parse(&binding.key()), Some(binding));
        }
    }

    #[test]
    fn garbled_text_is_none() {
        for text in [
            "", "object", "object:", "figure:", "mark::0", "mark:a", "mark:a:x", "wall:1",
        ] {
            assert_eq!(Binding::parse(text), None, "{text}");
        }
    }

    #[test]
    fn only_an_object_is_not_a_plan_binding() {
        assert!(!Binding::Object("o".into()).is_plan());
        assert!(Binding::Figure("f".into()).is_plan());
    }
}
