import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-15: the screenplay as it prints. The pages of `scriptPrintJson`, placed as the PDF places
// them (title page, page numbers, scene numbers at both sides), one under the other, scaled to
// the view's width and at most to the size of the paper on the screen. Read only.
//
//  ┌──────────────┐
//  │          2.  │   each text at its place in points; the page drawn in points, then scaled
//                         by `zoom` as a whole, so a line is as wide as on paper
//  │ INT. SAAL    │
//  │      ANNA    │
//  └──────────────┘
ListView {
    id: sheetList

    required property var controller
    required property var texts
    /// `{ fontPt, ascentPt, pages: [{ width, height, texts: [{ x, y, bold, text }] }] }`.
    property var sheet: ({ fontPt: 0, ascentPt: 0, pages: [] })
    /// Pixels per point: a 96 dpi screen shows the paper at its size; a narrower view less.
    readonly property real fullScale: 96 / 72
    readonly property real pageWidth: sheet.pages.length > 0 ? sheet.pages[0].width : 1
    readonly property real zoom: Math.min(fullScale, (width - 2 * Kirigami.Units.largeSpacing) / pageWidth)

    /// Lays the session's text out again.
    function refresh() {
        sheet = JSON.parse(controller.scriptPrintJson())
    }

    clip: true
    spacing: Kirigami.Units.largeSpacing
    topMargin: Kirigami.Units.largeSpacing
    bottomMargin: Kirigami.Units.largeSpacing
    model: sheet.pages
    QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
    Accessible.name: texts.t("screenplay.print")

    delegate: KanteCard {
        id: page
        required property var modelData
        required property int index

        objectName: "page." + index
        anchors.horizontalCenter: parent ? parent.horizontalCenter : undefined
        width: modelData.width * sheetList.zoom
        height: modelData.height * sheetList.zoom
        color: KanteStyle.cardColor
        borderColor: KanteStyle.frameColor
        chamfer: 0

        Item {
            width: page.modelData.width
            height: page.modelData.height
            transform: Scale { xScale: sheetList.zoom; yScale: sheetList.zoom }

            Repeater {
                model: page.modelData.texts
                delegate: Text {
                    required property var modelData
                    required property int index
                    objectName: "line." + index
                    x: modelData.x
                    y: modelData.y - sheetList.sheet.ascentPt
                    text: modelData.text
                    textFormat: Text.PlainText
                    font.family: KanteStyle.monoFamily
                    font.pixelSize: sheetList.sheet.fontPt
                    font.bold: modelData.bold
                    color: KanteStyle.textColor
                }
            }
        }
    }
}
