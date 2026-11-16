//! Placed images (LH-18) in the display list: the picture, the frame that is the element,
//! the grip.
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, PlanImage, Point, Project, SetPlan};
use klappe_stellprobe::{handles, hit, render, Role, Shape, Source};

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn set(rotation_deg: f64) -> SetPlan {
    SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        images: vec![PlanImage {
            id: "i1".into(),
            file: "bilder/a.png".into(),
            at: p(10.0, 6.0),
            width_m: 4.0,
            rotation_deg,
        }],
        ..Default::default()
    }
}

fn list(set: &SetPlan, px: Option<(u32, u32)>) -> klappe_stellprobe::DisplayList {
    let plan = FloorPlan::new("a", "A");

    render(&Project::new("T"), set, &plan, 0.0, None, &|_| px)
}

#[test]
fn a_known_size_gives_the_picture_and_the_frame() {
    let list = list(&set(0.0), Some((400, 100)));

    // 400 x 100 px at 4 m wide: 1 m deep, from the top left corner (8, 5.5).
    assert_eq!(list.items.len(), 2);
    assert_eq!(list.items[0].role, Role::Picture);
    assert_eq!(list.items[0].source, None);
    assert_eq!(
        list.items[0].shape,
        Shape::Image {
            file: "bilder/a.png".into(),
            at: p(8.0, 5.5),
            width_m: 4.0,
            depth_m: 1.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        }
    );
    assert_eq!(list.items[1].source, Some(Source::Image("i1".into())));
    assert_eq!(list.items[1].label, None);
    assert!(hit(&list.items[1], p(10.0, 6.0), 0.0));
    assert!(!hit(&list.items[1], p(10.0, 7.0), 0.0));
}

/// The corner the picture is drawn from turns with the image about its centre.
#[test]
fn a_turn_swings_the_corner_about_the_centre() {
    let list = list(&set(90.0), Some((400, 100)));

    let Shape::Image {
        at, rotation_deg, ..
    } = &list.items[0].shape
    else {
        assert_eq!(list.items[0].role, Role::Background);
        return;
    };
    assert!(at.distance_m(p(10.5, 4.0)) < 1e-9, "{at:?}");
    assert_eq!(*rotation_deg, 90.0);
}

#[test]
fn an_unknown_size_gives_a_named_frame_alone() {
    for px in [None, Some((0, 10))] {
        let list = list(&set(0.0), px);

        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].label.as_deref(), Some("bilder/a.png"));
        assert_eq!(list.items[0].source, Some(Source::Image("i1".into())));
        let Shape::Polygon { points } = &list.items[0].shape else {
            assert_eq!(list.items[0].role, Role::Background);
            return;
        };
        // The 4:3 placeholder: 4 m by 3 m.
        assert_eq!(points[0], p(8.0, 4.5));
        assert_eq!(points[2], p(12.0, 7.5));
    }
}

#[test]
fn the_grip_stands_for_its_image() {
    let set = set(0.0);
    let plan = FloorPlan::new("a", "A");
    let grips = |source: &Source| handles(&Project::new("T"), &set, &plan, source);

    for selected in [Source::Image("i1".into()), Source::ImageSize("i1".into())] {
        let items = grips(&selected);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].role, Role::Handle);
        assert_eq!(items[0].source, Some(Source::ImageSize("i1".into())));
        assert!(matches!(
            items[0].shape,
            Shape::Circle { center, .. } if center.distance_m(p(12.45, 6.0)) < 1e-9
        ));
    }
    assert!(grips(&Source::Image("x".into())).is_empty());
    assert_eq!(
        Source::ImageSize("i1".into()).owner(),
        Some(Source::Image("i1".into()))
    );
    assert_eq!(Source::Image("i1".into()).owner(), None);
}
