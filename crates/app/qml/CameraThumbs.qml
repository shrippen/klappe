import QtQuick
import org.kde.kirigami as Kirigami
import Kante

// What each camera sees, as a small picture beside its body in the plan (LH-113, LH-36). Only
// the cameras whose body is on screen get one (`controller.cameraSpots()` lists those the layer
// and the track filter leave, the canvas those in view), at the small width the picture needs, and
// the pictures are made again at most every `interval` ms: a burst of changes (playback, a drag)
// costs one round, so the plan stays fluid (NF-05). A camera whose picture did not change since
// keeps what it shows.
Item {
    id: thumbs

    required property var texts
    required property var controller
    /// The plan's canvas: where a body stands on screen.
    required property var canvas
    /// The element that shows an image (`show(image)`); a Rust type the tests do not have.
    property url surfaceSource: Qt.resolvedUrl("CameraSurface.qml")

    readonly property real pictureWidth: KanteStyle.unit(96)
    /// Gap between the body and its picture.
    readonly property real gap: KanteStyle.unit(14)
    readonly property int interval: 120
    readonly property bool canPaint: controller !== null && typeof controller.cameraSpots === "function"
                                     && typeof controller.cameraImageOf === "function"

    readonly property string paletteJson: colors.json
    // The cameras in view, `{ id, x_m, y_m, aspect }`, as the session lists them.
    property var spots: []
    // Counts the rounds: a picture is made again when its round is not the one it shows.
    property int round: 0

    ViewPalette { id: colors }

    // Asks for the spots and has every picture made again.
    function refresh() {
        if (!canPaint || !visible) {
            return
        }
        spots = JSON.parse(controller.cameraSpots())
        round++
    }

    onVisibleChanged: refresh()
    onPaletteJsonChanged: refresh()
    Component.onCompleted: refresh()

    Timer {
        id: pause
        interval: thumbs.interval
        onTriggered: thumbs.refresh()
    }

    Connections {
        target: thumbs.canPaint ? thumbs.controller : null
        function onPictureRevisionChanged() {
            if (thumbs.visible && !pause.running) {
                pause.start()
            }
        }
    }

    Repeater {
        model: thumbs.spots

        delegate: Item {
            id: thumb
            required property var modelData
            readonly property point body: thumbs.canvas.mapFromModel(modelData.x_m, modelData.y_m)
            readonly property bool inView: body.x >= 0 && body.y >= 0
                                           && body.x <= thumbs.width && body.y <= thumbs.height
            property int shownRound: -1

            objectName: "cameraThumb-" + modelData.id
            visible: inView
            x: body.x + thumbs.gap
            y: body.y - height / 2
            width: thumbs.pictureWidth
            height: width / modelData.aspect

            // Only a camera in view is painted, and only when its round has not been.
            function paint() {
                if (!inView || shownRound === thumbs.round || surface.item === null) {
                    return
                }
                const px = Math.max(1, Math.round(width * Screen.devicePixelRatio))
                const image = thumbs.controller.cameraImageOf(modelData.id, px, thumbs.paletteJson)
                surface.item.show(image)
                shownRound = thumbs.round
            }

            Connections {
                target: thumbs
                function onRoundChanged() {
                    thumb.paint()
                }
            }
            onInViewChanged: paint()

            Rectangle {
                anchors.fill: parent
                color: KanteStyle.backgroundColor
                border.width: KanteStyle.borderWidth
                border.color: KanteStyle.frameColor
            }
            Loader {
                id: surface
                anchors.fill: parent
                anchors.margins: KanteStyle.borderWidth
                source: thumbs.surfaceSource
                onLoaded: thumb.paint()
            }
        }
    }
}
