//! Scenes, locations and characters of the project.

use super::qobject;
use crate::manage;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{
    read_site, read_sun_time, Field, FigureColor, ProjectEdit, ServiceError, Site, SunTime,
};

impl qobject::Controller {
    pub(super) fn add_scene(
        self: Pin<&mut Self>,
        number: &QString,
        heading: &QString,
        location: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::AddScene {
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
        }))
    }

    pub(super) fn edit_scene(
        self: Pin<&mut Self>,
        id: &QString,
        number: &QString,
        heading: &QString,
        location: &QString,
        sun: &QString,
    ) -> bool {
        let edit = sun_of(sun).map(|sun| ProjectEdit::EditScene {
            id: id.to_string(),
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
            sun,
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_scene(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveScene { id: id.to_string() }))
    }

    pub(super) fn add_location(
        self: Pin<&mut Self>,
        name: &QString,
        width: &QString,
        depth: &QString,
        height: &QString,
        site: &QString,
    ) -> bool {
        let edit = sizes(width, depth, height).and_then(|(width_m, depth_m, wall_height_m)| {
            Ok(ProjectEdit::AddLocation {
                name: name.to_string(),
                width_m,
                depth_m,
                wall_height_m,
                site: site_of(site)?,
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn edit_location(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        width: &QString,
        depth: &QString,
        height: &QString,
        site: &QString,
    ) -> bool {
        let edit = sizes(width, depth, height).and_then(|(width_m, depth_m, wall_height_m)| {
            Ok(ProjectEdit::EditLocation {
                id: id.to_string(),
                name: name.to_string(),
                width_m,
                depth_m,
                wall_height_m,
                site: site_of(site)?,
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_location(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveLocation { id: id.to_string() }))
    }

    pub(super) fn add_character(
        self: Pin<&mut Self>,
        name: &QString,
        label: &QString,
        color: &QString,
        group: u32,
        size: &QString,
    ) -> bool {
        let edit = color_edit(color).and_then(|color| {
            Ok(ProjectEdit::AddCharacter {
                name: name.to_string(),
                short: label.to_string(),
                color,
                group: manage::group_of(group),
                size_m: manage::figure_size(&size.to_string())?,
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn edit_character(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        label: &QString,
        color: &QString,
        group: u32,
        size: &QString,
    ) -> bool {
        let edit = color_edit(color).and_then(|color| {
            Ok(ProjectEdit::EditCharacter {
                id: id.to_string(),
                name: name.to_string(),
                short: label.to_string(),
                color,
                group: manage::group_of(group),
                size_m: manage::figure_size(&size.to_string())?,
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_character(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveCharacter { id: id.to_string() }))
    }
}

/// Width, depth and room height typed in the location form.
fn sizes(
    width: &QString,
    depth: &QString,
    height: &QString,
) -> Result<(f64, f64, f64), ServiceError> {
    manage::sizes(&width.to_string(), &depth.to_string(), &height.to_string())
}

/// The three texts of a field group (LH-52): a JSON array of strings, as the form sends it.
fn texts3(json: &QString, field: Field) -> Result<[String; 3], ServiceError> {
    serde_json::from_str(&json.to_string()).map_err(|_| ServiceError::BadValue {
        field,
        value: json.to_string(),
    })
}

/// Latitude, longitude and north typed in the location form.
fn site_of(site: &QString) -> Result<Option<Site>, ServiceError> {
    let [latitude, longitude, north] = texts3(site, Field::Latitude)?;

    read_site(&latitude, &longitude, &north)
}

/// Date, time and offset from UTC typed in the scene form.
fn sun_of(sun: &QString) -> Result<Option<SunTime>, ServiceError> {
    let [date, time, offset] = texts3(sun, Field::Date)?;

    read_sun_time(&date, &time, &offset)
}

/// The colour a form sends by its role name.
fn color_edit(name: &QString) -> Result<FigureColor, ServiceError> {
    let name = name.to_string();

    FigureColor::from_name(&name).ok_or(ServiceError::BadValue {
        field: Field::Color,
        value: name,
    })
}
