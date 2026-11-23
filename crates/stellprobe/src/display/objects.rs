//! The set's objects on the plan: the body of each and the caption of its height (LH-93).

use klappe_model::{
    outline_box, outline_label_point, FigureColor, FloorPlan, ObjectShape, Point, SetObject,
    SetPlan,
};

use super::{Item, Role, Shape, Source, ELLIPSE_CORNERS, MARK_RADIUS_M};
use crate::opening::wall_items;

/// Where a height caption sits below the centre of a named object, clear of its name.
const HEIGHT_BELOW_NAME_M: f64 = 0.3;
const CM_PER_M: f64 = 100.0;

/// The outline of a set object, rotated about its centre: four corners for a rectangle, the
/// ellipse inscribed in that rectangle, a free form's points stretched to the size.
pub(crate) fn object_corners(o: &SetObject) -> Vec<Point> {
    let (hw, hd) = (o.width_m / 2.0, o.depth_m / 2.0);
    let local = match &o.shape {
        ObjectShape::Rect => vec![(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)],
        ObjectShape::Ellipse => (0..ELLIPSE_CORNERS)
            .map(|i| {
                let turn = std::f64::consts::TAU * i as f64 / ELLIPSE_CORNERS as f64;
                (hw * turn.cos(), hd * turn.sin())
            })
            .collect(),
        ObjectShape::Polygon { points } => stretched(points, o.width_m, o.depth_m),
    };

    local.iter().map(|(x, y)| o.placed(*x, *y)).collect()
}

/// A free form's `points` scaled from the size of their own box to `width_m` x `depth_m`, so a
/// resize of the object stretches the outline. A box without width or depth keeps its scale.
fn stretched(points: &[Point], width_m: f64, depth_m: f64) -> Vec<(f64, f64)> {
    let (_, own_w, own_d) = outline_box(points);
    let scale = |size_m: f64, own_m: f64| if own_m > 0.0 { size_m / own_m } else { 1.0 };
    let (sx, sy) = (scale(width_m, own_w), scale(depth_m, own_d));

    points.iter().map(|p| (p.x_m * sx, p.y_m * sy)).collect()
}

pub(super) fn set_items(set: &SetPlan, items: &mut Vec<Item>) {
    wall_items(set, items);

    for object in &set.objects {
        let name = Some(object.label.clone()).filter(|l| !l.is_empty());
        let free = matches!(object.shape, ObjectShape::Polygon { .. });

        // The canvas writes a body's name in the middle of its box; a free form's may lie
        // outside it (a U), so its name is a caption of its own at a point inside.
        items.push(Item {
            role: Role::Object,
            shape: Shape::Polygon {
                points: object_corners(object),
            },
            label: name.clone().filter(|_| !free),
            source: Some(Source::Object(object.id.clone())),
            opacity: crate::display::FULL_OPACITY,
        });
        if free && name.is_some() {
            items.push(name_item(object, name));
        }
        items.push(height_item(object));
    }
}

/// The colour an object's marks and way have: no figure's, a prop is not a person.
const TRACK_COLOR: FigureColor = FigureColor::Cyan;

/// The marks of the objects that move (LH-24), numbered 1, 2, 3 …, and their ways: a way
/// leads to a mark unless the object is carried from the mark before, then the figure's
/// path is its way.
pub(super) fn track_items(set: &SetPlan, plan: &FloorPlan, items: &mut Vec<Item>) {
    for track in &plan.object_tracks {
        // The object may be gone from the set another scene shares.
        if !set.objects.iter().any(|o| o.id == track.object) {
            continue;
        }

        let source = |index| Source::ObjectMark {
            object: track.object.clone(),
            index,
        };
        let carried: Vec<bool> = track.marks.iter().map(|m| m.carried_by.is_some()).collect();
        let marks: Vec<_> = track.marks.iter().enumerate().collect();

        for way in marks.chunk_by(|(a, _), _| !carried[*a]) {
            if way.len() < 2 {
                continue;
            }

            items.push(Item {
                role: Role::FigurePath(TRACK_COLOR),
                shape: Shape::Polyline {
                    points: way.iter().map(|(_, m)| m.at).collect(),
                },
                label: None,
                source: Some(Source::Object(track.object.clone())),
                opacity: crate::display::FULL_OPACITY,
            });
        }

        for (index, mark) in marks {
            items.push(Item {
                role: Role::Mark(TRACK_COLOR),
                shape: Shape::Circle {
                    center: mark.at,
                    radius_m: MARK_RADIUS_M,
                },
                label: Some((index + 1).to_string()),
                source: Some(source(index)),
                opacity: crate::display::FULL_OPACITY,
            });
        }
    }
}

/// A free form's name at its inside point, with no shape to draw and nothing to pick.
fn name_item(object: &SetObject, name: Option<String>) -> Item {
    Item {
        role: Role::Object,
        shape: Shape::Circle {
            center: label_point(object),
            radius_m: 0.0,
        },
        label: name,
        source: None,
        opacity: crate::display::FULL_OPACITY,
    }
}

