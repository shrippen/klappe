import QtQuick
import QtQuick.Controls as QQC2
import Kante

// "Delete …" of a version asks first: its text is lost, no undo step brings it back.
KanteDialog {
    id: confirm

    required property var texts
    required property var controller
    /// The version asked about, `{ id, name, … }`.
    property var version: null

    /// The version was deleted.
    signal removed()

    parent: QQC2.Overlay.overlay
    anchors.centerIn: parent
    modal: true
    size: KanteDialog.Size.Small
    destructive: true
    standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
    acceptText: texts.t("remove.confirm")
    rejectText: texts.t("form.cancel")
    title: texts.t("version.removeTitle")

    function ask(version) {
        confirm.version = version
        open()
    }

    onAccepted: {
        if (version && controller.removeVersion(version.id)) {
            removed()
        }
    }

    QQC2.Label {
        objectName: "question"
        width: parent ? parent.width : implicitWidth
        wrapMode: Text.WordWrap
        color: KanteStyle.textColor
        text: confirm.version ? confirm.texts.t("version.removeAsk", { name: confirm.version.name }) : ""
    }
}
