import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
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
    /// The tool of PlanTools: select, wall, object, figure, camera, light, scale, measure, note, line,
    /// arrow, rect, ellipse or area.
    property string tool: "select"
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
        visible: area.plan !== null
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
        onShapeClicked: (key, x, y) => area.controller.place(x, y)
        // The click before it selected the note; the double click edits its text.
        onShapeDoubleClicked: (key, x, y) => {
            if (area.tool === "select" && key.indexOf("note:") === 0) {
                area.controller.editSelectedNote()
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

    /// Zooms about the middle of the view: "in", "out" or "fit" (the whole plan).
    readonly property real zoomStep: 1.25
    function zoomBy(how) {
        switch (how) {
        case "in": canvas.zoom *= zoomStep; break
        case "out": canvas.zoom /= zoomStep; break
        case "fit": canvas.fit(); break
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
        // Measure tool, first point set: the pointer is the other end, the controller draws.
        HoverHandler {
            enabled: area.tool === "measure" && area.controller.measuring
            onPointChanged: {
                const m = canvas.mapToModel(point.position.x, point.position.y)
                area.controller.hover(m.x, m.y)
            }
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
        pointer: pointerSpy.point.position
        pointerOver: pointerSpy.hovered
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

    function setNote() {
        if (!controller.commitNote(noteField.text)) {
            noteField.invalid = true
        }
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

    // Area tool: Enter closes the area, Escape drops it (once its points are there).
    readonly property bool drawsArea: area.tool === "area" && controller.areaPoints > 0 && !area.dragging
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

    // Keys for the selection: Delete removes it, R and Shift+R turn it by 15 degrees (several
    // turn about their middle), Ctrl+D copies it, G makes its way straight.
    readonly property real turnStepDeg: 15
    // Not while a length is typed: Delete there edits the text.
    readonly property bool hasSelection: (!!controller.selectedKey || controller.selectedCount > 0) && !lengthField.activeFocus && !noteField.activeFocus
        && !selection.typing
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

    // Tooltip text from the controller's data and the catalogue.
    function tipLines(key) {
        if (key === "") {
            return []
        }
        const d = JSON.parse(controller.describe(key))
        const t = area.texts
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
            return d.power > 0
                ? [t.t("tip.light", { label: d.label, type: t.t("lightKind." + d.type), temp: t.num(d.temp, 0), power: t.num(d.power, 0), beam: t.num(d.beam, 0) })]
                : [t.t("tip.lightPassive", { label: d.label, type: t.t("lightKind." + d.type), beam: t.num(d.beam, 0) })]
        case "object":
            return [t.t("tip.object", { label: d.label }),
                    d.height !== null ? t.t("tip.objectHeight", { height: t.num(d.height, 2) }) : ""]
        case "note":
            return [t.t("tip.note", { text: d.text })]
        }
        return []
    }

    QQC2.ToolTip {
        readonly property var lines: area.tipLines(area.tipKey).filter(l => l !== "")
        parent: area
        x: area.tipAt.x + Kirigami.Units.largeSpacing
        y: area.tipAt.y + Kirigami.Units.largeSpacing
        visible: lines.length > 0
        text: lines.join("\n")
    }
}
