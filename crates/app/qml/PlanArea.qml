import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami
import Kante

// The floor plan: Stellprobe's display list drawn by KanteShapeCanvas (no geometry here,
// agent.md), with a tooltip for what the pointer is over (round 2, design 4). The drawing
// tools (PlanTools) pass clicks and drags on as they come, in metres; the controller edits.
Item {
    id: area

    required property var texts
    required property var controller

    readonly property var plan: controller.planJson ? JSON.parse(controller.planJson) : null
    /// The tool of PlanTools: select, wall, object, figure, camera, light, scale, measure, note, image, line,
    /// arrow, rect, ellipse or area.
    property string tool: "select"
    /// What the next click of the tool does (LH-101, PlanTools `hint`); empty for none.
    property string toolHint: ""
    /// Escape, with nothing else to end, goes back to the select tool (LH-101).
    signal selectToolRequested()
    /// The grid drawn and snapped to, in metres: the one `gridName` names in `choicesJson`.
    readonly property real gridStepM: {
        const grids = controller.choicesJson ? (JSON.parse(controller.choicesJson).grid || []) : []
        const grid = grids.find(g => g.id === controller.gridName)
        return grid ? grid.step_m : 1
    }
    /// What the canvas draws: the chosen grid, or 1 m while that is too fine for the zoom (it
    /// would not show at all, and an overview of a large set would have no grid).
    readonly property real drawnGridM: gridStepM * canvas.zoom >= KanteStyle.unit(minGridGapUnits) ? gridStepM : 1
    /// Smallest gap between grid lines on screen, as `KanteShapeCanvas` hides closer ones.
    readonly property int minGridGapUnits: 6
    property string tipKey: ""
    property bool dragging: false
    /// Escape aborted the drag in progress; the canvas goes on reporting until release.
    property bool dragAborted: false
    property point tipAt: Qt.point(0, 0)
    /// The 3D view shows instead of the plan (J).
    readonly property bool free: controller.viewFree === true

    KanteEmptyState {
        anchors.centerIn: parent
        width: Math.min(parent.width, Kirigami.Units.gridUnit * 24)
        visible: !area.controller.loaded
        title: area.texts.t("empty.title")
        text: area.texts.t("empty.text")
    }

    KanteShapeCanvas {
        id: canvas
        objectName: "planCanvas"
        anchors.fill: parent
        visible: area.plan !== null && !area.free
        contentWidth: area.plan ? area.plan.width : 1
        contentHeight: area.plan ? area.plan.depth : 1
        gridStep: area.drawnGridM
        shapes: area.plan ? area.plan.shapes : []
        accessibleName: area.texts.t("app.module")
        // A left drag always edits: the select tool moves what it grabbed, or on empty ground
        // pulls up a box to select in. The view pans with the middle button and with space.
        editing: true

        onShapeHovered: (key, x, y) => {
            area.tipKey = key
            area.tipAt = canvas.mapFromModel(x, y)
        }
        onShapeClicked: (key, x, y) => {
            // Only a click that selects opens the card again.
            if (area.tool === "select") {
                selection.dismissed = false
            }
            // The image tool asks for the file once its layer is open.
            if (area.controller.place(x, y) && area.tool === "image") {
                area.askImage(x, y)
            }
        }
        // The click before it selected the note or the bend; the double click edits the note's
        // text, or takes the bend out of its way (LH-21).
        onShapeDoubleClicked: (key, x, y) => {
            if (area.tool !== "select") {
                return
            }
            if (key.indexOf("note:") === 0) {
                area.controller.editSelectedNote()
            }
            if (area.isBend(key)) {
                area.controller.removeSelected()
            }
        }
        onShapeDragged: (key, fromX, fromY, x, y, done) => {
            if (area.dragAborted) {
                area.dragAborted = done ? false : true
                area.dragging = false
                return
            }
            area.dragging = !done
            area.controller.drag(fromX, fromY, x, y, done)
        }
    }

    // The free 3D view (LH-94) fills the plan's place; its clicks and drags go through the same
    // controller as the plan's.
    FreeView {
        id: freeView
        objectName: "freeView"
        anchors.fill: parent
        visible: area.free && area.plan !== null
        texts: area.texts
        controller: area.controller
    }

    /// Zooms about the middle of the view: "in", "out" or "fit" (the whole plan).
    readonly property real zoomStep: 1.25
    function zoomBy(how) {
        if (area.free) {
            freeView.zoomBy(how)
            return
        }
        switch (how) {
        case "in": canvas.zoom *= zoomStep; break
        case "out": canvas.zoom /= zoomStep; break
        case "fit": canvas.fit(); break
        }
    }

    // Image tool (LH-18): the file the click is to place, in the formats the background takes.
    // Made on the first click: a dialog that exists takes the keyboard focus of the plan's fields.
    property point imageAt: Qt.point(0, 0)
    Component {
        id: imageDialogComponent
        FileDialog {
            title: area.texts.t("image.pick")
            nameFilters: [area.texts.t("background.filter")]
            onAccepted: area.controller.addImage(selectedFile.toString(), area.imageAt.x, area.imageAt.y)
        }
    }
    property var imageDialog: null
    function askImage(x, y) {
        imageAt = Qt.point(x, y)
        if (!imageDialog) {
            imageDialog = imageDialogComponent.createObject(area, { objectName: "imageDialog" })
        }
        imageDialog.open()
    }

    // A file dragged in from the file manager lands where it is dropped, as the image tool
    // places one (LH-18): the first one in a format the background takes. A locked layer refuses
    // it in the controller, with the usual line in the status.
    readonly property var imageExtensions: ["png", "jpg", "jpeg"]
    function imageIn(urls) {
        const isImage = url => area.imageExtensions.indexOf(String(url).split(".").pop().toLowerCase()) >= 0
        return urls.map(String).find(isImage)
    }
    // Places the image of `urls` at (`x`, `y`), in the canvas's pixels; false when there is none.
    function dropImage(urls, x, y) {
        const file = area.imageIn(urls)
        if (file === undefined) {
            return false
        }
        const at = canvas.mapToModel(x, y)
        area.controller.addImage(file, at.x, at.y)
        return true
    }
    DropArea {
        id: imageDrop
        objectName: "imageDrop"
        anchors.fill: canvas
        enabled: canvas.visible
        onEntered: event => { event.accepted = event.hasUrls && area.imageIn(event.urls) !== undefined }
        onDropped: event => {
            if (event.hasUrls && area.dropImage(event.urls, event.x, event.y)) {
                event.accept()
            }
        }
    }

    LayerPopover {
        objectName: "layerPopover"
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.margins: Kirigami.Units.largeSpacing
        texts: area.texts
        controller: area.controller
    }

    // Shift at the press extends the selection (a click, a box or a lasso adds to it, a click
    // on a member takes it out); Ctrl makes the drag on empty ground a lasso. The canvas reports
    // no modifiers; this handler lies above it, only looks, and leaves the press to it.
    Item {
        anchors.fill: parent
        // Where the mouse pointer is: the card of the selection keeps clear of it.
        HoverHandler { id: pointerSpy }
        PointHandler {
            acceptedButtons: Qt.LeftButton
            onActiveChanged: {
                if (active) {
                    area.controller.extendSelection = (point.modifiers & Qt.ShiftModifier) !== 0
                    area.controller.lassoSelection = (point.modifiers & Qt.ControlModifier) !== 0
                }
            }
        }
        // Right click: the context menu (LH-100).
        TapHandler {
            acceptedButtons: Qt.RightButton
            onTapped: eventPoint => area.openMenuOn(eventPoint.position)
        }
        // Measure tool, first point set: the pointer is the other end, the controller draws.
        HoverHandler {
            enabled: area.tool === "measure" && area.controller.measuring
            onPointChanged: {
                const m = canvas.mapToModel(point.position.x, point.position.y)
                area.controller.hover(m.x, m.y)
            }
        }
    }

    // The context menu (LH-100). A click selects what is under the pointer first, as a left
    // click does; the Menu key and Shift+F10 take the selection as it is and open at the pointer,
    // or the middle of the view.
    PlanContextMenu {
        id: contextMenu
        objectName: "contextMenu"
        texts: area.texts
        controller: area.controller
        turnStepDeg: area.turnStepDeg
        onFitRequested: area.zoomBy("fit")
    }
    function openMenu(at) {
        if (!area.free && area.controller.loaded) {
            contextMenu.popup(area, at.x, at.y)
        }
    }
    function openMenuOn(at) {
        if (area.free || !area.controller.loaded) {
            return
        }
        const m = canvas.mapToModel(at.x, at.y)
        area.controller.selectAt(m.x, m.y)
        area.openMenu(at)
    }
    Shortcut {
        sequences: ["Menu", "Shift+F10"]
        enabled: !area.free && area.controller.loaded && canvas.activeFocus
        onActivated: area.openMenu(pointerSpy.hovered ? pointerSpy.point.position : Qt.point(area.width / 2, area.height / 2))
    }

    // The hint line (LH-101, LH-122): what the next click of the tool does.
    KanteCard {
        id: hintLine
        objectName: "hintLine"
        visible: area.toolHint !== "" && area.tool !== "select" && !area.free
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.margins: Kirigami.Units.largeSpacing
        implicitWidth: Math.min(hintLabel.implicitWidth, area.width / 2) + Kirigami.Units.largeSpacing * 2
        implicitHeight: hintLabel.implicitHeight + Kirigami.Units.largeSpacing * 2

        QQC2.Label {
            id: hintLabel
            objectName: "hintLabel"
            anchors.fill: parent
            anchors.margins: Kirigami.Units.largeSpacing
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: area.toolHint
        }
    }

    // Scale tool, two points picked: the length between them. Enter sets the scale, Escape
    // drops the points (it drops them while the field has the focus, too).
    readonly property bool asksLength: area.tool === "scale" && controller.scalePoints === 2
    KanteCard {
        id: lengthCard
        objectName: "lengthCard"
        visible: area.asksLength
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Kirigami.Units.largeSpacing
        implicitWidth: lengthRow.implicitWidth + Kirigami.Units.largeSpacing * 2
        implicitHeight: lengthRow.implicitHeight + Kirigami.Units.largeSpacing * 2

        RowLayout {
            id: lengthRow
            anchors.centerIn: parent
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: area.texts.t("scale.distance") }
            KanteTextField {
                id: lengthField
                objectName: "lengthField"
                Accessible.name: area.texts.t("scale.distance")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: area.setScale()
                Keys.onEscapePressed: area.controller.cancelScale()
            }
            KanteButton {
                text: area.texts.t("scale.apply")
                onClicked: area.setScale()
            }
        }
    }

    // Note tool, or F2 on a selected note: the text of the note. Enter sets it, Escape drops
    // the question; a blank text is refused.
    KanteCard {
        id: noteCard
        objectName: "noteCard"
        visible: area.controller.asksNote
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Kirigami.Units.largeSpacing
        implicitWidth: noteRow.implicitWidth + Kirigami.Units.largeSpacing * 2
        implicitHeight: noteRow.implicitHeight + Kirigami.Units.largeSpacing * 2

        RowLayout {
            id: noteRow
            anchors.centerIn: parent
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: area.texts.t("note.text") }
            KanteTextField {
                id: noteField
                objectName: "noteField"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 18
                Accessible.name: area.texts.t("note.text")
                onTextChanged: invalid = false
                onAccepted: area.setNote()
                Keys.onEscapePressed: area.controller.cancelNote()
            }
            KanteButton {
                objectName: "noteApply"
                text: area.texts.t("note.apply")
                onClicked: area.setNote()
            }
        }
    }

    // A drawing is selected (LH-19): colour, line and fill.
    DrawingCard {
        objectName: "drawingCard"
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        texts: area.texts
        controller: area.controller
        width: Math.min(area.width - Kirigami.Units.largeSpacing * 2, implicitWidth)
    }

    // An image is selected (LH-18): its opacity, and a new file for it.
    ImageCard {
        id: imageCard
        objectName: "imageCard"
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        texts: area.texts
        controller: area.controller
        width: Math.min(area.width - Kirigami.Units.largeSpacing * 2, implicitWidth)
    }

    // A lamp is selected (LH-50, LH-51): kind, library lamp, direction, beam, power and colour.
    LightCard {
        objectName: "lightCard"
        texts: area.texts
        controller: area.controller
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        width: Math.min(area.width - Kirigami.Units.largeSpacing * 2, implicitWidth)
    }

    // The card of the selection, beside the element (D-04): an object's size, a camera's lens,
    // a mark's pose and pause; "More" opens the rest.
    SelectionCard {
        id: selection
        objectName: "selectionCard"
        texts: area.texts
        controller: area.controller
        canvas: canvas
        free: freeView.visible
        freeAt: freeView.clickAt
        pointer: pointerSpy.point.position
        pointerOver: pointerSpy.hovered
    }

    // The card goes with the tool (Q1-04).
    onToolChanged: selection.dismissed = true

    // In 3D only a click with the select tool opens it again (as a click on the plan).
    Connections {
        target: freeView
        function onClickAtChanged() {
            if (area.tool === "select") {
                selection.dismissed = false
            }
        }
    }

    Connections {
        target: area.controller
        function onAsksNoteChanged() {
            if (!area.controller.asksNote) {
                return
            }
            noteField.text = area.controller.noteText
            noteField.forceActiveFocus()
            noteField.selectAll()
        }
    }

    /// The selected object's card takes the height (LH-93). In 3D the view turned to the object:
    /// the card opens beside the middle.
    function focusHeight() {
        freeView.clickAt = Qt.point(freeView.width / 2, freeView.height / 2)
        selection.focusHeight()
    }

    function setNote() {
        if (!controller.commitNote(noteField.text)) {
            noteField.invalid = true
        }
    }

    // Maßfeld (LH-95): Enter on the plan or in 3D opens an empty field: the length of the wall
    // being drawn, the width and depth of the room being drawn (`5;4`), else the height of the
    // selected object or wall. Enter applies, Escape closes. With nothing to apply it to, it
    // does not open.
    readonly property bool wallDrag: area.tool === "wall" && (area.dragging || freeView.dragging)
    readonly property bool roomDrag: area.tool === "room" && (area.dragging || freeView.dragging)
    readonly property string measureKind: {
        if (wallDrag) {
            return "length"
        }
        if (roomDrag) {
            return "room"
        }
        return /^(object|wall):/.test(controller.selectedKey) ? "height" : ""
    }
    // What the open field applies, fixed when it opens: empty while closed.
    property string asksMeasure: ""

    KanteCard {
        id: measureCard
        objectName: "measureCard"
        visible: area.asksMeasure !== ""
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Kirigami.Units.largeSpacing
        implicitWidth: measureRow.implicitWidth + Kirigami.Units.largeSpacing * 2
        implicitHeight: measureRow.implicitHeight + Kirigami.Units.largeSpacing * 2

        RowLayout {
            id: measureRow
            anchors.centerIn: parent
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: area.asksMeasure === "" ? "" : area.texts.t("measure." + area.asksMeasure) }
            KanteTextField {
                id: measureField
                objectName: "measureField"
                Accessible.name: area.texts.t("measure." + (area.asksMeasure || "height"))
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: area.applyMeasure()
                onActiveFocusChanged: {
                    if (!activeFocus) {
                        area.closeMeasure()
                    }
                }
                Keys.onEscapePressed: area.closeMeasure()
            }
        }
    }

    // Enter, not a digit, opens it: 1 to 4 pick the mode and 0 fits the plan.
    Shortcut {
        sequences: ["Return", "Enter"]
        enabled: area.measureKind !== "" && area.asksMeasure === "" && !area.drawsArea && (canvas.activeFocus || freeView.activeFocus)
        onActivated: area.openMeasure()
    }

    onWallDragChanged: {
        if (!wallDrag && asksMeasure === "length") {
            closeMeasure()
        }
    }

    onRoomDragChanged: {
        if (!roomDrag && asksMeasure === "room") {
            closeMeasure()
        }
    }

    function openMeasure() {
        asksMeasure = measureKind
        measureField.text = ""
        measureField.forceActiveFocus()
    }

    function closeMeasure() {
        if (asksMeasure === "") {
            return
        }
        const had = measureField.activeFocus
        asksMeasure = ""
        measureField.text = ""
        if (had) {
            (area.free ? freeView : canvas).forceActiveFocus()
        }
    }

    function applyMeasure() {
        if (!controller.typeMeasure(measureField.text)) {
            measureField.invalid = true
            return
        }
        closeMeasure()
    }

    onAsksLengthChanged: {
        lengthField.text = ""
        if (asksLength) {
            lengthField.forceActiveFocus()
        }
    }

    function setScale() {
        if (!controller.calibrate(lengthField.text)) {
            lengthField.invalid = true
        }
    }

    Shortcut {
        sequence: "Escape"
        enabled: area.tool === "scale" && area.controller.scalePoints > 0 && !area.dragging
        onActivated: area.controller.cancelScale()
    }

    // Measure tool: Escape drops the measurement.
    Shortcut {
        sequence: "Escape"
        enabled: area.tool === "measure" && area.controller.measuring && !area.dragging
        onActivated: area.controller.cancelMeasure()
    }

    // Area and free form tools: Enter closes the outline, Escape drops it (once its points are
    // there).
    readonly property bool drawsArea: (area.tool === "area" || area.tool === "freeform")
        && controller.areaPoints > 0 && !area.dragging
    Shortcut {
        sequence: "Escape"
        enabled: area.drawsArea
        onActivated: area.controller.cancelArea()
    }
    Shortcut {
        sequences: ["Return", "Enter"]
        enabled: area.drawsArea
        onActivated: area.controller.finishArea()
    }

    // Escape drops the drag in progress; only then, so it stays free for dialogs.
    Shortcut {
        sequence: "Escape"
        enabled: area.dragging
        onActivated: {
            area.dragAborted = true
            area.dragging = false
            area.controller.cancelDrag()
        }
    }

    // Escape clears the selection, which closes its card (Q1-04). Not while a drag, a measure,
    // a scale or an outline has its own Escape, and not in a text field anywhere in the window
    // (a cell of the shot table; the card's fields leave the field first).
    readonly property Item focused: Window.window ? Window.window.activeFocusItem : null
    readonly property bool textFocused: !!area.focused && area.focused.cursorPosition !== undefined
    // Something has an Escape of its own: a drag, an outline, the points of a scale or a measure.
    readonly property bool escBusy: area.dragging || freeView.dragging || area.drawsArea
        || (area.tool === "scale" && area.controller.scalePoints > 0)
        || (area.tool === "measure" && area.controller.measuring)
    // LH-101: Escape ends the operation, then the tool goes back to select, then the selection goes.
    Shortcut {
        sequence: "Escape"
        enabled: area.hasSelection && !area.textFocused && !area.escBusy && area.tool === "select"
        onActivated: area.controller.clearSelection()
    }
    Shortcut {
        sequence: "Escape"
        enabled: area.controller.loaded && area.tool !== "select" && !area.textFocused && !area.escBusy
        onActivated: area.selectToolRequested()
    }

    // Keys for the selection: Delete removes it, R and Shift+R turn it by 15 degrees (several
    // turn about their middle), Ctrl+D copies it, G makes its way straight.
    readonly property real turnStepDeg: 15
    // Not while a length is typed: Delete there edits the text.
    readonly property bool hasSelection: (!!controller.selectedKey || controller.selectedCount > 0) && !lengthField.activeFocus && !noteField.activeFocus && !measureField.activeFocus
        && !selection.typing && !imageCard.typing
    Shortcut {
        sequence: "Delete"
        enabled: area.hasSelection
        onActivated: area.controller.removeSelected()
    }
    Shortcut {
        sequence: "Ctrl+D"
        enabled: area.hasSelection
        onActivated: area.controller.duplicateSelected()
    }
    Shortcut {
        sequence: "F2"
        enabled: area.hasSelection && area.controller.selectedKey.indexOf("note:") === 0
        onActivated: area.controller.editSelectedNote()
    }
    Shortcut {
        sequence: "G"
        enabled: area.hasSelection
        onActivated: area.controller.straightenSelected()
    }
    Shortcut {
        sequence: "R"
        enabled: area.hasSelection
        onActivated: area.controller.turnSelected(area.turnStepDeg)
    }
    Shortcut {
        sequence: "Shift+R"
        enabled: area.hasSelection
        onActivated: area.controller.turnSelected(-area.turnStepDeg)
    }

    /// A key of a bend of a way (`figurevia:e:1:at0`), not of the grip that makes a new one
    /// (`...:new0`).
    function isBend(key) {
        return /^(figure|camera)via:.*:at\d+$/.test(key)
    }

    /// The text key of the tooltip of a grip (LH-102), `""` for a key that is none.
    function gripTip(key) {
        const parts = key.split(":")
        const last = parts[parts.length - 1]
        switch (parts[0]) {
        case "objectgrip":
            return { width: "tip.gripWidth", depth: "tip.gripDepth" }[last] || (last.indexOf("point") === 0 ? "tip.gripPoint" : "tip.gripSide")
        case "lightgrip": return last === "beam" ? "tip.gripBeam" : "tip.gripDirection"
        case "openingedge": return "tip.gripOpening"
        case "notewidth": return "tip.gripNote"
        case "imagesize": return "tip.gripImage"
        case "drawingpoint": return "tip.gripDrawing"
        case "drawingedge": return "tip.gripEdge"
        }
        return ""
    }

    /// The lines of the tooltip of a shape: a grip's one line, else what the element is.
    function tipOf(key) {
        const grip = area.gripTip(key)
        return grip !== "" ? [area.texts.t(grip)] : area.tipLines(key)
    }

    // Tooltip text from the controller's data and the catalogue.
    function tipLines(key) {
        if (key === "") {
            return []
        }
        const d = JSON.parse(controller.describe(key))
        const t = area.texts
        // A note and an image have one line of text each.
        if ({ note: 1, image: 1 }[d.kind]) {
            return [t.t("tip." + d.kind, { text: d.text, file: d.file })]
        }
        switch (d.kind) {
        case "camera":
            return [t.t("tip.camera", { label: d.label }), t.t("tip.lens", { lens: t.num(d.lens, 1), fov: t.num(d.fov, 1) }),
                    t.t("tip.height", { height: t.num(d.height, 2) }), d.shots ? t.t("tip.shots", { shots: d.shots }) : ""]
                .concat(d.rig ? [t.t("tip.rig", { rig: t.t("rig." + d.rig) })] : [])
                .concat(d.follows ? [t.t("tip.follows", { name: d.follows })] : [])
        case "figure":
            return [t.t("tip.figure", { name: d.name })]
        case "mark":
            return [t.t("tip.mark", { name: d.name, index: d.index }), t.t("tip.at", { time: t.num(d.time, 1) })]
        case "bend":
            return [t.t("tip.bend", { index: d.index })]
        case "light":
            // Bounce and flag make no light of their own: no power, no colour.
            return d.gives_light
                ? [t.t("tip.light", { label: d.label, type: t.t("lightKind." + d.type), temp: t.num(d.temp, 0), power: t.num(d.power, 0), beam: t.num(d.beam, 0) })]
                : [t.t("tip.lightPassive", { label: d.label, type: t.t("lightKind." + d.type), beam: t.num(d.beam, 0) })]
        case "object":
            return [t.t("tip.object", { label: d.label }),
                    d.height !== null ? t.t("tip.objectHeight", { height: t.num(d.height, 2) }) : ""]
        case "opening":
            return [t.t("tip.opening", { type: t.t("openingKind." + d.type), width: t.num(d.width, 2) })]
        }
        return []
    }

    QQC2.ToolTip {
        readonly property var lines: area.tipOf(area.tipKey).filter(l => l !== "")
        parent: area
        x: area.tipAt.x + Kirigami.Units.largeSpacing
        y: area.tipAt.y + Kirigami.Units.largeSpacing
        visible: lines.length > 0
        text: lines.join("\n")
    }
}
