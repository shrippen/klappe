import QtQuick
import Kante

// Over the plan: entries of the register wait for the answer which scene of the book they are, and
// place names of the scene headings for the scripted location they belong to.
// Stays while questions are open; the button opens the dialog.
KanteCallout {
    id: notice

    property var texts
    property var controller
    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : {}
    readonly property var pending: project.pending || []
    readonly property var places: project.pendingPlaces || []

    signal matchRequested()

    visible: controller.loaded && pending.length + places.length > 0
    kind: KanteCallout.Kind.Info
    text: [
        pending.length > 0 ? texts.t("book.pending", { count: pending.length }) : "",
        places.length > 0 ? texts.t("book.places", { count: places.length }) : ""
    ].filter(t => t !== "").join(" ")

    KanteButton {
        objectName: "match"
        text: notice.texts.t("book.match")
        onClicked: notice.matchRequested()
    }
}
