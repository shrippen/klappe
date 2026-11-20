#pragma once

#include <QtCore/QHash>
#include <QtCore/QObject>
#include <QtCore/QPointer>
#include <QtCore/QString>
#include <QtCore/QVariantMap>
#include <QtGui/QColor>
#include <QtGui/QSyntaxHighlighter>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickTextDocument>

namespace klappe {

// Sets the lines of the screenplay editor as the page does (D-15): each block gets the
// margins and alignment of its kind, headings bold, sections, synopses and notes muted.
// Only the look changes; the text stays plain Fountain.
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

    // How block `block` is drawn now: "bold", "muted" or "" (what the highlighter put there).
    // For the test of the look (tests/controller/script_look.qml); the window does not ask.
    Q_INVOKABLE QString drawnFace(int block) const;

    Look lookOf(int block) const;
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
    QColor m_muted;
};

}
