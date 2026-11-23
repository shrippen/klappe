//! A location's place and a scene's date and time (LH-52) in the project folder.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{FloorPlan, Location, Project, Scene, SetPlan, Site, SunTime};
use klappe_store::{load, save, StoreError};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

fn project(site: Option<Site>, sun: Option<SunTime>) -> Project {
    let mut project = Project::new("Film");
    project.locations.push(Location {
        id: "saal".into(),
        name: "Saal".into(),
        site,
        set: SetPlan::default(),
    });
    project.scenes = vec![Scene {
        id: "szene-001".into(),
        number: "1".into(),
        heading: "INT. SAAL – TAG".into(),
        location: Some("saal".into()),
        sun,
        plans: vec![FloorPlan {
            location: "saal".into(),
            ..FloorPlan::new("plan-a", "Plan A")
        }],
        ..Default::default()
    }];

    project
}

const BERLIN: Site = Site {
    latitude_deg: 52.52,
    longitude_deg: 13.405,
    north_deg: 90.0,
};

const NOON: SunTime = SunTime {
    year: 2026,
    month: 6,
    day: 21,
    hour: 13,
    minute: 8,
    utc_offset_min: Some(120),
};

#[test]
fn place_and_date_are_written_and_read_back() {
    let dir = test_dir("sun-round-trip");
    let written = project(Some(BERLIN), Some(NOON));

    save(&dir, &written, None).unwrap();
    let place = fs::read_to_string(dir.join("drehorte/saal.toml")).unwrap();
    let date = fs::read_to_string(dir.join("szenen/szene-001.json")).unwrap();

    assert!(
        place.contains("[site]") && place.contains("latitude_deg = 52.52"),
        "{place}"
    );
    assert!(
        date.contains("\"sun\"") && date.contains("\"utc_offset_min\": 120"),
        "{date}"
    );
    assert_eq!(load(&dir).unwrap().project, written);
}

#[test]
fn without_them_the_files_do_not_mention_the_sun() {
    let dir = test_dir("sun-none");

    save(&dir, &project(None, None), None).unwrap();
    let place = fs::read_to_string(dir.join("drehorte/saal.toml")).unwrap();
    let date = fs::read_to_string(dir.join("szenen/szene-001.json")).unwrap();

    assert!(!place.contains("site"), "{place}");
    assert!(!date.contains("sun"), "{date}");
    assert_eq!(load(&dir).unwrap().project, project(None, None));
}

#[test]
fn a_zone_that_follows_the_longitude_stores_no_offset() {
    let dir = test_dir("sun-no-offset");
    let written = project(
        Some(BERLIN),
        Some(SunTime {
            utc_offset_min: None,
            ..NOON
        }),
    );

    save(&dir, &written, None).unwrap();

    let date = fs::read_to_string(dir.join("szenen/szene-001.json")).unwrap();
    assert!(!date.contains("utc_offset_min"), "{date}");
    assert_eq!(load(&dir).unwrap().project, written);
}

#[test]
fn a_place_off_the_globe_or_a_date_that_does_not_exist_does_not_load() {
    let off = Site {
        latitude_deg: 95.0,
        ..BERLIN
    };
    let no_day = SunTime {
        month: 2,
        day: 30,
        ..NOON
    };

    for (name, written) in [
        ("sun-off-globe", project(Some(off), None)),
        ("sun-no-day", project(None, Some(no_day))),
    ] {
        let dir = test_dir(name);
        save(&dir, &written, None).unwrap();

        assert!(
            matches!(load(&dir), Err(StoreError::Parse { .. })),
            "{name}"
        );
    }
}
