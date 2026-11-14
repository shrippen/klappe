//! The fields of the selected camera or mark.

use super::q;
use super::qobject;
use crate::camera;
use crate::camera::CameraField;
use crate::mark;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Arrival, Departure, Pose, Session, Source};
use serde_json::json;

impl qobject::Controller {
    pub(super) fn set_camera_aim(self: Pin<&mut Self>, height: &QString, tilt: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = mark::aim_edit(selected, &height.to_string(), &tilt.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_camera_field(self: Pin<&mut Self>, field: &QString, value: &QString) -> bool {
        let Some(field) = CameraField::named(&field.to_string()) else {
            return false;
        };
        let Some(s) = self.rust().session.as_ref() else {
            return false;
        };
        let selected = self.rust().selected.only();
        let typed = camera::camera_edit(selected, s.camera_presets(), field, &value.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn mark_info(&self) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return q("{}");
        };

        q(&mark_json(s, self.rust().selected.only()).to_string())
    }

    pub(super) fn set_mark_pose(self: Pin<&mut Self>, pose: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = mark::pose_edit(selected, &pose.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn set_mark_arrival(self: Pin<&mut Self>, arrival: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = mark::arrival_edit(selected, &arrival.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn set_mark_departure(self: Pin<&mut Self>, departure: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = mark::departure_edit(selected, &departure.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn set_mark_hold(self: Pin<&mut Self>, seconds: &QString) -> bool {
        let selected = self.rust().selected.only();
        let typed = mark::hold_edit(selected, &seconds.to_string());

        self.commit_typed(typed)
    }

    pub(super) fn set_mark_pace(self: Pin<&mut Self>, pace: &QString) -> bool {
        let selected = self.rust().selected.only();
        let Some(edit) = mark::pace_edit(selected, &pace.to_string()) else {
            return false;
        };

        self.commit_edit(edit)
    }
}

/// The fields of the selected mark for QML; `{}` when the selection is no mark.
fn mark_json(s: &Session, selected: Option<&Source>) -> serde_json::Value {
    let Some(selected) = selected else {
        return json!({});
    };
    let Some(info) = s.mark_info(selected) else {
        return json!({});
    };
    let kind = match selected {
        Source::Camera(_) => "camera",
        _ => "figure",
    };

    json!({
        "kind": kind,
        "pose": info.pose.map(Pose::name).unwrap_or_default(),
        "arrival": info.arrival.map(Arrival::name).unwrap_or_default(),
        "departure": info.departure.map(Departure::name).unwrap_or_default(),
        "hold": info.hold_s,
        "hasWay": info.has_way,
        "pace": info.pace_m_s,
        "height": info.height_m,
        "tilt": info.tilt_deg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::tests::example;

    /// The mark fields read the selection: a figure's mark has a pose and a pace, a camera
    /// has neither a pose nor a first way; anything else has no fields.
    #[test]
    fn the_mark_fields_follow_the_selection() {
        let s = example();
        let mark = |index| Source::Mark {
            character: "elisabeth".into(),
            index,
        };

        let second = mark_json(&s, Some(&mark(1)));
        assert_eq!(second["kind"], "figure");
        assert_eq!(second["pose"], "standing");
        assert_eq!(second["arrival"], "walk");
        assert_eq!(second["departure"], "stay");
        assert_eq!(second["hold"], 4.0);
        assert_eq!(second["hasWay"], true);
        assert_eq!(second["pace"], 1.75);

        assert_eq!(mark_json(&s, Some(&mark(0)))["hasWay"], false);

        let camera = mark_json(&s, Some(&Source::Camera("a".into())));
        assert_eq!(camera["kind"], "camera");
        assert_eq!(camera["pose"], "");
        assert_eq!(camera["arrival"], "");
        assert_eq!(camera["departure"], "");
        assert_eq!(camera["hold"], 6.0);
        assert_eq!(camera["height"], 1.6);
        assert_eq!(camera["tilt"], 0.0);
        assert_eq!(second["height"], serde_json::Value::Null);
        assert_eq!(second["tilt"], serde_json::Value::Null);

        assert_eq!(mark_json(&s, Some(&Source::Wall(0))), json!({}));
        assert_eq!(mark_json(&s, None), json!({}));
    }
}
