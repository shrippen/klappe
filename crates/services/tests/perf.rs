//! NF-05: a large plan (50 figures, 10 cameras, 300 set objects) stays fluid. Timings are
//! only meaningful in a release build: `make perf` runs these ignored tests with
//! `--release`, prints the numbers and fails above four times the budget.
#![allow(clippy::unwrap_used)]

use std::path::Path;
use std::time::{Duration, Instant};

use klappe_model::{
    Arrival, Camera, CameraMark, Character, Dash, Departure, Drawing, DrawingShape, FigureColor,
    FigureTrack, Fill, FloorPlan, Light, LightKind, Location, Mark, Pose, Project, Scene, Sensor,
    SetObject, SetPlan, Shot, ShotSize, Wall,
};
use klappe_services::{Angle, Looping, Movement, PlanEdit, Point, Session, Source};

const FIGURES: usize = 50;
const MARKS_PER_FIGURE: usize = 4;
const CAMERAS: usize = 10;
const OBJECTS: usize = 300;
const WALLS: usize = 12;
const DRAWINGS: usize = 8;
const LIGHTS: usize = 6;
const SHOTS: usize = 10;
const SET_M: f64 = 40.0;
const MARK_GAP_S: f64 = 5.0;

/// Frame time of a playback tick (25 fps).
const TICK_S: f64 = 0.04;
/// One tick and one edit together (NF-05).
const BUDGET: Duration = Duration::from_millis(16);
/// CI machines vary; only a real regression crosses this.
const LIMIT_FACTOR: u32 = 4;
const RUNS: usize = 200;

/// A point on a fixed pseudo-random walk over the set, so runs repeat.
fn spot(n: usize) -> Point {
    let x = (n * 37 % 997) as f64 / 997.0 * (SET_M - 2.0) + 1.0;
    let y = (n * 91 % 991) as f64 / 991.0 * (SET_M - 2.0) + 1.0;
    Point::new(x, y)
}

fn mark(n: usize, at_s: f64, via: Option<Point>) -> Mark {
    Mark {
        at_s,
        at: spot(n),
        facing_deg: (n * 17 % 360) as f64,
        pose: Pose::Standing,
        hold_s: 1.0,
        via,
        arrival: Arrival::Walk,
        departure: Departure::Stay,
    }
}

fn character(i: usize) -> Character {
    Character {
        id: format!("c{i}"),
        name: format!("Figur {i}"),
        short: format!("F{i}"),
        color: FigureColor::ALL[i % FigureColor::ALL.len()],
        group: None,
    }
}

fn figure(i: usize) -> FigureTrack {
    let marks = (0..MARKS_PER_FIGURE)
        .map(|k| {
            let via = (k % 2 == 1).then(|| spot(i * 31 + k + 500));
            mark(i * 7 + k, k as f64 * MARK_GAP_S, via)
        })
        .collect();

    FigureTrack {
        character: format!("c{i}"),
        marks,
    }
}

fn camera(i: usize) -> Camera {
    let marks = (0..3)
        .map(|k| CameraMark {
            at_s: k as f64 * MARK_GAP_S * 2.0,
            at: spot(i * 13 + k + 1000),
            facing_deg: (i * 36 + k * 20) as f64,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s: 0.0,
            via: None,
        })
        .collect();

    Camera {
        id: format!("cam{i}"),
        label: format!("{}", (b'A' + i as u8) as char),
        sensor: Sensor::Super35,
        lens_mm: 35.0,
        aspect: 2.39,
        marks,
        follows: None,
    }
}

fn object(i: usize) -> SetObject {
    SetObject {
        id: format!("o{i}"),
        kind: "table".into(),
        label: String::new(),
        at: spot(i + 2000),
        width_m: 1.2,
        depth_m: 0.8,
        rotation_deg: (i * 11 % 360) as f64,
        shape: Default::default(),
        height_m: None,
    }
}

fn wall(i: usize) -> Wall {
    Wall {
        from: spot(i + 3000),
        to: spot(i + 3100),
        thickness_m: 0.2,
    }
}

fn drawing(i: usize) -> Drawing {
    Drawing {
        id: format!("d{i}"),
        shape: DrawingShape::Arrow {
            from: spot(i + 4000),
            to: spot(i + 4100),
        },
        color: FigureColor::Red,
        dash: Dash::Dashed,
        fill: Fill::Outline,
    }
}

