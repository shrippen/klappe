#include "application.h"

#include <QtWidgets/QApplication>

#include <vector>

namespace klappe {

namespace {

// QApplication keeps references to argc and argv for its whole life.
int g_argc = 0;
std::vector<QByteArray> g_args;
std::vector<char*> g_argv;

}

std::unique_ptr<QGuiApplication> newApplication(const rust::Vec<rust::String>& args)
{
  for (const auto& arg : args) {
    g_args.push_back(QByteArray(arg.data(), static_cast<qsizetype>(arg.size())));
  }
  for (auto& arg : g_args) {
    g_argv.push_back(arg.data());
  }
  g_argv.push_back(nullptr);
  g_argc = static_cast<int>(g_args.size());

  return std::make_unique<QApplication>(g_argc, g_argv.data());
}

bool isWidgetsApp()
{
  return qobject_cast<QApplication*>(QCoreApplication::instance()) != nullptr;
}

}
