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
    // The order of the exported shot list: the film's or the shoot's (LH-72).
    readonly property var shotOrders: ["cut", "shooting"]
    property string shotOrder: "cut"

    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }

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
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                texts: texts
                controller: ctl
                tools: planTools
                locked: backupNotice.blocking
                showScript: workspace.scriptShown
                showShot: workspace.shotShown
                showTimeline: root.showTimeline
                shotOrder: root.shotOrder
                onNewRequested: root.switchProject(() => newFolderDialog.open())
                onOpenRequested: root.switchProject(() => openFolderDialog.open())
                onProjectRequested: projectDialog.open()
                onTemplateImportRequested: templateImportDialog.open()
                onTemplateExportRequested: templateExportDialog.open()
                onBackgroundRequested: backgroundDialog.open()
                onViewToggled: (area, shown) => {
                    switch (area) {
                    case "script":
                    case "shot": workspace.setArea(area, shown); break
                    case "timeline": root.showTimeline = shown; break
                    }
                }
                onShotOrderPicked: order => root.shotOrder = order
                onZoomRequested: how => workspace.zoomPlan(how)
                onPlayRequested: workspace.togglePlay()
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
                onExampleRequested: ctl.openExample()
                onForgetRequested: path => ctl.forgetRecent(path)
            }

            // Tool bar: the drawing tools.
            RowLayout {
                visible: ctl.loaded
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
                    compact: workspace.tier !== Workspace.Tier.Wide
                    texts: texts
                    controller: ctl
                    onImportRequested: libraryImportDialog.open()
                    onExportRequested: libraryExportDialog.open()
                }
            }

            Kirigami.Separator { Layout.fillWidth: true; visible: ctl.loaded }

            BackupNotice {
                id: backupNotice
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                texts: texts
                controller: ctl
            }

            Workspace {
                id: workspace
                visible: ctl.loaded
                Layout.fillWidth: true
                Layout.fillHeight: true
                texts: texts
                controller: ctl
                tool: planTools.tool
                shot: root.selected
                showTimeline: root.showTimeline
                locked: backupNotice.blocking
                onProjectEntryRequested: (kind, id) => projectDialog.openAt(kind, id)
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

    // A project in the way of the next one: save it first or stay (the open one is backed
    // up either way when the next opens; saving spares the question at the next start).
    property var pendingSwitch: null

    function switchProject(action) {
        if (!ctl.loaded || !ctl.dirty) {
            action()
            return
        }
        pendingSwitch = action
        switchDialog.open()
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
    KanteDialog {
        id: switchDialog
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        title: texts.t("menu.save")
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: texts.t("project.switchSave")
        rejectText: texts.t("form.cancel")
        onAccepted: {
            if (ctl.save()) {
                root.pendingSwitch()
            }
        }
        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: texts.t("project.switchAsk")
        }
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
    onClosing: ctl.autosave()

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
