#include "application.h"

#include <QtCore/QStandardPaths>
#include <QtGui/QIcon>
#include <QtWidgets/QApplication>

#include <vector>

namespace klappe {

namespace {

// QApplication keeps references to argc and argv for its whole life.
int g_argc = 0;
std::vector<QByteArray> g_args;
std::vector<char*> g_argv;

// The icon theme Kante's symbols are named after.
const QString kBreeze = QStringLiteral("breeze");

// Without a desktop's settings (the QA session, another desktop) Qt has no icon theme, only
// "hicolor": buttons showed their text ("Ausblenden: Wärter") and menus a wrong glyph until
// Kirigami set its own theme later. A theme the user chose stays.
void useIconTheme()
{
  const QString theme = QIcon::themeName();
  if (!theme.isEmpty() && theme != QStringLiteral("hicolor")) {
    return;
  }

  QStringList paths = QIcon::themeSearchPaths();
  const QStringList found = QStandardPaths::locateAll(
    QStandardPaths::GenericDataLocation, QStringLiteral("icons"), QStandardPaths::LocateDirectory);
  for (const QString& dir : found) {
    if (!paths.contains(dir)) {
      paths.append(dir);
    }
  }
  QIcon::setThemeSearchPaths(paths);
  QIcon::setThemeName(kBreeze);
}

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

  auto app = std::make_unique<QApplication>(g_argc, g_argv.data());
  useIconTheme();

  return app;
}

bool isWidgetsApp()
{
  return qobject_cast<QApplication*>(QCoreApplication::instance()) != nullptr;
}

}
