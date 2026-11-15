//! The free 3D view (LH-94): switching, navigating, its picture, and clicks and drags in it.
//! The picture is the session's, coloured like the camera view's; a click becomes what the
//! plan's click would be: a pick of the select tool, or a plan point of the floor for a tool.

use super::picture::colored;
use super::qobject;
use crate::plan::Palette;
use crate::tools::{Grip, Tool};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QString};
use klappe_services::{Size, View};

impl qobject::Controller {
    pub(super) fn toggle_view(mut self: Pin<&mut Self>) {
        let Some(s) = self.as_mut().session() else {
            return;
        };

        s.toggle_view();
        let free = s.view() == View::Free;
        self.as_mut().set_view_free(free);
        self.bump_free();
    }

    pub(super) fn free_size(mut self: Pin<&mut Self>, width_px: f64, height_px: f64) {
        self.as_mut().rust_mut().get_mut().free_size = (width_px, height_px);
    }

    pub(super) fn free_image(
        mut self: Pin<&mut Self>,
        scale: f64,
        palette_json: &QString,
    ) -> QImage {
        self.as_mut().rust_mut().get_mut().free_frame = String::new();
        let made = self.rust().free_colored(scale, &palette_json.to_string());
        let Some((image, frame)) = made else {
            return QImage::default();
        };

        self.as_mut().rust_mut().get_mut().free_frame = frame;
        image
    }

    pub(super) fn free_frame(&self) -> QString {
        QString::from(self.rust().free_frame.as_str())
    }

    pub(super) fn turn_view(mut self: Pin<&mut Self>, dx: f64, dy: f64) {
        if let Some(s) = self.as_mut().session() {
            s.turn_view(dx, dy);
        }
        self.bump_free();
    }

    pub(super) fn pan_view(mut self: Pin<&mut Self>, dx: f64, dy: f64) {
        let size = self.rust().free_size;
        if let Some(s) = self.as_mut().session() {
            s.pan_view(dx, dy, size);
        }
        self.bump_free();
    }

    pub(super) fn zoom_view(mut self: Pin<&mut Self>, steps: f64, x: f64, y: f64) {
        let size = self.rust().free_size;
        if let Some(s) = self.as_mut().session() {
            s.zoom_view(steps, (x, y), size);
        }
        self.bump_free();
    }

    pub(super) fn free_place(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        let size = self.rust().free_size;
        let tool = self.rust().tool.clone();
        let floor = {
            let this = self.as_ref();
            this.rust()
                .session
                .as_ref()
                .and_then(|s| s.free_floor_at((x, y), size))
        };

        // Above the horizon there is no floor, and nothing to click.
        let Some(floor) = floor else {
            return false;
        };
        if tool != Tool::Select {
            return self.place(floor.x_m, floor.y_m);
        }

        // As on the plan: a click on nothing clears the selection (Shift keeps it).
        let picked = {
            let this = self.as_ref();
            this.rust()
                .session
                .as_ref()
                .and_then(|s| s.free_pick((x, y), size))
        };
        let how = self.rust().pick_mode();
        let found = picked.is_some();
        let selected = self.rust().selected.clone().click(picked, how);
        self.as_mut().rust_mut().get_mut().selected = selected;
        self.refresh();
        found
    }

    pub(super) fn free_drag(
        mut self: Pin<&mut Self>,
        from_x: f64,
        from_y: f64,
        x: f64,
        y: f64,
        done: bool,
    ) -> bool {
        let size = self.rust().free_size;
        let tool = self.rust().tool.clone();

        // The element a drag moves is the one seen where it starts, taken once.
        if tool == Tool::Select && self.rust().grip == Grip::Free {
            let grabbed = {
                let this = self.as_ref();
                this.rust()
                    .session
                    .as_ref()
                    .and_then(|s| s.free_pick((from_x, from_y), size))
            };
            self.as_mut().rust_mut().get_mut().grip = Grip::Held(grabbed);
        }
        let nothing_held = self.rust().grip == Grip::Held(None);

        let floors = {
            let this = self.as_ref();
            this.rust().session.as_ref().and_then(|s| {
                Some((
                    s.free_floor_at((from_x, from_y), size)?,
                    s.free_floor_at((x, y), size)?,
                ))
            })
        };

        // Empty ground has no box to pull up in 3D, and a pointer above the horizon no floor:
        // nothing moves, and a release lets go.
        let Some((from, to)) = floors.filter(|_| !(tool == Tool::Select && nothing_held)) else {
            if done {
                self.cancel_drag();
            }
            return false;
        };

        self.drag(from.x_m, from.y_m, to.x_m, to.y_m, done)
    }

    /// The 3D picture is made again.
    fn bump_free(mut self: Pin<&mut Self>) {
        let revision = self.free_revision().wrapping_add(1);
        self.as_mut().set_free_revision(revision);
    }
}

impl super::ControllerRust {
    /// The 3D view at `scale` times `free_size`, in the palette's colours, and its frame JSON.
    fn free_colored(&self, scale: f64, palette_json: &str) -> Option<(QImage, String)> {
        let s = self.session.as_ref()?;
        let palette = Palette::parse(palette_json)?;
        let size: Size = (
            (self.free_size.0 * scale).round(),
            (self.free_size.1 * scale).round(),
        );

        colored(&s.free_picture(size)?, &palette)
    }
}
