import QtQuick

// Zoom of the plan as commands: menu, palette and keys. Ctrl+= is the same key as Ctrl++
// without Shift. The window does the zooming (`requested`: "in", "out" or "fit").
Item {
    id: exports

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
        text: exports.texts.t("menu.zoomIn")
        shortcut: "Ctrl++"
        keys: exports.keyName("Ctrl++")
        enabled: exports.controller.loaded && !exports.locked
        onTriggered: exports.requested("in")
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
        text: exports.texts.t("menu.zoomOut")
        shortcut: "Ctrl+-"
        keys: exports.keyName("Ctrl+-")
        enabled: exports.controller.loaded && !exports.locked
        onTriggered: exports.requested("out")
    }
    CommandAction {
        id: actZoomFit
        cmd: "view.zoomFit"
        group: "view"
        text: exports.texts.t("menu.zoomFit")
        shortcut: "Ctrl+0"
        keys: exports.keyName("Ctrl+0")
        // The whole plan: the 3D view has no plan to fit.
        enabled: exports.controller.loaded && !exports.locked && !exports.controller.viewFree
        onTriggered: exports.requested("fit")
    }
}
