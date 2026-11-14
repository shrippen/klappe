import QtQuick
import QtQuick.Layouts
import Kante

// Over the plan: a backup of unsaved changes was found at open. The user restores it or
// discards it; the controller then sets backupState to "none" and the note goes.
KanteCallout {
    id: notice

    property var texts
    property var controller

    readonly property bool unreadable: controller.backupState === "unreadable"

    visible: controller.backupState === "found" || unreadable
    kind: KanteCallout.Kind.Warn
    text: unreadable
        ? texts.t("backup.unreadable", { message: controller.backupMessage })
        : texts.t("backup.found")

    KanteButton {
        objectName: "restore"
        visible: !notice.unreadable
        text: notice.texts.t("backup.restore")
        onClicked: notice.controller.restoreBackup()
    }
    KanteButton {
        objectName: "discard"
        text: notice.texts.t("backup.discard")
        onClicked: notice.controller.discardBackup()
    }
}
