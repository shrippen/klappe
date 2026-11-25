import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-02, D-03 (planung/DESIGN.md): the window's header. The mark, where we are (project ›
// scene › plan, the plan's name a menu), the menus, undo and redo, and the command palette.
// Without a project (D-07) only the mark stays: the start page has New and Open, and shows the status.
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
    /// The modules of the bar (`{ id, name, icon }`) and the id of the open one (D-11).
    property var modules: []
    property string module: "stellprobe"
    /// Camera pictures in the plan (LH-113), long shot description in the table (LH-112).
    property bool showCameraViews: true
    property bool showShotDescription: false
    /// The order of the exported shot list: "cut" or "shooting" (LH-72).
    property string shotOrder: "cut"

    signal newRequested()
    signal openRequested()
    signal closeRequested()
    signal projectRequested()
    /// The open questions about the book are to be answered.
    signal bookRequested()
    signal backgroundRequested()
    /// A screenplay file is to be loaded: the window asks for it.
    signal scriptImportRequested()
    /// The screenplay view sends what is typed (before a save).
    signal flushRequested()
    signal viewToggled(string area, bool shown)
    /// A module is asked for: its button, key or palette command (D-11, D-12).
    signal moduleRequested(string id)
    signal shotOrderPicked(string order)
    /// Zoom of the plan: "in", "out" or "fit".
    signal zoomRequested(string how)
    signal playRequested()
    signal speakRequested()
    /// The system has a speech voice.
    property bool speechAvailable: false
    /// The script is to show line `line` (-1: the scene's head), or the plan (DB-63).
    signal scriptRequested(int line)
    signal planRequested()
    /// An object without a height is selected (LH-93): its card takes the height.
    signal heightRequested()
    /// The plan menu's template files (LH-07): the window asks for the file.
    signal templateImportRequested()
    signal templateExportRequested()

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : { scenes: [], plans: [], plan: "" }
    readonly property var scene: project.scenes.find(s => s.id === controller.sceneId) || null
    property alias commandPalette: palette
    /// The actions of the menus and buttons, in the order of the palette.
    readonly property var entries: [actNew, actOpen, actProject, actBook, actSave, actClose, actBackground, actBackgroundRemove, actScriptImport,
                                    actFree, actWallCut, actThrough, actHeights,
                                    actScript, actShot, actTimeline,
                                    actUndo, actRedo].concat(moduleCommands.actions, exportCommands.actions, shotViews.actions, zoom.actions,
                                    pageExports.actions, reportCommands.actions, viewCommands.actions, objectCommands.actions,
                                    playCommands.actions, navCommands.actions, armCommands.actions, jumpCommands.actions)

    // "Ctrl+Shift+Z" as the keyboard of the language names it.
    function keyName(sequence) {
        return sequence.replace("Ctrl", texts.t("key.ctrl")).replace("Shift", texts.t("key.shift"))
            .replace("Space", texts.t("key.space"))
    }

    // "Undo: Move (Ctrl+Z)": the step the button takes back or does again (LH-103); the plain
    // name while the step has no text of its own.
    function stepTip(tipKey, step, action) {
        const key = "step." + step
        if (!step || texts.map[key] === undefined) {
            return texts.t("tool.tip", { name: action.text, key: action.keys })
        }
        return texts.t(tipKey, { step: texts.t(key), key: action.keys })
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
        visible: header.controller.loaded && header.module === "stellprobe"
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
        visible: header.controller.loaded && text !== ""

        // What the width cuts off shows whole on hover.
        HoverHandler { id: statusHover }
        QQC2.ToolTip.text: text
        QQC2.ToolTip.visible: statusHover.hovered && truncated
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
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
        visible: header.controller.loaded
        text: header.texts.t("menu.file")
        KanteMenuItem { objectName: "new"; action: actNew; iconRoom: true }
        KanteMenuItem { objectName: "open"; action: actOpen; iconRoom: true }
        QQC2.MenuSeparator { visible: header.controller.loaded }
        KanteMenuItem { objectName: "project"; action: actProject; iconRoom: true; visible: header.controller.loaded }
        KanteMenuItem { objectName: "save"; action: actSave; iconRoom: true; visible: header.controller.loaded }
        KanteMenuItem { objectName: "close"; action: actClose; iconRoom: true; visible: header.controller.loaded }
        QQC2.MenuSeparator { visible: header.controller.loaded }
        KanteMenuItem { objectName: "background"; action: actBackground; iconRoom: true; visible: header.controller.loaded }
        KanteMenuItem {
            objectName: "backgroundRemove"
            action: actBackgroundRemove
            iconRoom: true
            visible: header.controller.loaded && header.controller.hasBackground
        }
        QQC2.MenuSeparator { visible: header.controller.loaded }
        KanteMenuItem { objectName: "scriptImport"; action: actScriptImport; iconRoom: true; visible: header.controller.loaded }
    }
    HeaderMenu {
        objectName: "exportMenu"
        visible: header.controller.loaded
        text: header.texts.t("menu.export")
        KanteMenuItem { objectName: "pdf"; action: exportCommands.pdf; iconRoom: true }
        KanteMenuItem { objectName: "png"; action: exportCommands.png; iconRoom: true }
        KanteMenuItem { objectName: "svg"; action: exportCommands.svg; iconRoom: true }
        KanteMenuItem { objectName: "csv"; action: exportCommands.csv; iconRoom: true }
        KanteMenuItem { objectName: "ods"; action: exportCommands.ods; iconRoom: true }
        KanteMenuItem { objectName: "xlsx"; action: exportCommands.xlsx; iconRoom: true }
        KanteMenuItem { objectName: "frames"; action: exportCommands.frames; iconRoom: true }
        KanteMenuItem { objectName: "view3d"; action: exportCommands.view3d; iconRoom: true }
        KanteMenuItem { objectName: "camera"; action: exportCommands.camera; iconRoom: true }
        KanteMenuItem { objectName: "video"; action: exportCommands.video; iconRoom: true }
        QQC2.MenuSeparator {}
        KanteMenuItem { objectName: "shotPages"; action: pageExports.shotPages; iconRoom: true }
        KanteMenuItem { objectName: "cameraPages"; action: pageExports.cameraPages; iconRoom: true }
        KanteMenuItem { objectName: "shotFiles"; action: pageExports.shotFiles; iconRoom: true }
        KanteMenuItem { objectName: "cameraFiles"; action: pageExports.cameraFiles; iconRoom: true }
        KanteMenuSection { text: header.texts.t("menu.reports") }
        KanteMenuItem { objectName: "reportScenes"; action: reportCommands.scenes; iconRoom: true }
        KanteMenuItem { objectName: "reportPlaces"; action: reportCommands.places; iconRoom: true }
        KanteMenuItem { objectName: "reportCast"; action: reportCommands.cast; iconRoom: true }
        KanteMenuItem { objectName: "reportExtract"; action: reportCommands.extract; iconRoom: true }
    }
    HeaderMenu {
        id: viewMenu
        objectName: "viewMenu"
        visible: header.controller.loaded
        text: header.texts.t("menu.view")
        // The saved views (LH-119) come first: the section's label is item 0, the views follow it.
        KanteMenuSection { text: header.texts.t("menu.viewSaved"); rule: false }
        Instantiator {
            model: viewCommands.savedViews
            delegate: KanteMenuItem {
                required property var modelData
                objectName: "savedView." + modelData.id
                text: modelData.name
                enabled: !header.locked
                onTriggered: header.controller.applyView(modelData.id)
            }
            onObjectAdded: (index, object) => viewMenu.menu.insertItem(index + 1, object)
            onObjectRemoved: (index, object) => viewMenu.menu.removeItem(object)
        }
        KanteMenuItem { objectName: "saveView"; action: viewCommands.saveAction }
        KanteMenuItem { objectName: "removeView"; action: viewCommands.removeAction }
        KanteMenuSection { text: header.texts.t("menu.viewMode") }
        KanteMenuItem { objectName: "free"; action: actFree }
        KanteMenuItem { objectName: "wallCut"; action: actWallCut }
        KanteMenuItem { objectName: "through"; action: actThrough }
        KanteMenuItem { objectName: "heights"; action: actHeights }
        KanteMenuItem { objectName: "sketchClear"; action: viewCommands.clearAction }
        KanteMenuItem { objectName: "cameraViews"; action: shotViews.cameraViewsAction }
        KanteMenuItem { objectName: "shotDescription"; action: shotViews.descriptionAction }
        KanteMenuSection { text: header.texts.t("menu.viewAreas") }
        KanteMenuItem { objectName: "script"; action: actScript }
        KanteMenuItem { objectName: "shot"; action: actShot }
        KanteMenuItem { objectName: "timeline"; action: actTimeline }
        KanteMenuItem { objectName: "scriptShow"; action: jumpCommands.scriptAction }
        KanteMenuItem { objectName: "planShow"; action: jumpCommands.planAction }
        KanteMenuSection { text: header.texts.t("menu.viewZoom") }
        KanteMenuItem { objectName: "zoomIn"; action: zoom.inAction }
        KanteMenuItem { objectName: "zoomOut"; action: zoom.outAction }
        KanteMenuItem { objectName: "zoomFit"; action: zoom.fitAction }
        KanteMenuSection { text: header.texts.t("menu.viewOrder") }
        KanteMenuItem { objectName: "cut"; action: shotViews.cutAction }
        KanteMenuItem { objectName: "shooting"; action: shotViews.shootingAction }
    }

    KanteToolButton {
        objectName: "undo"
        visible: header.controller.loaded
        action: actUndo
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: header.stepTip("undo.tip", header.controller.undoStep, actUndo)
        QQC2.ToolTip.visible: hovered
        QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
    }
    KanteToolButton {
        objectName: "redo"
        visible: header.controller.loaded
        action: actRedo
        display: QQC2.AbstractButton.IconOnly
        QQC2.ToolTip.text: header.stepTip("redo.tip", header.controller.redoStep, actRedo)
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
        id: actBook
        cmd: "book.match"
        group: "file"
        text: header.texts.t("book.command")
        enabled: header.controller.loaded && !header.locked && (header.project.questions || []).length > 0
        onTriggered: header.bookRequested()
    }
    CommandAction {
        id: actSave
        cmd: "project.save"
        group: "file"
        text: header.texts.t("menu.save")
        shortcut: "Ctrl+S"
        keys: header.keyName("Ctrl+S")
        enabled: header.controller.loaded
        onTriggered: {
            header.flushRequested()
            header.controller.save()
        }
    }
    CommandAction {
        id: actClose
        cmd: "project.close"
        group: "file"
        text: header.texts.t("menu.close")
        shortcut: "Ctrl+W"
        keys: header.keyName("Ctrl+W")
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.closeRequested()
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
        id: actScriptImport
        cmd: "script.import"
        group: "file"
        text: header.texts.t("script.import")
        enabled: header.controller.loaded && !header.locked
        onTriggered: header.scriptImportRequested()
    }

    ExportDialog {
        id: exportDialog
        objectName: "exportDialog"
        texts: header.texts
        controller: header.controller
        order: header.shotOrder
        papers: header.project.papers || []
        orientations: header.project.orientations || []
        formats: header.project.formats || []
        reportFormats: header.project.report_formats || []
        projectSize: header.project.paper_size || ""
        projectOrientation: header.project.paper_orientation || ""
    }
    ExportCommands {
        id: exportCommands
        texts: header.texts
        controller: header.controller
        dialog: exportDialog
        keyName: header.keyName
    }

    // LH-87: one page per shot or per camera, its camera strong and the rest faded.
    PageExports {
        id: pageExports
        texts: header.texts
        controller: header.controller
        dialog: exportDialog
    }

    // DB-61: the reports, as the file dialog takes them.
    ReportCommands {
        id: reportCommands
        texts: header.texts
        controller: header.controller
        dialog: exportDialog
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
    // What the 3D view has on, as words (`freeLook`): "cut", "camera". It asks again whenever the
    // 3D picture is made again, and after any change of the session (another plan, undo): those
    // can end the look through a camera without a 3D command.
    readonly property string look: {
        header.controller.freeRevision
        header.controller.pictureRevision
        header.controller.viewFree
        return typeof header.controller.freeLook === "function" ? header.controller.freeLook("") : ""
    }
    // Walls cut at the chest (Y, LH-98): only in 3D.
    CommandAction {
        id: actWallCut
        cmd: "view.wallCut"
        group: "view"
        text: header.texts.t("view.wallCut")
        shortcut: "Y"
        keys: "Y"
        checkable: true
        enabled: header.controller.loaded && !header.locked && header.controller.viewFree === true
        onToggled: header.controller.freeLook("cut")
    }
    // Look through the selected shot's camera (LH-99). No key: Z, the one letter left, means
    // nothing here.
    CommandAction {
        id: actThrough
        cmd: "view.through"
        group: "view"
        text: header.texts.t("view.through")
        checkable: true
        enabled: header.controller.loaded && !header.locked && header.controller.viewFree === true
        // The entry shows what is true: a look that was refused leaves it unchecked.
        onToggled: actThrough.checked = header.controller.freeLook("camera").split(" ").includes("camera")
    }
    Binding {
        target: actWallCut
        property: "checked"
        value: header.look.split(" ").includes("cut")
    }
    Binding {
        target: actThrough
        property: "checked"
        value: header.look.split(" ").includes("camera")
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
    // The commands of an object and of a bound label (LH-114 to LH-116, LH-118).
    ObjectCommands { id: objectCommands; texts: header.texts; controller: header.controller; locked: header.locked; keyName: header.keyName }

    // Saved views (LH-119) and the sketch pen's clear command (LH-120).
    ViewCommands {
        id: viewCommands
        objectName: "viewCommands"
        texts: header.texts
        controller: header.controller
        locked: header.locked
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

    // D-12: Ctrl+1 to Ctrl+N and "Go to" in the palette; the window switches the module.
    ModuleCommands {
        id: moduleCommands
        texts: header.texts
        controller: header.controller
        modules: header.modules
        locked: header.locked
        keyName: header.keyName
        onRequested: id => header.moduleRequested(id)
    }

    ShotViewCommands {
        id: shotViews
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
        cameraViews: header.showCameraViews
        description: header.showShotDescription
        shotOrder: header.shotOrder
        onToggled: (area, shown) => header.viewToggled(area, shown)
        onOrderPicked: order => header.shotOrderPicked(order)
    }

    // Zoom of the plan; the window hands it on.
    ZoomCommands {
        id: zoom
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
        onRequested: how => header.zoomRequested(how)
    }

    NavCommands {
        id: navCommands
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
    }

    JumpCommands {
        id: jumpCommands
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
        onScriptRequested: line => header.scriptRequested(line)
        onPlanRequested: header.planRequested()
    }

    PlayCommands {
        id: playCommands
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
        speechAvailable: header.speechAvailable
        onRequested: header.playRequested()
        onSpeakRequested: header.speakRequested()
    }

    ArmCommands {
        id: armCommands
        texts: header.texts
        controller: header.controller
        locked: header.locked
        keyName: header.keyName
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
        const path = [header.controller.title, sceneText].filter(t => t !== "").join(" › ")
        // Stellprobe goes on with the plan's menu.
        return header.module === "stellprobe" ? path + " ›" : path
    }

    // What the palette lists: the modes, the tools, the actions, the scenes, the plans. Each is
    // listed with the key it has anywhere else.
    readonly property var commands: [].concat(modeCommands(), toolCommands(), actionCommands(entries), viewCommands.palette(), objectCommands.kindCommands(),
                                              sceneCommands(), actionCommands(planMenu.actions), planCommands())

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
        case "kind":
            controller.changeSelected("kind", target)
            return
        case "scene":
            controller.selectScene(target)
            return
        case "plan":
            controller.selectPlan(target)
            return
        case "savedView":
            controller.applyView(target)
            return
        case "removeView":
            controller.removeView(target)
            return
        case "fade":
            controller.cycleFade(target)
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
