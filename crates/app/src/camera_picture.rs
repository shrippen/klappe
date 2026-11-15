//! The camera's picture as a QML element (3D-KONZEPT.md, section 5.3): a `QQuickPaintedItem`
//! that holds a `QImage` and draws it scaled into its bounds. The controller makes the image
//! (`cameraImage`), QML hands it over with `show`.
//!
//! cxx-qt generates C++ glue with `unsafe` blocks; the workspace lint is relaxed for this file
//! only (agent.md allows `unsafe` in the cxx-qt bridge).
#![allow(unsafe_code)]

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QImage, QPainterRenderHint};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
        include!("cxx-qt-lib/qrect.h");
        type QRect = cxx_qt_lib::QRect;
        include!("cxx-qt-lib/qpainter.h");
        type QPainter = cxx_qt_lib::QPainter;
    }

    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickItem>);
        #[qobject]
        type QQuickItem;

        include!(<QtQuick/QQuickPaintedItem>);
        #[qobject]
        #[base = QQuickItem]
        type QQuickPaintedItem;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QQuickPaintedItem]
        #[qml_element]
        type CameraPicture = super::CameraPictureRust;

        /// Shows `image`, scaled into the item; a null image shows nothing.
        #[qinvokable]
        fn show(self: Pin<&mut Self>, image: &QImage);

        #[cxx_override]
        unsafe fn paint(self: Pin<&mut CameraPicture>, painter: *mut QPainter);
    }

    // A `QQuickItem` takes a `QQuickItem` as parent, not a `QObject`.
    impl cxx_qt::Constructor<(), BaseArguments = (*mut QQuickItem,)> for CameraPicture {}

    // Methods of the Qt base classes.
    unsafe extern "RustQt" {
        #[inherit]
        unsafe fn update(self: Pin<&mut CameraPicture>);

        #[inherit]
        unsafe fn width(self: &CameraPicture) -> f64;

        #[inherit]
        unsafe fn height(self: &CameraPicture) -> f64;
    }
}

use qobject::{QPainter, QQuickItem, QRect};

#[derive(Default)]
pub struct CameraPictureRust {
    image: QImage,
}

impl cxx_qt::Constructor<()> for qobject::CameraPicture {
    type BaseArguments = (*mut QQuickItem,);
    type InitializeArguments = ();
    type NewArguments = ();

    /// QML makes the item without a parent; it sets one afterwards.
    fn route_arguments(_: ()) -> ((), (*mut QQuickItem,), ()) {
        ((), (core::ptr::null_mut(),), ())
    }

    fn new(_: ()) -> CameraPictureRust {
        CameraPictureRust::default()
    }
}

impl qobject::CameraPicture {
    fn show(mut self: Pin<&mut Self>, image: &QImage) {
        self.as_mut().rust_mut().image = image.clone();
        // SAFETY: `update` only schedules a repaint of this live item, on the GUI thread QML
        // calls `show` from.
        unsafe { self.update() };
    }

    /// Draws the image over the whole item, smoothed: it is made at device pixels, so the
    /// scale is about 1.
    ///
    /// # Safety
    /// Qt calls it with a painter that is valid for the duration of the call.
    unsafe fn paint(self: Pin<&mut Self>, painter: *mut QPainter) {
        // SAFETY: Qt passes a live painter, and nothing else holds it during the call.
        let Some(painter) = (unsafe { painter.as_mut() }) else {
            return;
        };
        // SAFETY: width and height read the size of this live item.
        let target = QRect::new(
            0,
            0,
            self.width().round() as i32,
            self.height().round() as i32,
        );
        // SAFETY: the painter is not moved out of its place by the pin.
        let mut painter = unsafe { Pin::new_unchecked(painter) };

        painter
            .as_mut()
            .set_render_hint(QPainterRenderHint::SmoothPixmapTransform, true);
        painter.draw_image(&target, &self.rust().image);
    }
}
