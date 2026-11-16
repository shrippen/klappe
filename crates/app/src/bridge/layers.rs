//! Tracks, layers and their locks.

use super::qobject;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{Layer, Lock, Session, Visibility};
use serde_json::json;

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
        let Some(layer) = Layer::from_name(&layer.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_layer(layer, visibility(shown));
        }
        self.refresh();
    }

    pub(super) fn cycle_fade(mut self: Pin<&mut Self>, layer: &QString) {
        let Some(layer) = Layer::from_name(&layer.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_fade(layer, s.fade(layer).next());
        }
        self.refresh();
    }

    pub(super) fn set_lock(mut self: Pin<&mut Self>, layer: &QString, locked: &QString) {
        let Some(layer) = Layer::from_name(&layer.to_string()) else {
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

pub(super) fn layers_json(s: &Session) -> String {
    layer_flags(|l| s.layer(l) == Visibility::Shown)
}

pub(super) fn locks_json(s: &Session) -> String {
    layer_flags(|l| s.lock(l) == Lock::Locked)
}

/// The strength of each layer, by name, as the names of `Fade` (`100`, `60`, `30`).
pub(super) fn fades_json(s: &Session) -> String {
    let fades: serde_json::Map<_, _> = Layer::ALL
        .iter()
        .map(|l| (l.name().to_owned(), json!(s.fade(*l).name())))
        .collect();

    serde_json::Value::Object(fades).to_string()
}

/// One switch per layer, by name: what `layersJson` and `locksJson` hand to QML.
fn layer_flags(flag: impl Fn(Layer) -> bool) -> String {
    let flags: serde_json::Map<_, _> = Layer::ALL
        .iter()
        .map(|l| (l.name().to_owned(), json!(flag(*l))))
        .collect();

    json!(flags).to_string()
}
