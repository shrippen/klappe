import QtQuick
import QtQuick.Controls as QQC2
import Kante

// A project in the way of the next step (another project, or none): save it first or stay.
// The open one is backed up either way when the next opens; saving spares the question at the
// next start.
Item {
    id: guard

    required property var texts
    required property var controller
    readonly property alias dialog: dialog
    readonly property bool asking: dialog.opened || dialog.visible

    property var pending: null

    /// Runs `action` at once if nothing is unsaved, else after the user saved; `saveKey` is the
    /// key of the button that saves and goes on.
    function run(action, saveKey) {
        if (!controller.loaded || !controller.dirty) {
            action()
            return
        }
        pending = action
        dialog.acceptText = texts.t(saveKey)
        dialog.open()
    }

    KanteDialog {
        id: dialog
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        title: guard.texts.t("menu.save")
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        rejectText: guard.texts.t("form.cancel")
        onAccepted: {
            if (guard.controller.save()) {
                guard.pending()
            }
        }
        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: guard.texts.t("project.switchAsk")
        }
    }
}
