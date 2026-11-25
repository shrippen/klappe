import QtQuick
import QtQuick.Controls as QQC2
import QtQml
import org.kde.kirigami as Kirigami
import Kante

// The menu of a figure's look in the scene sheet: the character's looks (a tick on the one
// worn), "No look", then "New look …" and "Edit looks …", which only ask (the sheet answers).
//
//  Wärter · Ölzeug, nass ▾
//    ✓ Ölzeug, nass
//      Anzug
//      Kein Look
//    ──────────────
//      Neuer Look …
//      Looks bearbeiten …
QQC2.Menu {
    id: menu

    required property var texts
    required property var controller
    /// The scene on view and the figure asked about, `{ id, name, look }`.
    property string scene: ""
    property var figure: null
    /// The project's looks, `{ id, character, name }`.
    property var looks: []

    readonly property var own: figure ? looks.filter(l => l.character === figure.id) : []

    /// "New look …" was chosen for the figure.
    signal newRequested(var figure)
    /// "Edit looks …" was chosen for the figure.
    signal editRequested(var figure)

    // Opens below `item` for `figure`.
    function openFor(figure, item) {
        menu.figure = figure
        popup(item, 0, item.height)
    }

    KantePopupSkin { popup: menu }

    Instantiator {
        model: menu.own
        delegate: KanteMenuItem {
            required property var modelData
            objectName: "look." + modelData.id
            text: modelData.name
            checkable: true
            readonly property bool worn: menu.figure !== null && menu.figure.look === modelData.id
            checked: worn
            // A click toggles the tick; the tick is the look worn, so the binding goes back.
            onTriggered: {
                menu.controller.setSceneLook(menu.scene, menu.figure.id, modelData.id)
                checked = Qt.binding(() => worn)
            }
        }
        onObjectAdded: (index, object) => menu.insertItem(index, object)
        onObjectRemoved: (index, object) => menu.removeItem(object)
    }

    KanteMenuItem {
        objectName: "noLook"
        text: menu.texts.t("looks.none")
        enabled: menu.figure !== null && menu.figure.look !== null
        onTriggered: menu.controller.setSceneLook(menu.scene, menu.figure.id, "")
    }
    QQC2.MenuSeparator {}
    KanteMenuItem {
        objectName: "newLook"
        text: menu.texts.t("looks.new")
        onTriggered: menu.newRequested(menu.figure)
    }
    KanteMenuItem {
        objectName: "editLooks"
        text: menu.texts.t("looks.edit")
        onTriggered: menu.editRequested(menu.figure)
    }
}
