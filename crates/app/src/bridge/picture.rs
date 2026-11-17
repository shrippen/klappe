//! The camera's picture for the shot card: the session's pixels, coloured with Kante's colours
//! the surface reads from the theme.

use super::qobject;
use crate::plan::{frame_json, ppm, Palette, IMAGE_FORMAT};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};
use klappe_services::{Picture, SceneRole, Source};

impl qobject::Controller {
    pub(super) fn camera_image(
        mut self: Pin<&mut Self>,
        width_px: i32,
        palette_json: &QString,
    ) -> QImage {
        self.as_mut().rust_mut().camera_frame = String::new();
        let image = self
            .rust()
            .colored_picture(width_px, &palette_json.to_string());
        let Some((image, frame)) = image else {
            return QImage::default();
        };

        self.as_mut().rust_mut().camera_frame = frame;
        image
    }

    pub(super) fn camera_image_of(
        &self,
        id: &QString,
        width_px: i32,
        palette_json: &QString,
    ) -> QImage {
        let image = self.rust().session.as_ref().and_then(|s| {
            let palette = Palette::parse(&palette_json.to_string())?;
            let width = usize::try_from(width_px).ok()?;
            let picture = s.camera_picture_of(&id.to_string(), width)?;

            colored(&picture, &palette, &|_| FULL_STRENGTH)
        });

        image.map(|(image, _)| image).unwrap_or_default()
    }

    pub(super) fn shot_image(&self, id: &QString, width_px: i32, palette_json: &QString) -> QImage {
        let image = self.rust().session.as_ref().and_then(|s| {
            let palette = Palette::parse(&palette_json.to_string())?;
            let width = usize::try_from(width_px).ok()?;
            let picture = s.shot_picture(&id.to_string(), width)?;

            colored(&picture, &palette, &|_| FULL_STRENGTH)
        });

        image.map(|(image, _)| image).unwrap_or_default()
    }

    pub(super) fn camera_spots(&self) -> QString {
        let spots = self
            .rust()
            .session
            .as_ref()
            .map(|s| s.camera_spots())
            .unwrap_or_default();

        QString::from(super::view::to_json(&spots).as_str())
    }

    pub(super) fn camera_frame(&self) -> QString {
        QString::from(self.rust().camera_frame.as_str())
    }
}

impl super::ControllerRust {
    /// The image of the selected shot's camera and the JSON of its size and names; the camera
    /// picked on the plan is only the fallback, the panel shows the view with a shot.
    fn colored_picture(&self, width_px: i32, palette_json: &str) -> Option<(QImage, String)> {
        let s = self.session.as_ref()?;
        let palette = Palette::parse(palette_json)?;
        let picked = match self.selected.only() {
            Some(Source::Camera(id)) => Some(id.as_str()),
            _ => None,
        };
        let picture = s.camera_picture(picked, usize::try_from(width_px).ok()?)?;

        colored(&picture, &palette, &|_| FULL_STRENGTH)
    }
}

/// What a camera's picture takes from a layer's fade: nothing, a camera sees what is there.
const FULL_STRENGTH: f64 = 1.0;

/// `picture` as an image in the palette's colours and the JSON of its size and names;
/// `strength` is how much of its colour a role keeps (LH-15).
pub(super) fn colored(
    picture: &Picture,
    palette: &Palette,
    strength: &dyn Fn(SceneRole) -> f64,
) -> Option<(QImage, String)> {
    let image = QImage::from_data(&ppm(picture, palette, strength), Some(IMAGE_FORMAT))?;

    Some((image, frame_json(picture)))
}
