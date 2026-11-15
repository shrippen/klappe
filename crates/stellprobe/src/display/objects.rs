//! The set's objects on the plan: the body of each and the caption of its height (LH-93).

use klappe_model::{Point, SetObject, SetPlan};

use super::{object_corners, Item, Role, Shape, Source};
use crate::opening::wall_items;

/// Where a height caption sits below the centre of a named object, clear of its name.
const HEIGHT_BELOW_NAME_M: f64 = 0.3;
const CM_PER_M: f64 = 100.0;

pub(super) fn set_items(set: &SetPlan, items: &mut Vec<Item>) {
    wall_items(set, items);

    for object in &set.objects {
        items.push(Item {
            role: Role::Object,
            shape: Shape::Polygon {
                points: object_corners(object),
            },
            label: Some(object.label.clone()).filter(|l| !l.is_empty()),
            source: Some(Source::Object(object.id.clone())),
        });
        items.push(height_item(object));
    }
}

/// The caption of an object's height: at its centre, below the name when it has one.
fn height_item(object: &SetObject) -> Item {
    let named = !object.label.is_empty();
    let drop_m = if named { HEIGHT_BELOW_NAME_M } else { 0.0 };

    Item {
        role: Role::Height {
            cm: object.height_m.map(|h| (h * CM_PER_M).round() as u32),
        },
        shape: Shape::Circle {
            center: Point::new(object.at.x_m, object.at.y_m + drop_m),
            radius_m: 0.0,
        },
        label: None,
        source: None,
    }
}

#[cfg(test)]
mod tests {
    use klappe_model::ObjectShape;

    use super::*;

    fn object(label: &str, height_m: Option<f64>) -> SetObject {
        SetObject {
            id: "o1".into(),
            kind: "table".into(),
            label: label.into(),
            at: Point::new(10.0, 5.0),
            width_m: 4.0,
            depth_m: 2.0,
            rotation_deg: 0.0,
            shape: ObjectShape::Rect,
            height_m,
            base_m: None,
        }
    }

    /// 1.234 m shows as 123 cm; an object without height as `None`, at its centre.
    #[test]
    fn a_height_item_rounds_to_centimetres() {
        let missing = height_item(&object("", None));
        let item = height_item(&object("", Some(1.234)));

        assert_eq!(missing.role, Role::Height { cm: None });
        assert_eq!(item.role, Role::Height { cm: Some(123) });
        assert_eq!(item.source, None);
        assert!(matches!(
            item.shape,
            Shape::Circle { center, .. } if center == Point::new(10.0, 5.0)
        ));
    }

    /// A named object's number sits below its name.
    #[test]
    fn a_named_object_has_its_height_lower() {
        let item = height_item(&object("Tisch", None));

        assert!(matches!(
            item.shape,
            Shape::Circle { center, .. } if center == Point::new(10.0, 5.0 + HEIGHT_BELOW_NAME_M)
        ));
    }

    /// Each object brings its body and then its height.
    #[test]
    fn every_object_is_followed_by_its_height() {
        let set = SetPlan {
            objects: vec![object("", Some(0.8))],
            ..Default::default()
        };
        let mut items = Vec::new();
        set_items(&set, &mut items);

        let roles: Vec<_> = items.iter().map(|i| i.role).collect();
        assert_eq!(roles, vec![Role::Object, Role::Height { cm: Some(80) }]);
    }
}
