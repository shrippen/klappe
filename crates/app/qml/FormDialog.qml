import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A dialog with fields that sends them to the controller on accept. `submit` returns false
// when the controller refused (its message is in `controller.status`): the dialog then stays
// open and shows the message, so nothing typed is lost.
KanteDialog {
    id: form

    required property var texts
    required property var controller
    // Called on accept; true when the change was made.
    property var submit: function () { return true }
    // The fields; the message of a refusal stands above them.
    default property alias fields: column.data

    parent: QQC2.Overlay.overlay
    anchors.centerIn: parent
    modal: true
    size: KanteDialog.Size.Small
    standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
    rejectText: texts.t("form.cancel")

    // A refusal reopens the dialog; its message must survive that.
    property bool refused: false

    onAboutToShow: {
        if (!refused) {
            controller.clearStatus()
        }
        refused = false
    }
    onAccepted: {
        if (submit()) {
            return
        }
        refused = true
        Qt.callLater(open)
    }

    ColumnLayout {
        id: column
        width: parent ? parent.width : implicitWidth
        spacing: Kirigami.Units.smallSpacing

        KanteCallout {
            objectName: "error"
            Layout.fillWidth: true
            visible: form.controller.status !== ""
            kind: KanteCallout.Kind.Danger
            text: form.controller.status
        }
    }
}
