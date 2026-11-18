import QtQuick

// DB-63, the way between the script and the plan as commands: menu, palette and keys. "Show in
// script" takes the selected mark or shot to its line (the scene's head when no line names it);
// "Show plan" takes the script's scene to the plan. The window does the showing.
Item {
    id: jump

    required property var texts
    required property var controller
    property bool locked: false
    property var keyName: sequence => sequence

    /// Show line `line` of the script; -1 for the scene's head.
    signal scriptRequested(int line)
    signal planRequested()

    readonly property alias scriptAction: actScript
    readonly property alias planAction: actPlan
    /// In the order of the palette.
    readonly property var actions: [actScript, actPlan]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actScript
        cmd: "script.show"
        group: "view"
        text: jump.texts.t("script.show")
        shortcut: "Shift+D"
        keys: jump.keyName("Shift+D")
        enabled: jump.controller.loaded && !jump.locked
        onTriggered: jump.scriptRequested(jump.controller.scriptLine())
    }
    CommandAction {
        id: actPlan
        cmd: "script.plan"
        group: "view"
        text: jump.texts.t("script.plan")
        shortcut: "Shift+E"
        keys: jump.keyName("Shift+E")
        enabled: jump.controller.loaded && !jump.locked
        onTriggered: jump.planRequested()
    }
}
