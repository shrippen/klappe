import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// What the camera sees (LH-36, LH-62, LH-99): the controller's pixel picture, covering right
// per pixel, with the names written over it. The frame has the camera's aspect ratio (LH-38).
// Empty with a hint while the shot's camera has no marks.
Item {
    id: view

    required property var texts
    // `cameraImage(widthPx, paletteJson)`, `cameraFrame()` and `pictureRevision`: the picture
    // is made again when the revision, the size or the theme changes.
    property var controller: null
    // The element that shows the image (`show(image)`); a Rust type the tests do not have.
    property url surfaceSource: Qt.resolvedUrl("CameraSurface.qml")

    // A controller without `cameraImage` (the stand-in of a test that is not about the picture)
    // gets no surface.
    readonly property bool canPaint: controller !== null && typeof controller.cameraImage === "function"

    // `{ width, height, labels: [{ text, x, y }] }` of the picture shown, in its pixels; null
    // without one.
    property var frame: null

    readonly property real aspect: frame ? frame.height / frame.width : 0
    // The frame never takes more than this share of the width as height.
    readonly property real maxHeightShare: 0.6
    // Kante's colours as the theme has them now, for the controller to colour the roles.
    readonly property string paletteJson: colors.json

    implicitHeight: frame
                    ? Math.min(width * aspect, width * maxHeightShare)
                    : hint.implicitHeight + 2 * Kirigami.Units.largeSpacing

    ViewPalette { id: colors }

    // Asks the controller for the picture at the device pixels of the item's width.
    function refresh() {
        if (!canPaint || width <= 0 || surface.item === null) {
            return
        }

        const widthPx = Math.max(1, Math.round(width * Screen.devicePixelRatio))
        const image = controller.cameraImage(widthPx, paletteJson)
        const text = controller.cameraFrame()
        frame = text ? JSON.parse(text) : null
        surface.item.show(image)
    }

    onWidthChanged: refresh()
    onControllerChanged: refresh()
    onPaletteJsonChanged: refresh()

    Connections {
        target: view.canPaint ? view.controller : null
        function onPictureRevisionChanged() {
            view.refresh()
        }
    }

    QQC2.Label {
        id: hint
        objectName: "viewHint"
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        visible: view.frame === null
        wrapMode: Text.WordWrap
        color: KanteStyle.mutedTextColor
        text: view.texts.t("view.none")
    }

    // The picture, as large as fits with the camera's aspect, centred.
    Item {
        id: box
        objectName: "viewBox"
        anchors.horizontalCenter: parent.horizontalCenter
        height: parent.height
        width: view.frame ? Math.min(view.width, height / view.aspect) : 0
        visible: view.frame !== null
        clip: true

        Accessible.role: Accessible.Graphic
        Accessible.name: view.texts.t("view.title")

        Loader {
            id: surface
            objectName: "viewSurface"
            anchors.fill: parent
            active: view.canPaint
            source: view.surfaceSource
            onLoaded: view.refresh()
        }

        Repeater {
            model: view.frame ? view.frame.labels : []

            // A name above the point it belongs to; the outline keeps it readable on any pixel.
            Text {
                required property var modelData
                readonly property real ratio: box.width / view.frame.width

                objectName: "viewLabel"
                x: modelData.x * ratio - width / 2
                y: modelData.y * ratio - height
                text: modelData.text
                font: KanteStyle.smallFont
                color: KanteStyle.textColor
                style: Text.Outline
                styleColor: KanteStyle.backgroundColor
            }
        }
    }
}
