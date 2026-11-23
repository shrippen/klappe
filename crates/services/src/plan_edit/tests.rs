use super::*;
use klappe_model::Location;

/// One scene with an empty plan and no cameras, on an empty set.
pub(super) fn empty() -> Project {
    let mut project = Project::new("Test");
    project.locations = vec![Location {
        id: "l".into(),
        name: "L".into(),
        set: SetPlan::default(),
        site: None,
    }];
    project.scenes = vec![Scene {
        id: "s".into(),
        number: "1".into(),
        heading: String::new(),
        location: Some("l".into()),
        plans: vec![FloorPlan {
            location: "l".into(),
            ..FloorPlan::new("p", "Plan A")
        }],
        ..Default::default()
    }];
    project
}

#[test]
fn a_missing_location_refuses_set_edits() {
    let mut project = empty();
    project.locations.clear();
    let edit = PlanEdit::AddWall {
        from: Point::new(0.0, 0.0),
        to: Point::new(1.0, 0.0),
    };

    assert!(matches!(
        apply(Path::new("."), &mut project, 0, 0, 0.0, (Snapping::On, Grid::default()), &edit),
        Err(ServiceError::NoLocation(ref id)) if id == "l"
    ));
}
