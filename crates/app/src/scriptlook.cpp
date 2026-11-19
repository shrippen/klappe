#include "scriptlook.h"

#include <QtGui/QTextBlock>
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
        const ScriptLook::Look look = m_owner->lookOf(currentBlock().blockNumber());
        if (!look.bold && !look.muted) {
            return;
        }

        QTextCharFormat face;
        if (look.bold) {
            face.setFontWeight(QFont::Bold);
        }
        if (look.muted) {
            face.setForeground(m_owner->mutedColor());
        }
        setFormat(0, text.length(), face);
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
