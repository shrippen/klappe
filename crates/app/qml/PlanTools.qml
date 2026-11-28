import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQml
import QtCore
import org.kde.kirigami as Kirigami
import Kante

// The drawing tools of the tool bar (LH-08 ff.): one tool at a time, a list for the tools that
// need a choice (object from the library, figure, lamp), the snapping switch. The object
// tool also offers to import and export library files (LH-12): the window asks for the file. Keys V W O F C L M D N I A B
// E P pick the tool (NF-06) in every mode; a key of another mode's tool changes the mode. The
// drawing tools (line, arrow, rect, ellipse, area; LH-19) also
// show colour, line style and, for closed shapes, fill. The controller does the work: this only
// says which tool and which choice.
//
// D-01: four working modes (keys 1-4) decide which tools the bar shows. The mode is a view only
// (what can be picked and edited does not depend on it), so it lives here, in `_modes`, the one
// table that says which tool belongs to which mode; the settings keep it over a restart. Tools are
// buttons with symbol and word (D-47), the symbol alone below the wide step; one without an icon
// in the theme shows its name instead.
RowLayout {
    id: tools

    required property var texts
    required property var controller

    property string tool: "select"
    /// D-09: below the wide step the options (colour, line, fill, snapping, library) sit in a
    /// popover, and so do the tools that do not fit the bar. D-47: wide, the tools show symbol and
    /// word; below, the symbol alone (`words`).
    property bool compact: false
    /// How many of the `shown` tools the bar has room for; the rest are in the popover.
    property int fitCount: shown.length
    /// D-47: the tools show their word beside the symbol. The window's step decides, not `compact`:
    /// a module bar makes the bar narrower than the window.
    property bool words: !compact
    readonly property int toolDisplay: words ? QQC2.AbstractButton.TextBesideIcon : QQC2.AbstractButton.IconOnly
    /// The working mode: the id of an entry of `_modes`.
    property string mode: "build"
    /// The value picked in the current tool's list: library kind, character id, camera id (empty: a new camera) or lamp kind.
    readonly property string choice: {
        if (tool === "object") {
            const entry = objectBox.model[objectBox.currentIndex]
            return entry ? entry.value : ""
        }
        const box = tool === "figure" ? figureBox : tool === "camera" ? cameraBox : tool === "light" ? lightBox : null
        return box && box.currentValue !== undefined ? String(box.currentValue) : ""
    }

    /// What the picked entry is called, for the hint line.
    readonly property string choiceName: {
        if (tool === "object") {
            const entry = objectBox.model[objectBox.currentIndex]
            return entry ? entry.text : ""
        }
        const box = tool === "figure" ? figureBox : tool === "camera" ? cameraBox : tool === "light" ? lightBox : null
        return box && box.currentIndex >= 0 ? box.currentText : ""
    }
    /// LH-101, LH-122: what the next click of the tool does, and that the tool stays.
    readonly property string hint: {
        if (tool === "select") {
            return ""
        }
        const named = ["object", "figure", "light"].indexOf(tool) >= 0
        const key = tool === "camera" && choice === "" ? "hint.cameraNew" : "hint." + tool
        if (named && choice === "") {
            return texts.t("hint.noChoice")
        }
        const stays = ["height", "scale", "measure"].indexOf(tool) < 0
        return texts.t(key, { name: choiceName }) + " " + texts.t(stays ? "hint.stays" : "hint.esc")
    }

    /// The library as the list shows it: built in first, then the project's own, in groups. An
    /// own entry without a group is in "Own". Entries of one heading are put together.
    readonly property var _objects: {
        const shown = (_choices.object || []).map(o => ({
            value: o.kind,
            text: o.custom ? (o.label || o.kind) : texts.t("objectKind." + o.kind),
            group: !o.custom ? texts.t("objectGroup." + o.group) : (o.group || texts.t("library.custom"))
        }))
        const headings = shown.map(o => o.group).filter((g, i, all) => all.indexOf(g) === i)
        return headings.reduce((list, g) => list.concat(shown.filter(o => o.group === g)), [])
    }
    /// The own entry the list has picked: its kind ("" for none or a built-in one), and its data.
    readonly property string pickedOwnKind: {
        if (tool !== "object") {
            return ""
        }
        const entry = (_choices.object || []).find(o => o.kind === choice)
        return entry && entry.custom ? entry.kind : ""
    }
    readonly property var pickedOwn: (_choices.object || []).find(o => o.kind === pickedOwnKind) || null
    readonly property bool hasOwnObjects: (_choices.object || []).some(o => o.custom)

    /// How long the delete button waits for its second click.
    readonly property int _askMs: 5000

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
        { name: "room", key: "Q", icon: "draw-square-inverted-corners" },
        { name: "door", key: "T", icon: "application-exit" },
        { name: "window", key: "X", icon: "window-new" },
        { name: "object", key: "O", icon: "draw-cuboid" },
        { name: "freeform", key: "U", icon: "draw-path" },
        { name: "height", key: "H", icon: "transform-move-vertical" },
        { name: "figure", key: "F", icon: "user" },
        { name: "camera", key: "C", icon: "camera-video" },
        { name: "light", key: "L", icon: "flashlight-on" },
        { name: "scale", key: "M", icon: "transform-scale" },
        { name: "measure", key: "D", icon: "measure" },
        { name: "note", key: "N", icon: "view-pim-notes" },
        // Shift+B: every letter is taken, and B is the rectangle.
        { name: "image", key: "Shift+B", icon: "insert-image" },
        { name: "line", key: "I", icon: "draw-line" },
        { name: "arrow", key: "A", icon: "draw-arrow" },
        { name: "rect", key: "B", icon: "draw-rectangle" },
        { name: "ellipse", key: "E", icon: "draw-ellipse" },
        { name: "area", key: "P", icon: "draw-polygon" },
        // Shift+P: the pen next to the polygon (P); its mode is the stage's, where the bar has room
        // (in the draw mode the colour, line and fill lists leave none at the wide step).
        { name: "sketch", key: "Shift+P", icon: "draw-freehand" }
    ]
    /// The modes (D-01) with the tools of each in groups, separated in the bar. A tool in two modes
    /// (selecting, measuring) belongs to the first; the others show it too.
    readonly property var _modes: [
        { id: "build", key: "1", groups: [["select"], ["wall", "room", "door", "window", "object", "freeform", "height"], ["scale", "image", "measure"]] },
        { id: "stage", key: "2", groups: [["select"], ["figure"], ["sketch"], ["measure"]] },
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

    readonly property var _choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : { figure: [], object: [], light: [], color: [], dash: [], fill: [], grid: [] }

    spacing: Kirigami.Units.smallSpacing

    property bool _tookMode: false

    // The controller keeps the tool: its drags and clicks do not repeat it.
    onToolChanged: {
        _fit()
        // A tool of another mode (its key, the palette) takes its mode along.
        if (!_inMode(tool, mode)) {
            _tookMode = true
            mode = _ownerMode(tool)
            _tookMode = false
        }
        _tell()
    }
    onModeChanged: {
        // The tool must be on the bar: else back to selecting.
        if (!_inMode(tool, mode)) {
            tool = "select"
        }
        store.mode = mode
        // The card of the selection goes with the mode (Q1-04); a tool key that took the mode
        // along keeps the selection (W with a wall selected).
        if (!_tookMode) {
            controller.clearSelection()
        }
    }
    onChoiceChanged: _tell()
    onDrawColorChanged: _tellStyle()
    onDrawDashChanged: _tellStyle()
    onDrawFillChanged: _tellStyle()
    Component.onCompleted: {
        if (modes.indexOf(store.mode) >= 0) {
            mode = store.mode
        }
        // The last picks of the lists (LH-123); a list that is not filled yet takes its pick
        // when it is.
        objectBox.pickedKind = store.object
        figureBox.picked = store.figure
        cameraBox.picked = store.camera
        lightBox.picked = store.light
        objectBox.currentIndex = Math.max(0, objectBox.model.findIndex(o => o.value === store.object))
        figureBox.currentIndex = Math.max(0, figureBox.model.findIndex(m => m.value === store.figure))
        cameraBox.currentIndex = Math.max(0, cameraBox.model.findIndex(m => m.value === store.camera))
        lightBox.currentIndex = Math.max(0, lightBox.model.findIndex(m => m.value === store.light))
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
    // Tools beyond the room go to the popover. The room is what the width leaves after the mode
    // box, the popover's button and the tool's list.
    function _fit() {
        let room = width - modeBox.width - moreButton.implicitWidth - spacing * 2
        room -= optionsLoader.item ? optionsLoader.implicitWidth + spacing : 0
        room -= objectBox.visible ? objectBox.Layout.minimumWidth + spacing : 0
        room -= figureBox.visible ? figureBox.implicitWidth + spacing : 0
        room -= lightBox.visible ? lightBox.implicitWidth + spacing : 0
        let count = 0
        // A tool the repeater has not made yet counts as no fit; the next `_fit` sees it.
        for (let item = slots.itemAt(0); item && item.implicitWidth + spacing <= room; item = slots.itemAt(count)) {
            room -= item.implicitWidth + spacing
            count++
        }
        fitCount = count
    }
    onWidthChanged: _fit()
    onCompactChanged: _fit()
    /// Name and key of a tool; the tools with more to say have a text of their own.
    function toolTip(name, key) {
        const own = ["scale", "measure", "note", "image", "area", "door", "window", "height", "room", "freeform", "sketch"]
        if (own.indexOf(name) >= 0) {
            return texts.t("tool." + name + "Tip", { key: key })
        }
        return texts.t("tool.tip", { name: texts.t("tool." + name), key: key })
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
        Layout.minimumWidth: implicitWidth
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
        id: slots
        model: tools.shown
        onItemAdded: tools._fit()
        onItemRemoved: tools._fit()
        delegate: RowLayout {
            id: slot
            required property int index
            required property var modelData
            readonly property string name: modelData.tool.name
            readonly property string key: modelData.tool.key
            spacing: tools.spacing
            visible: index < tools.fitCount
            onImplicitWidthChanged: tools._fit()

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
                Layout.minimumWidth: implicitWidth
                text: tools.texts.t("tool." + slot.name)
                icon.name: slot.modelData.tool.icon
                // Kante shows the name when the theme lacks the icon.
                display: tools.toolDisplay
                checkable: true
                checked: tools.tool === slot.name
                QQC2.ButtonGroup.group: group
                onClicked: tools.tool = slot.name
                Accessible.name: text
                QQC2.ToolTip.text: tools.toolTip(slot.name, slot.key)
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
            }
        }
    }

    // Type to narrow the list; the groups keep the built-in and the own objects apart.
    KanteSearchCombo {
        id: objectBox
        objectName: "objectBox"
        onVisibleChanged: tools._fit()
        visible: tools.tool === "object"
        implicitWidth: Kirigami.Units.gridUnit * 12
        // Gives way before anything else does when the bar is narrow (only a filling item shrinks).
        Layout.fillWidth: true
        Layout.minimumWidth: Kirigami.Units.gridUnit * 6
        Layout.maximumWidth: implicitWidth
        Accessible.name: tools.texts.t("tool.pick.object")
        placeholderText: tools.texts.t("library.search")
        textRole: "text"
        sectionRole: "group"
        currentIndex: 0
        model: tools._objects
        // A new list (an object was filed, a file imported) keeps the kind that was picked.
        property string pickedKind: ""
        onActivated: index => {
            pickedKind = model[index].value
            store.object = pickedKind
        }
        onModelChanged: currentIndex = Math.max(0, model.findIndex(o => o.value === pickedKind))
    }
    KanteComboBox {
        id: figureBox
        objectName: "figureBox"
        onVisibleChanged: tools._fit()
        visible: tools.tool === "figure"
        Accessible.name: tools.texts.t("tool.pick.figure")
        textRole: "text"
        valueRole: "value"
        displayText: count > 0 ? currentText : tools.texts.t("tool.noFigures")
        model: tools._choices.figure.map(f => ({ value: f.id, text: f.name }))
        // LH-123: the pick outlives a new list and a restart.
        property string picked: ""
        onActivated: {
            picked = String(currentValue)
            store.figure = picked
        }
        onModelChanged: currentIndex = Math.max(0, model.findIndex(m => m.value === picked))
    }
    KanteComboBox {
        id: cameraBox
        objectName: "cameraBox"
        visible: tools.tool === "camera"
        Accessible.name: tools.texts.t("tool.pick.camera")
        textRole: "text"
        valueRole: "value"
        model: [{ value: "", text: tools.texts.t("tool.newCamera") }]
            .concat((tools._choices.cameras || []).map(c => ({ value: c.id, text: c.label })))
        property string picked: ""
        onActivated: {
            picked = String(currentValue)
            store.camera = picked
        }
        onModelChanged: currentIndex = Math.max(0, model.findIndex(m => m.value === picked))
    }
    KanteComboBox {
        id: lightBox
        objectName: "lightBox"
        onVisibleChanged: tools._fit()
        visible: tools.tool === "light"
        Accessible.name: tools.texts.t("tool.pick.light")
        textRole: "text"
        valueRole: "value"
        model: tools._choices.light.map(k => ({ value: k, text: tools.texts.t("lightPreset." + k) }))
        property string picked: ""
        onActivated: {
            picked = String(currentValue)
            store.light = picked
        }
        onModelChanged: currentIndex = Math.max(0, model.findIndex(m => m.value === picked))
    }

    // D-09: the buttons of the options, inline while wide, in a popover below that.
    Component {
        id: optionsItems
        GridLayout {
            id: ownItems
            // Whether the delete button has asked and waits for the second click.
            property bool asking: false

            // The fields start with the name and group of the picked entry.
            function fill() {
                asking = false
                ownName.text = tools.pickedOwn ? (tools.pickedOwn.label || tools.pickedOwn.kind) : ""
                ownGroup.text = tools.pickedOwn ? tools.pickedOwn.group : ""
            }
            function sureText() {
                const uses = tools.pickedOwn ? tools.pickedOwn.uses : 0
                return uses === 0 ? tools.texts.t("library.deleteSureNone") : tools.texts.t("library.deleteSure", { count: uses })
            }
            function change() {
                ownName.invalid = !tools.controller.editLibraryItem(tools.pickedOwnKind, ownName.text, ownGroup.text)
            }
            function remove() {
                if (!asking) {
                    asking = true
                    askTimer.restart()
                    return
                }
                asking = false
                tools.controller.removeLibraryItem(tools.pickedOwnKind)
            }
            Component.onCompleted: fill()
            Connections {
                target: tools
                function onPickedOwnChanged() { ownItems.fill() }
            }
            Timer {
                id: askTimer
                interval: tools._askMs
                onTriggered: ownItems.asking = false
            }

            rowSpacing: tools.spacing
            columnSpacing: tools.spacing
            // One column stacks them in the popover; many put them in a row.
            columns: tools.compact ? 1 : tools._tools.length

            // Rename, regroup and delete the own entry the list has picked.
            KanteTextField {
                id: ownName
                objectName: "ownName"
                visible: tools.pickedOwn !== null
                Layout.preferredWidth: Kirigami.Units.gridUnit * 8
                Accessible.name: tools.texts.t("library.name")
                placeholderText: tools.texts.t("library.name")
                onTextChanged: invalid = false
                onAccepted: ownItems.change()
            }
            KanteTextField {
                id: ownGroup
                objectName: "ownGroup"
                visible: tools.pickedOwn !== null
                Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                Accessible.name: tools.texts.t("library.group")
                placeholderText: tools.texts.t("library.group")
                onAccepted: ownItems.change()
            }
            KanteToolButton {
                objectName: "libraryRename"
                visible: tools.pickedOwn !== null
                text: tools.texts.t("library.rename")
                onClicked: ownItems.change()
                QQC2.ToolTip.text: tools.texts.t("library.renameTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
            }
            // The first click asks, naming how many objects of the plan use the entry.
            KanteToolButton {
                objectName: "libraryDelete"
                visible: tools.pickedOwn !== null
                text: ownItems.asking ? ownItems.sureText() : tools.texts.t("library.delete")
                onClicked: ownItems.remove()
                QQC2.ToolTip.text: tools.texts.t("library.deleteTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
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
            // Sketch pen (LH-120): one command takes every stroke off the plan.
            KanteToolButton {
                objectName: "sketchClear"
                visible: tools.tool === "sketch"
                text: tools.texts.t("sketch.clear")
                onClicked: tools.controller.clearSketches()
                QQC2.ToolTip.text: tools.texts.t("sketch.clearTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
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
            KanteComboBox {
                objectName: "gridBox"
                Accessible.name: tools.texts.t("tool.grid")
                textRole: "text"
                valueRole: "value"
                model: (tools._choices.grid || []).map(g => ({ value: g.id, text: tools.texts.t("grid." + g.id) }))
                currentIndex: (tools._choices.grid || []).findIndex(g => g.id === tools.controller.gridName)
                onActivated: tools.controller.setGrid(String(currentValue))
                QQC2.ToolTip.text: tools.texts.t("tool.gridTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
            }
        }
    }

    Loader {
        id: optionsLoader
        active: !tools.compact
        visible: active
        sourceComponent: optionsItems
        onImplicitWidthChanged: tools._fit()
        onItemChanged: tools._fit()
    }
    KanteToolButton {
        id: moreButton
        objectName: "optionsButton"
        // Tools that did not fit are in the popover at every step.
        visible: tools.compact || tools.fitCount < tools.shown.length
        Layout.minimumWidth: implicitWidth
        display: QQC2.AbstractButton.IconOnly
        icon.name: "overflow-menu"
        text: tools.texts.t("tool.options")
        Accessible.name: text
        onClicked: optionsPopover.openAt(moreButton)
        QQC2.ToolTip.text: text
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }
    KantePopover {
        id: optionsPopover
        objectName: "optionsPopover"
        title: tools.texts.t("tool.options")

        Flow {
            objectName: "overflowTools"
            width: parent.width
            spacing: tools.spacing
            Repeater {
                model: tools.shown.slice(tools.fitCount)
                delegate: KanteToolButton {
                    required property var modelData
                    objectName: "overflow." + modelData.tool.name
                    text: tools.texts.t("tool." + modelData.tool.name)
                    icon.name: modelData.tool.icon
                    display: tools.toolDisplay
                    checkable: true
                    checked: tools.tool === modelData.tool.name
                    Accessible.name: text
                    QQC2.ToolTip.text: tools.toolTip(modelData.tool.name, modelData.tool.key)
                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                    onClicked: {
                        tools.tool = modelData.tool.name
                        optionsPopover.close()
                    }
                }
            }
        }
        Loader {
            active: tools.compact
            sourceComponent: optionsItems
        }
    }

    // A layout has the width its parent gives it only with a filling child; without, it keeps
    // its own width and `_fit` sees no room.
    Item { Layout.fillWidth: true }

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
        /// The last pick of each tool's list (LH-123): library kind, character id, camera id
        /// (empty: a new camera) and lamp kind.
        property string object: ""
        property string figure: ""
        property string camera: ""
        property string light: ""
    }
}
