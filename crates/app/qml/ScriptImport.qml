import QtQuick
import QtQuick.Controls as QQC2
import QtQuick.Dialogs
import Kante

// "Load screenplay …": the file dialog for a Fountain file. A project that has a screenplay
// is asked first; the file replaces the text, and nothing is saved by it.
Item {
    id: imp

    required property var texts
    required property var controller
    readonly property alias dialog: ask
    readonly property alias picker: picker

    /// Asks (only for a project with a screenplay), then offers the file dialog.
    function start() {
        if (controller.scriptText.trim() === "") {
            picker.open()
            return
        }
        ask.open()
    }

    KanteDialog {
        id: ask
        objectName: "importAsk"
        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        title: imp.texts.t("script.import")
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: imp.texts.t("script.importReplace")
        rejectText: imp.texts.t("form.cancel")
        onAccepted: picker.open()
        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: imp.texts.t("script.importAsk")
        }
    }

    FileDialog {
        id: picker
        onVisibleChanged: {
            if (!visible && Window.window) {
                Window.window.requestActivate()
            }
        }
        title: imp.texts.t("script.importPick")
        nameFilters: [imp.texts.t("script.filter")]
        onAccepted: imp.controller.importScript(selectedFile.toString())
    }
}
