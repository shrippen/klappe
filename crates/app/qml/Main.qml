import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import QtQuick.Dialogs
import io.github.shrippen.klappe

// E-14 (planung/KONZEPT.md): header and tool bar on top, script left, plan in the middle,
// shot right, timeline below. Script, shot and timeline fold away.
//
//  ┌ Klappe · Wartburg · Szene 12 …            Rückgängig  Speichern  PDF  CSV ┐
//  ├ Auswahl Wand Objekt Figur Kamera Licht │ Drehbuch Shot Zeitleiste    ┤
//  │ Drehbuch │            Plan             │ Shot 12.3                   │
//  ├──────────┴─────────────────────────────┴─────────────────────────────┤
//  │ Zeitleiste: Schnittfolge, Figuren, Kameras, Licht                    │
//  └──────────────────────────────────────────────────────────────────────┘
Kirigami.ApplicationWindow {
    id: root

    title: texts.t("app.name") + (ctl.loaded ? " – " + ctl.title : "")
    width: Kirigami.Units.gridUnit * 80
    height: Kirigami.Units.gridUnit * 50
    minimumWidth: Kirigami.Units.gridUnit * 40
    minimumHeight: Kirigami.Units.gridUnit * 26
    pageStack.globalToolBar.style: Kirigami.ApplicationHeaderStyle.None
    color: KanteStyle.backgroundColor

    property bool showScript: true
    property bool showShot: true
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

            // Header: where we are, and what applies to the whole project.
            RowLayout {
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                spacing: Kirigami.Units.largeSpacing

                KanteHeading {
                    text: texts.t("app.name")
                    level: 2
                }
                KanteSectionLabel {
                    text: texts.t("app.module")
                }
                Kirigami.Heading {
                    Layout.fillWidth: true
                    level: 4
                    elide: Text.ElideRight
                    color: KanteStyle.mutedTextColor
                    text: ctl.loaded ? ctl.title + " · " + texts.t("app.scene", { number: ctl.planTitle }) : ""
                }
                Kirigami.Heading {
                    level: 5
                    color: KanteStyle.mutedTextColor
                    text: ctl.status !== "" ? ctl.status : (ctl.dirty ? texts.t("menu.unsaved") : "")
                    visible: text !== ""
                }
                KanteButton {
                    text: texts.t("menu.undo")
                    enabled: ctl.canUndo
                    onClicked: ctl.undo()
                }
                KanteButton {
                    text: texts.t("menu.redo")
                    enabled: ctl.canRedo
                    onClicked: ctl.redo()
                }
                KanteButton {
                    text: texts.t("menu.save")
                    enabled: ctl.loaded
                    onClicked: ctl.save()
                }
                KanteSegmented {
                    objectName: "shotOrderChooser"
                    enabled: ctl.loaded
                    model: root.shotOrders.map(o => texts.t("export.order." + o))
                    currentIndex: root.shotOrders.indexOf(root.shotOrder)
                    onActivated: index => root.shotOrder = root.shotOrders[index]
                }
                KanteButton {
                    objectName: "backgroundButton"
                    text: texts.t("menu.background")
                    enabled: ctl.loaded
                    onClicked: backgroundDialog.open()
                }
                KanteButton {
                    objectName: "removeBackgroundButton"
                    text: texts.t("menu.backgroundRemove")
                    visible: ctl.hasBackground
                    onClicked: ctl.removeBackground()
                }
                KanteButton {
                    text: texts.t("menu.pdf")
                    enabled: ctl.loaded
                    onClicked: ctl.exportPdf(root.shotOrder)
                }
                KanteButton {
                    text: texts.t("menu.png")
                    enabled: ctl.loaded
                    onClicked: ctl.exportPng()
                }
                KanteButton {
                    text: texts.t("menu.svg")
                    enabled: ctl.loaded
                    onClicked: ctl.exportSvg()
                }
                KanteButton {
                    text: texts.t("menu.csv")
                    enabled: ctl.loaded
                    onClicked: ctl.exportCsv(root.shotOrder)
                }
            }

            // Tool bar: the drawing tools, then what to show.
            RowLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Kirigami.Units.smallSpacing
                Layout.rightMargin: Kirigami.Units.smallSpacing
                spacing: Kirigami.Units.smallSpacing

                // Locked while the backup question is open (its buttons dim themselves); the
                // view toggles stay usable.
                PlanTools {
                    id: planTools
                    enabled: !backupNotice.blocking
                    texts: texts
                    controller: ctl
                }
                Item { Layout.fillWidth: true }
                KanteToolButton {
                    text: texts.t("toggle.script")
                    checkable: true
                    checked: root.showScript
                    onToggled: root.showScript = checked
                }
                KanteToolButton {
                    text: texts.t("toggle.shot")
                    checkable: true
                    checked: root.showShot
                    onToggled: root.showShot = checked
                }
                KanteToolButton {
                    text: texts.t("toggle.timeline")
                    checkable: true
                    checked: root.showTimeline
                    onToggled: root.showTimeline = checked
                }
            }

            Kirigami.Separator { Layout.fillWidth: true }

            BackupNotice {
                id: backupNotice
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.smallSpacing
                texts: texts
                controller: ctl
            }

            RowLayout {
                enabled: !backupNotice.blocking
                opacity: enabled ? 1 : KanteStyle.dimOpacity
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                ScriptPanel {
                    Layout.preferredWidth: Kirigami.Units.gridUnit * 20
                    Layout.fillHeight: true
                    visible: root.showScript && ctl.loaded
                    texts: texts
                    lines: ctl.scriptJson ? JSON.parse(ctl.scriptJson) : []
                    onChipClicked: (kind, target, at) => kind === "shot" ? ctl.selectShot(target) : ctl.seek(at)
                }
                Kirigami.Separator { Layout.fillHeight: true; visible: root.showScript && ctl.loaded }

                PlanArea {
                    tool: planTools.tool
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    texts: texts
                    controller: ctl
                }

                Kirigami.Separator { Layout.fillHeight: true; visible: root.showShot && ctl.loaded }
                ShotPanel {
                    Layout.preferredWidth: Kirigami.Units.gridUnit * 20
                    Layout.fillHeight: true
                    visible: root.showShot && ctl.loaded
                    texts: texts
                    controller: ctl
                    shot: root.selected
                }
            }

            Kirigami.Separator { Layout.fillWidth: true; visible: root.showTimeline && ctl.loaded }
            Timeline {
                enabled: !backupNotice.blocking
                opacity: enabled ? 1 : KanteStyle.dimOpacity
                Layout.fillWidth: true
                Layout.preferredHeight: Kirigami.Units.gridUnit * 16
                visible: root.showTimeline && ctl.loaded
                texts: texts
                controller: ctl
            }
        }
    }

    FileDialog {
        id: backgroundDialog
        title: texts.t("background.pick")
        nameFilters: [texts.t("background.filter")]
        onAccepted: ctl.setBackground(selectedFile.toString())
    }

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
