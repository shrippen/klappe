import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A button in the header that opens a menu below it: `Datei ▾`. The entries (KanteMenuItem,
// KanteMenuSection, QQC2.MenuSeparator) go inside as children.
KanteToolButton {
    id: button

    default property alias entries: menu.contentData
    readonly property alias menu: menu

    rightPadding: leftPadding + arrow.width + Kirigami.Units.smallSpacing
    onClicked: menu.popup(button, 0, button.height)

    KanteIcon {
        id: arrow
        anchors.right: parent.right
        anchors.rightMargin: button.leftPadding
        anchors.verticalCenter: parent.verticalCenter
        width: Kirigami.Units.iconSizes.small
        height: width
        iconSource: "arrow-down"
        color: KanteStyle.mutedTextColor
    }

    QQC2.Menu {
        id: menu
        KantePopupSkin { popup: menu }
    }
}
