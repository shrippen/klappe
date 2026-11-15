#pragma once

#include <memory>

#include <QtGui/QGuiApplication>

#include "rust/cxx.h"

namespace klappe {

// A QApplication (Qt Widgets), returned as its base class.
std::unique_ptr<QGuiApplication> newApplication(const rust::Vec<rust::String>& args);

// Whether the running application object is a QApplication.
bool isWidgetsApp();

}
