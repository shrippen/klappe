import QtQuick
import QtQuick.Controls as QQC2
import Kante

// The card of the selection (D-04): the object's, the camera's and the mark's fields in one
// card beside the selected element, with a pointer at it. A camera shows two sections
// (camera, mark at the playhead). It follows the element when the plan is panned, zoomed
// or played, stays inside the plan (on the left of the element when the right has no room)
// and keeps clear of the mouse pointer: it does not open under it. Nothing is selected, or
// several things: no card.
KantePopover {
    id: card

    required property var texts
    required property var controller
    /// The plan's KanteShapeCanvas: the element's place, in its pixels.
    required property Item canvas
    /// The mouse pointer in the canvas's pixels, when it is over it.
    property point pointer: Qt.point(0, 0)
    property bool pointerOver: false

    readonly property string key: controller.selectedKey
    readonly property bool wanted: objectSection.shown || cameraSection.shown || markSection.shown
    /// The rest of the fields of every section.
    property bool expanded: false
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: objectSection.typing || cameraSection.typing || markSection.typing

    // The plan keeps the keyboard (zoom keys, Escape for a drag); a field takes it on a click.
    // The card goes with the selection, not with a click or a key.
    focus: false
    closeOnOutside: false
    closePolicy: QQC2.Popup.NoAutoClose

    onKeyChanged: place()
    onWantedChanged: place()
    onImplicitWidthChanged: place()
    onImplicitHeightChanged: place()

    Connections {
        target: card.canvas
        function onPanXChanged() { card.place() }
        function onPanYChanged() { card.place() }
        function onZoomChanged() { card.place() }
        function onShapesChanged() { card.place() }
        function onWidthChanged() { card.place() }
        function onHeightChanged() { card.place() }
        function onVisibleChanged() { card.place() }
    }

    // The shape that stands for the element: the last of its key with a caption (the body of
    // a camera or a figure, not its path or view cone), else the last of its key.
    function shapeOf(shapes) {
        let last = null
        let captioned = null
        for (const s of shapes) {
            if (s.key !== card.key) {
                continue
            }
            last = s
            if (s.text !== "") {
                captioned = s
            }
        }
        return captioned || last
    }

    // Its box in the canvas's pixels, kept inside the canvas.
    function boxOf(shape) {
        const c = card.canvas
        let left = Infinity, right = -Infinity, top = Infinity, bottom = -Infinity
        const grow = shape.kind === "circle" ? shape.radius : 0
        for (let i = 0; i + 1 < shape.points.length; i += 2) {
            const p = c.mapFromModel(shape.points[i], shape.points[i + 1])
            left = Math.min(left, p.x - grow * c.zoom)
            right = Math.max(right, p.x + grow * c.zoom)
            top = Math.min(top, p.y - grow * c.zoom)
            bottom = Math.max(bottom, p.y + grow * c.zoom)
        }
        const keep = (v, max) => Math.max(0, Math.min(v, max))
        return { left: keep(left, c.width), right: keep(right, c.width),
                 mid: keep((top + bottom) / 2, c.height) }
    }

    // The element's box as the rectangle the card opens beside, its height only the middle.
    function rectOf(box) {
        return Qt.rect(box.left, box.mid, box.right - box.left, 0)
    }

    // Where the card would stand on `side` of the box, in canvas pixels, inside the canvas.
    function slot(box, side) {
        return card.rectSlot(card.canvas, card.rectOf(box), card.canvas, side)
    }

    function holds(s) {
        const p = card.pointer
        return card.pointerOver && p.x >= s.x && p.x <= s.x + card.implicitWidth
            && p.y >= s.y && p.y <= s.y + card.implicitHeight
    }

    // The mouse pointer is over the slot, which the card would cover; one the user is already
    // over (the card itself) does not count, or the card would run from the hand.
    function covers(s) {
        return card.holds(s) && !over.hovered
    }

    // The side of the element for the card: the one it has, else right; then the other one;
    // a side where it fits and does not cover the pointer first. No room on either side: Auto,
    // the card covers the element inside the plan.
    function sideFor(box) {
        const right = KantePopover.Side.Right
        const left = KantePopover.Side.Left
        const kept = card.visible && card.pointerSide === "right" ? left : right
        const sides = [kept, kept === right ? left : right]
        const fits = side => card.slot(box, side).fits
        const free = sides.find(side => fits(side) && !card.covers(card.slot(box, side)))
        return free || sides.find(fits) || KantePopover.Side.Auto
    }

    // Placing lays the card out, which changes its size and asks to place it again: that
    // asks wait for the run in progress, which then goes over with the new size.
    readonly property int maxRuns: 4
    property bool placing: false
    property bool placeAgain: false

    function place() {
        if (card.placing) {
            card.placeAgain = true
            return
        }
        card.placing = true
        for (let run = 0; run < card.maxRuns; run++) {
            card.placeAgain = false
            card.placeOnce()
            if (!card.placeAgain) {
                break
            }
        }
        card.placing = false
    }

    function placeOnce() {
        // A hidden plan (another tab of the narrow step) takes the card with it: a popup would
        // stay in the overlay.
        if (!card.wanted || card.canvas.width <= 0 || !card.canvas.visible) {
            card.close()
            return
        }
        const shape = card.shapeOf(card.canvas.shapes)
        if (!shape) {
            card.close()
            return
        }
        const box = card.boxOf(shape)
        card.openAtRect(card.canvas, card.rectOf(box), card.canvas, card.sideFor(box))
    }

    HoverHandler { id: over }

    ObjectCard {
        id: objectSection
        expanded: card.expanded
        showMore: false
        objectName: "objectCard"
        texts: card.texts
        controller: card.controller
    }
    CameraCard {
        id: cameraSection
        expanded: card.expanded
        showMore: false
        objectName: "cameraCard"
        texts: card.texts
        controller: card.controller
    }
    MarkCard {
        id: markSection
        expanded: card.expanded
        showMore: false
        objectName: "markCard"
        texts: card.texts
        controller: card.controller
    }

    KanteButton {
        objectName: "cardMore"
        emphasis: KanteButton.Emphasis.Quiet
        size: KanteButton.Size.Small
        text: card.texts.t(card.expanded ? "card.less" : "card.more")
        onClicked: card.expanded = !card.expanded
    }
}
