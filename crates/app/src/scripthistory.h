#pragma once

#include <QtCore/QElapsedTimer>
#include <QtCore/QList>
#include <QtCore/QObject>
#include <QtCore/QPointer>
#include <QtCore/QString>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickTextDocument>

namespace klappe {

// The undo history of the screenplay editor's text, in place of the document's own. That one
// records the look of the lines (ScriptLook) too, and a look joined to a typed character
// becomes a step of its own: Ctrl+Z would take back the indent and leave the text. This one
// records only the text, as steps of what was replaced where:
//
//   "Hallo"  typed letter by letter, without a pause  ─►  one step { at, removed "", put "Hallo" }
//
// Undo puts `removed` back in place of `put`; redo the other way round.
class ScriptHistory : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickTextDocument* document READ document WRITE setDocument NOTIFY documentChanged)
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY stepsChanged)
    Q_PROPERTY(bool canRedo READ canRedo NOTIFY stepsChanged)

public:
    struct Step {
        int at = 0;
        QString removed;
        QString put;
    };

    explicit ScriptHistory(QObject* parent = nullptr);

    QQuickTextDocument* document() const;
    void setDocument(QQuickTextDocument* document);
    bool canUndo() const;
    bool canRedo() const;

    // Forgets every step: the text came from outside (open, import, a restored backup).
    Q_INVOKABLE void reset();
    // Puts `text` in place of the text from `from` to `to`, as one change (capitals as typed).
    Q_INVOKABLE void replace(int from, int to, const QString& text);
    // Takes back the last step; returns where the cursor goes, -1 for none.
    Q_INVOKABLE int undo();
    Q_INVOKABLE int redo();

Q_SIGNALS:
    void documentChanged();
    void stepsChanged();

private:
    void changed();
    bool joins(const Step& step) const;
    int replay(const Step& step);

    QPointer<QQuickTextDocument> m_document;
    // The text before the last change, to see what it replaced.
    QString m_text;
    QList<Step> m_undo;
    QList<Step> m_redo;
    QElapsedTimer m_since;
    bool m_replaying = false;
};

}
