import QtQuick
import org.kde.kirigami as Kirigami
import Kante

// What each camera sees, as a small picture beside its body in the plan (LH-113, LH-36). Only
// the cameras whose body is on screen get one (`controller.cameraSpots()` lists those the layer
// and the track filter leave, the canvas those in view), at the small width the picture needs.
// A picture costs a few milliseconds (`make perf`), so a tick of `interval` ms makes one picture,
// the next camera that is out of date, and a change leaves the rest as it was: ten cameras bring
// the plan no lost frame (NF-05). The items stay while the cameras stay; only their place moves.
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
    // The cameras listed, `{ id, x_m, y_m, aspect }` by id; `ids` changes only when a camera
    // comes or goes, so the items below are not made again with every change.
    property var spotById: ({})
    property var ids: []
    // Counts the changes; a picture is out of date when it shows an earlier one.
    property int round: 0

    ViewPalette { id: colors }

    // Reads the spots again: where each body stands, and which cameras there are.
    function readSpots() {
        const spots = JSON.parse(controller.cameraSpots())
        const byId = {}
        for (const spot of spots) {
            byId[spot.id] = spot
        }
        spotById = byId
        const next = spots.map(s => s.id)
        if (next.join("\n") !== ids.join("\n")) {
            ids = next
        }
    }

    // One tick: the spots, then the picture of the first camera in view that is out of date; the
    // next tick follows while another is.
    function step() {
        if (!canPaint || !visible) {
            return
        }
        readSpots()
        let stale = 0
        for (let i = 0; i < items.count; i++) {
            const item = items.itemAt(i)
            if (!item || !item.inView || item.shownRound === round) {
                continue
            }
            if (stale === 0) {
                item.paint()
            }
            stale++
        }
        if (stale > 1) {
            pause.start()
        }
    }

    function refresh() {
        round++
        if (!pause.running) {
            pause.start()
        }
    }

    onVisibleChanged: refresh()
    onPaletteJsonChanged: refresh()
    Component.onCompleted: refresh()

    Timer {
        id: pause
        interval: thumbs.interval
        onTriggered: thumbs.step()
    }

    Connections {
        target: thumbs.canPaint ? thumbs.controller : null
        function onPictureRevisionChanged() {
            if (thumbs.visible) {
                thumbs.refresh()
            }
        }
    }

    Repeater {
        id: items
        model: thumbs.ids

        delegate: Item {
            id: thumb
            required property string modelData
            readonly property var spot: thumbs.spotById[modelData] || { x_m: 0, y_m: 0, aspect: 1.5 }
            readonly property point body: thumbs.canvas.mapFromModel(spot.x_m, spot.y_m)
            readonly property bool inView: body.x >= 0 && body.y >= 0
                                           && body.x <= thumbs.width && body.y <= thumbs.height
            property int shownRound: -1

            objectName: "cameraThumb-" + modelData
            visible: inView
            x: body.x + thumbs.gap
            y: body.y - height / 2
            width: thumbs.pictureWidth
            height: width / spot.aspect

            // Only a camera in view is painted, and only when it shows an earlier round.
            function paint() {
                if (!inView || surface.item === null) {
                    return
                }
                const px = Math.max(1, Math.round(width * Screen.devicePixelRatio))
                const image = thumbs.controller.cameraImageOf(modelData, px, thumbs.paletteJson)
                surface.item.show(image)
                shownRound = thumbs.round
            }

            // A camera that comes into view gets its picture with the next tick.
            onInViewChanged: {
                if (inView) {
                    thumbs.refresh()
                }
            }

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
                onLoaded: thumbs.refresh()
            }
        }
    }
}
