import QtQuick

// The reports (DB-61): the actions of the export menu and the palette. `dialog` is the
// export dialog that asks for the file.
Item {
    id: reports

    required property var texts
    required property var controller
    required property var dialog

    readonly property alias scenes: actScenes
    readonly property alias places: actPlaces
    readonly property alias cast: actCast
    readonly property alias extract: actExtract
    readonly property alias contacts: actContacts
    /// In the order of the palette.
    readonly property var actions: [actScenes, actPlaces, actCast, actExtract, actContacts]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actScenes
        cmd: "report.szenen"
        group: "export"
        text: reports.texts.t("report.szenen")
        enabled: reports.controller.loaded
        onTriggered: reports.dialog.requestReport("szenen")
    }
    CommandAction {
        id: actPlaces
        cmd: "report.spielorte"
        group: "export"
        text: reports.texts.t("report.spielorte")
        enabled: reports.controller.loaded
        onTriggered: reports.dialog.requestReport("spielorte")
    }
    CommandAction {
        id: actCast
        cmd: "report.figuren"
        group: "export"
        text: reports.texts.t("report.figuren")
        enabled: reports.controller.loaded
        onTriggered: reports.dialog.requestReport("figuren")
    }
    CommandAction {
        id: actExtract
        cmd: "report.auszug"
        group: "export"
        text: reports.texts.t("report.auszug")
        enabled: reports.controller.loaded
        onTriggered: reports.dialog.requestReport("auszug")
    }
    CommandAction {
        id: actContacts
        cmd: "report.kontakte"
        group: "export"
        text: reports.texts.t("report.kontakte")
        enabled: reports.controller.loaded
        onTriggered: reports.dialog.requestReport("kontakte")
    }
}
