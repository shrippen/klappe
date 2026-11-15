//! NF-05 at the bridge: the JSON of a 50 figure, 10 camera, 300 object plan, as `shapes_json`
//! makes it for every redraw. Timing: `make perf` (release).

use std::path::Path;
use std::time::{Duration, Instant};

use klappe_services::{FigureColor, PlanEdit, Point, ProjectEdit, Session};

use super::{shapes_json, HeightTexts};

const FIGURES: usize = 50;
const MARKS: usize = 4;
const CAMERAS: usize = 10;
const OBJECTS: usize = 300;
const SET_M: f64 = 40.0;
const RUNS: usize = 200;
/// The JSON may take half the 16 ms budget of NF-05 (the canvas needs the rest); the test, like
/// all of `make perf`, fails only above four times its share: 4 × 8 ms.
const LIMIT: Duration = Duration::from_millis(32);

/// A point on a fixed pseudo-random walk over the set, so runs repeat.
fn at(n: usize) -> Point {
    Point::new(
        (n * 37 % 997) as f64 / 997.0 * (SET_M - 2.0) + 1.0,
        (n * 91 % 991) as f64 / 991.0 * (SET_M - 2.0) + 1.0,
    )
}

/// The big plan, built with the edits a user would make.
fn big_session(dir: &Path) -> Session {
    let mut session = Session::create(dir, "Perf").unwrap_or_else(|e| panic!("{e}"));
    let mut project = |edit| session.edit_project(edit).unwrap_or_else(|e| panic!("{e}"));
    project(ProjectEdit::AddLocation {
        name: "Halle".to_owned(),
        width_m: SET_M,
        depth_m: SET_M,
        wall_height_m: 2.6,
    });
    project(ProjectEdit::AddScene {
        number: "1".to_owned(),
        heading: "INT. HALLE".to_owned(),
        location: "halle".to_owned(),
    });
    for i in 0..FIGURES {
        project(ProjectEdit::AddCharacter {
            name: format!("Figur {i}"),
            short: format!("F{i}"),
            color: FigureColor::Red,
            group: None,
            size_m: None,
        });
    }

    let mut plan = |edit| session.edit_plan(edit).unwrap_or_else(|e| panic!("{e}"));
    for i in 0..FIGURES * MARKS {
        plan(PlanEdit::AddMark {
            character: format!("figur-{}", i % FIGURES),
            at: at(i),
        });
    }
    for i in 0..CAMERAS {
        plan(PlanEdit::AddCamera {
            at: at(i + 1000),
            facing_deg: 0.0,
        });
    }
    for i in 0..OBJECTS {
        plan(PlanEdit::AddObject {
            kind: "table".to_owned(),
            at: at(i + 2000),
        });
    }

    session
}

#[test]
#[ignore = "timing, run by `make perf` in release"]
fn a_large_plan_becomes_json_fast() {
    // Next to the test binary, in the target folder.
    let dir = std::env::current_exe()
        .map(|e| e.with_file_name("plan-perf"))
        .unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let list = big_session(&dir).display_list();

    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(shapes_json(
                &list,
                &dir,
                &|_| false,
                &[],
                None,
                None,
                &HeightTexts::default(),
            ));
            start.elapsed()
        })
        .collect();
    times.sort();
    let median = times[RUNS / 2];

    println!(
        "perf plan json: {} items, {:.3} ms",
        list.items.len(),
        median.as_secs_f64() * 1e3
    );
    assert!(list.items.len() > OBJECTS, "the plan is not big");
    assert!(median < LIMIT, "{median:?} over {LIMIT:?}");
}
