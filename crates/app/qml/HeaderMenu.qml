import QtQuick
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A button in the header that opens a menu below it: `Datei ▾`. The entries (KanteMenuItem,
// KanteMenuSection, QQC2.MenuSeparator) go inside as children. As in a menu bar: while a
// menu beside it is open, the mouse on this button opens this menu and closes that one.
KanteToolButton {
    id: button

    default property alias entries: menu.contentData
    readonly property alias menu: menu

    rightPadding: leftPadding + arrow.width + Kirigami.Units.smallSpacing
    hoverEnabled: true
    onClicked: menu.popup(button, 0, button.height)
    onHoveredChanged: takeOver()

    // The open menu of a sibling HeaderMenu passes to this one.
    function takeOver() {
        if (!hovered || menu.visible || !parent) {
            return
        }
        const open = parent.children.find(c => c !== button && c.menu instanceof QQC2.Menu && c.menu.visible)
        if (!open) {
            return
        }

        open.menu.close()
        menu.popup(button, 0, button.height)
    }

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
