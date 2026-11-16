use super::*;

fn drawn(shape: DrawingShape, fill: Fill) -> Drawing {
    Drawing {
        id: "d1".into(),
        shape,
        color: FigureColor::Red,
        dash: Dash::Dashed,
        fill,
        rotation_deg: 0.0,
    }
}

/// The drawing's items, in list order.
fn items_of_drawing(drawing: Drawing) -> Vec<Item> {
    let mut p = plan();
    p.drawings = vec![drawing];

    render(&project(), &set(), &p, 0.0, None, &|_| None)
        .items
        .into_iter()
        .filter(|i| i.source == Some(Source::Drawing("d1".into())))
        .collect()
}

/// An arrow from (0, 0) to (4, 0): the shaft, and a head whose tip is (4, 0) and whose
/// wings lie 0.45 m back at 25° either side of the shaft: x = 4 + 0.45 cos 155°.
#[test]
fn an_arrow_is_a_shaft_and_a_filled_head() {
    let arrow = drawn(
        DrawingShape::Arrow {
            from: Point::new(0.0, 0.0),
            to: Point::new(4.0, 0.0),
        },
        Fill::Outline,
    );

    let items = items_of_drawing(arrow);

    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].role,
        Role::Drawing {
            color: FigureColor::Red,
            dash: Dash::Dashed,
            fill: Fill::Outline
        }
    );
    assert_eq!(items[1].role, Role::ArrowHead(FigureColor::Red));
    let Shape::Polygon { points } = &items[1].shape else {
        panic!("head is no polygon: {:?}", items[1].shape);
    };
    let (wing_x, wing_y) = (
        4.0 + 0.45 * 155.0_f64.to_radians().cos(),
        0.45 * 25.0_f64.to_radians().sin(),
    );
    assert_eq!(points[1], Point::new(4.0, 0.0));
    assert!((points[0].x_m - wing_x).abs() < 1e-6 && (points[0].y_m - wing_y).abs() < 1e-6);
    assert!((points[2].x_m - wing_x).abs() < 1e-6 && (points[2].y_m + wing_y).abs() < 1e-6);
}

/// A 0.4 m arrow gets a head of 0.2 m: it never outgrows the shaft.
#[test]
fn a_short_arrow_has_a_short_head() {
    let head = arrow_head(Point::new(0.0, 0.0), Point::new(0.4, 0.0));

    let Some(Shape::Polygon { points }) = head else {
        panic!("no head");
    };
    assert!(points[0].distance_m(points[1]) <= 0.2 + 1e-9);
    assert_eq!(arrow_head(Point::new(1.0, 1.0), Point::new(1.0, 1.0)), None);
}

/// Corners dragged from bottom right to top left still come out clockwise from the top
/// left; filled it is a polygon, as an outline a closed path.
#[test]
fn a_rectangle_has_four_corners_whichever_way_it_was_dragged() {
    let shape = |fill| {
        let rect = drawn(
            DrawingShape::Rect {
                from: Point::new(3.0, 4.0),
                to: Point::new(1.0, 2.0),
            },
            fill,
        );
        items_of_drawing(rect)[0].shape.clone()
    };
    let corners = vec![
        Point::new(1.0, 2.0),
        Point::new(3.0, 2.0),
        Point::new(3.0, 4.0),
        Point::new(1.0, 4.0),
    ];

    let mut closed = corners.clone();
    closed.push(corners[0]);
    assert_eq!(shape(Fill::Filled), Shape::Polygon { points: corners });
    assert_eq!(shape(Fill::Outline), Shape::Polyline { points: closed });
}

/// Ellipse in the box (0, 0)-(6, 2): centre (3, 1), radii 3 and 1; starts at its right
/// end (6, 1); every corner satisfies ((x-3)/3)^2 + ((y-1)/1)^2 = 1.
#[test]
fn an_ellipse_is_inscribed_in_its_box() {
    let ellipse = drawn(
        DrawingShape::Ellipse {
            from: Point::new(6.0, 2.0),
            to: Point::new(0.0, 0.0),
        },
        Fill::Filled,
    );

    let items = items_of_drawing(ellipse);

    let Shape::Polygon { points } = &items[0].shape else {
        panic!("not a polygon: {:?}", items[0].shape);
    };
    assert_eq!(points.len(), ELLIPSE_CORNERS);
    assert!((points[0].x_m - 6.0).abs() < 1e-9 && (points[0].y_m - 1.0).abs() < 1e-9);
    for p in points {
        let r = ((p.x_m - 3.0) / 3.0).powi(2) + (p.y_m - 1.0).powi(2);
        assert!((r - 1.0).abs() < 1e-9, "{p:?}");
    }
}

/// A click inside a hollow shape reaches what lies below it; inside a filled one it
/// takes the shape.
#[test]
fn only_a_filled_shape_is_taken_from_inside() {
    let at = |fill| {
        let rect = drawn(
            DrawingShape::Rect {
                from: Point::new(0.0, 0.0),
                to: Point::new(4.0, 4.0),
            },
            fill,
        );
        let item = items_of_drawing(rect).remove(0);
        (
            crate::hit::hit(&item, Point::new(2.0, 2.0), 0.15),
            crate::hit::hit(&item, Point::new(4.05, 2.0), 0.15),
        )
    };

    assert_eq!(at(Fill::Outline), (false, true));
    assert_eq!(at(Fill::Filled), (true, true));
}

/// An area is as drawn: its points, as a polygon when filled.
#[test]
fn an_area_keeps_its_points() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(2.0, 0.0),
        Point::new(1.0, 2.0),
    ];

    let items = items_of_drawing(drawn(
        DrawingShape::Area {
            points: points.clone(),
        },
        Fill::Filled,
    ));

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].shape, Shape::Polygon { points });
}

/// Drawings lie on the set (after the wall) and under the lamps, cameras and figures.
#[test]
fn drawings_lie_between_set_and_figures() {
    let mut p = plan();
    p.drawings = vec![drawn(
        DrawingShape::Line {
            from: Point::new(1.0, 1.0),
            to: Point::new(2.0, 2.0),
        },
        Fill::Outline,
    )];

    let list = render(&project(), &set(), &p, 0.0, None, &|_| None);

    let roles: Vec<_> = list.items.iter().map(|i| i.role).collect();
    let at = |f: &dyn Fn(&Role) -> bool| roles.iter().position(f);
    let drawing = at(&|r| matches!(r, Role::Drawing { .. }));
    assert!(at(&|r| *r == Role::Wall) < drawing);
    assert!(drawing < at(&|r| *r == Role::Light));
    assert!(drawing < at(&|r| matches!(r, Role::Figure(_))));
}
