use super::*;
use crate::figure::GROUP_DOTS;
use klappe_model::Note;
use klappe_model::{
    Arrival, Camera, CameraMark, Character, Departure, Drawing, FigureTrack, Light, LightKind,
    Mark, Movement, ObjectShape, Pose, Sensor, SetObject, Shot, ShotSize, Wall,
};

fn project() -> Project {
    let mut p = Project::new("Test");
    p.characters = vec![
        Character {
            id: "e".into(),
            name: "Elisabeth".into(),
            short: "E".into(),
            color: FigureColor::Purple,
            group: None,
            size_m: None,
        },
        Character {
            id: "v".into(),
            name: "Burgvogt".into(),
            short: "B".into(),
            color: FigureColor::Blue,
            group: None,
            size_m: None,
        },
    ];
    p
}

fn set() -> SetPlan {
    SetPlan {
        width_m: 20.0,
        depth_m: 13.0,
        walls: vec![Wall::new(Point::new(0.0, 0.0), Point::new(20.0, 0.0), 0.2)],
        ..Default::default()
    }
}

fn mark(at_s: f64, x: f64, y: f64) -> Mark {
    Mark {
        at_s,
        at: Point::new(x, y),
        facing_deg: 0.0,
        pose: Pose::Standing,
        hold_s: 0.0,
        via: Vec::new(),
        arrival: Arrival::Walk,
        departure: Departure::Stay,
    }
}

/// The grips of `selected` on a plan whose set has no notes.
fn grips_of(plan: &FloorPlan, selected: &Source) -> Vec<Item> {
    handles(&project(), &SetPlan::default(), plan, selected)
}

fn plan() -> FloorPlan {
    let mut p = FloorPlan::new("a", "Plan A");
    p.figures = vec![
        FigureTrack {
            character: "e".into(),
            marks: vec![mark(0.0, 2.0, 6.0), mark(8.0, 7.0, 6.0)],
        },
        FigureTrack {
            character: "v".into(),
            marks: vec![mark(0.0, 12.0, 6.0)],
        },
    ];
    p.cameras = vec![Camera {
        id: "a".into(),
        label: "A".into(),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks: vec![CameraMark {
            at_s: 0.0,
            at: Point::new(8.0, 12.0),
            facing_deg: -90.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: Vec::new(),
            rig: Default::default(),
            lens_mm: None,
        }],
        follows: None,
    }];
    p.lights = vec![Light {
        id: "l1".into(),
        label: "L1".into(),
        kind: LightKind::LedPanel,
        at: Point::new(18.0, 2.0),
        facing_deg: 135.0,
        beam_deg: 60.0,
        color_temp_k: 5600.0,
        power_w: 200.0,
    }];
    p
}

fn count(list: &DisplayList, role: Role) -> usize {
    list.items.iter().filter(|i| i.role == role).count()
}

#[test]
fn draws_every_part_of_the_plan() {
    let list = render(&project(), &set(), &plan(), 0.0, None);

    assert_eq!(count(&list, Role::Wall), 1);
    assert_eq!(count(&list, Role::Light), 1);
    assert_eq!(count(&list, Role::Axis), 1);
    assert_eq!(count(&list, Role::Frustum), 1);
    assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 2);
    assert_eq!(count(&list, Role::Figure(FigureColor::Blue)), 2);
}

fn group_list() -> DisplayList {
    let mut p = project();
    p.characters[0].group = Some(12);
    render(&p, &set(), &plan(), 0.0, None)
}

/// A group is one larger body with three dots, however many people it counts, and its
/// label names the count (LH-23).
#[test]
fn a_group_is_drawn_as_a_group() {
    let list = group_list();
    let purple = |i: &&Item| i.role == Role::Figure(FigureColor::Purple);
    let radii: Vec<f64> = list
        .items
        .iter()
        .filter(purple)
        .filter_map(|i| match i.shape {
            Shape::Circle { radius_m, .. } => Some(radius_m),
            _ => None,
        })
        .collect();
    let label = list
        .items
        .iter()
        .filter(purple)
        .find_map(|i| i.label.clone());

    assert_eq!(radii.len(), 1);
    assert!(radii[0] > FIGURE_RADIUS_M);
    assert_eq!(label.as_deref(), Some("Elisabeth (12)"));
}

