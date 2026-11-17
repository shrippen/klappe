import QtQuick
import QtQuick.Controls as QQC2

// The shot views as commands: menu, palette and keys. The small camera pictures beside the
// camera bodies in the plan (Shift+C, LH-113) and the long description as a second line of the
// selected shot in the shot table (Shift+S, LH-112), and the order of the shot list, cut or
// shooting (LH-72). The window keeps the state (`cameraViews`, `description`, `shotOrder`) and
// hands it back; `toggled` names the area ("cameraViews", "shotDescription").
Item {
    id: views

    required property var texts
    required property var controller
    property bool locked: false
    property var keyName: sequence => sequence
    property bool cameraViews: true
    property bool description: false

    property string shotOrder: "cut"

    signal toggled(string area, bool shown)
    signal orderPicked(string order)

    readonly property alias cameraViewsAction: actCameraViews
    readonly property alias descriptionAction: actDescription
    readonly property alias cutAction: actCut
    readonly property alias shootingAction: actShooting
    /// In the order of the palette.
    readonly property var actions: [actCameraViews, actDescription, actCut, actShooting]

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

    QQC2.ActionGroup { id: orderGroup }
    CommandAction {
        id: actCut
        cmd: "order.cut"
        group: "view"
        text: views.texts.t("export.order.cut")
        QQC2.ActionGroup.group: orderGroup
        checkable: true
        checked: views.shotOrder === "cut"
        enabled: views.controller.loaded
        onTriggered: views.orderPicked("cut")
    }
    CommandAction {
        id: actShooting
        cmd: "order.shooting"
        group: "view"
        text: views.texts.t("export.order.shooting")
        QQC2.ActionGroup.group: orderGroup
        checkable: true
        checked: views.shotOrder === "shooting"
        enabled: views.controller.loaded
        onTriggered: views.orderPicked("shooting")
    }
}
