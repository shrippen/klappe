//! The photos and sketches placed on the plan (LH-18): the picture and its frame.

use klappe_model::{PlanImage, SetPlan};

use super::{Item, Role, Shape, Source};

/// The aspect of the frame (depth to width) for a file whose size is not known: a missing or
/// unreadable one is drawn as an empty 4:3 frame with its name.
const GAP_ASPECT: f64 = 0.75;

/// The corners of the image's box, clockwise from the top left, turned about its centre.
pub(super) fn image_corners(image: &PlanImage, depth_m: f64) -> [klappe_model::Point; 4] {
    let (half_w, half_d) = (image.width_m / 2.0, depth_m / 2.0);

    [
        image.placed(-half_w, -half_d),
        image.placed(half_w, -half_d),
        image.placed(half_w, half_d),
        image.placed(-half_w, half_d),
    ]
}

/// What `image_px` tells of the files of the set's images: for each one the picture (when its
/// size is known) and a frame that carries the element, so a click takes it. Without a size
/// the frame stands alone and names the file. The picture has no source: the frame is the
/// element; a picture is only looked at.
pub(super) fn image_items(
    set: &SetPlan,
    image_px: &dyn Fn(&str) -> Option<(u32, u32)>,
    items: &mut Vec<Item>,
) {
    for image in &set.images {
        let size = image_px(&image.file).filter(|(w, h)| *w > 0 && *h > 0);
        let aspect = size.map_or(GAP_ASPECT, |(w, h)| f64::from(h) / f64::from(w));
        let depth_m = image.width_m * aspect;
        let corners = image_corners(image, depth_m);

        if size.is_some() {
            items.push(Item {
                role: Role::Picture,
                shape: Shape::Image {
                    file: image.file.clone(),
                    at: corners[0],
                    width_m: image.width_m,
                    depth_m,
                    rotation_deg: image.rotation_deg,
                    opacity: image.opacity,
                },
                label: None,
                source: None,
                opacity: crate::display::FULL_OPACITY,
            });
        }

        items.push(Item {
            role: Role::Picture,
            shape: Shape::Polygon {
                points: corners.to_vec(),
            },
            label: size.is_none().then(|| image.file.clone()),
            source: Some(Source::Image(image.id.clone())),
            opacity: crate::display::FULL_OPACITY,
        });
    }
}
