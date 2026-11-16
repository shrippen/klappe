import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-02, D-03 (planung/DESIGN.md): the window's header. The mark, where we are (project ›
// scene › plan, the plan's name a menu), the menus, undo and redo, and the command palette.
// Without a project (D-07) only the mark and the File menu with New and Open stay.
//
//  KLAPPE  Wartburg › Szene 12 › Plan A ▾    Datei ▾  Ausgeben ▾  Ansicht ▾   ↶ ↷ ⌕
//
// Every command is one action: the menus, the buttons and the palette (Ctrl+K) run the same.
// The window does what needs its dialogs (the signals) and keeps what is on view (the toggles).
RowLayout {
    id: header

    required property var texts
    required property var controller
    /// The tool bar (PlanTools), for the palette: `names`, `keys`, `modes`, `modeKeys`, `tool`,
    /// `mode`, `enabled`.
    property var tools: null
    /// While the backup question is open: what changes or replaces the project is locked.
    property bool locked: false
    property bool showScript: true
    property bool showShot: true
    property bool showTimeline: true
    /// The order of the exported shot list: "cut" or "shooting" (LH-72).
    property string shotOrder: "cut"

    signal newRequested()
    signal openRequested()
    signal projectRequested()
    signal backgroundRequested()
    signal viewToggled(string area, bool shown)
    signal shotOrderPicked(string order)
    /// Zoom of the plan: "in", "out" or "fit".
    signal zoomRequested(string how)
    signal playRequested()
    /// An object without a height is selected (LH-93): its card takes the height.
    signal heightRequested()
    /// The plan menu's template files (LH-07): the window asks for the file.
    signal templateImportRequested()
    signal templateExportRequested()

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : { scenes: [], plans: [], plan: "" }
    readonly property var scene: project.scenes.find(s => s.id === controller.sceneId) || null
    property alias commandPalette: palette
    /// The actions of the menus and buttons, in the order of the palette.
    readonly property var entries: [actNew, actOpen, actProject, actSave, actBackground, actBackgroundRemove,
                                    actPdf, actPng, actSvg, actCsv, actOds, actXlsx, actFrames,
                                    actFree, actHeights, actScript, actShot, actTimeline, actCut, actShooting,
                                    actZoomIn, actZoomOut, actZoomFit,
                                    actPlay, actMarkPrev, actMarkNext,
                                    actUndo, actRedo]

    // "Ctrl+Shift+Z" as the keyboard of the language names it.
    function keyName(sequence) {
        return sequence.replace("Ctrl", texts.t("key.ctrl")).replace("Shift", texts.t("key.shift"))
            .replace("Space", texts.t("key.space"))
    }

    spacing: Kirigami.Units.largeSpacing

    KanteHeading {
        text: header.texts.t("app.name").toUpperCase()
        level: 2
    }

    // The path gives way first: it elides before anything else is cut. The plan menu follows
    // right after it, the free room is behind.
    Kirigami.Heading {
        objectName: "path"
        Layout.fillWidth: true
        Layout.minimumWidth: 0
        Layout.maximumWidth: implicitWidth
        visible: header.controller.loaded
        level: 4
        elide: Text.ElideRight
        color: KanteStyle.mutedTextColor
        text: header.pathText()
    }
    PlanMenu {
        id: planMenu
        objectName: "planMenu"
        Layout.minimumWidth: Kirigami.Units.gridUnit * 5
        Layout.maximumWidth: Kirigami.Units.gridUnit * 14
        visible: header.controller.loaded
        enabled: !header.locked && plans.length > 0
        texts: header.texts
        controller: header.controller
        locked: header.locked
        onTemplateImportRequested: header.templateImportRequested()
        onTemplateExportRequested: header.templateExportRequested()
    }

    Item { Layout.fillWidth: true }

    Kirigami.Heading {
        objectName: "status"
        Layout.maximumWidth: Kirigami.Units.gridUnit * 10
        level: 5
        elide: Text.ElideRight
        color: KanteStyle.mutedTextColor
        text: header.controller.status !== "" ? header.controller.status : (header.controller.dirty ? header.texts.t("menu.unsaved") : "")
        visible: text !== ""
    }

    // Objects without a height (LH-93): the count, and a click goes on to the next.
    KanteButton {
        objectName: "missingHeights"
        visible: header.controller.loaded && header.controller.missingHeights > 0
        emphasis: KanteButton.Emphasis.Quiet
        size: KanteButton.Size.Small
        text: header.texts.t("height.missingHeader", { n: header.controller.missingHeights })
        onClicked: actHeights.trigger()
    }

    HeaderMenu {
        objectName: "fileMenu"
        text: header.texts.t("menu.file")
        KanteMenuItem { objectName: "new"; action: actNew; iconRoom: true }
        KanteMenuItem { objectName: "open"; action: actOpen; iconRoom: true }
        QQC2.MenuSeparator { visible: header.controller.loaded }
        KanteMenuItem { objectName: "project"; action: actProject; iconRoom: true; visible: header.controller.loaded }
        KanteMenuItem { objectName: "save"; action: actSave; iconRoom: true; visible: header.controller.loaded }
        QQC2.MenuSeparator { visible: header.controller.loaded }
        KanteMenuItem { objectName: "background"; action: actBackground; iconRoom: true; visible: header.controller.loaded }
        KanteMenuItem {
            objectName: "backgroundRemove"
            action: actBackgroundRemove
            iconRoom: true
            visible: header.controller.loaded && header.controller.hasBackground
        }
    }
    HeaderMenu {
        objectName: "exportMenu"
        visible: header.controller.loaded
        text: header.texts.t("menu.export")
        KanteMenuItem { objectName: "pdf"; action: actPdf; iconRoom: true }
        KanteMenuItem { objectName: "png"; action: actPng; iconRoom: true }
        KanteMenuItem { objectName: "svg"; action: actSvg; iconRoom: true }
        KanteMenuItem { objectName: "csv"; action: actCsv; iconRoom: true }
        KanteMenuItem { objectName: "ods"; action: actOds; iconRoom: true }
        KanteMenuItem { objectName: "xlsx"; action: actXlsx; iconRoom: true }
        KanteMenuItem { objectName: "frames"; action: actFrames; iconRoom: true }
    }
    HeaderMenu {
        objectName: "viewMenu"
        visible: header.controller.loaded
        text: header.texts.t("menu.view")
        KanteMenuSection { text: header.texts.t("menu.viewMode"); rule: false }
        KanteMenuItem { objectName: "free"; action: actFree }
        KanteMenuItem { objectName: "heights"; action: actHeights }
        KanteMenuSection { text: header.texts.t("menu.viewAreas") }
        KanteMenuItem { objectName: "script"; action: actScript }
        KanteMenuItem { objectName: "shot"; action: actShot }
        KanteMenuItem { objectName: "timeline"; action: actTimeline }
        KanteMenuSection { text: header.texts.t("menu.viewZoom") }
        KanteMenuItem { objectName: "zoomIn"; action: actZoomIn }
        KanteMenuItem { objectName: "zoomOut"; action: actZoomOut }
        KanteMenuItem { objectName: "zoomFit"; action: actZoomFit }
        KanteMenuSection { text: header.texts.t("menu.viewOrder") }
        KanteMenuItem { objectName: "cut"; action: actCut }
        KanteMenuItem { objectName: "shooting"; action: actShooting }
    }

    KanteToolButton {
        objectName: "undo"
        visible: header.controller.loaded
        action: actUndo
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: header.texts.t("tool.tip", { name: actUndo.text, key: actUndo.keys })
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }
    KanteToolButton {
        objectName: "redo"
        visible: header.controller.loaded
        action: actRedo
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: header.texts.t("tool.tip", { name: actRedo.text, key: actRedo.keys })
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }
    KanteToolButton {
        objectName: "paletteButton"
        visible: header.controller.loaded
        action: actPalette
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: header.texts.t("tool.tip", { name: actPalette.text, key: actPalette.keys })
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }

    CommandAction {
        id: actNew
        cmd: "project.new"
        group: "file"
        text: header.texts.t("menu.new")
        shortcut: "Ctrl+N"
        keys: header.keyName("Ctrl+N")
        enabled: !header.locked
        onTriggered: header.newRequested()
    }
    CommandAction {
        id: actOpen
        cmd: "project.open"
        group: "file"
        text: header.texts.t("menu.open")
        shortcut: "Ctrl+O"
        keys: header.keyName("Ctrl+O")
        enabled: !header.locked
        onTriggered: header.openRequested()
    }
    CommandAction {
        id: actProject
        cmd: "project.edit"
        group: "file"
        text: header.texts.t("menu.project")
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.projectRequested()
    }
    CommandAction {
        id: actSave
        cmd: "project.save"
        group: "file"
        text: header.texts.t("menu.save")
        shortcut: "Ctrl+S"
        keys: header.keyName("Ctrl+S")
        enabled: header.controller.loaded
        onTriggered: header.controller.save()
    }
    CommandAction {
        id: actBackground
        cmd: "background.set"
        group: "file"
        text: header.texts.t("menu.background")
        enabled: !header.locked && header.controller.loaded
        onTriggered: header.backgroundRequested()
    }
    CommandAction {
        id: actBackgroundRemove
        cmd: "background.remove"
        group: "file"
        text: header.texts.t("menu.backgroundRemove")
        enabled: !header.locked && header.controller.loaded && header.controller.hasBackground
        onTriggered: header.controller.removeBackground()
    }

    CommandAction {
        id: actPdf
        cmd: "export.pdf"
        group: "export"
        text: header.texts.t("menu.pdf")
        shortcut: "Ctrl+P"
        keys: header.keyName("Ctrl+P")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("pdf")
    }
    ExportDialog {
        id: exportDialog
        objectName: "exportDialog"
        texts: header.texts
        controller: header.controller
        order: header.shotOrder
    }
    CommandAction {
        id: actPng
        cmd: "export.png"
        group: "export"
        text: header.texts.t("menu.png")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("png")
    }
    CommandAction {
        id: actSvg
        cmd: "export.svg"
        group: "export"
        text: header.texts.t("menu.svg")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("svg")
    }
    CommandAction {
        id: actCsv
        cmd: "export.csv"
        group: "export"
        text: header.texts.t("menu.csv")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("csv")
    }
    CommandAction {
        id: actOds
        cmd: "export.ods"
        group: "export"
        text: header.texts.t("menu.ods")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("ods")
    }
    CommandAction {
        id: actXlsx
        cmd: "export.xlsx"
        group: "export"
        text: header.texts.t("menu.xlsx")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("xlsx")
    }
    CommandAction {
        id: actFrames
        cmd: "export.frames"
        group: "export"
        text: header.texts.t("menu.frames")
        enabled: header.controller.loaded
        onTriggered: exportDialog.request("frames")
    }

    // Plan or 3D (J): the controller keeps which one shows.
    CommandAction {
        id: actFree
        cmd: "view.free"
        group: "view"
        text: header.texts.t("view.free")
        shortcut: "J"
        keys: "J"
        checkable: true
        enabled: header.controller.loaded && !header.locked
        onToggled: header.controller.toggleView()
    }
    // Height missing (Shift+H, LH-93): selects the next object without one; its card takes the
    // typed height.
    CommandAction {
        id: actHeights
        cmd: "view.heights"
        group: "view"
        text: header.texts.t("height.next")
        shortcut: "Shift+H"
        keys: header.keyName("Shift+H")
        enabled: header.controller.loaded && !header.locked && header.controller.missingHeights > 0
        onTriggered: {
            if (header.controller.nextMissingHeight()) {
                header.heightRequested()
            }
        }
    }
    // The checkmark follows the controller, which also leaves 3D when another project opens.
    Binding {
        target: actFree
        property: "checked"
        value: header.controller.viewFree === true
    }

    // The areas toggle; the window keeps what is shown and hands it back.
    CommandAction {
        id: actScript
        cmd: "view.script"
        group: "view"
        text: header.texts.t("toggle.script")
        checkable: true
        checked: header.showScript
        enabled: header.controller.loaded
        onToggled: header.viewToggled("script", checked)
    }
    CommandAction {
        id: actShot
        cmd: "view.shot"
        group: "view"
        text: header.texts.t("toggle.shot")
        checkable: true
        checked: header.showShot
        enabled: header.controller.loaded
        onToggled: header.viewToggled("shot", checked)
    }
    CommandAction {
        id: actTimeline
        cmd: "view.timeline"
        group: "view"
        text: header.texts.t("toggle.timeline")
        checkable: true
        checked: header.showTimeline
        enabled: header.controller.loaded
        onToggled: header.viewToggled("timeline", checked)
    }
    QQC2.ActionGroup { id: orderGroup }
    CommandAction {
        id: actCut
        cmd: "order.cut"
        group: "view"
        text: header.texts.t("export.order.cut")
        QQC2.ActionGroup.group: orderGroup
        checkable: true
        checked: header.shotOrder === "cut"
        enabled: header.controller.loaded
        onTriggered: header.shotOrderPicked("cut")
    }
    CommandAction {
        id: actShooting
        cmd: "order.shooting"
        group: "view"
        text: header.texts.t("export.order.shooting")
        QQC2.ActionGroup.group: orderGroup
        checkable: true
        checked: header.shotOrder === "shooting"
        enabled: header.controller.loaded
        onTriggered: header.shotOrderPicked("shooting")
    }

    // Zoom of the plan; the window hands it on. Ctrl+= is the same key as Ctrl++ without Shift.
    CommandAction {
        id: actZoomIn
        cmd: "view.zoomIn"
        group: "view"
        text: header.texts.t("menu.zoomIn")
        shortcut: "Ctrl++"
        keys: header.keyName("Ctrl++")
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.zoomRequested("in")
    }
    Shortcut {
        sequence: "Ctrl+="
        enabled: actZoomIn.enabled
        onActivated: actZoomIn.trigger()
    }
    CommandAction {
        id: actZoomOut
        cmd: "view.zoomOut"
        group: "view"
        text: header.texts.t("menu.zoomOut")
        shortcut: "Ctrl+-"
        keys: header.keyName("Ctrl+-")
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.zoomRequested("out")
    }
    CommandAction {
        id: actZoomFit
        cmd: "view.zoomFit"
        group: "view"
        text: header.texts.t("menu.zoomFit")
        shortcut: "Ctrl+0"
        keys: header.keyName("Ctrl+0")
        // The whole plan: the 3D view has no plan to fit.
        enabled: header.controller.loaded && !header.locked && !header.controller.viewFree
        onTriggered: header.zoomRequested("fit")
    }

    // Play or pause: K works everywhere. Space does too, except where it already means
    // something: the plan pans with Space held (Kante), a button presses with it, a text field
    // types it. A text field keeps its keys anyway: it takes the key before a shortcut.
    CommandAction {
        id: actPlay
        cmd: "playback.toggle"
        group: "playback"
        text: header.texts.t("menu.play")
        shortcut: "K"
        keys: header.keyName("Space") + " / K"
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.playRequested()
    }
    readonly property Item focused: Window.window ? Window.window.activeFocusItem : null
    Shortcut {
        sequence: "Space"
        enabled: actPlay.enabled && !(header.focused && header.focused.down !== undefined)
            && !(header.focused && header.focused.objectName === "planCanvas")
        onActivated: actPlay.trigger()
    }

    // Marks: the playhead jumps to the previous or next mark of the shot.
    CommandAction {
        id: actMarkPrev
        cmd: "playback.markPrev"
        group: "playback"
        text: header.texts.t("menu.markPrev")
        shortcut: ","
        keys: ","
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.controller.stepMark(-1)
    }
    CommandAction {
        id: actMarkNext
        cmd: "playback.markNext"
        group: "playback"
        text: header.texts.t("menu.markNext")
        shortcut: "."
        keys: "."
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.controller.stepMark(1)
    }

    CommandAction {
        id: actUndo
        cmd: "edit.undo"
        group: "edit"
        text: header.texts.t("menu.undo")
        icon.name: "edit-undo"
        shortcut: "Ctrl+Z"
        keys: header.keyName("Ctrl+Z")
        enabled: header.controller.loaded && header.controller.canUndo
        onTriggered: header.controller.undo()
    }
    CommandAction {
        id: actRedo
        cmd: "edit.redo"
        group: "edit"
        text: header.texts.t("menu.redo")
        icon.name: "edit-redo"
        shortcut: "Ctrl+Shift+Z"
        keys: header.keyName("Ctrl+Shift+Z")
        enabled: header.controller.loaded && header.controller.canRedo
        onTriggered: header.controller.redo()
    }
    CommandAction {
        id: actPalette
        text: header.texts.t("menu.commands")
        icon.name: "edit-find"
        shortcut: "Ctrl+K"
        keys: header.keyName("Ctrl+K")
        enabled: header.controller.loaded
        onTriggered: palette.open()
    }

    function pathText() {
        const scene = header.scene
        const sceneText = scene ? header.texts.t("app.scene", { number: scene.number }) + " · " + scene.heading : ""
        return [header.controller.title, sceneText].filter(t => t !== "").join(" › ") + " ›"
    }

    // What the palette lists: the modes, the tools, the actions, the scenes, the plans. Each is
    // listed with the key it has anywhere else.
    readonly property var commands: [].concat(modeCommands(), toolCommands(), actionCommands(entries), sceneCommands(),
                                              actionCommands(planMenu.actions), planCommands())

    function actionCommands(actions) {
        return actions.map(a => ({
            id: a.cmd,
            text: a.text,
            group: texts.t("palette.group." + a.group),
            keys: a.keys,
            enabled: a.enabled
        }))
    }

    function modeCommands() {
        if (!tools) {
            return []
        }
        return tools.modes.map((id, i) => ({
            id: "mode:" + id,
            text: texts.t("palette.mode", { name: texts.t("mode." + id) }),
            group: texts.t("palette.group.modes"),
            keys: tools.modeKeys[i],
            enabled: tools.enabled
        }))
    }

    function toolCommands() {
        if (!tools) {
            return []
        }
        return tools.names.map((name, i) => ({
            id: "tool:" + name,
            text: texts.t("tool." + name),
            group: texts.t("palette.group.tools"),
            keys: tools.keys[i],
            enabled: tools.enabled
        }))
    }

    function sceneCommands() {
        return project.scenes.map(s => ({
            id: "scene:" + s.id,
            text: texts.t("app.scene", { number: s.number }) + " · " + s.heading,
            group: texts.t("palette.group.scenes"),
            enabled: !locked && s.id !== controller.sceneId
        }))
    }

    function planCommands() {
        return project.plans.map(p => ({
            id: "plan:" + p.id,
            text: p.name,
            group: texts.t("palette.group.plans"),
            enabled: !locked && p.id !== project.plan
        }))
    }

    // The palette hands back the id; what it names is found again in the same places.
    function run(id) {
        const at = id.indexOf(":")
        const kind = at < 0 ? id : id.substring(0, at)
        const target = id.substring(at + 1)
        switch (kind) {
        case "mode":
            tools.mode = target
            return
        case "tool":
            tools.tool = target
            return
        case "scene":
            controller.selectScene(target)
            return
        case "plan":
            controller.selectPlan(target)
            return
        }
        const action = entries.concat(planMenu.actions).find(a => a.cmd === id)
        if (action) {
            action.trigger()
        }
    }

    KanteCommandPalette {
        id: palette
        objectName: "palette"
        commands: header.commands
        placeholderText: header.texts.t("palette.placeholder")
        emptyText: header.texts.t("palette.empty")
        accessibleName: header.texts.t("palette.name")
        onTriggered: id => header.run(id)
    }
}