/// The dots in a group's body are a role of their own: in the body's colour they vanish.
#[test]
fn group_dots_differ_from_the_body() {
    let list = group_list();
    let dots: Vec<&Item> = list
        .items
        .iter()
        .filter(
            |i| matches!(i.shape, Shape::Circle { radius_m, .. } if radius_m == GROUP_DOT_RADIUS_M),
        )
        .collect();

    assert_eq!(dots.len(), GROUP_DOTS);
    assert!(dots
        .iter()
        .all(|i| i.role != Role::Figure(FigureColor::Purple)));
}

/// A single person keeps the one small circle and the plain name.
#[test]
fn a_single_figure_has_no_dots() {
    let list = render(&project(), &set(), &plan(), 0.0, None);
    let circles = list
        .items
        .iter()
        .filter(|i| i.role == Role::Figure(FigureColor::Purple))
        .filter(|i| matches!(i.shape, Shape::Circle { .. }))
        .count();
    let label = list.items.iter().find_map(|i| match i.role {
        Role::Figure(FigureColor::Purple) => i.label.clone(),
        _ => None,
    });

    assert_eq!(circles, 1);
    assert_eq!(label.as_deref(), Some("Elisabeth"));
}

/// Every part of the group, dots and rim included, selects the group's one track.
#[test]
fn the_whole_group_is_hit() {
    let list = group_list();
    let centre = Point::new(2.0, 6.0);
    // Beyond a single figure's radius, inside the group's.
    let rim = Point::new(2.0 + 0.5, 6.0);
    let hits = |at: Point| -> Vec<Option<Source>> {
        list.items
            .iter()
            .filter(|i| i.role == Role::Figure(FigureColor::Purple))
            .filter(|i| crate::hit::hit(i, at, 0.0))
            .map(|i| i.source.clone())
            .collect()
    };

    assert!(hits(rim).contains(&Some(Source::Figure("e".into()))));
    assert!(hits(centre)
        .iter()
        .all(|s| *s == Some(Source::Figure("e".into()))));
    assert!(!hits(Point::new(2.0 + 1.5, 6.0)).contains(&Some(Source::Figure("e".into()))));
}

#[test]
fn walls_are_selectable_by_index() {
    let list = render(&project(), &set(), &plan(), 0.0, None);

    assert_eq!(list.items[0].source, Some(Source::Wall(0)));
}

#[test]
fn figure_moves_with_time() {
    let at = |t: f64| {
        render(&project(), &set(), &plan(), t, None)
            .items
            .into_iter()
            .find_map(|i| match (i.role, i.shape) {
                (Role::Figure(FigureColor::Purple), Shape::Circle { center, .. }) => Some(center),
                _ => None,
            })
    };

    assert_eq!(at(0.0), Some(Point::new(2.0, 6.0)));
    assert_eq!(at(4.0), Some(Point::new(4.5, 6.0)));
}

/// A note is a card with its wrapped text as label, on top of the figures.
#[test]
fn notes_come_last_with_their_text() {
    let mut room = set();
    room.notes = vec![Note {
        id: "n1".into(),
        at: Point::new(3.0, 4.0),
        text: "Vorhang schließen".into(),
        width_m: 2.4,
    }];

    let list = render(&project(), &room, &plan(), 0.0, None);

    let last = list.items.last();
    assert_eq!(last.map(|i| i.role), Some(Role::Note));
    assert_eq!(
        last.and_then(|i| i.label.as_deref()),
        Some("Vorhang schließen")
    );
    assert_eq!(
        last.and_then(|i| i.source.clone()),
        Some(Source::Note("n1".into()))
    );
    let corners = match last.map(|i| i.shape.clone()) {
        Some(Shape::Polygon { points }) => points,
        other => panic!("not a card: {other:?}"),
    };
    assert_eq!(corners[0], Point::new(3.0, 4.0));
    assert!(corners[2].x_m > 3.0 && corners[2].y_m > 4.0);
}

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

    render(&project(), &set(), &p, 0.0, None)
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

    let list = render(&project(), &set(), &p, 0.0, None);

    let roles: Vec<_> = list.items.iter().map(|i| i.role).collect();
    let at = |f: &dyn Fn(&Role) -> bool| roles.iter().position(f);
    let drawing = at(&|r| matches!(r, Role::Drawing { .. }));
    assert!(at(&|r| *r == Role::Wall) < drawing);
    assert!(drawing < at(&|r| *r == Role::Light));
    assert!(drawing < at(&|r| matches!(r, Role::Figure(_))));
}

