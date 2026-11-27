#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QClipboard>
#include <QtGui/QGuiApplication>
#include <QtQml/qqmlregistration.h>

namespace klappe {

// Puts text on the clipboard for the screenplay editor: a copy of the shown text is the
// Fountain source of it, which the text area cannot give by itself.
class ScriptClipboard : public QObject {
    Q_OBJECT
    QML_ELEMENT

public:
    explicit ScriptClipboard(QObject* parent = nullptr)
        : QObject(parent)
    {
    }

    Q_INVOKABLE void setText(const QString& text)
    {
        QGuiApplication::clipboard()->setText(text);
    }
};

}
