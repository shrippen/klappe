import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The project view (LH-01 to LH-03): scenes, locations and characters in three tabs, each
// with add, edit and delete. A dialog and not a fourth column of the main window (E-14): the
// work area keeps its room, and the lists are used in short visits. Deleting asks first;
// what the service refuses (a location with scenes, a character in a plan) shows as its
// message in the dialog.
Item {
    id: root

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson
        ? JSON.parse(controller.projectJson)
        : { scenes: [], locations: [], characters: [] }

    // What a delete asks about: { kind, item } or null.
    property var pending: null

    function open() {
        dialog.open()
    }

    implicitWidth: 0
    implicitHeight: 0

    KanteDialog {
        id: dialog
        objectName: "dialog"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Large
        title: root.texts.t("project.title")
        standardButtons: QQC2.Dialog.Close
        rejectText: root.texts.t("project.close")

        onAboutToShow: {
            root.controller.clearStatus()
            sceneList.currentId = root.controller.sceneId
            tabs.currentIndex = 0
        }

        ColumnLayout {
            width: parent ? parent.width : implicitWidth
            spacing: Kirigami.Units.smallSpacing

            KanteTabBar {
                id: tabs
                objectName: "tabs"
                Layout.fillWidth: true
                model: [
                    root.texts.t("project.scenes"),
                    root.texts.t("project.locations"),
                    root.texts.t("project.characters")
                ]
                counts: [
                    String(root.project.scenes.length),
                    String(root.project.locations.length),
                    String(root.project.characters.length)
                ]
                onActivated: index => currentIndex = index
            }

            KanteCallout {
                objectName: "error"
                Layout.fillWidth: true
                visible: root.controller.status !== ""
                kind: KanteCallout.Kind.Danger
                text: root.controller.status
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.preferredHeight: Kirigami.Units.gridUnit * 18
                currentIndex: tabs.currentIndex

                ManageList {
                    id: sceneList
                    objectName: "sceneList"
                    texts: root.texts
                    kind: "scene"
                    items: root.project.scenes
                    onAddRequested: sceneForm.show(null)
                    onEditRequested: item => sceneForm.show(item)
                    onRemoveRequested: item => root.ask("scene", item)
                    onShowRequested: item => {
                        if (root.controller.selectScene(item.id)) {
                            dialog.close()
                        }
                    }
                }
                ManageList {
                    id: locationList
                    objectName: "locationList"
                    texts: root.texts
                    kind: "location"
                    items: root.project.locations
                    onAddRequested: locationForm.show(null)
                    onEditRequested: item => locationForm.show(item)
                    onRemoveRequested: item => root.ask("location", item)
                }
                ManageList {
                    id: characterList
                    objectName: "characterList"
                    texts: root.texts
                    kind: "character"
                    items: root.project.characters
                    onAddRequested: characterForm.show(null)
                    onEditRequested: item => characterForm.show(item)
                    onRemoveRequested: item => root.ask("character", item)
                }
            }
        }
    }

    function ask(kind, item) {
        root.controller.clearStatus()
        pending = { kind: kind, item: item }
        confirm.open()
    }

    SceneForm {
        id: sceneForm
        objectName: "sceneForm"
        texts: root.texts
        controller: root.controller
        locations: root.project.locations
    }
    LocationForm {
        id: locationForm
        objectName: "locationForm"
        texts: root.texts
        controller: root.controller
    }
    CharacterForm {
        id: characterForm
        objectName: "characterForm"
        texts: root.texts
        controller: root.controller
        characters: root.project.characters
    }

    KanteDialog {
        id: confirm
        objectName: "confirm"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        destructive: true
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: root.texts.t("remove.confirm")
        rejectText: root.texts.t("form.cancel")
        title: root.pending ? root.texts.t("project.remove") : ""

        QQC2.Label {
            objectName: "question"
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: {
                if (!root.pending) {
                    return ""
                }
                const item = root.pending.item
                const name = root.pending.kind === "scene" ? item.number + " " + item.heading : item.name
                return root.texts.t("remove." + root.pending.kind, { name: name.trim() })
            }
        }

        onAccepted: {
            const item = root.pending.item
            switch (root.pending.kind) {
            case "scene": root.controller.removeScene(item.id); break
            case "location": root.controller.removeLocation(item.id); break
            case "character": root.controller.removeCharacter(item.id); break
            }
        }
    }
}