#[test]
fn walls_come_first_figures_last() {
    let list = render(&project(), &set(), &plan(), 0.0, None);

    assert_eq!(list.items.first().map(|i| i.role), Some(Role::Wall));
    assert!(matches!(
        list.items.last().map(|i| i.role),
        Some(Role::Figure(_))
    ));
}

/// 1000 x 500 px set 12.5 m wide is 6.25 m deep; it is the first, hindmost item.
#[test]
fn background_is_first_and_keeps_its_aspect() {
    let mut room = set();
    room.background = Some(Background {
        file: "bilder/saal.png".into(),
        at: Point::new(1.0, 2.0),
        width_m: 12.5,
        rotation_deg: 30.0,
        opacity: 0.5,
    });

    let list = render(&project(), &room, &plan(), 0.0, Some((1000, 500)));

    assert_eq!(
        list.items.first(),
        Some(&Item {
            role: Role::Background,
            shape: Shape::Image {
                file: "bilder/saal.png".into(),
                at: Point::new(1.0, 2.0),
                width_m: 12.5,
                depth_m: 6.25,
                rotation_deg: 30.0,
                opacity: 0.5,
            },
            label: None,
            source: None,
        })
    );
    assert_eq!(list.items.get(1).map(|i| i.role), Some(Role::Wall));
}

#[test]
fn background_needs_a_set_entry_and_a_size() {
    let mut room = set();
    let none_set = render(&project(), &room, &plan(), 0.0, Some((10, 10)));
    room.background = Some(Background {
        file: "bilder/saal.png".into(),
        at: Point::default(),
        width_m: 5.0,
        rotation_deg: 0.0,
        opacity: 1.0,
    });
    let unknown_size = render(&project(), &room, &plan(), 0.0, None);
    let zero_size = render(&project(), &room, &plan(), 0.0, Some((0, 10)));

    for list in [none_set, unknown_size, zero_size] {
        assert_eq!(count(&list, Role::Background), 0);
    }
}

/// A 2 m wide, flat object turned by 90° stands upright: corners at y = -1 and y = 1.
#[test]
fn objects_turn_clockwise() {
    let o = SetObject {
        id: "t".into(),
        kind: "table".into(),
        label: String::new(),
        at: Point::new(0.0, 0.0),
        width_m: 2.0,
        depth_m: 0.0,
        rotation_deg: 90.0,
        shape: ObjectShape::Rect,
        height_m: None,
        base_m: None,
    };
    let corners = object_corners(&o);

    assert!((corners[0].x_m).abs() < 1e-9 && (corners[0].y_m + 1.0).abs() < 1e-9);
    assert!((corners[1].x_m).abs() < 1e-9 && (corners[1].y_m - 1.0).abs() < 1e-9);
}

fn object(kind: &str, shape: ObjectShape, rotation_deg: f64) -> SetObject {
    SetObject {
        id: "o1".into(),
        kind: kind.into(),
        label: String::new(),
        at: Point::new(10.0, 5.0),
        width_m: 4.0,
        depth_m: 2.0,
        rotation_deg,
        shape,
        height_m: None,
        base_m: None,
    }
}

