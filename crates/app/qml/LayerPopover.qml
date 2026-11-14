import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Layers (design D-06): a button over the plan that opens a list, one row per layer with an
// eye (shown) and a lock (a locked layer is drawn but cannot be picked or changed). A dot on
// the button tells that a layer is hidden or locked while the list is closed.
Item {
    id: layers

    required property var texts
    required property var controller

    readonly property var names: ["set", "figures", "cameras", "lights", "notes", "drawings"]
    readonly property var shown: controller.layersJson ? JSON.parse(controller.layersJson) : ({})
    readonly property var locks: controller.locksJson ? JSON.parse(controller.locksJson) : ({})
    readonly property bool unusual: names.some(n => shown[n] === false || locks[n] === true)

    implicitWidth: card.implicitWidth
    implicitHeight: card.implicitHeight

    // A card under the button: the plan's wall runs under it, and a bare button is unreadable there.
    KanteCard {
        id: card
        anchors.fill: parent
        implicitWidth: button.implicitWidth + Kirigami.Units.smallSpacing * 2
        implicitHeight: button.implicitHeight + Kirigami.Units.smallSpacing * 2

        KanteButton {
            id: button
            objectName: "layersButton"
            anchors.centerIn: parent
            text: layers.texts.t("layer.title")
            icon.name: "layer-visible-on"
            Accessible.name: layers.unusual ? layers.texts.t("layer.titleOff") : text
            onClicked: popover.openAt(button)
        }

        KanteLamp {
            objectName: "layersDot"
            visible: layers.unusual
            color: KanteStyle.warningColor
            width: KanteStyle.unit(8)
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: -width / 2
        }
    }

    KantePopover {
        id: popover
        objectName: "layersPopover"
        title: layers.texts.t("layer.title")

        Repeater {
            model: layers.names
            delegate: RowLayout {
                required property string modelData
                readonly property string layerName: layers.texts.t("layer." + modelData)
                spacing: Kirigami.Units.smallSpacing

                KanteSectionLabel {
                    Layout.fillWidth: true
                    Layout.minimumWidth: Kirigami.Units.gridUnit * 6
                    text: layerName
                }
                KanteToolButton {
                    objectName: "layerButton_" + modelData
                    display: QQC2.AbstractButton.IconOnly
                    icon.name: checked ? "view-visible" : "view-hidden"
                    checkable: true
                    checked: layers.shown[modelData] !== false
                    fillChecked: false
                    Accessible.name: layers.texts.t("layer.showTip", { layer: layerName })
                    QQC2.ToolTip.text: Accessible.name
                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                    onToggled: layers.controller.setLayer(modelData, checked ? "shown" : "hidden")
                }
                KanteToolButton {
                    objectName: "lockButton_" + modelData
                    display: QQC2.AbstractButton.IconOnly
                    icon.name: checked ? "object-locked" : "object-unlocked"
                    checkable: true
                    checked: layers.locks[modelData] === true
                    fillChecked: false
                    Accessible.name: layers.texts.t("layer.lockTip", { layer: layerName })
                    QQC2.ToolTip.text: Accessible.name
                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                    onToggled: layers.controller.setLock(modelData, checked ? "locked" : "unlocked")
                }
            }
        }
    }
}
