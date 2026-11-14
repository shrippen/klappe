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
    /// The tool of PlanTools: select, wall, object, figure, camera, light, scale, note, line,
    /// arrow, rect, ellipse or area.
    property string tool: "select"
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
        anchors.fill: parent
        visible: area.plan !== null
        contentWidth: area.plan ? area.plan.width : 1
        contentHeight: area.plan ? area.plan.depth : 1
        gridStep: 1
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

    // Shift at the press extends the selection (a click or a box adds to it, a click on a
    // member takes it out). The canvas reports no modifiers; this handler lies above it, only
    // looks, and leaves the press to it.
    Item {
        anchors.fill: parent
        PointHandler {
            acceptedButtons: Qt.LeftButton
            onActiveChanged: {
                if (active) {
                    area.controller.extendSelection = (point.modifiers & Qt.ShiftModifier) !== 0
                }
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

    // An object is selected (LH-25, LH-12): its size as two fields, and filing it in the
    // library under a name. Enter in a size field sets the size; a size that is no length
    // marks the field and changes nothing.
    readonly property var objectInfo: {
        // The plan changes with every edit; the key alone would not tell the new size.
        area.controller.planJson
        const key = area.controller.selectedKey
        if (!key || key.indexOf("object:") !== 0) {
            return null
        }
        const d = JSON.parse(area.controller.describe(key))
        return d.kind === "object" ? d : null
    }
    onObjectInfoChanged: area.fillObjectCard()

    // The name a new library entry starts with: the object's own label, else its kind's name.
    function fillObjectCard() {
        const info = area.objectInfo
        if (!info) {
            return
        }
        if (!widthField.activeFocus) {
            widthField.text = area.texts.num(info.width, 2)
        }
        if (!depthField.activeFocus) {
            depthField.text = area.texts.num(info.depth, 2)
        }
        const kindName = area.texts.t("objectKind." + info.library)
        if (!libraryName.activeFocus) {
            libraryName.text = info.label !== "" ? info.label : (kindName !== "objectKind." + info.library ? kindName : "")
        }
    }

    function setObjectSize() {
        const ok = controller.resizeSelected(widthField.text, depthField.text)
        widthField.invalid = !ok
        depthField.invalid = !ok
    }

    function fileObject() {
        if (!controller.saveToLibrary(libraryName.text)) {
            libraryName.invalid = true
        }
    }

    KanteCard {
        id: objectCard
        objectName: "objectCard"
        visible: area.objectInfo !== null
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        implicitWidth: Math.min(area.width - Kirigami.Units.largeSpacing * 2,
                                objectRow.implicitWidth + Kirigami.Units.largeSpacing * 2)
        implicitHeight: objectRow.implicitHeight + Kirigami.Units.largeSpacing * 2

        RowLayout {
            id: objectRow
            anchors.centerIn: parent
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: area.texts.t("object.size") }
            KanteTextField {
                id: widthField
                objectName: "widthField"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 5
                Accessible.name: area.texts.t("field.width")
                placeholderText: area.texts.t("field.width")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: area.setObjectSize()
            }
            KanteTextField {
                id: depthField
                objectName: "depthField"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 5
                Accessible.name: area.texts.t("field.depth")
                placeholderText: area.texts.t("field.depth")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: area.setObjectSize()
            }
            KanteButton {
                objectName: "objectApply"
                text: area.texts.t("form.apply")
                onClicked: area.setObjectSize()
            }

            Kirigami.Separator { Layout.fillHeight: true }

            KanteTextField {
                id: libraryName
                objectName: "libraryName"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 10
                Accessible.name: area.texts.t("library.name")
                placeholderText: area.texts.t("library.name")
                onTextChanged: invalid = false
                onAccepted: area.fileObject()
            }
            KanteButton {
                objectName: "libraryFile"
                text: area.texts.t("library.save")
                onClicked: area.fileObject()
                QQC2.ToolTip.text: area.texts.t("library.saveTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
            }
        }
    }

    // A camera is selected (LH-32): body, sensor, lens and frame.
    CameraCard {
        id: cameraCard
        objectName: "cameraCard"
        texts: area.texts
        controller: area.controller
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        width: Math.min(area.width - Kirigami.Units.largeSpacing * 2, implicitWidth)
    }

    // A mark is selected (LH-22, LH-63): pose, pause and pace.
    MarkCard {
        objectName: "markCard"
        anchors.horizontalCenter: parent.horizontalCenter
        // Below the camera card: a camera shows both.
        anchors.top: cameraCard.visible ? cameraCard.bottom : parent.top
        anchors.topMargin: Kirigami.Units.largeSpacing
        texts: area.texts
        controller: area.controller
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
        && !widthField.activeFocus && !depthField.activeFocus && !libraryName.activeFocus
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
                .concat(d.follows ? [t.t("tip.follows", { name: d.follows })] : [])
        case "figure":
            return [t.t("tip.figure", { name: d.name })]
        case "mark":
            return [t.t("tip.mark", { name: d.name, index: d.index }), t.t("tip.at", { time: t.num(d.time, 1) })]
        case "bend":
            return [t.t("tip.bend", { index: d.index })]
        case "light":
            return [t.t("tip.light", { label: d.label, temp: d.temp, power: d.power })]
        case "object":
            return [t.t("tip.object", { label: d.label })]
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