/// A 4 by 2 m ellipse at (10, 5): rightmost point (12, 5), lowest (10, 6), every corner
/// on the curve; turned by 90° the long axis stands upright.
#[test]
fn an_ellipse_object_is_inscribed_in_its_box() {
    let corners = object_corners(&object("pillar", ObjectShape::Ellipse, 0.0));

    assert_eq!(corners.len(), ELLIPSE_CORNERS);
    assert!((corners[0].x_m - 12.0).abs() < 1e-9 && (corners[0].y_m - 5.0).abs() < 1e-9);
    let quarter = corners[ELLIPSE_CORNERS / 4];
    assert!((quarter.x_m - 10.0).abs() < 1e-9 && (quarter.y_m - 6.0).abs() < 1e-9);
    for c in &corners {
        let (dx, dy) = ((c.x_m - 10.0) / 2.0, (c.y_m - 5.0) / 1.0);
        assert!((dx * dx + dy * dy - 1.0).abs() < 1e-9);
    }

    let upright = object_corners(&object("pillar", ObjectShape::Ellipse, 90.0));
    assert!((upright[0].x_m - 10.0).abs() < 1e-9 && (upright[0].y_m - 7.0).abs() < 1e-9);
}

/// An object whose kind no library knows (an old project) is a rectangle of its own size.
#[test]
fn an_unknown_kind_is_drawn_as_a_rectangle() {
    let corners = object_corners(&object("zeppelin", ObjectShape::default(), 0.0));

    assert_eq!(corners.len(), 4);
    assert_eq!((corners[0].x_m, corners[0].y_m), (8.0, 4.0));
    assert_eq!((corners[2].x_m, corners[2].y_m), (12.0, 6.0));
}

/// The nose points where the body faces: 90° is down.
#[test]
fn nose_points_where_it_faces() {
    let tip = nose(Point::new(0.0, 0.0), 90.0, 1.0)[1];

    assert!(tip.x_m.abs() < 1e-9);
    assert!((tip.y_m - (1.0 + NOSE_M)).abs() < 1e-9);
}

/// The axis runs through both figures and on by the set's diagonal on each side.
#[test]
fn axis_reaches_across_the_set() {
    let list = render(&project(), &set(), &plan(), 0.0, None);
    let axis = list.items.iter().find_map(|i| match (i.role, &i.shape) {
        (Role::Axis, Shape::Line { from, to, .. }) => Some((*from, *to)),
        _ => None,
    });
    let diagonal = 20.0_f64.hypot(13.0);

    let (from, to) = axis.unwrap();
    assert!((from.x_m - (2.0 - diagonal)).abs() < 1e-9 && (from.y_m - 6.0).abs() < 1e-9);
    assert!((to.x_m - (12.0 + diagonal)).abs() < 1e-9);
}

fn shot(lens_mm: Option<f64>, from_s: f64, to_s: f64) -> Shot {
    Shot {
        id: "s".into(),
        number: "1.1".into(),
        camera: "a".into(),
        size: ShotSize::Wide,
        angle: Default::default(),
        movement: Movement::Static,
        lens_mm,
        from_s,
        to_s,
        description: String::new(),
        props: String::new(),
        gear: String::new(),
        crew: String::new(),
        note: String::new(),
        setup_of: None,
    }
}

fn frustum_of(plan: &FloorPlan, t_s: f64) -> Vec<Point> {
    render(&project(), &set(), plan, t_s, None)
        .items
        .into_iter()
        .find_map(|i| match (i.role, i.shape) {
            (Role::Frustum, Shape::Polygon { points }) => Some(points),
            _ => None,
        })
        .unwrap_or_default()
}

/// The shot's own lens wins while it runs; outside it the camera's lens applies.
#[test]
fn frustum_follows_the_shots_lens() {
    let mut p = plan();
    p.shots = vec![shot(Some(40.0), 0.0, 4.0)];
    let fov = |lens_mm: f64| horizontal_fov_deg(Sensor::Super35, lens_mm);
    let expect = |lens_mm: f64| {
        frustum(
            Point::new(8.0, 12.0),
            -90.0,
            fov(lens_mm),
            20.0_f64.hypot(13.0),
        )
    };

    let during = frustum_of(&p, 2.0);
    let after = frustum_of(&p, 5.0);

    assert!(
        (during[1].x_m - expect(40.0)[1].x_m).abs() < 1e-6,
        "{during:?}"
    );
    assert!(
        (after[1].x_m - expect(35.0)[1].x_m).abs() < 1e-6,
        "{after:?}"
    );
}

