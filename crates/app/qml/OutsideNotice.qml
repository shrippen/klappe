import QtQuick
import Kante

// Over the plan: a file changed outside Klappe (the terminal, Git) and in the window. The
// window keeps its version until the user settles it: reload drops the window's changes,
// overwrite saves them over the files. Stays while the conflict is open.
KanteCallout {
    id: notice

    property var texts
    property var controller

    visible: controller.loaded && controller.outsideFiles !== ""
    kind: KanteCallout.Kind.Warn
    text: texts.t("outside.both", { files: controller.outsideFiles })

    KanteButton {
        objectName: "reload"
        text: notice.texts.t("outside.reload")
        onClicked: notice.controller.reloadProject()
    }
    KanteButton {
        objectName: "overwrite"
        text: notice.texts.t("outside.overwrite")
        onClicked: notice.controller.saveOverwrite()
    }
}
