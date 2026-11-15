//! The application object. Qt's platform theme (on KDE: plasma-integration) offers its native
//! file dialogs only to a `QApplication`; with a `QGuiApplication` QML's `FileDialog` and
//! `FolderDialog` fall back to Qt's own QML dialogs. cxx-qt-lib has no `QApplication`, so
//! `src/application.cpp` makes one and hands it over as the `QGuiApplication` it derives from.
//!
//! cxx-qt generates C++ glue with `unsafe` blocks; the workspace lint is relaxed for this file
//! only (agent.md allows `unsafe` in the cxx-qt bridge).
#![allow(unsafe_code)]

use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("klappe-app/src/application.h");
        include!("cxx-qt-lib/qguiapplication.h");
        #[namespace = ""]
        type QGuiApplication = cxx_qt_lib::QGuiApplication;

        #[namespace = "klappe"]
        #[rust_name = "new_application"]
        fn newApplication(args: &Vec<String>) -> UniquePtr<QGuiApplication>;

        #[namespace = "klappe"]
        #[rust_name = "is_widgets_app"]
        fn isWidgetsApp() -> bool;
    }
}

/// Initializes the window system and makes the application object, a `QApplication`.
pub fn new() -> UniquePtr<QGuiApplication> {
    let args: Vec<String> = std::env::args_os()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    ffi::new_application(&args)
}

/// Whether the running application object is a `QApplication`.
pub fn is_widgets() -> bool {
    ffi::is_widgets_app()
}
