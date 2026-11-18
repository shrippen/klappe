import QtQuick

// The file exports of the export menu and the palette (Ctrl+P and the rest). `dialog` is the
// export dialog that asks for paper and file.
Item {
    id: exports

    required property var texts
    required property var controller
    required property var dialog
    property var keyName: sequence => sequence

    readonly property alias pdf: actPdf
    readonly property alias png: actPng
    readonly property alias svg: actSvg
    readonly property alias csv: actCsv
    readonly property alias ods: actOds
    readonly property alias xlsx: actXlsx
    readonly property alias frames: actFrames
    readonly property alias video: actVideo
    readonly property alias view3d: actView3d
    readonly property alias camera: actCameraPng
    /// In the order of the palette.
    readonly property var actions: [actPdf, actPng, actSvg, actCsv, actOds, actXlsx, actFrames, actView3d, actCameraPng, actVideo]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actPdf
        cmd: "export.pdf"
        group: "export"
        text: exports.texts.t("menu.pdf")
        shortcut: "Ctrl+P"
        keys: exports.keyName("Ctrl+P")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("pdf")
    }
    CommandAction {
        id: actPng
        cmd: "export.png"
        group: "export"
        text: exports.texts.t("menu.png")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("png")
    }
    CommandAction {
        id: actSvg
        cmd: "export.svg"
        group: "export"
        text: exports.texts.t("menu.svg")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("svg")
    }
    CommandAction {
        id: actCsv
        cmd: "export.csv"
        group: "export"
        text: exports.texts.t("menu.csv")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("csv")
    }
    CommandAction {
        id: actOds
        cmd: "export.ods"
        group: "export"
        text: exports.texts.t("menu.ods")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("ods")
    }
    CommandAction {
        id: actXlsx
        cmd: "export.xlsx"
        group: "export"
        text: exports.texts.t("menu.xlsx")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("xlsx")
    }
    CommandAction {
        id: actFrames
        cmd: "export.frames"
        group: "export"
        text: exports.texts.t("menu.frames")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("frames")
    }
    // LH-86: the 3D view as it is on screen (so only in 3D), and the shot card's camera view.
    CommandAction {
        id: actView3d
        cmd: "export.view3d"
        group: "export"
        text: exports.texts.t("menu.view3d")
        enabled: exports.controller.loaded && exports.controller.viewFree === true
        onTriggered: exports.dialog.request("view3d")
    }
    CommandAction {
        id: actCameraPng
        cmd: "export.camera"
        group: "export"
        text: exports.texts.t("menu.camera")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("camera")
    }
    // LH-84: the playback as MP4; the file only, a sheet is no part of a video.
    CommandAction {
        id: actVideo
        cmd: "export.mp4"
        group: "export"
        text: exports.texts.t("menu.mp4")
        enabled: exports.controller.loaded
        onTriggered: exports.dialog.request("mp4")
    }
}
