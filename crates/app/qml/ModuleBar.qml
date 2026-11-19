import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-11 (planung/DESIGN.md): the module bar. A narrow bar of icon buttons at the window's left
// edge, one per module, in the order of `modules`; the open module is checked and has a bar at
// its edge, so the mark does not rest on colour alone. The window keeps which module shows.
//
//  ┌──┐
//  │▌D│  Drehbuch   (Ctrl+1)
//  │ S│  Stellprobe (Ctrl+2)
//  └──┘
Rectangle {
    id: bar

    required property var texts
    /// The modules in the order of the bar: `{ id, name, icon }`; `name` is a text key.
    required property var modules
    /// The id of the open module.
    property string module: ""
    /// Names a key sequence as the language of the keyboard does.
    property var keyName: sequence => sequence

    /// The module `id` is asked for.
    signal picked(string id)

    implicitWidth: column.implicitWidth + Kirigami.Units.smallSpacing * 2
    color: KanteStyle.backgroundColor
    Accessible.role: Accessible.ToolBar
    Accessible.name: texts.t("module.bar")

    ColumnLayout {
        id: column
        x: Kirigami.Units.smallSpacing
        y: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        Repeater {
            model: bar.modules

            delegate: KanteToolButton {
                id: button

                required property var modelData
                required property int index
                readonly property bool open: bar.module === modelData.id
                readonly property string moduleName: bar.texts.t(modelData.name)

                objectName: "module." + modelData.id
                // One width, whatever the theme has: without the icon the name is cut to fit.
                implicitWidth: Kirigami.Units.gridUnit * 2
                Layout.preferredWidth: implicitWidth
                Layout.maximumWidth: implicitWidth
                text: moduleName
                icon.name: modelData.icon
                // Kante shows the name when the theme lacks the icon.
                display: QQC2.AbstractButton.IconOnly
                checkable: true
                checked: open
                // The state is the bar's: a click asks, the window answers with `module`.
                onClicked: {
                    checked = Qt.binding(() => button.open)
                    bar.picked(modelData.id)
                }
                Accessible.name: moduleName
                QQC2.ToolTip.text: bar.texts.t("tool.tip", { name: moduleName, key: bar.keyName("Ctrl+" + (index + 1)) })
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay

                // The edge mark of the open module.
                Rectangle {
                    objectName: "edge"
                    visible: button.open
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: Kirigami.Units.smallSpacing / 2
                    color: KanteStyle.focusColor
                }
            }
        }
    }

    Kirigami.Separator {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
    }
}
