#pragma once

#include <QtCore/QHash>
#include <QtCore/QList>
#include <QtCore/QObject>
#include <QtCore/QPointer>
#include <QtCore/QString>
#include <QtCore/QVariantList>
#include <QtCore/QVariantMap>
#include <QtGui/QColor>
#include <QtGui/QSyntaxHighlighter>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickTextDocument>

namespace klappe {

// Sets the lines of the screenplay editor as the page does (D-15): each block gets the
// margins and alignment of its kind, headings bold, sections, synopses and notes muted, and
// the stretches the script emphasises (D-36) bold, italic, underlined. Only the look changes;
// the text is what the editor shows.
//
//   QML: kinds "h cd"  ──apply──►  block 2: left 22 columns     (QTextBlockFormat)
//                                  block 0: bold                (ScriptFaces)
//
// Block formats would go into the document's undo history; the editor keeps its own of the
// text (ScriptHistory), so the look is no step there.
class ScriptLook : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickTextDocument* document READ document WRITE setDocument NOTIFY documentChanged)
    Q_PROPERTY(QColor mutedColor READ mutedColor WRITE setMutedColor)

public:
    struct Look {
        qreal left = 0;
        qreal right = 0;
        Qt::Alignment align = Qt::AlignLeft;
        bool bold = false;
        bool muted = false;
    };

    explicit ScriptLook(QObject* parent = nullptr);

    QQuickTextDocument* document() const;
    void setDocument(QQuickTextDocument* document);

    // `kinds`: one letter per block; `looks`: per letter `{ left, right, align, bold, muted }`,
    // margins in pixels, `align` "left", "right" or "center".
    Q_INVOKABLE void apply(const QString& kinds, const QVariantMap& looks);

    // The emphasis of the blocks `from` to `from + removed` is now that of `lines`: per block a
    // list of numbers, three for each stretch (first character, end, 1 bold + 2 italic +
    // 4 underline). `removed` -1: all the rest, not drawn yet.
    Q_INVOKABLE void setRuns(int from, int removed, const QVariantList& lines);

    // Draws all blocks again, after `setRuns` for all of them with the text already set.
    Q_INVOKABLE void redraw();

    // How block `block` is drawn now: "bold", "muted" or "" (what the highlighter put there).
    // For the test of the look (tests/controller/script_look.qml); the window does not ask.
    Q_INVOKABLE QString drawnFace(int block) const;
    // The emphasis drawn in block `block` as "from-to" and the letters b, i, u per stretch,
    // joined by blanks: "4-8b 9-12i". For the tests as well.
    Q_INVOKABLE QString drawnRuns(int block) const;

    // One stretch of emphasised characters.
    struct Run {
        int from = 0;
        int to = 0;
        int flags = 0;
    };
    // Flags of `Run`.
    static constexpr int BOLD = 1;
    static constexpr int ITALIC = 2;
    static constexpr int UNDERLINE = 4;

    Look lookOf(int block) const;
    QList<Run> runsOf(int block) const;
    QColor mutedColor() const;
    void setMutedColor(const QColor& color);

Q_SIGNALS:
    void documentChanged();

private:
    void setMargins();
    void refaceChanged(const QString& before);

    QPointer<QQuickTextDocument> m_document;
    // Owned by the document: gone with it.
    QPointer<QSyntaxHighlighter> m_faces;
    QString m_kinds;
    QHash<QChar, Look> m_looks;
    // By block; blocks past the end have none.
    QList<QList<Run>> m_runs;
    QColor m_muted;
};

}
