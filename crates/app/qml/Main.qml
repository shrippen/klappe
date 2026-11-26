import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import QtQuick.Dialogs
import QtCore
import io.github.shrippen.klappe

// E-14 and D-02, D-03 (planung/DESIGN.md): header on top, tool bar below, script left, plan in
// the middle, shot right, timeline below. Script, shot and timeline fold away (View menu). The
// work area (Workspace.qml) changes with the window's width (D-09).
//
//  ┌ KLAPPE  Projekt › Szene 12 › Plan A ▾    Datei  Ausgeben  Ansicht  ↶ ↷ ⌕ ┐
//  ├ Auswahl Wand Objekt Figur Kamera Licht …                 Einrasten        ┤
//  │ Drehbuch │            Plan             │ Shot 12.3                        │
//  ├──────────┴─────────────────────────────┴──────────────────────────────────┤
//  │ Zeitleiste: Schnittfolge, Figuren, Kameras, Licht                         │
//  └───────────────────────────────────────────────────────────────────────────┘
Kirigami.ApplicationWindow {
    id: root

    title: texts.t("app.name") + (ctl.loaded ? " – " + ctl.title : "")
    width: Kirigami.Units.gridUnit * 80
    height: Kirigami.Units.gridUnit * 50
    minimumWidth: Kirigami.Units.gridUnit * 40
    minimumHeight: Kirigami.Units.gridUnit * 26
    pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.None
    color: KanteStyle.backgroundColor

    property bool showTimeline: true
    property bool showCameraViews: true
    property bool showShotDescription: false
    // D-11: the modules of the bar, in its order (D-12: Ctrl+1 first). A new module is one entry
    // here, its view below with `visible: root.module === <id>`, and its texts `module.<id>`.
    readonly property var modules: [
        { id: "screenplay", name: "module.screenplay", icon: "document-edit" },
        { id: "stellprobe", name: "module.stellprobe", icon: "view-split-left-right" },
        { id: "breakdown", name: "module.breakdown", icon: "view-list-details" },
        { id: "casting", name: "module.casting", icon: "view-media-artist" },
        { id: "team", name: "module.team", icon: "system-users" }
    ]
    // The open module; the settings keep it over a restart (an unknown stored id gives Stellprobe).
    readonly property string module: modules.some(m => m.id === windowStore.module) ? windowStore.module : "stellprobe"
    readonly property bool inStellprobe: module === "stellprobe"

    // Typing in the screenplay goes to the session at once, then the module changes.
    function openModule(id) {
        screenplay.flush()
        windowStore.module = id
    }
    // The order of the exported shot list: the film's or the shoot's (LH-72).
    readonly property var shotOrders: ["cut", "shooting"]
    property string shotOrder: "cut"

    Controller { id: ctl }
    SpeechVoice { id: speechVoice }
    // Reading the dialogue aloud stays over a restart (LH-64).
    Settings {
        id: playback
        category: "playback"
        property bool speak: false
    }
    Texts { id: texts; source: ctl.texts }
    // The window's state that stays over a restart, like the layout of the workspace.
    Settings {
        id: windowStore
        category: "window"
        property string module: "stellprobe"
    }

    Binding { target: KanteStyle; property: "kind"; value: KanteStyle.Kind.Kante }
    KanteScope { target: root.contentItem }

    readonly property var shots: ctl.shotsJson ? JSON.parse(ctl.shotsJson) : []
    readonly property var selected: shots.find(s => s.id === ctl.selectedShot) || null

    pageStack.initialPage: Kirigami.Page {
        id: page
        padding: 0
        background: Rectangle { color: KanteStyle.backgroundColor }

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            AppHeader {
                objectName: "header"
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                texts: texts
                controller: ctl
                tools: planTools
                locked: backupNotice.blocking
                showScript: workspace.scriptShown
                showShot: workspace.shotShown
                showTimeline: root.showTimeline
                modules: root.modules
                module: root.module
                showCameraViews: root.showCameraViews
                showShotDescription: root.showShotDescription
                shotOrder: root.shotOrder
                onNewRequested: root.switchProject(() => newFolderDialog.open())
                onOpenRequested: root.switchProject(() => openFolderDialog.open())
                onCloseRequested: guard.run(() => ctl.closeProject(), "project.closeSave")
                onProjectRequested: projectDialog.open()
                onBookRequested: bookDialog.open()
                onTemplateImportRequested: templateImportDialog.open()
                onTemplateExportRequested: templateExportDialog.open()
                onBackgroundRequested: backgroundDialog.open()
                onScriptImportRequested: scriptImport.start()
                onTitlePageRequested: {
                    root.openModule("screenplay")
                    screenplay.showTitlePage()
                }
                onFlushRequested: screenplay.flush()
                onViewToggled: (area, shown) => {
                    switch (area) {
                    case "script":
                    case "shot": workspace.setArea(area, shown); break
                    case "timeline": root.showTimeline = shown; break
                    case "cameraViews": root.showCameraViews = shown; break
                    case "shotDescription": root.showShotDescription = shown; break
                    }
                }
                onShotOrderPicked: order => root.shotOrder = order
                onZoomRequested: how => workspace.zoomPlan(how)
                onPlayRequested: workspace.togglePlay()
                speechAvailable: speechVoice.available
                onSpeakRequested: playback.speak = !playback.speak
                onModuleRequested: id => root.openModule(id)
                onScriptRequested: line => {
                    root.openModule("stellprobe")
                    workspace.revealScript(line)
                }
                onPlanRequested: {
                    root.openModule("stellprobe")
                    workspace.revealPlan()
                }
                onHeightRequested: workspace.focusHeight()
            }

            StartPage {
                Layout.fillWidth: true
                Layout.fillHeight: true
                visible: !ctl.loaded
                texts: texts
                controller: ctl
                onNewRequested: root.switchProject(() => newFolderDialog.open())
                onOpenRequested: root.switchProject(() => openFolderDialog.open())
                onRecentRequested: path => ctl.openProject(path)
                onBlankRequested: ctl.openBlank()
                onExampleRequested: ctl.openExample()
                onForgetRequested: path => ctl.forgetRecent(path)
            }

            // D-11: the module bar at the left edge, the module's own area beside it.
            RowLayout {
                visible: ctl.loaded
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                ModuleBar {
                    Layout.fillHeight: true
                    // Like the keys: no switch while the backup question waits.
                    enabled: !backupNotice.blocking
                    texts: texts
                    modules: root.modules
                    module: root.module
                    keyName: sequence => sequence.replace("Ctrl", texts.t("key.ctrl"))
                    onPicked: id => root.openModule(id)
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 0

                    // Tool bar: the drawing tools.
                    RowLayout {
                        visible: ctl.loaded && root.inStellprobe
                        Layout.fillWidth: true
                        // Its overflow is for the narrow-window step (D-09); it must not widen the header.
                        Layout.minimumWidth: 0
                        Layout.leftMargin: Kirigami.Units.smallSpacing
                        Layout.rightMargin: Kirigami.Units.smallSpacing
                        spacing: Kirigami.Units.smallSpacing

                        // Locked while the backup question is open (its buttons dim themselves); the
                        // view toggles stay usable.
                        PlanTools {
                            id: planTools
                            enabled: !backupNotice.blocking
                            Layout.fillWidth: true
                            Layout.minimumWidth: 0
                            // The options need the full 1400 px of D-09; a module bar beside the work area takes some.
                            compact: workspace.width < workspace.wideMin
                            texts: texts
                            controller: ctl
                            onImportRequested: libraryImportDialog.open()
                            onExportRequested: libraryExportDialog.open()
                        }
                    }

                    Kirigami.Separator { Layout.fillWidth: true; visible: ctl.loaded && root.inStellprobe }

                    AsideNotice {
                        Layout.fillWidth: true
                        Layout.margins: Kirigami.Units.smallSpacing
                        controller: ctl
                    }

                    BookNotice {
                        Layout.fillWidth: true
                        Layout.margins: Kirigami.Units.smallSpacing
                        texts: texts
                        controller: ctl
                        onMatchRequested: bookDialog.open()
                    }

                    OutsideNotice {
                        Layout.fillWidth: true
                        Layout.margins: Kirigami.Units.smallSpacing
                        texts: texts
                        controller: ctl
                    }

                    BackupNotice {
                        id: backupNotice
                        Layout.fillWidth: true
                        Layout.margins: Kirigami.Units.smallSpacing
                        texts: texts
                        controller: ctl
                    }

                    Workspace {
                        id: workspace
                        objectName: "workspace"
                        tierWidth: root.width
                        visible: ctl.loaded && root.inStellprobe
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        texts: texts
                        controller: ctl
                        tool: planTools.tool
                        toolHint: planTools.hint
                        onSelectToolRequested: planTools.tool = "select"
                        shot: root.selected
                        showTimeline: root.showTimeline
                        cameraViews: root.showCameraViews
                        shotDescription: root.showShotDescription
                        locked: backupNotice.blocking
                        voice: speechVoice
                        speak: playback.speak
                        onSpeakToggled: on => playback.speak = on
                        onProjectEntryRequested: (kind, id) => projectDialog.openAt(kind, id)
                        onNewLocationRequested: (name, place) => projectDialog.openNewLocation(name, place)
                        onScriptImportRequested: scriptImport.start()
                        onScreenplayRequested: root.openModule("screenplay")
                    }

                    BreakdownView {
                        objectName: "breakdown"
                        visible: ctl.loaded && root.module === "breakdown"
                        enabled: !backupNotice.blocking
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        texts: texts
                        controller: ctl
                    }

                    CastingView {
                        objectName: "casting"
                        visible: ctl.loaded && root.module === "casting"
                        enabled: !backupNotice.blocking
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        texts: texts
                        controller: ctl
                    }

                    TeamView {
                        objectName: "team"
                        visible: ctl.loaded && root.module === "team"
                        enabled: !backupNotice.blocking
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        texts: texts
                        controller: ctl
                    }

                    ScreenplayView {
                        id: screenplay
                        objectName: "screenplay"
                        visible: ctl.loaded && root.module === "screenplay"
                        enabled: !backupNotice.blocking
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        texts: texts
                        controller: ctl
                        onImportRequested: scriptImport.start()
                    }
                }
            }
        }
    }

    FileDialog {
        id: backgroundDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("background.pick")
        nameFilters: [texts.t("background.filter")]
        onAccepted: ctl.setBackground(selectedFile.toString())
    }

    FileDialog {
        id: libraryImportDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("library.importPick")
        nameFilters: [texts.t("library.filter")]
        onAccepted: ctl.importLibrary(selectedFile.toString(), "objects")
    }
    FileDialog {
        id: libraryExportDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("library.exportPick")
        nameFilters: [texts.t("library.filter")]
        fileMode: FileDialog.SaveFile
        defaultSuffix: "toml"
        onAccepted: ctl.exportLibrary(selectedFile.toString(), "objects")
    }

    FileDialog {
        id: templateImportDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("template.importPick")
        nameFilters: [texts.t("template.filter")]
        onAccepted: ctl.importLibrary(selectedFile.toString(), "templates")
    }
    FileDialog {
        id: templateExportDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("template.exportPick")
        nameFilters: [texts.t("template.filter")]
        fileMode: FileDialog.SaveFile
        defaultSuffix: "toml"
        onAccepted: ctl.exportLibrary(selectedFile.toString(), "templates")
    }

    ScriptImport {
        id: scriptImport
        texts: texts
        controller: ctl
        onQuestionsRequested: bookDialog.open()
    }

    // A file may have changed outside while the window was in the background (the terminal
    // beside it, a Git pull): typing is sent first, then the folder is compared.
    onActiveChanged: {
        if (!active || !ctl.loaded) {
            return
        }
        screenplay.flush()
        ctl.checkOutside()
    }

    OutsideToast { controller: ctl }
    OutsideDialog {
        id: outsideDialog
        texts: texts
        controller: ctl
    }
    Connections {
        target: ctl
        function onOutsideAsked() {
            outsideDialog.open()
        }
    }

    ProjectGuard {
        id: guard
        texts: texts
        controller: ctl
        flush: () => screenplay.flush()
    }

    function switchProject(action) {
        guard.run(action, "project.switchSave")
    }

    FolderDialog {
        id: newFolderDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("project.pickNew")
        onAccepted: newProjectDialog.show(selectedFolder.toString())
    }
    FolderDialog {
        id: openFolderDialog
        onVisibleChanged: if (!visible) root.requestActivate()
        title: texts.t("project.pickOpen")
        onAccepted: ctl.openProject(selectedFolder.toString())
    }
    NewProjectDialog {
        id: newProjectDialog
        texts: texts
        controller: ctl
    }
    ProjectDialog {
        id: projectDialog
        texts: texts
        controller: ctl
    }
    BookDialog {
        id: bookDialog
        texts: texts
        controller: ctl
    }

    // The projects opened last: the app's settings keep them, the controller orders them.
    Settings {
        id: recent
        category: "recent"
        property string projects: "[]"
    }
    Connections {
        target: ctl
        function onRecentStoreChanged() {
            recent.projects = ctl.recentStore
        }
    }
    Component.onCompleted: ctl.loadRecent(recent.projects)

    // Closing keeps the last seconds too: they go to the backup, offered at the next open.
    onClosing: {
        screenplay.flush()
        ctl.autosave()
    }

    // Backs up unsaved changes while a project is open; errors reach the status line.
    Timer {
        interval: ctl.autosaveMs
        running: ctl.loaded
        repeat: true
        onTriggered: ctl.autosave()
    }

    // Tests and release screenshots: --screenshot PATH [--screenshot-after MS], then quit.
    Timer {
        interval: Math.max(100, ctl.screenshotAfter)
        running: ctl.screenshotPath !== ""
        onTriggered: {
            // The window itself has no engine to grab with; the page has (Kontra's lesson).
            page.grabToImage(function(result) {
                result.saveToFile(ctl.screenshotPath)
                Qt.quit()
            })
        }
    }
}
