//! The body forms of the objects in a set (LH-97): which boxes an object of a kind is made of
//! in 3D. The caller looks them up (the library knows which kind has which form); the scene
//! only reads them.

use std::collections::HashMap;

use klappe_model::FormBox;

/// The boxes of each kind of object that has a body form.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Forms {
    boxes: HashMap<String, Vec<FormBox>>,
}

impl Forms {
    /// Gives objects of `kind` the body made of `boxes`.
    pub fn insert(&mut self, kind: &str, boxes: &[FormBox]) {
        self.boxes.insert(kind.to_owned(), boxes.to_vec());
    }

    /// The boxes of an object of `kind`; `None` for a plain prism.
    pub(crate) fn get(&self, kind: &str) -> Option<&[FormBox]> {
        self.boxes.get(kind).map(Vec::as_slice)
    }
}
