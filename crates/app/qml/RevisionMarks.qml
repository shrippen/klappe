import QtQuick
import QtQuick.Controls as QQC2
import Kante

// The revision's marks in the flowing text (DB-41): beside every line that is new or changed
// since the last version, an `*` and a thin stroke in the colour of the running revision. They
// sit on the text's left margin and move nothing. Place it in the text area; `ranges` are the
// marked lines as text positions, `{ from, to }`.
Item {
    id: marks

    required property var area
    required property var doc
    required property var texts
    /// The marked lines as text positions, `{ from, to }` (`ScreenplayView.lineRanges`).
    property var ranges: []
    property color ink: KanteStyle.textColor
    /// Of an older text than the one typed: pale like the page breaks.
    property bool stale: false

    anchors.fill: parent

    Repeater {
        model: marks.ranges
        delegate: Item {
            id: mark
            required property var modelData
            objectName: "revisionMark"
            // The text may have lost the line since the last layout.
            readonly property bool placed: modelData.from >= 0
            readonly property rect first: marks.area.contentHeight >= 0 && placed
                ? marks.doc.rectOf(modelData.from) : Qt.rect(0, 0, 0, 0)
            readonly property rect last: marks.area.contentHeight >= 0 && placed
                ? marks.doc.rectOf(modelData.to) : Qt.rect(0, 0, 0, 0)
            visible: placed
            x: 0
            y: first.y
            width: marks.area.leftPadding
            height: last.y + last.height - first.y
            opacity: marks.stale ? KanteStyle.dimOpacity : 1
            Accessible.role: Accessible.StaticText
            Accessible.name: marks.texts.t("version.revisionMark")

            Rectangle {
                width: KanteStyle.borderWidth * 2
                height: parent.height
                color: marks.ink
            }
            QQC2.Label {
                x: KanteStyle.borderWidth * 3
                text: "*"
                color: marks.ink
                font: KanteStyle.smallFont
            }
        }
    }
}
