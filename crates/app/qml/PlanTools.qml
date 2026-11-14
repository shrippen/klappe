import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQml
import org.kde.kirigami as Kirigami
import Kante

// The drawing tools of the tool bar (LH-08 ff.): one tool at a time, a list for the tools that
// need a choice (object kind, figure, lamp kind), the snapping switch. Keys V W O F C L M N I A B
// E P pick the tool (NF-06). The drawing tools (line, arrow, rect, ellipse, area; LH-19) also
// show colour, line style and, for closed shapes, fill. The controller does the work: this only
// says which tool and which choice.
RowLayout {
    id: tools

    required property var texts
    required property var controller

    property string tool: "select"
    /// The value picked in the current tool's list: object kind, character id or lamp kind.
    readonly property string choice: {
        const box = tool === "figure" ? figureBox : tool === "object" ? objectBox : tool === "light" ? lightBox : null
        return box && box.currentValue !== undefined ? String(box.currentValue) : ""
    }

    /// What the drawing tools draw with: names as the controller lists them in `choicesJson`.
    property string drawColor: "red"
    property string drawDash: "solid"
    property string drawFill: "outline"
    readonly property bool drawing: ["line", "arrow", "rect", "ellipse", "area"].indexOf(tool) >= 0
    /// Rectangle, ellipse and area have an inside to fill.
    readonly property bool closedShape: ["rect", "ellipse", "area"].indexOf(tool) >= 0

    readonly property var _names: ["select", "wall", "object", "figure", "camera", "light", "scale", "note",
                                   "line", "arrow", "rect", "ellipse", "area"]
    readonly property var _keys: ["V", "W", "O", "F", "C", "L", "M", "N", "I", "A", "B", "E", "P"]
    readonly property var _choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : { figure: [], object: [], light: [], color: [], dash: [], fill: [] }

    spacing: Kirigami.Units.smallSpacing

    // The controller keeps the tool: its drags and clicks do not repeat it.
    onToolChanged: _tell()
    onChoiceChanged: _tell()
    onDrawColorChanged: _tellStyle()
    onDrawDashChanged: _tellStyle()
    onDrawFillChanged: _tellStyle()
    Component.onCompleted: {
        _tell()
        _tellStyle()
    }
    function _tell() {
        controller.setTool(tool, choice)
    }
    function _tellStyle() {
        controller.setDrawStyle(drawColor, drawDash, drawFill)
    }

    QQC2.ButtonGroup { id: group }

    Repeater {
        model: tools._names
        delegate: KanteToolButton {
            required property int index
            required property string modelData
            text: tools.texts.t("tool." + modelData)
            checkable: true
            checked: tools.tool === modelData
            QQC2.ButtonGroup.group: group
            onClicked: tools.tool = modelData
            QQC2.ToolTip.text: modelData === "scale" || modelData === "note" || modelData === "area"
                ? tools.texts.t("tool." + modelData + "Tip", { key: tools._keys[index] })
                : tools.texts.t("tool.tip", { name: text, key: tools._keys[index] })
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }

    KanteComboBox {
        id: objectBox
        objectName: "objectBox"
        visible: tools.tool === "object"
        Accessible.name: tools.texts.t("tool.pick.object")
        textRole: "text"
        valueRole: "value"
        model: tools._choices.object.map(k => ({ value: k, text: tools.texts.t("objectKind." + k) }))
    }
    KanteComboBox {
        id: figureBox
        objectName: "figureBox"
        visible: tools.tool === "figure"
        Accessible.name: tools.texts.t("tool.pick.figure")
        textRole: "text"
        valueRole: "value"
        displayText: count > 0 ? currentText : tools.texts.t("tool.noFigures")
        model: tools._choices.figure.map(f => ({ value: f.id, text: f.name }))
    }
    KanteComboBox {
        id: lightBox
        objectName: "lightBox"
        visible: tools.tool === "light"
        Accessible.name: tools.texts.t("tool.pick.light")
        textRole: "text"
        valueRole: "value"
        model: tools._choices.light.map(k => ({ value: k, text: tools.texts.t("lightKind." + k) }))
    }

    KanteSwatchPicker {
        objectName: "colorPicker"
        visible: tools.drawing
        Accessible.name: tools.texts.t("draw.color")
        colors: (tools._choices.color || []).map(n => KanteStyle.labelColor(n))
        names: (tools._choices.color || []).map(n => tools.texts.t("color." + n))
        currentIndex: (tools._choices.color || []).indexOf(tools.drawColor)
        onPicked: index => tools.drawColor = tools._choices.color[index]
    }
    KanteComboBox {
        id: dashBox
        objectName: "dashBox"
        visible: tools.drawing
        Accessible.name: tools.texts.t("draw.dash")
        textRole: "text"
        valueRole: "value"
        model: (tools._choices.dash || []).map(k => ({ value: k, text: tools.texts.t("dash." + k) }))
        currentIndex: Math.max(0, (tools._choices.dash || []).indexOf(tools.drawDash))
        onActivated: tools.drawDash = String(currentValue)
    }
    KanteComboBox {
        id: fillBox
        objectName: "fillBox"
        visible: tools.closedShape
        Accessible.name: tools.texts.t("draw.fill")
        textRole: "text"
        valueRole: "value"
        model: (tools._choices.fill || []).map(k => ({ value: k, text: tools.texts.t("fill." + k) }))
        currentIndex: Math.max(0, (tools._choices.fill || []).indexOf(tools.drawFill))
        onActivated: tools.drawFill = String(currentValue)
    }

    KanteToolButton {
        objectName: "snapButton"
        text: tools.texts.t("tool.snapping")
        checkable: true
        checked: tools.controller.snappingOn
        onToggled: tools.controller.setSnapping(checked)
        QQC2.ToolTip.text: tools.texts.t("tool.snappingTip")
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }

    Instantiator {
        model: tools._names
        delegate: Shortcut {
            required property int index
            required property string modelData
            sequence: tools._keys[index]
            // Like the buttons: no tool change while the tool bar is locked.
            enabled: tools.controller.loaded && tools.enabled
            onActivated: tools.tool = modelData
        }
    }
}
