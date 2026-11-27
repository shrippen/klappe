#include "scripthistory.h"

namespace klappe {

namespace {

// Typing that rests this long starts a new step.
constexpr qint64 PAUSE_MS = 1000;
// Steps kept; the oldest go first.
constexpr int STEPS_MAX = 1000;

constexpr QChar LINE_BREAK = u'\n';

// A change of as many characters inside what the last step put: capitals as the writer
// types, part of the same step.
bool within(const ScriptHistory::Step& step, const ScriptHistory::Step& last)
{
    const int end = step.at + step.removed.size();

    return !step.put.isEmpty() && step.removed.size() == step.put.size() && step.at >= last.at
        && end <= last.at + last.put.size();
}

}

ScriptHistory::ScriptHistory(QObject* parent)
    : QObject(parent)
{
}

bool ScriptHistory::canUndo() const
{
    return !m_undo.isEmpty();
}

bool ScriptHistory::canRedo() const
{
    return !m_redo.isEmpty();
}

void ScriptHistory::reset()
{
    m_undo.clear();
    m_redo.clear();
    m_since.invalidate();
    Q_EMIT stepsChanged();
}

// Typing on at the end of the last step, or deleting on before it, without a pause and within a
// line, is the same step.
bool ScriptHistory::joins(const Step& step) const
{
    if (m_undo.isEmpty() || !m_since.isValid() || m_since.elapsed() > PAUSE_MS) {
        return false;
    }

    const Step& last = m_undo.last();
    const bool typing = step.removed.isEmpty() && last.removed.isEmpty()
        && step.at == last.at + last.put.size() && !step.put.contains(LINE_BREAK)
        && !last.put.contains(LINE_BREAK);
    const bool deleting = step.put.isEmpty() && last.put.isEmpty()
        && step.at + step.removed.size() == last.at && !step.removed.contains(LINE_BREAK)
        && !last.removed.contains(LINE_BREAK);

    return typing || deleting || within(step, last);
}

void ScriptHistory::record(int at, const QString& removed, const QString& put)
{
    const Step step { at, removed, put };

    if (joins(step)) {
        Step& last = m_undo.last();
        if (within(step, last)) {
            last.put.replace(step.at - last.at, step.put.size(), step.put);
        } else if (step.put.isEmpty()) {
            last.at = step.at;
            last.removed = step.removed + last.removed;
        } else {
            last.put += step.put;
        }
    } else {
        m_undo.append(step);
        if (m_undo.size() > STEPS_MAX) {
            m_undo.removeFirst();
        }
    }
    m_redo.clear();
    m_since.start();
    Q_EMIT stepsChanged();
}

namespace {

QVariantMap asMap(const ScriptHistory::Step& step)
{
    return {
        { QStringLiteral("at"), step.at },
        { QStringLiteral("removed"), step.removed },
        { QStringLiteral("put"), step.put },
    };
}

}

QVariantMap ScriptHistory::undo()
{
    if (m_undo.isEmpty()) {
        return {};
    }

    const Step step = m_undo.takeLast();
    m_redo.append(step);
    m_since.invalidate();
    Q_EMIT stepsChanged();

    return asMap({ step.at, step.put, step.removed });
}

QVariantMap ScriptHistory::redo()
{
    if (m_redo.isEmpty()) {
        return {};
    }

    const Step step = m_redo.takeLast();
    m_undo.append(step);
    m_since.invalidate();
    Q_EMIT stepsChanged();

    return asMap(step);
}

}
