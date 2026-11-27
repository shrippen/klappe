#include "scriptlook.h"

#include <QtCore/QStringList>
#include <QtGui/QTextBlock>
#include <QtGui/QTextLayout>
#include <QtGui/QTextCursor>
#include <QtGui/QTextDocument>

namespace klappe {

// The character look of a block (bold, muted), from the kind its owner knows for it.
// A highlighter's formats are only drawn, never part of the text or its undo history.
class ScriptFaces : public QSyntaxHighlighter {
public:
    ScriptFaces(ScriptLook* owner, QTextDocument* document)
        : QSyntaxHighlighter(document)
        , m_owner(owner)
    {
    }

protected:
    void highlightBlock(const QString& text) override
    {
        const int block = currentBlock().blockNumber();
        const ScriptLook::Look look = m_owner->lookOf(block);
        const QList<ScriptLook::Run> runs = m_owner->runsOf(block);
        if (!look.bold && !look.muted && runs.isEmpty()) {
            return;
        }

        QTextCharFormat face;
        if (look.bold) {
            face.setFontWeight(QFont::Bold);
        }
        if (look.muted) {
            face.setForeground(m_owner->mutedColor());
        }
        if (look.bold || look.muted) {
            setFormat(0, text.length(), face);
        }

        for (const ScriptLook::Run& run : runs) {
            QTextCharFormat format = face;
            if (run.flags & ScriptLook::BOLD) {
                format.setFontWeight(QFont::Bold);
            }
            if (run.flags & ScriptLook::ITALIC) {
                format.setFontItalic(true);
            }
            if (run.flags & ScriptLook::UNDERLINE) {
                format.setFontUnderline(true);
            }
            setFormat(run.from, run.to - run.from, format);
        }
    }

private:
    ScriptLook* m_owner;
};

namespace {

Qt::Alignment alignOf(const QString& name)
{
    if (name == QStringLiteral("right")) {
        return Qt::AlignRight;
    }
    if (name == QStringLiteral("center")) {
        return Qt::AlignHCenter;
    }

    return Qt::AlignLeft;
}

bool sameMargins(const QTextBlockFormat& format, const ScriptLook::Look& look)
{
    return qFuzzyCompare(format.leftMargin() + 1, look.left + 1)
        && qFuzzyCompare(format.rightMargin() + 1, look.right + 1)
        && (format.alignment() & Qt::AlignHorizontal_Mask) == look.align;
}

}

ScriptLook::ScriptLook(QObject* parent)
    : QObject(parent)
{
}

QQuickTextDocument* ScriptLook::document() const
{
    return m_document;
}

void ScriptLook::setDocument(QQuickTextDocument* document)
{
    if (document == m_document) {
        return;
    }

    delete m_faces;
    m_document = document;
    if (document) {
        // The editor has its own history of the source (ScriptHistory). The area's list would
        // undo half a step from its context menu.
        document->textDocument()->setUndoRedoEnabled(false);
        m_faces = new ScriptFaces(this, document->textDocument());
    }
    Q_EMIT documentChanged();
}

ScriptLook::Look ScriptLook::lookOf(int block) const
{
    if (block < 0 || block >= m_kinds.size()) {
        return {};
    }

    return m_looks.value(m_kinds.at(block));
}

QList<ScriptLook::Run> ScriptLook::runsOf(int block) const
{
    return block >= 0 && block < m_runs.size() ? m_runs.at(block) : QList<Run>();
}

void ScriptLook::setRuns(int from, int removed, const QVariantList& lines)
{
    if (!m_document) {
        return;
    }

    QList<QList<Run>> runs;
    for (const QVariant& line : lines) {
        const QVariantList numbers = line.toList();
        QList<Run> stretches;
        for (int i = 0; i + 2 < numbers.size(); i += 3) {
            stretches.append({ numbers.at(i).toInt(), numbers.at(i + 1).toInt(), numbers.at(i + 2).toInt() });
        }
        runs.append(stretches);
    }

    while (m_runs.size() < from) {
        m_runs.append(QList<Run>());
    }
    const int count = removed < 0 ? m_runs.size() - from : qMin(removed, m_runs.size() - from);
    m_runs.erase(m_runs.begin() + from, m_runs.begin() + from + count);
    for (int i = 0; i < runs.size(); ++i) {
        m_runs.insert(from + i, runs.at(i));
    }

    // All lines: the text is set next and drawn with them; only the call that follows a text
    // already set draws it again (`redraw`).
    if (removed < 0) {
        return;
    }

    // The lines the text area drew while their runs were not known yet are drawn again.
    QTextDocument* doc = m_document->textDocument();
    for (int i = 0; i < runs.size() && m_faces; ++i) {
        const QTextBlock block = doc->findBlockByNumber(from + i);
        if (block.isValid()) {
            m_faces->rehighlightBlock(block);
        }
    }
}

void ScriptLook::redraw()
{
    if (m_faces) {
        m_faces->rehighlight();
    }
}

QString ScriptLook::drawnRuns(int block) const
{
    if (!m_document) {
        return {};
    }

    const QTextBlock found = m_document->textDocument()->findBlockByNumber(block);
    if (!found.isValid()) {
        return {};
    }

    QStringList drawn;
    for (const QTextLayout::FormatRange& range : found.layout()->formats()) {
        QString letters;
        if (range.format.fontWeight() == QFont::Bold) {
            letters += QLatin1Char('b');
        }
        if (range.format.fontItalic()) {
            letters += QLatin1Char('i');
        }
        if (range.format.fontUnderline()) {
            letters += QLatin1Char('u');
        }
        // The look of the line itself is no emphasis.
        const Look look = lookOf(block);
        if ((look.bold || look.muted) && range.start == 0 && range.length == found.length() - 1) {
            continue;
        }
        drawn.append(QStringLiteral("%1-%2%3").arg(range.start).arg(range.start + range.length).arg(letters));
    }

    return drawn.join(QLatin1Char(' '));
}

QString ScriptLook::drawnFace(int block) const
{
    if (!m_document) {
        return {};
    }

    const QTextBlock found = m_document->textDocument()->findBlockByNumber(block);
    if (!found.isValid() || found.layout()->formats().isEmpty()) {
        return {};
    }

    const QTextCharFormat face = found.layout()->formats().constFirst().format;
    if (face.fontWeight() == QFont::Bold) {
        return QStringLiteral("bold");
    }

    return face.hasProperty(QTextFormat::ForegroundBrush) ? QStringLiteral("muted") : QString();
}

QColor ScriptLook::mutedColor() const
{
    return m_muted;
}

void ScriptLook::setMutedColor(const QColor& color)
{
    if (color == m_muted) {
        return;
    }

    m_muted = color;
    if (m_faces) {
        m_faces->rehighlight();
    }
}

void ScriptLook::apply(const QString& kinds, const QVariantMap& looks)
{
    if (!m_document) {
        return;
    }

    m_looks.clear();
    for (auto it = looks.cbegin(); it != looks.cend(); ++it) {
        if (it.key().isEmpty()) {
            continue;
        }
        const QVariantMap look = it.value().toMap();
        m_looks.insert(it.key().at(0), Look {
            look.value(QStringLiteral("left")).toReal(),
            look.value(QStringLiteral("right")).toReal(),
            alignOf(look.value(QStringLiteral("align")).toString()),
            look.value(QStringLiteral("bold")).toBool(),
            look.value(QStringLiteral("muted")).toBool(),
        });
    }

    const QString before = m_kinds;
    m_kinds = kinds;
    refaceChanged(before);
    setMargins();
}

// Draws again the blocks whose kind changed; the highlighter drew the edited ones already.
// Lines put in or taken out move the blocks below to other numbers: then all are drawn again.
void ScriptLook::refaceChanged(const QString& before)
{
    if (!m_faces) {
        return;
    }
    if (before.size() != m_kinds.size()) {
        m_faces->rehighlight();
        return;
    }
    QTextDocument* doc = m_document->textDocument();

    for (QTextBlock block = doc->begin(); block.isValid(); block = block.next()) {
        const int at = block.blockNumber();
        const QChar was = at < before.size() ? before.at(at) : QChar();
        const QChar now = at < m_kinds.size() ? m_kinds.at(at) : QChar();
        if (was != now) {
            m_faces->rehighlightBlock(block);
        }
    }
}

void ScriptLook::setMargins()
{
    QTextDocument* doc = m_document->textDocument();

    QList<QTextBlock> stale;
    for (QTextBlock block = doc->begin(); block.isValid(); block = block.next()) {
        if (!sameMargins(block.blockFormat(), lookOf(block.blockNumber()))) {
            stale.append(block);
        }
    }
    if (stale.isEmpty()) {
        return;
    }

    QTextCursor cursor(doc);
    cursor.beginEditBlock();
    for (const QTextBlock& block : stale) {
        const Look look = lookOf(block.blockNumber());
        QTextBlockFormat format = block.blockFormat();
        format.setLeftMargin(look.left);
        format.setRightMargin(look.right);
        format.setAlignment(look.align);
        cursor.setPosition(block.position());
        cursor.setBlockFormat(format);
    }
    cursor.endEditBlock();
}

}
