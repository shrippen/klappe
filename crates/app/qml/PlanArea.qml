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
    /// The pointer is over something the select tool can move: a left drag then moves it,
    /// else it pans (middle button and space always pan).
    property bool overGrab: false
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
        editing: area.tool !== "select" || area.overGrab

        onShapeHovered: (key, x, y) => {
            area.tipKey = key
            area.tipAt = canvas.mapFromModel(x, y)
            area.overGrab = key !== "" && area.controller.grabbable(x, y)
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

    // Keys for the selection: Delete removes it, R and Shift+R turn it by 15 degrees.
    readonly property real turnStepDeg: 15
    // Not while a length is typed: Delete there edits the text.
    readonly property bool hasSelection: !!controller.selectedKey && !lengthField.activeFocus && !noteField.activeFocus
    Shortcut {
        sequence: "Delete"
        enabled: area.hasSelection
        onActivated: area.controller.removeSelected()
    }
    Shortcut {
        sequence: "F2"
        enabled: area.hasSelection && area.controller.selectedKey.indexOf("note:") === 0
        onActivated: area.controller.editSelectedNote()
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
        case "figure":
            return [t.t("tip.figure", { name: d.name })]
        case "mark":
            return [t.t("tip.mark", { name: d.name, index: d.index }), t.t("tip.at", { time: t.num(d.time, 1) })]
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