/// Where an object's height caption and the 3D aim go: its centre, but for a free form a point
/// inside the outline (the centre of a U lies in its gap).
pub fn label_point(object: &SetObject) -> Point {
    match object.shape {
        ObjectShape::Polygon { .. } => outline_label_point(&object_corners(object)),
        _ => object.at,
    }
}

/// The caption of an object's height: at its label point, below the name when it has one.
fn height_item(object: &SetObject) -> Item {
    let named = !object.label.is_empty();
    let drop_m = if named { HEIGHT_BELOW_NAME_M } else { 0.0 };
    let at = label_point(object);

    Item {
        role: Role::Height {
            cm: object.height_m.map(|h| (h * CM_PER_M).round() as u32),
        },
        shape: Shape::Circle {
            center: Point::new(at.x_m, at.y_m + drop_m),
            radius_m: 0.0,
        },
        label: None,
        source: None,
        opacity: crate::display::FULL_OPACITY,
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
            mirrored: false,
            element: None,
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

    /// A notched free form (a U): the caption sits inside it, not in the gap at the box middle.
    #[test]
    fn a_free_form_caption_lies_inside_the_outline() {
        let points: Vec<Point> = [
            (-1.5, -1.5),
            (-0.5, -1.5),
            (-0.5, 0.5),
            (0.5, 0.5),
            (0.5, -1.5),
            (1.5, -1.5),
            (1.5, 1.5),
            (-1.5, 1.5),
        ]
        .iter()
        .map(|&(x, y)| Point::new(x, y))
        .collect();
        let mut form = object("", None);
        form.shape = ObjectShape::Polygon { points };
        form.width_m = 3.0;
        form.depth_m = 3.0;

        let inside = label_point(&form);
        assert_eq!(inside, Point::new(9.0, 5.0));

        let item = height_item(&form);
        assert!(matches!(item.shape, Shape::Circle { center, .. } if center == inside));
    }

    /// A named U: the canvas writes a body's name in the middle of its box, in the gap, so a
    /// free form's name comes as a caption of its own at the inside point, the height below.
    #[test]
    fn a_named_free_form_has_its_name_inside() {
        let points: Vec<Point> = [
            (-1.5, -1.5),
            (-0.5, -1.5),
            (-0.5, 0.5),
            (0.5, 0.5),
            (0.5, -1.5),
            (1.5, -1.5),
            (1.5, 1.5),
            (-1.5, 1.5),
        ]
        .iter()
        .map(|&(x, y)| Point::new(x, y))
        .collect();
        let mut form = object("Tresen", Some(1.1));
        form.shape = ObjectShape::Polygon { points };
        form.width_m = 3.0;
        form.depth_m = 3.0;
        let set = SetPlan {
            objects: vec![form],
            ..Default::default()
        };
        let mut items = Vec::new();
        set_items(&set, &mut items);

        let inside = Point::new(9.0, 5.0);
        assert_eq!(items[0].label, None);
        assert_eq!(items[1].label.as_deref(), Some("Tresen"));
        assert_eq!(items[1].source, None);
        assert!(matches!(items[1].shape, Shape::Circle { center, .. } if center == inside));
        let below = Point::new(9.0, 5.0 + HEIGHT_BELOW_NAME_M);
        assert!(matches!(items[2].shape, Shape::Circle { center, .. } if center == below));
    }

    /// A rectangle keeps its centre.
    #[test]
    fn a_rect_keeps_its_centre() {
        assert_eq!(label_point(&object("", None)), Point::new(10.0, 5.0));
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

    /// A free form of 2 x 3 m in its own box, set to 4 x 3 m, turned a quarter: its points are
    /// stretched by 2 in x, then turned about the centre.
    #[test]
    fn a_free_form_is_stretched_and_turned() {
        let points = [(-1.0, -1.5), (1.0, -1.5), (1.0, 1.5)];
        let mut o = object("", None);
        o.at = Point::new(10.0, 5.0);
        o.width_m = 4.0;
        o.depth_m = 3.0;
        o.shape = ObjectShape::Polygon {
            points: points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
        };

        let upright = object_corners(&o);
        assert_eq!(
            upright,
            vec![
                Point::new(8.0, 3.5),
                Point::new(12.0, 3.5),
                Point::new(12.0, 6.5)
            ]
        );

        o.rotation_deg = 90.0;
        let turned = object_corners(&o);
        let want = [(11.5, 3.0), (11.5, 7.0), (8.5, 7.0)];
        for (got, (x, y)) in turned.iter().zip(want) {
            assert!(
                (got.x_m - x).abs() < 1e-9 && (got.y_m - y).abs() < 1e-9,
                "{got:?} is not ({x}, {y})"
            );
        }
    }
}
