//! Fading layers (LH-15): a layer can be drawn paler, so that another one stands out. The
//! plan, the 3D view and the printout read the strength from here.
//!
//! ```text
//! Layer ──► Fade (100 / 60 / 30 %) ──► strength per item of the display list (`item_fades`)
//!                                  └─► strength per body of the 3D view (`scene_fade`)
//! ```

use std::collections::HashMap;

use klappe_stellprobe::{DisplayList, Item, SceneRole};

use super::layers::layer_of;
use super::{Layer, Session};

/// How strongly a layer is drawn. Steps, not a slider: the same three everywhere, so a
/// printout and the screen agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fade {
    #[default]
    Full,
    Medium,
    Faint,
}

impl Fade {
    /// Every step, strongest first, in the order the layer list offers them.
    pub const ALL: [Fade; 3] = [Fade::Full, Fade::Medium, Fade::Faint];

    /// How QML calls the step; the same as its share in percent.
    pub fn name(self) -> &'static str {
        match self {
            Fade::Full => "100",
            Fade::Medium => "60",
            Fade::Faint => "30",
        }
    }

    /// The share of the colour that is left, 1 for full.
    pub fn strength(self) -> f64 {
        match self {
            Fade::Full => 1.0,
            Fade::Medium => 0.6,
            Fade::Faint => 0.3,
        }
    }

    /// The next step down; after the faintest, full again.
    pub fn next(self) -> Self {
        match self {
            Fade::Full => Fade::Medium,
            Fade::Medium => Fade::Faint,
            Fade::Faint => Fade::Full,
        }
    }
}

/// The strength of each item of `list` with the layers in `faded` drawn paler.
pub(super) fn fades_of(list: &DisplayList, faded: &HashMap<Layer, Fade>) -> Vec<f64> {
    let strength = |item: &Item| {
        let layer = layer_of(item.role, item.source.as_ref());

        faded.get(&layer).copied().unwrap_or_default().strength()
    };

    list.items.iter().map(strength).collect()
}

impl Session {
    /// Sets how strongly `layer` is drawn in plan, 3D view and printout. A view setting like
    /// `set_layer`, not part of the project, and no undo step.
    pub fn set_fade(&mut self, layer: Layer, fade: Fade) {
        match fade {
            Fade::Full => self.plan_view.faded_layers.remove(&layer),
            other => self.plan_view.faded_layers.insert(layer, other),
        };
    }

    pub fn fade(&self, layer: Layer) -> Fade {
        self.plan_view
            .faded_layers
            .get(&layer)
            .copied()
            .unwrap_or_default()
    }

    /// The strength of each item of `list`, in its order, 1 for full.
    pub fn item_fades(&self, list: &DisplayList) -> Vec<f64> {
        fades_of(list, &self.plan_view.faded_layers)
    }

    /// The strength of a body of the 3D view: that of the layer it belongs to.
    pub fn scene_fade(&self, role: SceneRole) -> f64 {
        let layer = match role {
            SceneRole::Floor | SceneRole::Wall | SceneRole::Object | SceneRole::Missing => {
                Layer::Set
            }
            SceneRole::Figure(_) => Layer::Figures,
            SceneRole::Camera => Layer::Cameras,
        };

        self.fade(layer).strength()
    }
}