/// A lens a mark states reaches the frustum; a shot's lens does not count then (LH-33).
#[test]
fn frustum_follows_the_lens_of_the_marks() {
    let mut by_mark = plan();
    by_mark.cameras[0].marks[0].lens_mm = Some(70.0);
    by_mark.shots = vec![shot(Some(40.0), 0.0, 4.0)];
    let mut by_shot = plan();
    by_shot.shots = vec![shot(Some(70.0), 0.0, 4.0)];

    assert_eq!(frustum_of(&by_mark, 2.0), frustum_of(&by_shot, 2.0));
}

#[test]
fn unknown_character_is_skipped() {
    let mut p = plan();
    p.figures[0].character = "nobody".into();

    let list = render(&project(), &set(), &p, 0.0, None);

    assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 0);
}

fn path_of(list: &DisplayList, role: Role) -> Vec<Point> {
    let item = list.items.iter().find(|i| i.role == role).unwrap();
    match &item.shape {
        Shape::Polyline { points } => points.clone(),
        other => panic!("{other:?}"),
    }
}

/// A way with a `via` is drawn along the curve; without, it stays two points.
#[test]
fn a_curved_way_is_a_polyline_along_the_curve() {
    let mut p = plan();
    let straight = render(&project(), &set(), &p, 0.0, None);
    p.figures[0].marks[1].via = vec![Point::new(4.5, 3.0)];
    let curved = render(&project(), &set(), &p, 0.0, None);

    let role = Role::FigurePath(FigureColor::Purple);
    assert_eq!(path_of(&straight, role).len(), 2);

    // From (2, 6) over (4.5, 3) to (7, 6): the way passes the bend.
    let points = path_of(&curved, role);
    assert!(points.len() > 2);
    assert!(points.contains(&Point::new(4.5, 3.0)));
    assert_eq!(points.first(), Some(&Point::new(2.0, 6.0)));
    assert_eq!(points.last(), Some(&Point::new(7.0, 6.0)));
}

/// The camera's path bends the same way.
#[test]
fn a_curved_camera_path_is_a_polyline_too() {
    let mut p = plan();
    let mut second = p.cameras[0].marks[0].clone();
    second.at = Point::new(12.0, 12.0);
    second.via = vec![Point::new(10.0, 6.0)];
    p.cameras[0].marks.push(second);

    let list = render(&project(), &set(), &p, 0.0, None);

    assert!(path_of(&list, Role::CameraPath { dash: Dash::Dashed }).len() > 2);
}

/// Grips: one per bend of the selected way and one to pull a new bend out of each piece
/// between them, none in `render`.
#[test]
fn only_the_selected_way_has_handles() {
    let mut p = plan();
    p.figures[0].marks[1].via = vec![Point::new(4.5, 3.0)];

    assert!(render(&project(), &set(), &p, 0.0, None)
        .items
        .iter()
        .all(|i| !matches!(i.source, Some(Source::FigureVia { .. }))));

    let via = |bend| Source::FigureVia {
        character: "e".into(),
        index: 1,
        bend,
    };
    let grips = grips_of(&p, &Source::Figure("e".into()));
    let sources: Vec<_> = grips.iter().map(|g| g.source.clone()).collect();
    assert_eq!(
        sources,
        [
            Some(via(Bend::New(0))),
            Some(via(Bend::New(1))),
            Some(via(Bend::At(0)))
        ]
    );
    assert!(grips
        .iter()
        .all(|g| g.role == Role::FigureHandle(FigureColor::Purple)));
    assert!(matches!(
        grips[2].shape,
        Shape::Circle { center, .. } if center == Point::new(4.5, 3.0)
    ));

    // The grip of a selected grip, and of a mark, belong to the same way.
    assert_eq!(grips_of(&p, &via(Bend::At(0))), grips);
    let mark = Source::Mark {
        character: "e".into(),
        index: 0,
    };
    assert_eq!(grips_of(&p, &mark), grips);

    // Another way's, a one-mark way's, and a wall's: none.
    assert!(grips_of(&p, &Source::Figure("v".into())).is_empty());
    assert!(grips_of(&p, &Source::Wall(0)).is_empty());
}

