import QtQuick
import QtQuick.Controls as QQC2
import Kante

// A file changed outside Klappe and the window took it over: says which (the controller words
// it), then goes by itself.
KanteToast {
    id: toast

    required property var controller

    parent: QQC2.Overlay.overlay
    anchors.right: parent ? parent.right : undefined
    anchors.bottom: parent ? parent.bottom : undefined
    anchors.margins: KanteStyle.unit(16)
    kind: KanteToast.Kind.Info

    Connections {
        target: toast.controller
        function onOutsideTaken(text) {
            toast.text = text
            toast.show()
        }
    }
}
