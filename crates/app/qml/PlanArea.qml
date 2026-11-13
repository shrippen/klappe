import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The floor plan: Stellprobe's display list drawn by KanteShapeCanvas (no geometry here,
// agent.md), with a tooltip for what the pointer is over (round 2, design 4).
Item {
    id: area

    required property var texts
    required property var controller

    readonly property var plan: controller.planJson ? JSON.parse(controller.planJson) : null
    property string tipKey: ""
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

        onShapeHovered: (key, x, y) => {
            area.tipKey = key
            area.tipAt = canvas.mapFromModel(x, y)
        }
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
            return [t.t("tip.camera", { label: d.label }), t.t("tip.lens", { lens: d.lens, fov: d.fov }),
                    t.t("tip.height", { height: d.height }), d.shots ? t.t("tip.shots", { shots: d.shots }) : ""]
        case "figure":
            return [t.t("tip.figure", { name: d.name })]
        case "mark":
            return [t.t("tip.mark", { name: d.name, index: d.index }), t.t("tip.at", { time: d.time })]
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
