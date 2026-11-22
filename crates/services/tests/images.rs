//! Photos and sketches placed on the plan (LH-18), on a copy of the example project. The
//! images are bare PNG headers: only the pixel size is ever read from them.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::PlanImage;
use klappe_services::{
    Hint, Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Shape, Snapping, Source,
    Visibility,
};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const FILE: &str = "bilder/skizze.png";

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

/// A PNG header of `w` x `h` pixels, 8-bit RGB.
fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n".to_vec();
    b.extend_from_slice(&13_u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(&[8, 2, 0, 0, 0]);
    b
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}

/// A copy of the example in a fresh folder, and `skizze.png` (1000 x 500 px) beside it.
fn project_dir(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("projekt.klappe");
    copy_dir(Path::new(EXAMPLE), &dir);
    let source = root.join("skizze.png");
    std::fs::write(&source, png(1000, 500)).unwrap();

    (dir, source)
}

/// A session whose set has the image `i1`, 2 m wide, centred on (10, 3), not turned; its file
/// is in `bilder/`. Snapping is off, so a drag lands where it says.
fn with_image(name: &str) -> (Session, PathBuf) {
    let (dir, source) = project_dir(name);
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s.edit_plan(PlanEdit::AddImage {
        source,
        at: p(10.0, 3.0),
    })
    .unwrap();

    (s, dir)
}

fn images(s: &Session) -> Vec<PlanImage> {
    s.project().locations[0].set.images.clone()
}

fn image() -> Source {
    Source::Image("i1".into())
}

fn grip() -> Source {
    Source::ImageSize("i1".into())
}

fn shift(source: Source, dx_m: f64, dy_m: f64) -> PlanEdit {
    PlanEdit::Shift { source, dx_m, dy_m }
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

/// The frame of the image `i1` in the display list.
fn frame(s: &Session) -> Vec<Point> {
    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| i.source.as_ref() == Some(&image()))
        .unwrap();
    let Shape::Polygon { points } = &item.shape else {
        return Vec::new();
    };

    points.clone()
}

#[test]
fn adds_an_image_copies_its_file_and_undoes_in_one_step() {
    let (s0, dir) = with_image("img-add");
    let mut s = s0;

    assert_eq!(
        images(&s),
        vec![PlanImage {
            id: "i1".into(),
            file: FILE.into(),
            at: p(10.0, 3.0),
            width_m: 2.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        }]
    );
    assert!(dir.join(FILE).is_file());

    s.undo();
    assert!(images(&s).is_empty());
    assert!(!s.can_undo());
    // The copy is the user's file.
    assert!(dir.join(FILE).is_file());
}

#[test]
fn the_picture_and_its_frame_are_drawn_with_the_height_of_the_aspect() {
    let (s, _) = with_image("img-list");
    let list = s.display_list();
    let picture = list
        .items
        .iter()
        .find(|i| matches!(i.shape, Shape::Image { .. }))
        .unwrap();

    // 1000 x 500 px at 2 m wide: 1 m deep; `at` is the top left corner.
    assert_eq!(picture.role, Role::Picture);
    assert_eq!(picture.source, None);
    assert!(matches!(
        &picture.shape,
        Shape::Image { file, at, width_m, depth_m, rotation_deg, opacity }
            if file == FILE && near(*at, p(9.0, 2.5)) && *width_m == 2.0 && *depth_m == 1.0
                && *rotation_deg == 0.0 && *opacity == 1.0
    ));
    assert_eq!(
        frame(&s),
        vec![p(9.0, 2.5), p(11.0, 2.5), p(11.0, 3.5), p(9.0, 3.5)]
    );
}

#[test]
fn a_missing_file_is_an_empty_frame_with_its_name() {
    let (s, dir) = with_image("img-gone");
    std::fs::remove_file(dir.join(FILE)).unwrap();

    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| i.source.as_ref() == Some(&image()))
        .unwrap();

    assert!(list
        .items
        .iter()
        .all(|i| !matches!(i.shape, Shape::Image { .. })));
    assert_eq!(item.role, Role::Picture);
    assert_eq!(item.label.as_deref(), Some(FILE));
    // The frame can still be picked, moved and removed.
    assert_eq!(s.pick(p(10.0, 3.0)), Some(image()));
}

