//! Tracks, layers and their locks.

use super::qobject;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{Layer, Lock, Visibility};

const SHOWN: &str = "shown";
const LOCKED: &str = "locked";

impl qobject::Controller {
    pub(super) fn set_track(mut self: Pin<&mut Self>, id: &QString, shown: &QString) {
        if let Some(s) = self.as_mut().session() {
            s.set_track(&id.to_string(), visibility(shown));
        }
        self.refresh();
    }

    pub(super) fn set_layer(mut self: Pin<&mut Self>, layer: &QString, shown: &QString) {
        let Some(layer) = layer_of(&layer.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_layer(layer, visibility(shown));
        }
        self.refresh();
    }

    pub(super) fn set_lock(mut self: Pin<&mut Self>, layer: &QString, locked: &QString) {
        let Some(layer) = layer_of(&layer.to_string()) else {
            return;
        };
        let lock = match locked.to_string() == LOCKED {
            true => Lock::Locked,
            false => Lock::Unlocked,
        };
        if let Some(s) = self.as_mut().session() {
            s.set_lock(layer, lock);
        }
        self.refresh();
    }
}

fn visibility(text: &QString) -> Visibility {
    if text.to_string() == SHOWN {
        return Visibility::Shown;
    }
    Visibility::Hidden
}

fn layer_of(name: &str) -> Option<Layer> {
    match name {
        "set" => Some(Layer::Set),
        "figures" => Some(Layer::Figures),
        "cameras" => Some(Layer::Cameras),
        "lights" => Some(Layer::Lights),
        "notes" => Some(Layer::Notes),
        "drawings" => Some(Layer::Drawings),
        _ => None,
    }
}
