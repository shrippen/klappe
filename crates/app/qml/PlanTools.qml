import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQml
import QtCore
import QtQuick.Controls.impl
import org.kde.kirigami as Kirigami
import Kante

// The drawing tools of the tool bar (LH-08 ff.): one tool at a time, a list for the tools that
// need a choice (object from the library, figure, lamp kind), the snapping switch. The object
// tool also offers to import and export library files (LH-12): the window asks for the file. Keys V W O F C L M D N I A B
// E P pick the tool (NF-06) in every mode; a key of another mode's tool changes the mode. The
// drawing tools (line, arrow, rect, ellipse, area; LH-19) also
// show colour, line style and, for closed shapes, fill. The controller does the work: this only
// says which tool and which choice.
//
// D-01: four working modes (keys 1-4) decide which tools the bar shows. The mode is a view only
// (what can be picked and edited does not depend on it), so it lives here, in `_modes`, the one
// table that says which tool belongs to which mode; the settings keep it over a restart. Tools are
// icon buttons; one without an icon in the theme shows its name instead.
RowLayout {
    id: tools

    required property var texts
    required property var controller

    property string tool: "select"
    /// The working mode: the id of an entry of `_modes`.
    property string mode: "build"
    /// The value picked in the current tool's list: library kind, character id or lamp kind.
    readonly property string choice: {
        if (tool === "object") {
            const entry = objectBox.model[objectBox.currentIndex]
            return entry ? entry.value : ""
        }
        const box = tool === "figure" ? figureBox : tool === "light" ? lightBox : null
        return box && box.currentValue !== undefined ? String(box.currentValue) : ""
    }

    /// The library as the list shows it: built in first, then the project's own, in groups.
    readonly property var _objects: (_choices.object || []).map(o => ({
        value: o.kind,
        text: o.custom ? (o.label || o.kind) : texts.t("objectKind." + o.kind),
        group: texts.t(o.custom ? "library.custom" : "library.builtin")
    }))
    readonly property bool hasOwnObjects: (_choices.object || []).some(o => o.custom)

    signal importRequested()
    signal exportRequested()

    /// What the drawing tools draw with: names as the controller lists them in `choicesJson`.
    property string drawColor: "red"
    property string drawDash: "solid"
    property string drawFill: "outline"
    readonly property bool drawing: ["line", "arrow", "rect", "ellipse", "area"].indexOf(tool) >= 0
    /// Rectangle, ellipse and area have an inside to fill.
    readonly property bool closedShape: ["rect", "ellipse", "area"].indexOf(tool) >= 0

    // Tool, key, theme icon. The order is the palette's.
    readonly property var _tools: [
        { name: "select", key: "V", icon: "edit-select" },
        { name: "wall", key: "W", icon: "view-grid" },
        { name: "object", key: "O", icon: "draw-cuboid" },
        { name: "figure", key: "F", icon: "user" },
        { name: "camera", key: "C", icon: "camera-video" },
        { name: "light", key: "L", icon: "flashlight-on" },
        { name: "scale", key: "M", icon: "transform-scale" },
        { name: "measure", key: "D", icon: "measure" },
        { name: "note", key: "N", icon: "view-pim-notes" },
        { name: "line", key: "I", icon: "draw-line" },
        { name: "arrow", key: "A", icon: "draw-arrow" },
        { name: "rect", key: "B", icon: "draw-rectangle" },
        { name: "ellipse", key: "E", icon: "draw-ellipse" },
        { name: "area", key: "P", icon: "draw-polygon" }
    ]
    /// The modes (D-01) with the tools of each in groups, separated in the bar. A tool in two modes
    /// (selecting, measuring) belongs to the first; the others show it too.
    readonly property var _modes: [
        { id: "build", key: "1", groups: [["select"], ["wall", "object"], ["scale", "measure"]] },
        { id: "stage", key: "2", groups: [["select"], ["figure"], ["measure"]] },
        { id: "resolve", key: "3", groups: [["select"], ["camera", "light"], ["measure"]] },
        { id: "draw", key: "4", groups: [["select"], ["note"], ["line", "arrow", "rect", "ellipse", "area"], ["measure"]] }
    ]
    /// The tools and their keys, in the order of the palette.
    readonly property var names: _tools.map(t => t.name)
    readonly property var keys: _tools.map(t => t.key)
    /// The modes and their keys, in the order of the segments (the command palette lists them).
    readonly property var modes: _modes.map(m => m.id)
    readonly property var modeKeys: _modes.map(m => m.key)
    /// What the bar shows in the current mode: the tools in order, `gap` where a group starts.
    readonly property var shown: {
        const entries = []
        const mode = _modes.find(m => m.id === tools.mode) || _modes[0]
        mode.groups.forEach((group, g) => group.forEach((name, i) => entries.push({
            tool: _tools.find(t => t.name === name),
            gap: g > 0 && i === 0
        })))
        return entries
    }

    readonly property var _choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : { figure: [], object: [], light: [], color: [], dash: [], fill: [] }

    spacing: Kirigami.Units.smallSpacing

    // The controller keeps the tool: its drags and clicks do not repeat it.
    onToolChanged: {
        // A tool of another mode (its key, the palette) takes its mode along.
        if (!_inMode(tool, mode)) {
            mode = _ownerMode(tool)
        }
        _tell()
    }
    onModeChanged: {
        // The tool must be on the bar: else back to selecting.
        if (!_inMode(tool, mode)) {
            tool = "select"
        }
        store.mode = mode
    }
    onChoiceChanged: _tell()
    onDrawColorChanged: _tellStyle()
    onDrawDashChanged: _tellStyle()
    onDrawFillChanged: _tellStyle()
    Component.onCompleted: {
        if (modes.indexOf(store.mode) >= 0) {
            mode = store.mode
        }
        _tell()
        _tellStyle()
    }
    function _inMode(name, id) {
        const mode = _modes.find(m => m.id === id)
        return !!mode && mode.groups.some(group => group.indexOf(name) >= 0)
    }
    function _ownerMode(name) {
        const owner = _modes.find(m => _inMode(name, m.id))
        return owner ? owner.id : _modes[0].id
    }
    function _tell() {
        controller.setTool(tool, choice)
    }
    function _tellStyle() {
        controller.setDrawStyle(drawColor, drawDash, drawFill)
    }

    QQC2.ButtonGroup { id: group }

    // The segments set `currentIndex` themselves, which ends a binding: Binding keeps it.
    KanteSegmented {
        id: modeBox
        objectName: "modeBox"
        Accessible.name: tools.texts.t("mode.name")
        model: tools.modes.map(id => tools.texts.t("mode." + id))
        tooltips: tools.modes.map((id, i) => tools.texts.t("tool.tip", { name: tools.texts.t("mode." + id), key: tools.modeKeys[i] }))
        onActivated: index => tools.mode = tools.modes[index]
    }
    Binding {
        target: modeBox
        property: "currentIndex"
        value: tools.modes.indexOf(tools.mode)
    }

    Repeater {
        model: tools.shown
        delegate: RowLayout {
            id: slot
            required property var modelData
            readonly property string name: modelData.tool.name
            readonly property string key: modelData.tool.key
            spacing: tools.spacing

            Kirigami.Separator {
                visible: slot.modelData.gap
                // Not fillHeight: it would let the whole bar grow to the height of its parent.
                Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
                Layout.alignment: Qt.AlignVCenter
                Layout.leftMargin: tools.spacing
                Layout.rightMargin: tools.spacing
            }
            KanteToolButton {
                objectName: "tool." + slot.name
                text: tools.texts.t("tool." + slot.name)
                icon.name: slot.modelData.tool.icon
                // No icon in the theme (offscreen, a bare system): the name instead.
                display: probe.status === Image.Ready ? QQC2.AbstractButton.IconOnly : QQC2.AbstractButton.TextOnly
                checkable: true
                checked: tools.tool === slot.name
                QQC2.ButtonGroup.group: group
                onClicked: tools.tool = slot.name
                Accessible.name: text
                QQC2.ToolTip.text: slot.name === "scale" || slot.name === "measure" || slot.name === "note" || slot.name === "area"
                    ? tools.texts.t("tool." + slot.name + "Tip", { key: slot.key })
                    : tools.texts.t("tool.tip", { name: text, key: slot.key })
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay

                // Asks the theme whether it has the icon.
                IconImage {
                    id: probe
                    visible: false
                    name: parent.icon.name
                    sourceSize: Qt.size(Kirigami.Units.iconSizes.smallMedium, Kirigami.Units.iconSizes.smallMedium)
                }
            }
        }
    }

    // Type to narrow the list; the groups keep the built-in and the own objects apart.
    KanteSearchCombo {
        id: objectBox
        objectName: "objectBox"
        visible: tools.tool === "object"
        implicitWidth: Kirigami.Units.gridUnit * 12
        Accessible.name: tools.texts.t("tool.pick.object")
        placeholderText: tools.texts.t("library.search")
        textRole: "text"
        sectionRole: "group"
        currentIndex: 0
        model: tools._objects
        // A new list (an object was filed, a file imported) keeps the kind that was picked.
        property string pickedKind: ""
        onActivated: index => pickedKind = model[index].value
        onModelChanged: currentIndex = Math.max(0, model.findIndex(o => o.value === pickedKind))
    }
    KanteToolButton {
        objectName: "libraryImport"
        visible: tools.tool === "object"
        text: tools.texts.t("library.import")
        onClicked: tools.importRequested()
        QQC2.ToolTip.text: tools.texts.t("library.importTip")
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }
    KanteToolButton {
        objectName: "libraryExport"
        visible: tools.tool === "object"
        enabled: tools.hasOwnObjects
        text: tools.texts.t("library.export")
        onClicked: tools.exportRequested()
        QQC2.ToolTip.text: tools.texts.t("library.exportTip")
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
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

    // A list in focus takes letters and digits to pick its entry, like a text field does.
    readonly property bool listFocused: Window.window !== null && Window.window.activeFocusItem instanceof QQC2.ComboBox

    Instantiator {
        model: tools.names
        delegate: Shortcut {
            required property int index
            required property string modelData
            sequence: tools.keys[index]
            // Like the buttons: no tool change while the tool bar is locked.
            enabled: tools.controller.loaded && tools.enabled && !tools.listFocused
            onActivated: tools.tool = modelData
        }
    }
    Instantiator {
        model: tools.modes
        delegate: Shortcut {
            required property int index
            required property string modelData
            sequence: tools.modeKeys[index]
            enabled: tools.controller.loaded && tools.enabled && !tools.listFocused
            onActivated: tools.mode = modelData
        }
    }

    Settings {
        id: store
        category: "tools"
        property string mode: "build"
    }
}
