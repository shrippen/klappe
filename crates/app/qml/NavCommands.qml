import QtQuick

// The keyboard's way to what only the mouse did (NF-06): turning and shifting the 3D view, and
// moving the selected mark or the edges of the selected shot on the timeline by a tenth of a
// second. The palette lists them all. The arrows and + - of the 3D view are keys of FreeView
// while it has the focus (a global arrow key would take the cursor from every text field), so
// their actions show the key but own no shortcut; Alt+Left and Alt+Right move a mark anywhere.
Item {
    id: nav

    required property var texts
    required property var controller
    property bool locked: false
    property var keyName: sequence => sequence

    /// One step of the timeline's fine adjustment, in seconds.
    readonly property real stepS: 0.1

    readonly property bool free: controller.loaded && !locked && controller.viewFree
    readonly property bool markSelected: controller.loaded && !locked && /^mark:/.test(controller.selectedKey)
    readonly property bool shotSelected: controller.loaded && !locked && controller.selectedShot !== ""

    /// In the order of the palette.
    readonly property var actions: [actTurnLeft, actTurnRight, actTiltUp, actTiltDown,
                                    actPanLeft, actPanRight, actPanUp, actPanDown,
                                    actMarkEarlier, actMarkLater,
                                    actFromEarlier, actFromLater, actToEarlier, actToLater]

    visible: false
    width: 0
    height: 0

    component View3d: CommandAction {
        property string step: ""
        group: "view"
        enabled: nav.free
        onTriggered: nav.controller.stepView(step)
    }
    component Edge: CommandAction {
        property string what: ""
        property real direction: 1
        group: "playback"
        onTriggered: nav.controller.nudgeTime(what, direction * nav.stepS)
    }

    View3d { id: actTurnLeft; cmd: "view.turnLeft"; step: "turnLeft"; text: nav.texts.t("view.turnLeft"); keys: "←" }
    View3d { id: actTurnRight; cmd: "view.turnRight"; step: "turnRight"; text: nav.texts.t("view.turnRight"); keys: "→" }
    View3d { id: actTiltUp; cmd: "view.tiltUp"; step: "turnUp"; text: nav.texts.t("view.tiltUp"); keys: "↑" }
    View3d { id: actTiltDown; cmd: "view.tiltDown"; step: "turnDown"; text: nav.texts.t("view.tiltDown"); keys: "↓" }
    View3d { id: actPanLeft; cmd: "view.panLeft"; step: "panLeft"; text: nav.texts.t("view.panLeft"); keys: nav.keyName("Shift+←") }
    View3d { id: actPanRight; cmd: "view.panRight"; step: "panRight"; text: nav.texts.t("view.panRight"); keys: nav.keyName("Shift+→") }
    View3d { id: actPanUp; cmd: "view.panUp"; step: "panUp"; text: nav.texts.t("view.panUp"); keys: nav.keyName("Shift+↑") }
    View3d { id: actPanDown; cmd: "view.panDown"; step: "panDown"; text: nav.texts.t("view.panDown"); keys: nav.keyName("Shift+↓") }

    Edge {
        id: actMarkEarlier
        cmd: "retime.markEarlier"
        what: "mark"
        direction: -1
        text: nav.texts.t("retime.markEarlier")
        shortcut: "Alt+Left"
        keys: nav.keyName("Alt+Left")
        enabled: nav.markSelected
    }
    Edge {
        id: actMarkLater
        cmd: "retime.markLater"
        what: "mark"
        text: nav.texts.t("retime.markLater")
        shortcut: "Alt+Right"
        keys: nav.keyName("Alt+Right")
        enabled: nav.markSelected
    }
    Edge { id: actFromEarlier; cmd: "retime.fromEarlier"; what: "shot-from"; direction: -1; text: nav.texts.t("retime.fromEarlier"); enabled: nav.shotSelected }
    Edge { id: actFromLater; cmd: "retime.fromLater"; what: "shot-from"; text: nav.texts.t("retime.fromLater"); enabled: nav.shotSelected }
    Edge { id: actToEarlier; cmd: "retime.toEarlier"; what: "shot-to"; direction: -1; text: nav.texts.t("retime.toEarlier"); enabled: nav.shotSelected }
    Edge { id: actToLater; cmd: "retime.toLater"; what: "shot-to"; text: nav.texts.t("retime.toLater"); enabled: nav.shotSelected }
}
