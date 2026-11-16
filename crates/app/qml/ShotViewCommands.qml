import QtQuick

// Two view switches as commands: menu, palette and keys. The small camera pictures beside the
// camera bodies in the plan (Shift+C, LH-113) and the long description as a second line of the
// selected shot in the shot table (Shift+S, LH-112). The window keeps the state (`cameraViews`,
// `description`) and hands it back; `toggled` names the area ("cameraViews", "shotDescription").
Item {
    id: views

    required property var texts
    required property var controller
    property bool locked: false
    property var keyName: sequence => sequence
    property bool cameraViews: true
    property bool description: false

    signal toggled(string area, bool shown)

    readonly property alias cameraViewsAction: actCameraViews
    readonly property alias descriptionAction: actDescription
    /// In the order of the palette.
    readonly property var actions: [actCameraViews, actDescription]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actCameraViews
        cmd: "view.cameraViews"
        group: "view"
        text: views.texts.t("view.cameraViews")
        shortcut: "Shift+C"
        keys: views.keyName("Shift+C")
        checkable: true
        checked: views.cameraViews
        enabled: views.controller.loaded && !views.locked
        onToggled: views.toggled("cameraViews", checked)
    }
    CommandAction {
        id: actDescription
        cmd: "view.shotDescription"
        group: "view"
        text: views.texts.t("view.shotDescription")
        shortcut: "Shift+S"
        keys: views.keyName("Shift+S")
        checkable: true
        checked: views.description
        enabled: views.controller.loaded && !views.locked
        onToggled: views.toggled("shotDescription", checked)
    }
}
