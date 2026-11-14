import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQml
import org.kde.kirigami as Kirigami
import Kante

// The drawing tools of the tool bar (LH-08 ff.): one tool at a time, a list for the tools that
// need a choice (object kind, figure, lamp kind), the snapping switch. Keys V W O F C L M pick
// the tool (NF-06). The controller does the work: this only says which tool and which choice.
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

    readonly property var _names: ["select", "wall", "object", "figure", "camera", "light", "scale"]
    readonly property var _keys: ["V", "W", "O", "F", "C", "L", "M"]
    readonly property var _choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : { figure: [], object: [], light: [] }

    spacing: Kirigami.Units.smallSpacing

    // The controller keeps the tool: its drags and clicks do not repeat it.
    onToolChanged: _tell()
    onChoiceChanged: _tell()
    Component.onCompleted: _tell()
    function _tell() {
        controller.setTool(tool, choice)
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
            QQC2.ToolTip.text: modelData === "scale"
                ? tools.texts.t("tool.scaleTip", { key: tools._keys[index] })
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
            enabled: tools.controller.loaded
            onActivated: tools.tool = modelData
        }
    }
}
