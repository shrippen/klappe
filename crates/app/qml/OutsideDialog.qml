import QtQuick
import QtQuick.Controls as QQC2
import Kante

// A save found a file changed outside and in the window and wrote nothing: reload (drop the
// window's changes), overwrite (the window's version goes to the disk) or cancel.
Item {
    id: ask

    required property var texts
    required property var controller
    readonly property alias dialog: dialog

    function open() {
        dialog.open()
    }

    KanteDialog {
        id: dialog
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        destructive: true
        title: ask.texts.t("outside.dialogTitle")
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: ask.texts.t("outside.overwrite")
        rejectText: ask.texts.t("form.cancel")
        footerStart: [
            KanteButton {
                objectName: "reload"
                text: ask.texts.t("outside.reloadShort")
                onClicked: {
                    dialog.close()
                    ask.controller.reloadProject()
                }
            }
        ]
        onAccepted: ask.controller.saveOverwrite()
        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: ask.texts.t("outside.dialogAsk", { files: ask.controller.outsideFiles })
        }
    }
}