#[test]
fn a_click_takes_the_image_and_a_box_takes_it_whole() {
    let (s, _) = with_image("img-pick");

    assert_eq!(s.pick(p(10.5, 3.2)), Some(image()));
    assert_eq!(s.pick(p(14.0, 3.0)), None);
    assert_eq!(s.pick_in(p(8.0, 2.0), p(12.0, 4.0)), vec![image()]);
    assert!(s.pick_in(p(8.0, 2.0), p(10.5, 4.0)).is_empty());
}

#[test]
fn the_tooltip_names_the_file() {
    let (s, _) = with_image("img-hint");

    assert_eq!(
        s.image_hint("i1"),
        Some(Hint::Image {
            file: FILE.into(),
            opacity: 1.0
        })
    );
    assert_eq!(s.image_hint("i9"), None);
}

#[test]
fn moves_the_image_and_undoes_in_one_step() {
    let (mut s, _) = with_image("img-move");

    s.edit_plan(shift(image(), 3.0, -1.0)).unwrap();
    assert_eq!(images(&s)[0].at, p(13.0, 2.0));

    s.edit_plan(PlanEdit::Move {
        source: image(),
        to: p(4.0, 4.0),
    })
    .unwrap();
    assert_eq!(images(&s)[0].at, p(4.0, 4.0));

    s.undo();
    assert_eq!(images(&s)[0].at, p(13.0, 2.0));
}

#[test]
fn the_grip_sits_outside_the_right_edge_while_the_image_is_selected() {
    let (s, _) = with_image("img-grip");
    let grip_at = |selected: Option<&Source>| {
        s.display_list_for(selected)
            .items
            .into_iter()
            .find(|i| i.role == Role::Handle)
            .map(|i| i.shape)
    };

    assert_eq!(grip_at(None), None);
    assert!(matches!(
        grip_at(Some(&image())),
        Some(Shape::Circle { center, .. }) if near(center, p(11.45, 3.0))
    ));
    // The grip is picked before the image, and stands for the image in the selection.
    assert_eq!(s.pick_for(p(11.45, 3.0), Some(&image())), Some(grip()));
    assert_eq!(grip().owner(), Some(image()));
}

/// The left edge stays: the centre moves half the change.
#[test]
fn dragging_the_grip_sets_the_width_and_undoes_in_one_step() {
    let (mut s, _) = with_image("img-size");

    s.edit_plan(shift(grip(), 1.0, 0.0)).unwrap();
    assert!((images(&s)[0].width_m - 3.0).abs() < 1e-9);
    assert!(near(images(&s)[0].at, p(10.5, 3.0)));

    s.edit_plan(shift(grip(), -100.0, 0.0)).unwrap();
    assert!((images(&s)[0].width_m - 0.05).abs() < 1e-9);

    s.undo();
    assert!((images(&s)[0].width_m - 3.0).abs() < 1e-9);
}

/// With snapping on, the edge lands on the grid, not the grip that sits outside it: the right
/// edge (x = 11) pulled by half a metre makes the image 2.5 m wide.
#[test]
fn a_snapped_drag_puts_the_edge_on_the_grid() {
    let (mut s, _) = with_image("img-size-snap");
    s.set_snapping(Snapping::On);

    s.edit_plan(shift(grip(), 0.5, 0.0)).unwrap();

    assert!((images(&s)[0].width_m - 2.5).abs() < 1e-9);
}

#[test]
fn the_grip_follows_the_turn() {
    let (mut s, _) = with_image("img-size-turned");
    s.edit_plan(PlanEdit::Turn {
        source: image(),
        facing_deg: 90.0,
    })
    .unwrap();

    // Turned a quarter clockwise, the right edge points down the plan.
    s.edit_plan(shift(grip(), 0.0, 1.0)).unwrap();

    assert!((images(&s)[0].width_m - 3.0).abs() < 1e-9);
    assert!(near(images(&s)[0].at, p(10.0, 3.5)));
}

#[test]
fn turns_the_image_about_its_centre() {
    let (mut s, _) = with_image("img-turn");

    s.edit_plan(PlanEdit::Spin {
        source: image(),
        by_deg: 90.0,
    })
    .unwrap();
    assert_eq!(images(&s)[0].rotation_deg, 90.0);
    assert_eq!(images(&s)[0].at, p(10.0, 3.0));
    // The top left corner (9, 2.5) swings to the top right.
    let corners = frame(&s);
    assert!(near(corners[0], p(10.5, 2.0)));
    assert!(near(corners[2], p(9.5, 4.0)));

    s.undo();
    assert_eq!(images(&s)[0].rotation_deg, 0.0);
}

