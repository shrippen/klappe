import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The free 3D view of the set (LH-94, 3D-KONZEPT.md section 6): the controller's pixel picture
// filling the item, with the names written over it. No geometry here: the controller turns a
// pixel into what it shows and the floor under it. The mouse:
//   left click / drag        select, set, move: as on the plan, through the floor under the pointer
//   right drag, Alt + left   turn         middle drag, Shift + right   shift
//   wheel                    zoom to the pointer; a touchpad's two-finger scroll shifts, a pinch zooms
Item {
    id: view

    required property var texts
    // `freeSize(w, h)`, `freeImage(scale, paletteJson)`, `freeFrame()`, `turnView`, `panView`,
    // `zoomView`, `freePlace`, `freeDrag`, `cancelDrag` and the revisions `pictureRevision` and
    // `freeRevision`: the picture is made again when a revision, the size or the theme changes.
    property var controller: null
    // The element that shows the image (`show(image)`); a Rust type the tests do not have.
    property url surfaceSource: Qt.resolvedUrl("CameraSurface.qml")

    // A controller without `freeImage` (the stand-in of a test that is not about 3D) gets no
    // surface.
    readonly property bool canPaint: controller !== null && typeof controller.freeImage === "function"

    // `{ width, height, labels: [{ text, x, y }] }` of the picture shown, in its pixels; null
    // without one.
    property var frame: null
    // A turn or a shift is under way: the picture is made smaller, so it keeps up with the hand.
    readonly property bool interacting: gesture === "turn" || gesture === "pan"
    // How much of the device pixels the picture has while it is under way.
    readonly property real handScale: 0.5
    // A drag of the left button that edits is under way.
    readonly property bool dragging: gesture === "edit" && moved
    // Where the last click or release was, in this item's pixels: the card of the selection
    // opens there.
    property point clickAt: Qt.point(width / 2, height / 2)
    readonly property real dpr: Screen.devicePixelRatio
    // A step of the wheel in the units of Qt, and of a touchpad's scroll in pixels.
    readonly property real wheelStep: 120
    readonly property real scrollPixelsPerStep: 60
    // Wheel steps one of the menu's zoom entries makes.
    readonly property real menuStep: 1
    // How much a pinch scales for one step of the wheel: the core's step (0.85) the other way.
    readonly property real pinchPerStep: 1.18

    // "", or what the pressed button does: "turn", "pan" or "edit" (click or drag).
    property string gesture: ""
    property bool moved: false
    property point pressAt: Qt.point(0, 0)
    property point lastAt: Qt.point(0, 0)

    ViewPalette { id: colors }

    // Zooms about the middle of the view: "in" or "out".
    function zoomBy(how) {
        if (!canPaint) {
            return
        }
        controller.freeSize(width * dpr, height * dpr)
        controller.zoomView(how === "in" ? menuStep : how === "out" ? -menuStep : 0,
                            width * dpr / 2, height * dpr / 2)
    }

    // Asks the controller for the picture at the device pixels of the item.
    function refresh() {
        if (!canPaint || !visible || width <= 0 || height <= 0 || surface.item === null) {
            return
        }

        controller.freeSize(width * dpr, height * dpr)
        const image = controller.freeImage(interacting ? handScale : 1, colors.json)
        reframe()
        surface.item.show(image)
    }

    // The names of the picture again, with the height drag's caption at the pointer.
    function reframe() {
        const text = controller.freeFrame()
        frame = text ? JSON.parse(text) : null
    }

    onWidthChanged: refresh()
    onHeightChanged: refresh()
    onVisibleChanged: refresh()
    onControllerChanged: refresh()
    onInteractingChanged: refresh()
    Connections {
        target: view.canPaint ? view.controller : null
        function onPictureRevisionChanged() { view.refresh() }
        function onFreeRevisionChanged() { view.refresh() }
    }
    Connections {
        target: colors
        function onJsonChanged() { view.refresh() }
    }

    Accessible.role: Accessible.Graphic
    Accessible.name: view.texts.t("view.free")

    Loader {
        id: surface
        objectName: "freeSurface"
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
            readonly property real ratio: view.width / view.frame.width

            objectName: "freeLabel"
            x: modelData.x * ratio - width / 2
            y: modelData.y * ratio - height
            text: modelData.text
            font: KanteStyle.smallFont
            color: KanteStyle.textColor
            style: Text.Outline
            styleColor: KanteStyle.backgroundColor
        }
    }

    QQC2.Label {
        objectName: "freeHint"
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        visible: view.frame === null
        wrapMode: Text.WordWrap
        color: KanteStyle.mutedTextColor
        text: view.texts.t("view.freeNone")
    }

    // Escape drops the drag in progress; only then, so it stays free for dialogs.
    Shortcut {
        sequence: "Escape"
        enabled: view.dragging
        onActivated: {
            view.gesture = ""
            view.controller.cancelDrag()
            view.reframe()
        }
    }

    function press(mouse) {
        const shift = (mouse.modifiers & Qt.ShiftModifier) !== 0
        const alt = (mouse.modifiers & Qt.AltModifier) !== 0
        view.forceActiveFocus()
        pressAt = Qt.point(mouse.x, mouse.y)
        lastAt = pressAt
        moved = false
        switch (mouse.button) {
        case Qt.RightButton: gesture = shift ? "pan" : "turn"; break
        case Qt.MiddleButton: gesture = "pan"; break
        case Qt.LeftButton: gesture = alt ? "turn" : "edit"; break
        default: gesture = ""
        }
    }

    function move(mouse) {
        const dx = mouse.x - lastAt.x
        const dy = mouse.y - lastAt.y
        lastAt = Qt.point(mouse.x, mouse.y)
        if (gesture === "turn") {
            controller.turnView(dx, dy)
        } else if (gesture === "pan") {
            controller.panView(dx * dpr, dy * dpr)
        } else if (gesture === "edit") {
            dragTo(mouse, false)
        }
    }

    // A left drag starts once the pointer has left the click's slop.
    function dragTo(mouse, done) {
        const far = Math.hypot(mouse.x - pressAt.x, mouse.y - pressAt.y) > Qt.styleHints.startDragDistance
        moved = moved || far
        if (!moved) {
            return
        }
        controller.freeDrag(pressAt.x * dpr, pressAt.y * dpr, mouse.x * dpr, mouse.y * dpr, done)
        reframe()
    }

    function release(mouse) {
        const edits = gesture === "edit"
        const dragged = moved
        if (edits) {
            clickAt = Qt.point(mouse.x, mouse.y)
            dragged ? dragTo(mouse, true) : controller.freePlace(mouse.x * dpr, mouse.y * dpr)
        }
        gesture = ""
        moved = false
    }

    // Steps of a wheel, or of a touchpad's scroll with Ctrl held (a pinch on some systems).
    function wheeled(wheel) {
        const touchpad = wheel.pixelDelta.x !== 0 || wheel.pixelDelta.y !== 0
        const zooms = (wheel.modifiers & Qt.ControlModifier) !== 0 || !touchpad
        if (!zooms) {
            controller.panView(wheel.pixelDelta.x * dpr, wheel.pixelDelta.y * dpr)
            return
        }
        const steps = touchpad ? wheel.pixelDelta.y / scrollPixelsPerStep : wheel.angleDelta.y / wheelStep
        controller.zoomView(steps, wheel.x * dpr, wheel.y * dpr)
    }

    MouseArea {
        id: input
        objectName: "freeInput"
        anchors.fill: parent
        enabled: view.canPaint
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onPressed: mouse => view.press(mouse)
        onPositionChanged: mouse => view.move(mouse)
        onReleased: mouse => view.release(mouse)
        onCanceled: { view.gesture = ""; view.moved = false }
        onWheel: wheel => view.wheeled(wheel)

        // A pinch on a touchpad or a touch screen zooms to the middle of the fingers.
        PinchHandler {
            id: pinch
            target: null
            property real last: 1
            onActiveChanged: last = 1
            onActiveScaleChanged: {
                if (!active) {
                    return
                }
                const steps = Math.log(activeScale / last) / Math.log(view.pinchPerStep)
                last = activeScale
                view.controller.zoomView(steps, centroid.position.x * view.dpr, centroid.position.y * view.dpr)
            }
        }
    }

}
