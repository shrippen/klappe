#include "scripthistory.h"

#include <QtGui/QTextCursor>
#include <QtGui/QTextDocument>

namespace klappe {

namespace {

// Typing that rests this long starts a new step.
constexpr qint64 PAUSE_MS = 1000;
// Steps kept; the oldest go first.
constexpr int STEPS_MAX = 1000;

constexpr QChar LINE_BREAK = u'\n';

// What changed from `before` to `after`: the text between their common start and end.
ScriptHistory::Step difference(const QString& before, const QString& after)
{
    const int shorter = qMin(before.size(), after.size());
    int start = 0;
    while (start < shorter && before.at(start) == after.at(start)) {
        ++start;
    }

    int end = 0;
    while (end < shorter - start
           && before.at(before.size() - 1 - end) == after.at(after.size() - 1 - end)) {
        ++end;
    }

    return {
        start,
        before.mid(start, before.size() - start - end),
        after.mid(start, after.size() - start - end),
    };
}

}

ScriptHistory::ScriptHistory(QObject* parent)
    : QObject(parent)
{
}

QQuickTextDocument* ScriptHistory::document() const
{
    return m_document;
}

void ScriptHistory::setDocument(QQuickTextDocument* document)
{
    if (document == m_document) {
        return;
    }
    if (m_document) {
        disconnect(m_document->textDocument(), nullptr, this, nullptr);
    }

    m_document = document;
    if (document) {
        QTextDocument* doc = document->textDocument();
        doc->setUndoRedoEnabled(false);
        connect(doc, &QTextDocument::contentsChanged, this, &ScriptHistory::changed);
    }
    reset();
    Q_EMIT documentChanged();
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
    m_text = m_document ? m_document->textDocument()->toPlainText() : QString();
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

    return typing || deleting;
}

void ScriptHistory::changed()
{
    const QString now = m_document->textDocument()->toPlainText();
    if (m_replaying || now == m_text) {
        m_text = now;
        return;
    }

    const Step step = difference(m_text, now);
    m_text = now;

    if (joins(step)) {
        Step& last = m_undo.last();
        if (step.put.isEmpty()) {
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

// Puts `step.put` in place of `step.removed`; returns the position after it.
int ScriptHistory::replay(const Step& step)
{
    QTextCursor cursor(m_document->textDocument());
    cursor.setPosition(step.at);
    cursor.setPosition(step.at + step.removed.size(), QTextCursor::KeepAnchor);

    m_replaying = true;
    cursor.insertText(step.put);
    m_replaying = false;
    m_text = m_document->textDocument()->toPlainText();
    m_since.invalidate();

    return step.at + step.put.size();
}

int ScriptHistory::undo()
{
    if (!m_document || m_undo.isEmpty()) {
        return -1;
    }

    const Step step = m_undo.takeLast();
    m_redo.append(step);
    const int at = replay({ step.at, step.put, step.removed });
    Q_EMIT stepsChanged();

    return at;
}

int ScriptHistory::redo()
{
    if (!m_document || m_redo.isEmpty()) {
        return -1;
    }

    const Step step = m_redo.takeLast();
    m_undo.append(step);
    const int at = replay(step);
    Q_EMIT stepsChanged();

    return at;
}

}
