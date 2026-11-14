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
    /// The tool of PlanTools: select, wall, object, figure, camera or light.
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
    readonly property bool hasSelection: !!controller.selectedKey
    Shortcut {
        sequence: "Delete"
        enabled: area.hasSelection
        onActivated: area.controller.removeSelected()
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
