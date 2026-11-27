#pragma once

#include <QtCore/QElapsedTimer>
#include <QtCore/QList>
#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtCore/QVariantMap>
#include <QtQml/qqmlregistration.h>

namespace klappe {

// The undo history of the screenplay editor, in place of the text area's own. The area shows
// the script without the marks of Fountain, so the history keeps the steps of the Fountain
// source, as what was replaced where (positions of the source):
//
//   "Hallo"  typed letter by letter, without a pause  ─►  one step { at, removed "", put "Hallo" }
//
// The editor records each change of the source; undo and redo give back the change to make,
// and the editor makes it (and shows the lines it touched again).
class ScriptHistory : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY stepsChanged)
    Q_PROPERTY(bool canRedo READ canRedo NOTIFY stepsChanged)

public:
    struct Step {
        int at = 0;
        QString removed;
        QString put;
    };

    explicit ScriptHistory(QObject* parent = nullptr);

    bool canUndo() const;
    bool canRedo() const;

    // Forgets every step: the text came from outside (open, import, a restored backup).
    Q_INVOKABLE void reset();
    // The source had `removed` at `at` replaced by `put`; capitals as typed join the step.
    Q_INVOKABLE void record(int at, const QString& removed, const QString& put);
    // Takes back the last step: `{ at, removed, put }`, the change to make; empty for none.
    Q_INVOKABLE QVariantMap undo();
    Q_INVOKABLE QVariantMap redo();

Q_SIGNALS:
    void stepsChanged();

private:
    bool joins(const Step& step) const;

    QList<Step> m_undo;
    QList<Step> m_redo;
    QElapsedTimer m_since;
};

}
