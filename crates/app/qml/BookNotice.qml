import QtQuick
import Kante

// Over the plan: entries of the register wait for the answer which scene of the book they are.
// Stays while questions are open; the button opens the dialog.
KanteCallout {
    id: notice

    property var texts
    property var controller
    readonly property var pending: controller.projectJson ? (JSON.parse(controller.projectJson).pending || []) : []

    signal matchRequested()

    visible: controller.loaded && pending.length > 0
    kind: KanteCallout.Kind.Info
    text: texts.t("book.pending", { count: pending.length })

    KanteButton {
        objectName: "match"
        text: notice.texts.t("book.match")
        onClicked: notice.matchRequested()
    }
}
