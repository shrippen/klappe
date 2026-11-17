import QtQuick

// What the selected plan element offers by key and palette: arm the next click (LH-104, LH-105,
// LH-109) and the shape of a way (LH-108).
Item {
    id: arming

    required property var texts
    required property var controller
    property bool locked: false
    property var keyName: sequence => sequence

    /// In the order of the palette.
    readonly property var actions: [actNextMark, actMarkBefore, actAxisTo, actWaySmooth, actWayStraight]

    visible: false
    width: 0
    height: 0

    // The selected figure or camera (LH-104, LH-105, LH-109): the key arms the next click on the
    // plan, no tool is switched; the controller says what the click does and Escape cancels.
    // The selected way (LH-108): its shape.
    readonly property bool planSelected: controller.loaded && !locked && !!controller.selectedKey
    readonly property string wayCurve: {
        controller.pictureRevision
        controller.selectedKey
        return controller.wayCurve ? controller.wayCurve() : ""
    }
    CommandAction {
        id: actNextMark
        cmd: "plan.nextMark"
        group: "edit"
        text: arming.texts.t("arm.nextMark")
        shortcut: "Shift+F"
        keys: arming.keyName("Shift+F")
        enabled: arming.planSelected && arming.controller.canArm("nextMark")
        onTriggered: arming.controller.arm("nextMark")
    }
    CommandAction {
        id: actMarkBefore
        cmd: "plan.markBefore"
        group: "edit"
        text: arming.texts.t("arm.markBefore")
        shortcut: "Shift+Z"
        keys: arming.keyName("Shift+Z")
        enabled: arming.planSelected && arming.controller.canArm("markBefore")
        onTriggered: arming.controller.arm("markBefore")
    }
    CommandAction {
        id: actAxisTo
        cmd: "plan.axisTo"
        group: "edit"
        text: arming.texts.t("arm.axisTo")
        shortcut: "Shift+A"
        keys: arming.keyName("Shift+A")
        enabled: arming.planSelected && arming.controller.canArm("axisTo")
        onTriggered: arming.controller.arm("axisTo")
    }
    CommandAction {
        id: actWaySmooth
        cmd: "plan.waySmooth"
        group: "edit"
        text: arming.texts.t("way.smooth")
        enabled: arming.planSelected && arming.wayCurve === "straight"
        onTriggered: arming.controller.setWayCurve("smooth")
    }
    CommandAction {
        id: actWayStraight
        cmd: "plan.wayStraight"
        group: "edit"
        text: arming.texts.t("way.straight")
        enabled: arming.planSelected && arming.wayCurve === "smooth"
        onTriggered: arming.controller.setWayCurve("straight")
    }
}
