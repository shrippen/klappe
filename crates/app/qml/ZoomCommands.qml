import QtQuick

// Zoom of the plan as commands: menu, palette and keys. Ctrl+= is the same key as Ctrl++
// without Shift. The window does the zooming (`requested`: "in", "out" or "fit").
Item {
    id: zoomCommands

    required property var texts
    required property var controller
    property bool locked: false
    /// Names a key as the language does ("Ctrl++" -> "Strg++").
    property var keyName: sequence => sequence

    signal requested(string how)

    readonly property alias inAction: actZoomIn
    readonly property alias outAction: actZoomOut
    readonly property alias fitAction: actZoomFit
    /// In the order of the palette.
    readonly property var actions: [actZoomIn, actZoomOut, actZoomFit]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actZoomIn
        cmd: "view.zoomIn"
        group: "view"
        text: zoomCommands.texts.t("menu.zoomIn")
        shortcut: "Ctrl++"
        keys: zoomCommands.keyName("Ctrl++")
        enabled: zoomCommands.controller.loaded && !zoomCommands.locked
        onTriggered: zoomCommands.requested("in")
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
        text: zoomCommands.texts.t("menu.zoomOut")
        shortcut: "Ctrl+-"
        keys: zoomCommands.keyName("Ctrl+-")
        enabled: zoomCommands.controller.loaded && !zoomCommands.locked
        onTriggered: zoomCommands.requested("out")
    }
    CommandAction {
        id: actZoomFit
        cmd: "view.zoomFit"
        group: "view"
        text: zoomCommands.texts.t("menu.zoomFit")
        shortcut: "Ctrl+0"
        keys: zoomCommands.keyName("Ctrl+0")
        // The whole plan: the 3D view has no plan to fit.
        enabled: zoomCommands.controller.loaded && !zoomCommands.locked && !zoomCommands.controller.viewFree
        onTriggered: zoomCommands.requested("fit")
    }
}
