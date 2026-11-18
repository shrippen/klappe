//! Appearing and leaving at a mark (LH-26), on the example project: Elisabeth's marks lie at
//! 0, 6 and 14 s with holds of 2, 4 and 0 s; camera A has marks.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Arrival, Departure, DisplayList, Fade, Layer, Lock, Pace, PlanEdit, Role, ServiceError, Session,
    Snapping, Source, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn mark(index: usize) -> Source {
    Source::Mark {
        character: "elisabeth".into(),
        index,
    }
}

fn arrival_of(s: &Session, index: usize) -> Arrival {
    s.mark_info(&mark(index)).unwrap().arrival.unwrap()
}

fn departure_of(s: &Session, index: usize) -> Departure {
    s.mark_info(&mark(index)).unwrap().departure.unwrap()
}

#[test]
fn a_mark_can_appear_and_leave() {
    let mut s = open();

    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
    assert_eq!(departure_of(&s, 1), Departure::Stay);

    s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    })
    .unwrap();
    s.edit_plan(PlanEdit::SetDeparture {
        source: mark(2),
        departure: Departure::Leave,
    })
    .unwrap();

    assert_eq!(arrival_of(&s, 1), Arrival::Appear);
    assert_eq!(arrival_of(&s, 0), Arrival::Walk);
    assert_eq!(departure_of(&s, 2), Departure::Leave);
    assert_eq!(departure_of(&s, 1), Departure::Stay);

    // One undo step each.
    s.undo();
    assert_eq!(departure_of(&s, 2), Departure::Stay);
    assert_eq!(arrival_of(&s, 1), Arrival::Appear);
    s.undo();
    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
}

/// The figure at the playhead stands for its mark there: at 7 s that is the second.
#[test]
fn the_figure_stands_for_its_mark_at_the_playhead() {
    let mut s = open();
    s.set_time(7.0);

    s.edit_plan(PlanEdit::SetDeparture {
        source: Source::Figure("elisabeth".into()),
        departure: Departure::Leave,
    })
    .unwrap();

    assert_eq!(departure_of(&s, 1), Departure::Leave);
}

/// The edit shows in the display list: after the pause at the leaving mark (6 s + 4 s) the
/// figure is not drawn.
#[test]
fn a_figure_that_left_is_not_drawn() {
    let mut s = open();
    let bodies = |s: &Session| {
        s.display_list()
            .items
            .iter()
            .filter(|i| matches!(i.role, Role::Figure(_)))
            .filter(|i| matches!(i.source, Some(Source::Figure(ref c)) if c == "elisabeth"))
            .count()
    };
    s.edit_plan(PlanEdit::SetDeparture {
        source: mark(1),
        departure: Departure::Leave,
    })
    .unwrap();

    s.set_time(8.0);
    assert_eq!(bodies(&s), 2);
    s.set_time(11.0);
    assert_eq!(bodies(&s), 0);
}

#[test]
fn a_camera_mark_has_neither() {
    let mut s = open();
    let camera = Source::Camera("a".into());

    let info = s.mark_info(&camera).unwrap();
    assert_eq!((info.arrival, info.departure), (None, None));

    let result = s.edit_plan(PlanEdit::SetArrival {
        source: camera,
        arrival: Arrival::Appear,
    });
    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
}

#[test]
fn the_edits_follow_the_figure_lock() {
    let mut s = open();
    s.set_lock(Layer::Figures, Lock::Locked);

    let arrival = s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    });
    let departure = s.edit_plan(PlanEdit::SetDeparture {
        source: mark(1),
        departure: Departure::Leave,
    });

    assert!(matches!(arrival, Err(ServiceError::Locked(Layer::Figures))));
    assert!(matches!(
        departure,
        Err(ServiceError::Locked(Layer::Figures))
    ));
    assert_eq!(arrival_of(&s, 1), Arrival::Walk);
    assert_eq!(departure_of(&s, 1), Departure::Stay);
}

#[test]
fn an_unknown_mark_is_gone() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::SetArrival {
        source: mark(9),
        arrival: Arrival::Appear,
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
}

/// Review of PR 35: the figure does not walk to a mark it appears at, so that mark has no way
/// and no pace, and setting one is refused instead of moving the appearance in time.
#[test]
fn an_appearing_mark_has_no_way_to_pace() {
    let mut s = open();
    s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    })
    .unwrap();
    let info = s.mark_info(&mark(1)).unwrap();

    assert!(!info.has_way);
    assert_eq!(info.pace_m_s, None);
    assert!(s
        .edit_plan(PlanEdit::SetPace {
            source: mark(1),
            pace: Pace::Run,
        })
        .is_err());
}

