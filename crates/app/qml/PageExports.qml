import QtQuick

// The exports with one page or picture per shot or per camera (LH-87): the actions of the
// export menu and the palette. `dialog` is the export dialog that asks for paper and file.
Item {
    id: exports

    required property var texts
    required property var controller
    required property var dialog

    readonly property alias shotPages: actShotPages
    readonly property alias cameraPages: actCameraPages
    readonly property alias shotFiles: actShotFiles
    readonly property alias cameraFiles: actCameraFiles
    /// In the order of the palette.
    readonly property var actions: [actShotPages, actCameraPages, actShotFiles, actCameraFiles]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actShotPages
        cmd: "export.shotpages"
        group: "export"
        text: exports.texts.t("menu.shotpages")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("shotpages")
    }
    CommandAction {
        id: actCameraPages
        cmd: "export.camerapages"
        group: "export"
        text: exports.texts.t("menu.camerapages")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("camerapages")
    }
    CommandAction {
        id: actShotFiles
        cmd: "export.shotfiles"
        group: "export"
        text: exports.texts.t("menu.shotfiles")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("shotfiles")
    }
    CommandAction {
        id: actCameraFiles
        cmd: "export.camerafiles"
        group: "export"
        text: exports.texts.t("menu.camerafiles")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("camerafiles")
    }
}