#[test]
fn removes_the_image_and_undoes_in_one_step() {
    let (mut s, _) = with_image("img-remove");

    s.edit_plan(PlanEdit::Remove { source: image() }).unwrap();
    assert!(images(&s).is_empty());

    s.undo();
    assert_eq!(images(&s).len(), 1);
    // The grip is no element: nothing to remove.
    assert!(s.edit_plan(PlanEdit::Remove { source: grip() }).is_err());
}

#[test]
fn copies_the_image_under_a_new_id() {
    let (mut s, _) = with_image("img-copy");

    s.edit_plan(PlanEdit::Duplicate {
        sources: vec![image()],
        dx_m: 2.5,
        dy_m: 1.0,
    })
    .unwrap();

    let all = images(&s);
    assert_eq!(all.len(), 2);
    assert_eq!(all[1].id, "i2");
    assert_eq!(all[1].file, FILE);
    assert_eq!(all[1].at, p(12.5, 4.0));
    s.undo();
    assert_eq!(images(&s).len(), 1);
}

#[test]
fn an_image_takes_part_in_a_group_with_a_note() {
    let (mut s, _) = with_image("img-group");
    let note = Source::Note("n1".into());
    let note_at = s.project().locations[0].set.notes[0].at;

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![image(), note.clone()],
        dx_m: 1.0,
        dy_m: 2.0,
    })
    .unwrap();
    assert_eq!(images(&s)[0].at, p(11.0, 5.0));
    assert_eq!(
        s.project().locations[0].set.notes[0].at,
        p(note_at.x_m + 1.0, note_at.y_m + 2.0)
    );

    s.edit_plan(PlanEdit::SpinGroup {
        sources: vec![image(), note],
        by_deg: 90.0,
    })
    .unwrap();
    assert_eq!(images(&s)[0].rotation_deg, 90.0);

    s.edit_plan(PlanEdit::RemoveGroup {
        sources: vec![image()],
    })
    .unwrap();
    assert!(images(&s).is_empty());
}

#[test]
fn the_notes_layer_locks_and_hides_the_image() {
    let (mut s, _) = with_image("img-layer");

    s.set_lock(Layer::Notes, Lock::Locked);
    assert_eq!(s.pick(p(10.0, 3.0)), None);
    for edit in [
        shift(image(), 1.0, 0.0),
        shift(grip(), 1.0, 0.0),
        PlanEdit::Remove { source: image() },
        PlanEdit::Duplicate {
            sources: vec![image()],
            dx_m: 1.0,
            dy_m: 0.0,
        },
        PlanEdit::AddImage {
            source: PathBuf::from("x.png"),
            at: p(1.0, 1.0),
        },
    ] {
        assert!(matches!(
            s.edit_plan(edit),
            Err(ServiceError::Locked(Layer::Notes))
        ));
    }
    assert_eq!(images(&s)[0].at, p(10.0, 3.0));

    s.set_lock(Layer::Notes, Lock::Unlocked);
    s.set_layer(Layer::Notes, Visibility::Hidden);
    assert!(s
        .display_list()
        .items
        .iter()
        .all(|i| i.role != Role::Picture));
}

#[test]
fn a_file_that_is_no_image_is_refused_and_leaves_the_plan() {
    let (mut s, dir) = with_image("img-bad");
    let text = dir.join("notiz.txt");
    std::fs::write(&text, "x").unwrap();

    let result = s.edit_plan(PlanEdit::AddImage {
        source: text,
        at: p(1.0, 1.0),
    });

    assert!(result.is_err());
    assert_eq!(images(&s).len(), 1);
}

#[test]
fn a_second_image_gets_the_next_id() {
    let (mut s, dir) = with_image("img-ids");
    let other = dir.join("../zweite.png");
    std::fs::write(&other, png(300, 600)).unwrap();

    s.edit_plan(PlanEdit::AddImage {
        source: other,
        at: p(3.0, 3.0),
    })
    .unwrap();

    assert_eq!(images(&s)[1].id, "i2");
    assert_eq!(images(&s)[1].file, "bilder/zweite.png");
}

#[test]
fn saved_and_reopened_the_image_is_there() {
    let (mut s, dir) = with_image("img-save");
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    assert_eq!(images(&Session::open(&dir).unwrap()), images(&s));
}
