import QtQuick

// DB-40, DB-42: "Lock version …" and "Compare …" of the screenplay's Fassung menu as palette
// commands. The screenplay view asks for the dialogs; they work only while it is on view and
// holds a text.
Item {
    id: versions

    required property var texts
    required property var controller
    property bool locked: false
    /// The open module's id.
    property string module: ""

    /// "lock" or "compare" is asked for.
    signal requested(string what)

    readonly property alias lockAction: actLock
    readonly property alias compareAction: actCompare
    /// In the order of the palette.
    readonly property var actions: [actLock, actCompare]

    readonly property bool available: controller.loaded && !locked && module === "screenplay"
        && controller.scriptText !== ""

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actLock
        cmd: "version.lock"
        group: "version"
        text: versions.texts.t("version.lock")
        enabled: versions.available
        onTriggered: versions.requested("lock")
    }
    CommandAction {
        id: actCompare
        cmd: "version.compare"
        group: "version"
        text: versions.texts.t("version.compare")
        enabled: versions.available
        onTriggered: versions.requested("compare")
    }
}