#[test]
fn a_camera_has_handles_when_selected() {
    let mut p = plan();
    let mut second = p.cameras[0].marks[0].clone();
    second.at = Point::new(12.0, 12.0);
    p.cameras[0].marks.push(second);

    let grips = grips_of(&p, &Source::Camera("a".into()));

    assert_eq!(grips.len(), 1);
    assert_eq!(grips[0].role, Role::CameraHandle);
    assert_eq!(
        grips[0].source,
        Some(Source::CameraVia {
            camera: "a".into(),
            index: 1,
            bend: Bend::New(0)
        })
    );
}

/// Elisabeth: mark 1 at 0 s, mark 2 (leaves) at 10 s, mark 3 (appears) at 20 s.
fn coming_and_going() -> FloorPlan {
    let mut p = plan();
    let mut track = FigureTrack {
        character: "e".into(),
        marks: vec![
            mark(0.0, 1.0, 1.0),
            mark(10.0, 5.0, 1.0),
            mark(20.0, 9.0, 1.0),
        ],
    };
    track.marks[1].departure = Departure::Leave;
    track.marks[2].arrival = Arrival::Appear;
    p.figures = vec![track];

    p
}

fn bodies(list: &DisplayList) -> usize {
    count(list, Role::Figure(FigureColor::Purple))
}

/// A figure that is not in the scene is not drawn; its marks and way stay (LH-26).
#[test]
fn a_figure_out_of_the_scene_is_not_drawn() {
    let p = coming_and_going();
    let at = |t_s| render(&project(), &set(), &p, t_s, None);

    assert_eq!(bodies(&at(5.0)), 2);
    assert_eq!(bodies(&at(15.0)), 0);
    assert_eq!(bodies(&at(20.0)), 2);
    assert_eq!(count(&at(15.0), Role::Mark(FigureColor::Purple)), 3);
    assert_eq!(count(&at(15.0), Role::FigurePath(FigureColor::Purple)), 1);
}

/// The way to a mark that is appeared at is not drawn: two marks and no way after it.
#[test]
fn no_way_leads_to_an_appearing_mark() {
    let mut p = coming_and_going();
    let way = |p: &FloorPlan| {
        let list = render(&project(), &set(), p, 0.0, None);
        list.items
            .iter()
            .filter_map(|i| match (&i.role, &i.shape) {
                (Role::FigurePath(_), Shape::Polyline { points }) => Some(points.len()),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(way(&p), [2]);

    // Review of PR 35: after a leave the figure is gone at a walked-to mark as well, so
    // no way leads there either.
    p.figures[0].marks[2].arrival = Arrival::Walk;
    assert_eq!(way(&p), [2]);

    p.figures[0].marks[1].departure = Departure::Stay;
    assert_eq!(way(&p), [3]);
}

/// Review of PR 35: no grip on a way after a leave, which the figure does not walk.
#[test]
fn a_way_after_a_leave_has_no_grip() {
    let mut p = coming_and_going();
    p.figures[0].marks[2].arrival = Arrival::Walk;
    let grips = grips_of(&p, &Source::Figure("e".into()));

    assert_eq!(grips.len(), 1);
}

/// An appearing mark has no way, so no grip to bend it (LH-26).
#[test]
fn an_appearing_mark_has_no_grip() {
    let p = coming_and_going();
    let grips = grips_of(&p, &Source::Figure("e".into()));

    assert_eq!(grips.len(), 1);
    assert_eq!(
        grips[0].source,
        Some(Source::FigureVia {
            character: "e".into(),
            index: 1,
            bend: Bend::New(0)
        })
    );
}

/// The mark's number says it: `+` where the figure appears, `-` where it leaves.
#[test]
fn marks_show_appearing_and_leaving() {
    let list = render(&project(), &set(), &coming_and_going(), 0.0, None);
    let labels: Vec<_> = list
        .items
        .iter()
        .filter(|i| matches!(i.role, Role::Mark(FigureColor::Purple)))
        .filter_map(|i| i.label.as_deref())
        .collect();

    assert_eq!(labels, ["1", "2-", "+3"]);
}
