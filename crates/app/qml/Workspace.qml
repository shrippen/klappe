import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtCore
import org.kde.kirigami as Kirigami
import Kante
import "ProjectData.js" as ProjectData

// D-09 (planung/DESIGN.md): the work area under the tool bar, in three steps by its width.
//
//  Wide (>= 1400)         Medium (1000-1400)       Narrow (< 1000)
//  ┌────┬─────────┬────┐   ┌───────────────────┐    ┌ Plan │ Drehbuch │ Shot ┐
//  │Skr.│  Plan   │Shot│   │ Drehbuch   Shot   │    │ one of the three      │
//  ├────┴─────────┴────┤   │       Plan        │    ├───────────────────────┤
//  │     Zeitleiste    │   ├───────────────────┤    │       Zeitleiste      │
//  └───────────────────┘   │     Zeitleiste    │    └───────────────────────┘
//                          └───────────────────┘
// Wide: script and shot beside the plan, each with a split handle (drag sets the width, a double
// click folds it); the widths are kept in the settings. Medium: both are drawers over the plan,
// opened by the buttons above it or the View menu. Narrow: the plan, script and shot are tabs.
Item {
    id: work

    enum Tier {
        Wide,
        Medium,
        Narrow
    }

    required property var texts
    required property var controller
    /// Dialogue is read aloud while playing (LH-64) by `voice` (SpeechVoice), the window's choice.
    property var voice: null
    property bool speak: false
    signal speakToggled(bool on)

    // The scene on view as the script's head shows it: { id, number, heading, inBook } or null.
    readonly property var sceneOnView: {
        if (!controller.projectJson || !controller.sceneId) {
            return null
        }
        return ProjectData.of(controller.projectJson).scenes.find(s => s.id === controller.sceneId) || null
    }

    // The script's cards ask to change a scene (`kind` "scene") or a character in the project dialog.
    signal projectEntryRequested(string kind, string id)
    /// A scene without a location asks for a new one, named `name`, for the scripted location `place` or, when that is empty, for itself (D-21).
    signal newLocationRequested(string name, string place)
    /// The script's empty column asks to load a screenplay or to open the screenplay view (D-17).
    signal scriptImportRequested()
    signal screenplayRequested()
    /// The tool bar's tool.
    property string tool: "select"
    /// What the tool's next click does (PlanTools `hint`).
    property string toolHint: ""
    /// The plan asks for the select tool (Escape).
    signal selectToolRequested()
    /// The shot on view in the shot panel.
    property var shot: null
    property bool showTimeline: true
    /// The camera pictures in the plan (LH-113) and the long description in the shot table (LH-112).
    property bool cameraViews: true
    property bool shotDescription: false
    /// The backup question is open: the work area dims and is locked.
    property bool locked: false
    /// Wide: script and shot beside the plan can be hidden (View menu).
    property bool showScript: true
    property bool showShot: true
    /// What is on view besides the plan in Medium (drawer) and Narrow (tab): "plan", "script", "shot".
    property string area: "plan"

    readonly property int wideMin: 1400
    readonly property int mediumMin: 1000
    /// The width the step is judged by: the window's (D-09), which is more than this item's
    /// when a module bar stands beside it.
    property real tierWidth: width
    readonly property int tier: tierWidth >= wideMin ? Workspace.Tier.Wide
                              : tierWidth >= mediumMin ? Workspace.Tier.Medium : Workspace.Tier.Narrow
    readonly property real paneMin: Kirigami.Units.gridUnit * 12
    readonly property real paneMax: Kirigami.Units.gridUnit * 40
    readonly property real paneDefault: Kirigami.Units.gridUnit * 20
    /// The plan keeps at least this much beside the panes.
    readonly property real planMin: Kirigami.Units.gridUnit * 24
    readonly property var _tabs: ["plan", "script", "shot"]
    readonly property var _leftTabs: ["scenes", "text"]
    /// What the left column is called in a drawer button or a tab: after the tab it shows.
    readonly property string leftName: texts.t(leftTab === "scenes" ? "layout.tabScenes" : "toggle.script")

    /// What the View menu shows as on: the pane beside the plan, the open drawer, the tab.
    readonly property bool scriptShown: tier === Workspace.Tier.Wide ? showScript && !scriptFolded : area === "script"
    readonly property bool shotShown: tier === Workspace.Tier.Wide ? showShot && !shotFolded : area === "shot"
    /// Widths in px, as the handles set them; `folded` panes show only their handle.
    property alias scriptWidth: store.scriptWidth
    property alias shotWidth: store.shotWidth
    property alias scriptFolded: store.scriptFolded
    property alias shotFolded: store.shotFolded
    /// The left column's tab, remembered (D-14): "scenes" or "text".
    property alias leftTab: store.leftTab
    /// The shot list's view and order, remembered (D-05).
    property alias shotView: store.shotView
    property alias shotOrder: store.shotOrder
    /// The timeline shows all tracks, not only the cut order and the selection's (D-45), remembered.
    property alias allTracks: store.allTracks

    /// The selected object's card takes the height (LH-93).
    function focusHeight() {
        plan.focusHeight()
    }

    /// Zooms the plan: "in", "out" or "fit".
    function zoomPlan(how) {
        plan.zoomBy(how)
    }

    function togglePlay() {
        timeline.togglePlay()
    }

    /// The View menu's toggle for `name` ("script" or "shot"): what each step does with it.
    function setArea(name, shown) {
        if (tier === Workspace.Tier.Wide) {
            work[name === "script" ? "showScript" : "showShot"] = shown
            // Showing it from the menu also opens a pane folded by its handle.
            if (shown) {
                work[name === "script" ? "scriptFolded" : "shotFolded"] = false
            }
            return
        }
        area = shown ? name : "plan"
    }

    /// DB-63: the script on view with line `line` shown and focused (-1: the scene's head).
    function revealScript(line) {
        setArea("script", true)
        leftTab = "text"
        // The pane of a narrow window or a drawer is made by the step above.
        Qt.callLater(() => {
            if (!_script) {
                return
            }
            line < 0 ? _script.showHead() : _script.showLine(line)
        })
    }

    /// DB-63: the plan on view with the focus on it.
    function revealPlan() {
        if (tier !== Workspace.Tier.Wide) {
            area = "plan"
        }
        plan.focusCanvas()
    }

    /// The script panel on screen, for `revealScript`.
    property var _script: null

    function _limit(other) {
        return Math.max(paneMin, Math.min(paneMax, width - other - planMin - scriptHandle.width - shotHandle.width))
    }

    // The remembered widths may not fit this window: the panes are cut to leave the plan its
    // room (the script first, the shot takes what is left); the remembered values stay.
    readonly property real _shotMin: showShot && !shotFolded ? paneMin : 0
    readonly property real scriptApplied: Math.min(scriptWidth, _limit(_shotMin))
    readonly property real shotApplied: Math.min(shotWidth, _limit(_scriptTaken))
    readonly property real _scriptTaken: showScript && !scriptFolded ? scriptApplied : 0
    readonly property real _shotTaken: showShot && !shotFolded ? shotApplied : 0

    onTierChanged: area = "plan"

    // The left column (D-14): tabs "Szenen" (the project's scene list, D-13) and "Text" (the
    // scene's script). Both stay alive, so the script can be shown from a jump (`revealScript`).
    Component {
        id: scriptPane
        ColumnLayout {
            spacing: 0

            KanteTabBar {
                objectName: "leftTabs"
                Layout.fillWidth: true
                accessibleName: work.texts.t("layout.leftTabs")
                model: [work.texts.t("layout.tabScenes"), work.texts.t("layout.tabText")]
                currentIndex: work.leftTab === "scenes" ? 0 : 1
                onActivated: index => work.leftTab = work._leftTabs[index]
            }
            SceneList {
                visible: work.leftTab === "scenes"
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.margins: Kirigami.Units.smallSpacing
                texts: work.texts
                controller: work.controller
            }
            ScriptPanel {
                id: scriptPanel
                visible: work.leftTab === "text"
                Layout.fillWidth: true
                Layout.fillHeight: true
                texts: work.texts
                controller: work.controller
                scene: work.sceneOnView
                onEditRequested: (kind, id) => work.projectEntryRequested(kind, id)
                lines: work.controller.scriptJson ? JSON.parse(work.controller.scriptJson) : []
                onChipClicked: (kind, target, index) => work.controller.showLink(kind, target, index)
                onPlanRequested: work.revealPlan()
                onImportRequested: work.scriptImportRequested()
                onScreenplayRequested: work.screenplayRequested()
                Component.onCompleted: work._script = scriptPanel
                Component.onDestruction: if (work._script === scriptPanel) work._script = null
            }
        }
    }
    Component {
        id: shotPane
        ShotPanel {
            texts: work.texts
            controller: work.controller
            shot: work.shot
            view: work.shotView
            order: work.shotOrder
            descriptionOpen: work.shotDescription
            onViewPicked: v => work.shotView = v
            onOrderPicked: o => work.shotOrder = o
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // The body: its parts depend on the step.
        Item {
            id: body
            Layout.fillWidth: true
            Layout.fillHeight: true
            enabled: !work.locked
            opacity: enabled ? 1 : KanteStyle.dimOpacity

            // Wide: the panes beside the plan.
            RowLayout {
                anchors.fill: parent
                spacing: 0

                Loader {
                    objectName: "scriptSlot"
                    active: work.tier === Workspace.Tier.Wide
                    visible: active && work.showScript && !work.scriptFolded
                    Layout.preferredWidth: work.scriptApplied
                    Layout.fillHeight: true
                    sourceComponent: scriptPane
                }
                KanteSplitHandle {
                    id: scriptHandle
                    objectName: "scriptHandle"
                    visible: work.tier === Workspace.Tier.Wide && work.showScript
                    Layout.fillHeight: true
                    orientation: Qt.Horizontal
                    accessibleName: work.texts.t("layout.splitScript")
                    from: work.paneMin
                    to: work._limit(work._shotTaken)
                    value: work.scriptApplied
                    defaultValue: work.paneDefault
                    collapsible: true
                    collapsed: work.scriptFolded
                    onMoved: v => { work.scriptWidth = v; work.scriptFolded = false }
                    onCollapseToggled: folded => work.scriptFolded = folded
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.minimumWidth: 0
                    spacing: 0

                    // Medium: the drawers' buttons; Narrow: the tabs.
                    RowLayout {
                        id: drawerButtons
                        objectName: "drawerButtons"
                        visible: work.tier === Workspace.Tier.Medium
                        Layout.fillWidth: true
                        Layout.margins: Kirigami.Units.smallSpacing
                        spacing: Kirigami.Units.smallSpacing

                        KanteToolButton {
                            objectName: "scriptButton"
                            text: work.leftName
                            checkable: true
                            checked: work.area === "script"
                            onClicked: work.setArea("script", checked)
                        }
                        Item { Layout.fillWidth: true }
                        KanteToolButton {
                            objectName: "shotButton"
                            text: work.texts.t("toggle.shot")
                            checkable: true
                            checked: work.area === "shot"
                            onClicked: work.setArea("shot", checked)
                        }
                    }
                    KanteTabBar {
                        id: tabs
                        objectName: "tabs"
                        visible: work.tier === Workspace.Tier.Narrow
                        Layout.fillWidth: true
                        accessibleName: work.texts.t("layout.areas")
                        model: [work.texts.t("layout.plan"), work.leftName, work.texts.t("toggle.shot")]
                        currentIndex: work._tabs.indexOf(work.area)
                        onActivated: index => work.area = work._tabs[index]
                    }

                    // The plan stays alive behind the other tabs: it keeps its view.
                    PlanArea {
                        id: plan
                        objectName: "plan"
                        visible: work.tier !== Workspace.Tier.Narrow || work.area === "plan"
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.minimumWidth: 0
                        tool: work.tool
                        cameraViews: work.cameraViews
                        toolHint: work.toolHint
                        onSelectToolRequested: work.selectToolRequested()
                        onNewLocationRequested: (name, place) => work.newLocationRequested(name, place)
                        texts: work.texts
                        controller: work.controller
                    }
                    Loader {
                        objectName: "tabScript"
                        active: work.tier === Workspace.Tier.Narrow && work.area === "script"
                        visible: active
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        sourceComponent: scriptPane
                    }
                    Loader {
                        objectName: "tabShot"
                        active: work.tier === Workspace.Tier.Narrow && work.area === "shot"
                        visible: active
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        sourceComponent: shotPane
                    }
                }

                KanteSplitHandle {
                    id: shotHandle
                    objectName: "shotHandle"
                    visible: work.tier === Workspace.Tier.Wide && work.showShot
                    Layout.fillHeight: true
                    orientation: Qt.Horizontal
                    inverted: true
                    accessibleName: work.texts.t("layout.splitShot")
                    from: work.paneMin
                    to: work._limit(work._scriptTaken)
                    value: work.shotApplied
                    defaultValue: work.paneDefault
                    collapsible: true
                    collapsed: work.shotFolded
                    onMoved: v => { work.shotWidth = v; work.shotFolded = false }
                    onCollapseToggled: folded => work.shotFolded = folded
                }
                Loader {
                    objectName: "shotSlot"
                    active: work.tier === Workspace.Tier.Wide
                    visible: active && work.showShot && !work.shotFolded
                    Layout.preferredWidth: work.shotApplied
                    Layout.fillHeight: true
                    sourceComponent: shotPane
                }
            }

            // Medium: drawers over the plan.
            QQC2.Drawer {
                id: scriptDrawer
                objectName: "scriptDrawer"
                parent: body
                edge: Qt.LeftEdge
                width: Math.min(work.paneDefault, body.width)
                // Below the buttons' row, so the button that opened it stays in view.
                y: drawerButtons.height + drawerButtons.Layout.margins * 2
                height: body.height - y
                modal: false
                interactive: false
                closePolicy: QQC2.Popup.CloseOnEscape | QQC2.Popup.CloseOnPressOutside
                visible: work.tier === Workspace.Tier.Medium && work.area === "script"
                onClosed: if (work.area === "script") work.area = "plan"

                KanteSheetSkin { popup: scriptDrawer; drawer: true }
                contentItem: Loader {
                    active: work.tier === Workspace.Tier.Medium
                    sourceComponent: scriptPane
                }
            }
            QQC2.Drawer {
                id: shotDrawer
                objectName: "shotDrawer"
                parent: body
                edge: Qt.RightEdge
                width: Math.min(work.paneDefault, body.width)
                // Below the buttons' row, so the button that opened it stays in view.
                y: drawerButtons.height + drawerButtons.Layout.margins * 2
                height: body.height - y
                modal: false
                interactive: false
                closePolicy: QQC2.Popup.CloseOnEscape | QQC2.Popup.CloseOnPressOutside
                visible: work.tier === Workspace.Tier.Medium && work.area === "shot"
                onClosed: if (work.area === "shot") work.area = "plan"

                KanteSheetSkin { popup: shotDrawer; drawer: true }
                contentItem: Loader {
                    active: work.tier === Workspace.Tier.Medium
                    sourceComponent: shotPane
                }
            }
        }

        Kirigami.Separator { Layout.fillWidth: true; visible: work.showTimeline }
        Timeline {
            id: timeline
            objectName: "timeline"
            enabled: !work.locked
            opacity: enabled ? 1 : KanteStyle.dimOpacity
            Layout.fillWidth: true
            Layout.preferredHeight: implicitHeight
            Layout.maximumHeight: work.height / 2
            visible: work.showTimeline
            texts: work.texts
            controller: work.controller
            voice: work.voice
            speak: work.speak
            allTracks: work.allTracks
            onSpeakToggled: on => work.speakToggled(on)
            onAllTracksToggled: on => work.allTracks = on
        }
    }

    // The widths and folds the handles set stay over a restart, like the working mode.
    Settings {
        id: store
        category: "layout"
        property real scriptWidth: Kirigami.Units.gridUnit * 20
        property real shotWidth: Kirigami.Units.gridUnit * 20
        property bool scriptFolded: false
        property bool shotFolded: false
        property string leftTab: "text"
        // The shot list's view ("tiles", "table") and order ("cut", "shooting"), D-05.
        property string shotView: "tiles"
        property string shotOrder: "cut"
        property bool allTracks: false
    }
}
