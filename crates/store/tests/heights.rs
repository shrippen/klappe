//! Heights of the set on disk (LH-90, LH-91; format version 2).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Opening, OpeningKind, Project, FORMAT_VERSION};
use klappe_store::{load, save, StoreError};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const LOCATION: &str = "drehorte/saengersaal.toml";

fn example() -> Project {
    load(Path::new(EXAMPLE)).unwrap().project
}

/// A fresh, empty folder for one test.
fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

/// Empty heights write no key; given ones are written and read back.
#[test]
fn heights_are_written_only_when_given() {
    let dir = test_dir("set-heights");
    let mut project = example();
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();
    for key in ["\nheight_m", "sill_m", "head_m", "base_m"] {
        assert!(!text.contains(key), "{key}: {text}");
    }

    let set = &mut project.locations[0].set;
    set.wall_height_m = 4.5;
    set.walls[0].height_m = Some(1.2);
    let mut window = Opening::new(OpeningKind::Window, 3.0, 1.2);
    window.sill_m = Some(1.0);
    window.head_m = Some(2.4);
    set.walls[0].openings = vec![window];
    set.objects[0].base_m = Some(0.5);
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join(LOCATION)).unwrap();

    for key in [
        "wall_height_m = 4.5",
        "height_m = 1.2",
        "sill_m = 1.0",
        "head_m = 2.4",
        "base_m = 0.5",
    ] {
        assert!(text.contains(key), "{key}: {text}");
    }
    assert_eq!(load(&dir).unwrap().project, project);
}

/// A version 1 project has no room height: it loads with 2.6 m, the old fixed wall height, and
/// is saved as the current version.
#[test]
fn a_version_1_project_loads_and_saves_as_the_current_version() {
    let dir = test_dir("format-1");
    save(&dir, &example(), None).unwrap();
    let projekt = dir.join("projekt.toml");
    let text = fs::read_to_string(&projekt).unwrap();
    fs::write(
        &projekt,
        text.replace(
            &format!("format_version = {FORMAT_VERSION}"),
            "format_version = 1",
        ),
    )
    .unwrap();
    let location = fs::read_to_string(dir.join(LOCATION)).unwrap();
    fs::write(
        dir.join(LOCATION),
        location.replace("wall_height_m = 2.6\n", ""),
    )
    .unwrap();

    let project = load(&dir).unwrap().project;
    assert_eq!(project.locations[0].set.wall_height_m, 2.6);

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(&projekt).unwrap();
    assert!(
        text.contains(&format!("format_version = {FORMAT_VERSION}")),
        "{text}"
    );
}

/// Makes one height of the example unusable.
type Spoil = fn(&mut Project);

/// A height no edit would take is refused when the file is opened.
#[test]
fn an_unusable_height_is_refused() {
    let cases: [(&str, Spoil); 6] = [
        ("room below the floor", |p| {
            p.locations[0].set.wall_height_m = -1.0
        }),
        ("wall of no number", |p| {
            p.locations[0].set.walls[0].height_m = Some(f64::NAN)
        }),
        ("object under the floor", |p| {
            p.locations[0].set.objects[0].base_m = Some(-2.0)
        }),
        ("sill above the head", |p| {
            let mut window = Opening::new(OpeningKind::Window, 3.0, 1.2);
            window.sill_m = Some(3.0);
            p.locations[0].set.walls[0].openings = vec![window];
        }),
        ("head of nothing", |p| {
            let mut door = Opening::new(OpeningKind::Door, 3.0, 0.9);
            door.head_m = Some(0.0);
            p.locations[0].set.walls[0].openings = vec![door];
        }),
        ("absurd wall", |p| {
            p.locations[0].set.walls[0].height_m = Some(1e300)
        }),
    ];

    for (name, spoil) in cases {
        let dir = test_dir("bad-height");
        let mut project = example();
        spoil(&mut project);
        save(&dir, &project, None).unwrap();

        let refused = matches!(load(&dir), Err(StoreError::Parse { .. }));
        assert!(refused, "{name}");
    }
}
