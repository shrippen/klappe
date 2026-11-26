import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import Kante

// Locks the book as a version (DB-40): a name, and the note that this saves the project and
// locks the scene numbers. An empty or used name is refused by the session; its reason stands
// under the field and the dialog stays open.
FormDialog {
    id: form

    /// The reason the last try was refused, shown at the field.
    property string error: ""
    readonly property alias nameField: nameField

    /// The book is locked.
    signal locked()

    title: texts.t("version.lockTitle")
    acceptText: texts.t("version.lockConfirm")

    function show() {
        nameField.text = ""
        error = ""
        open()
    }

    onOpened: nameField.forceActiveFocus()

    submit: function () {
        error = controller.lockVersion(nameField.text)
        if (error !== "") {
            return false
        }
        locked()
        return true
    }

    KanteSectionLabel { text: form.texts.t("version.lockName") }
    KanteTextField {
        id: nameField
        objectName: "versionName"
        Layout.fillWidth: true
        invalid: form.error !== ""
        Accessible.name: form.texts.t("version.lockName")
        onTextChanged: form.error = ""
        onAccepted: form.accept()
    }
    QQC2.Label {
        objectName: "versionError"
        Layout.fillWidth: true
        visible: form.error !== ""
        wrapMode: Text.WordWrap
        color: KanteStyle.negativeTextColor
        text: form.error
    }
    KanteCallout {
        objectName: "versionHint"
        Layout.fillWidth: true
        kind: KanteCallout.Kind.Info
        text: form.texts.t("version.lockHint")
    }
}
