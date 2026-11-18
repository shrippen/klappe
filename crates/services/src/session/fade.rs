//! Fading layers (LH-15): a layer can be drawn paler, so that another one stands out. The
//! plan, the 3D view and the printout read the strength from here, finished: the layer's fade
//! times the opacity of a figure fading in or out. They only apply the number.
//!
//! ```text
//! Layer ──► Fade (100 / 60 / 30 %) ─┬─ × item opacity ──► per item of the display list
//!                                   │                     (`item_fades`)
//!                                   └─ × source opacity ─► per pixel of the 3D view
//!                                                         (`picture_strength`)
//! ```

use std::collections::HashMap;

use klappe_stellprobe::{DisplayList, Item, Picture, SceneRole, Source};

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

/// How strongly the layer of `item` is drawn, 1 for full.
pub(super) fn layer_fade_of(item: &Item, faded: &HashMap<Layer, Fade>) -> f64 {
    let layer = layer_of(item.role, item.source.as_ref());

    faded.get(&layer).copied().unwrap_or_default().strength()
}

/// The strength of each item of `list`: its layer's fade, with the layers in `faded` drawn
/// paler, times its own opacity (a figure fading in or out, LH-26).
pub(super) fn fades_of(list: &DisplayList, faded: &HashMap<Layer, Fade>) -> Vec<f64> {
    let strength = |item: &Item| layer_fade_of(item, faded) * item.opacity;

    list.items.iter().map(strength).collect()
}

/// The strength of each item of `list` on the page of `camera`: that camera as it is, the rest
/// at most `Fade::Faint` (LH-87). The same rule as `fades_of`, with the layers set for the page.
pub(super) fn focused_fades(list: &DisplayList, camera: &str) -> Vec<f64> {
    let faint: HashMap<Layer, Fade> = Layer::ALL.into_iter().map(|l| (l, Fade::Faint)).collect();
    let strong = fades_of(list, &HashMap::new());
    let pale = fades_of(list, &faint);

    list.items
        .iter()
        .zip(strong.into_iter().zip(pale))
        .map(|(item, (strong, pale))| match &item.source {
            Some(Source::Camera(id)) if id == camera => strong,
            _ => pale,
        })
        .collect()
}

/// Every role code of a pixel fits a byte.
const ROLE_CODES: usize = 256;

/// `role_fade` of each role code, 1 for a code without a role; made once, not per pixel.
fn fade_by_code(role_fade: impl Fn(SceneRole) -> f64) -> Vec<f64> {
    (0..ROLE_CODES)
        .map(|code| {
            let role = u8::try_from(code).ok().and_then(SceneRole::of_code);

            role.map_or(1.0, &role_fade)
        })
        .collect()
}

/// The fade of the pixel's role times the opacity of its source.
fn strength_of(picture: &Picture, by_code: Vec<f64>) -> impl Fn(usize) -> f64 + '_ {
    move |i| by_code[usize::from(picture.frame.role[i])] * picture.opacity_at(i)
}

impl Session {
    /// Sets how strongly `layer` is drawn in plan, 3D view and printout. A view setting like
    /// `set_layer`, not part of the project, and no undo step.
    pub fn set_fade(&mut self, layer: Layer, fade: Fade) {
        match fade {
            Fade::Full => self.faded_layers.remove(&layer),
            other => self.faded_layers.insert(layer, other),
        };
    }

    pub fn fade(&self, layer: Layer) -> Fade {
        self.faded_layers.get(&layer).copied().unwrap_or_default()
    }

    /// The strength of each item of `list`, in its order, 1 for full: layer fade times the
    /// item's opacity. The plan and the printout draw with exactly this number.
    pub fn item_fades(&self, list: &DisplayList) -> Vec<f64> {
        fades_of(list, &self.faded_layers)
    }

    /// The strength of pixel `i` of a 3D picture: the fade of its body's layer times the
    /// opacity of its source (LH-26). The picture takes only this number.
    pub fn picture_strength<'a>(&self, picture: &'a Picture) -> impl Fn(usize) -> f64 + 'a {
        strength_of(picture, fade_by_code(|role| self.scene_fade(role)))
    }

    /// The same without layers: a camera's picture shows what is there, only a figure
    /// fading in or out is paler.
    pub fn camera_strength(picture: &Picture) -> impl Fn(usize) -> f64 + '_ {
        strength_of(picture, fade_by_code(|_| 1.0))
    }

    /// The strength of a body of the 3D view: that of the layer it belongs to.
    fn scene_fade(&self, role: SceneRole) -> f64 {
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
