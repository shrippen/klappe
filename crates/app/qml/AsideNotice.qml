import QtQuick
import Kante

// Over the plan: the example's old copy no longer fitted the format and was set aside. The
// note names the whole path and stays until the user closes it (the controller empties it when
// another project opens); the header's status line is too short for a path.
KanteCallout {
    id: notice

    property var controller

    visible: controller.exampleAside !== ""
    kind: KanteCallout.Kind.Info
    dismissible: true
    // The binding on `visible` decides; hiding by hand would cut it for a later note.
    autoHide: false
    text: controller.exampleAside
    onDismissed: controller.exampleAside = ""
}
