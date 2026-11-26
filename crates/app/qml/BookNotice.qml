import QtQuick
import Kante
import "ProjectData.js" as ProjectData

// Over the plan: entries of the register wait for the answer which scene of the book they are,
// names of the cues for the character they belong to, and place names of the scene headings for
// the scripted location they belong to.
// Stays while questions are open; the button opens the dialog.
KanteCallout {
    id: notice

    property var texts
    property var controller
    readonly property var project: ProjectData.of(controller.projectJson)
    readonly property var questions: project.questions || []
    readonly property var pending: questions.filter(q => q.kind === "scene")
    readonly property var figures: questions.filter(q => q.kind === "figure")
    readonly property var places: questions.filter(q => q.kind === "place")

    signal matchRequested()

    visible: controller.loaded && pending.length + figures.length + places.length > 0
    kind: KanteCallout.Kind.Info
    text: [
        pending.length > 0 ? texts.t("book.pending", { count: pending.length }) : "",
        figures.length > 0 ? texts.t("book.figures", { count: figures.length }) : "",
        places.length > 0 ? texts.t("book.places", { count: places.length }) : ""
    ].filter(t => t !== "").join(" ")

    KanteButton {
        objectName: "match"
        text: notice.texts.t("book.match")
        onClicked: notice.matchRequested()
    }
}