fn light(i: usize) -> Light {
    Light {
        id: format!("l{i}"),
        label: format!("L{i}"),
        kind: LightKind::Fresnel,
        at: spot(i + 5000),
        facing_deg: 45.0,
        beam_deg: 30.0,
        color_temp_k: 5600.0,
        power_w: 650.0,
    }
}

fn shot(i: usize) -> Shot {
    Shot {
        id: format!("s{i}"),
        number: format!("1.{i}"),
        camera: format!("cam{i}"),
        size: ShotSize::Medium,
        angle: Angle::EyeLevel,
        movement: Movement::Static,
        lens_mm: None,
        from_s: 0.0,
        to_s: MARK_GAP_S * MARKS_PER_FIGURE as f64,
        description: String::new(),
        props: String::new(),
        gear: String::new(),
        crew: String::new(),
        note: String::new(),
    }
}

/// The NF-05 plan as a session on disk (`name` names the folder under the target's tmp).
fn big_session(name: &str) -> Session {
    let mut plan = FloorPlan::new("p1", "Plan A");
    plan.figures = (0..FIGURES).map(figure).collect();
    plan.cameras = (0..CAMERAS).map(camera).collect();
    plan.lights = (0..LIGHTS).map(light).collect();
    plan.drawings = (0..DRAWINGS).map(drawing).collect();
    plan.shots = (0..SHOTS).map(shot).collect();

    let set = SetPlan {
        width_m: SET_M,
        depth_m: SET_M,
        walls: (0..WALLS).map(wall).collect(),
        objects: (0..OBJECTS).map(object).collect(),
        background: None,
        notes: Vec::new(),
    };
    let mut project = Project::new("Perf");
    project.characters = (0..FIGURES).map(character).collect();
    project.locations = vec![Location {
        id: "l1".into(),
        name: "Halle".into(),
        set,
    }];
    project.scenes = vec![Scene {
        id: "s1".into(),
        number: "1".into(),
        heading: "INT. HALLE".into(),
        location: "l1".into(),
        plans: vec![plan],
    }];

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    klappe_store::save(&dir, &project, None).unwrap();

    Session::open(&dir).unwrap()
}

/// Median of `RUNS` runs of `work`.
fn median(mut work: impl FnMut()) -> Duration {
    let mut times: Vec<Duration> = (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            work();
            start.elapsed()
        })
        .collect();

    times.sort();
    times[RUNS / 2]
}

fn report(what: &str, took: Duration) {
    println!("perf {what}: {:.3} ms", took.as_secs_f64() * 1e3);
}

/// One edit and its undo, on an object in the middle of the plan.
fn edit_and_undo(s: &mut Session) {
    s.edit_plan(PlanEdit::Shift {
        source: Source::Object("o150".into()),
        dx_m: 0.5,
        dy_m: 0.0,
    })
    .unwrap();
    s.undo();
}

#[test]
#[ignore = "timing, run by `make perf` in release"]
fn a_large_plan_stays_fluid() {
    let mut s = big_session("perf-large");
    s.set_time(7.0);
    let items = s.display_list().items.len();
    println!("perf plan: {items} display items");

    let render = median(|| {
        std::hint::black_box(s.display_list());
    });
    report("display list", render);

    let miss = Point::new(0.01, 0.01);
    let pick = median(|| {
        std::hint::black_box(s.pick(miss));
    });
    report("pick (miss, scans all)", pick);

    let tick = median(|| {
        s.advance(TICK_S, 1.0, Looping::Shot);
        std::hint::black_box(s.display_list());
    });
    report("playback tick + display list", tick);

    let edit = median(|| edit_and_undo(&mut s));
    report("edit + undo", edit);

    let both = median(|| {
        s.advance(TICK_S, 1.0, Looping::Shot);
        std::hint::black_box(s.display_list());
        edit_and_undo(&mut s);
    });
    report("tick + display list + edit + undo", both);

    assert!(
        both < BUDGET * LIMIT_FACTOR,
        "{both:?} over {LIMIT_FACTOR}x the {BUDGET:?} budget"
    );
}
