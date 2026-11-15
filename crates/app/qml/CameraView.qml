import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// What the camera sees (LH-36, LH-62): Stellprobe's projection, drawn as it comes (no
// geometry here). The frame has the camera's aspect ratio (LH-38). Empty with a hint while
// no camera is selected.
Item {
    id: view

    required property var texts
    // `{ width, depth, shapes }` from the controller; empty without a camera.
    property string json: ""

    readonly property var picture: json ? JSON.parse(json) : null
    // The frame never takes more than this share of the width as height.
    readonly property real maxHeightShare: 0.6

    implicitHeight: picture
                    ? Math.min(width * picture.depth / picture.width, width * maxHeightShare)
                    : hint.implicitHeight + 2 * Kirigami.Units.largeSpacing

    QQC2.Label {
        id: hint
        objectName: "viewHint"
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        visible: view.picture === null
        wrapMode: Text.WordWrap
        color: KanteStyle.mutedTextColor
        text: view.texts.t("view.none")
    }

    KanteShapeCanvas {
        id: canvas
        objectName: "viewCanvas"
        anchors.fill: parent
        visible: view.picture !== null
        contentWidth: view.picture ? view.picture.width : 1
        contentHeight: view.picture ? view.picture.depth : 1
        shapes: view.picture ? view.picture.shapes : []
        accessibleName: view.texts.t("view.title")

        // The frame is the whole picture: a new aspect ratio or size fits it again.
        onWidthChanged: fit()
        onHeightChanged: fit()
        onContentHeightChanged: fit()
    }
}