/// Elisabeth appears at 6 s; at 6.25 s she is half faded and Burgvogt is not.
fn fading() -> Session {
    let mut s = open();
    s.edit_plan(PlanEdit::SetArrival {
        source: mark(1),
        arrival: Arrival::Appear,
    })
    .unwrap();
    s.set_time(6.25);

    s
}

/// The opacities of the figures' bodies by character, not their ways.
fn figure_opacities(list: &DisplayList) -> Vec<(String, f64)> {
    let figure = |s: &Option<Source>| match s {
        Some(Source::Figure(id)) => Some(id.clone()),
        _ => None,
    };

    list.items
        .iter()
        .filter(|i| matches!(i.role, Role::Figure(_)))
        .filter_map(|i| Some((figure(&i.source)?, i.opacity)))
        .collect()
}

/// Review of PR 255: a hidden layer or track takes items out of the list; the opacity of the
/// others stays with them.
#[test]
fn the_opacity_stays_with_its_figure_when_items_are_hidden() {
    let camera = fading().plan().unwrap().cameras[0].id.clone();
    let hide: [&dyn Fn(&mut Session); 3] = [
        &|_| {},
        &|s| s.set_layer(Layer::Cameras, Visibility::Hidden),
        &|s| s.set_track(&camera, Visibility::Hidden),
    ];

    for hide in hide {
        let mut s = fading();
        hide(&mut s);
        let figures = figure_opacities(&s.display_list());

        assert!(figures.iter().any(|(id, _)| id == "burgvogt"));
        for (id, opacity) in figures {
            let want = if id == "elisabeth" { 0.5 } else { 1.0 };
            assert_eq!(opacity, want, "{id}");
        }
    }
}

/// The same in the print: Elisabeth (purple) is the translucent one, Burgvogt (blue) is not.
#[test]
fn the_print_fades_the_same_figure_when_a_layer_is_hidden() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("opacity-print");
    std::fs::create_dir_all(&dir).unwrap();
    let print = |hidden: bool| {
        let mut s = fading();
        if hidden {
            s.set_layer(Layer::Cameras, Visibility::Hidden);
        }
        let path = dir.join(format!("{hidden}.svg"));
        s.export_svg(
            &path,
            klappe_services::PaperSetup::default(),
            klappe_services::Language::English.print_texts(),
        )
        .unwrap();

        std::fs::read_to_string(path).unwrap()
    };
    let figure_lines = |svg: &str, color: &str| -> Vec<String> {
        svg.lines()
            .filter(|l| l.contains(&format!("class=\"item figure {color}\"")))
            .map(str::to_owned)
            .collect()
    };

    for hidden in [false, true] {
        let svg = print(hidden);
        let elisabeth = figure_lines(&svg, "purple");
        let burgvogt = figure_lines(&svg, "blue");

        assert!(!elisabeth.is_empty() && !burgvogt.is_empty(), "{svg}");
        assert!(
            elisabeth.iter().all(|l| l.contains("fill-opacity")),
            "{hidden}"
        );
        assert!(
            burgvogt.iter().all(|l| !l.contains("fill-opacity")),
            "{hidden}"
        );
    }
}

/// The print draws a figure at its layer's fade times its own opacity: Elisabeth at 6.25 s
/// is half there, so with her layer at 60 % she keeps 30 % of her fill.
#[test]
fn the_print_multiplies_layer_fade_and_opacity() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("opacity-fade-print");
    std::fs::create_dir_all(&dir).unwrap();
    let fill_opacities = |fade: Fade| -> Vec<f64> {
        let mut s = fading();
        s.set_fade(Layer::Figures, fade);
        let path = dir.join(format!("{}.svg", fade.name()));
        s.export_svg(
            &path,
            klappe_services::PaperSetup::default(),
            klappe_services::Language::English.print_texts(),
        )
        .unwrap();
        let svg = std::fs::read_to_string(path).unwrap();

        svg.lines()
            .filter(|l| l.contains("class=\"item figure purple\""))
            .filter_map(|l| {
                let at = l.find("fill-opacity=\"")? + "fill-opacity=\"".len();

                l[at..].split('"').next()?.parse().ok()
            })
            .collect()
    };

    let half = fill_opacities(Fade::Full);
    let product = fill_opacities(Fade::Medium);

    assert!(!half.is_empty());
    assert_eq!(half.len(), product.len());
    for (half, product) in half.iter().zip(&product) {
        assert!((product - half * 0.6).abs() < 1e-3, "{half} -> {product}");
    }
}
