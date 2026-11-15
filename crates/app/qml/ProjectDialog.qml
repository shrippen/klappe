import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The project (LH-01 to LH-03, D-08): scenes, locations and characters.
//
//   ┌ Szenen │ Drehorte │ Figuren ┬───────────────────────────┐
//   │ 12 INT. SÄNGERSAAL – TAG    │ Szene bearbeiten          │
//   │ 13 EXT. HOF                 │ Nummer [ 12             ] │
//   │                             │ Szenenkopf [ INT. …     ] │
//   │ [ Neu… ]                    │ [ Übernehmen ] [ Löschen… ]│
//   └─────────────────────────────┴───────────────────────────┘
//
// The list on the left picks, the form on the right changes: no dialog opens over this one,
// and a question (a change that alters the script, a delete) stands in the form. A dialog and
// not a fourth column of the main window (E-14): the work area keeps its room, and the lists
// are used in short visits. What the service refuses (a location with scenes, a character in
// a plan) shows as its message in the form.
Item {
    id: root

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson
        ? JSON.parse(controller.projectJson)
        : { scenes: [], locations: [], characters: [] }

    // The kinds, in the order of the tabs.
    readonly property var kinds: ["scene", "location", "character"]

    // The form is for a new entry, not for the selected one.
    property bool adding: false

    function open() {
        dialog.open()
    }

    // Opens on the scene or the character `id` of `kind`, from a card in the script.
    function openAt(kind, id) {
        dialog.open()
        tabs.currentIndex = kinds.indexOf(kind)
        listOf(kind).currentId = id
        adding = false
    }

    function listOf(kind) {
        return kind === "scene" ? sceneList : kind === "location" ? locationList : characterList
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
            root.adding = false
            sceneList.currentId = root.controller.sceneId
            locationList.currentId = ""
            characterList.currentId = ""
            tabs.currentIndex = 0
        }

        RowLayout {
            width: parent ? parent.width : implicitWidth
            spacing: Kirigami.Units.largeSpacing

            ColumnLayout {
                Layout.preferredWidth: Kirigami.Units.gridUnit * 18
                Layout.maximumWidth: Kirigami.Units.gridUnit * 18
                Layout.alignment: Qt.AlignTop
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
                    onActivated: index => {
                        currentIndex = index
                        root.adding = false
                    }
                }

                StackLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Kirigami.Units.gridUnit * 20
                    currentIndex: tabs.currentIndex

                    ManageList {
                        id: sceneList
                        objectName: "sceneList"
                        texts: root.texts
                        kind: "scene"
                        items: root.project.scenes
                        onAddRequested: root.add(sceneList, sceneForm)
                        onCurrentIdChanged: root.adding = false
                    }
                    ManageList {
                        id: locationList
                        objectName: "locationList"
                        texts: root.texts
                        kind: "location"
                        items: root.project.locations
                        onAddRequested: root.add(locationList, locationForm)
                        onCurrentIdChanged: root.adding = false
                    }
                    ManageList {
                        id: characterList
                        objectName: "characterList"
                        texts: root.texts
                        kind: "character"
                        items: root.project.characters
                        onAddRequested: root.add(characterList, characterForm)
                        onCurrentIdChanged: root.adding = false
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop
                spacing: Kirigami.Units.smallSpacing

                QQC2.Label {
                    objectName: "pick"
                    Layout.fillWidth: true
                    visible: !root.adding && root.listOf(root.kinds[tabs.currentIndex]).current === null
                    wrapMode: Text.Wrap
                    color: KanteStyle.mutedTextColor
                    text: root.texts.t("project.pick")
                }

                SceneEditor {
                    id: sceneForm
                    objectName: "sceneForm"
                    Layout.fillWidth: true
                    visible: tabs.currentIndex === 0 && (root.adding || sceneList.current !== null)
                    texts: root.texts
                    controller: root.controller
                    locations: root.project.locations
                    item: root.adding ? null : sceneList.current
                    onDone: root.finished(sceneList)
                    onShowRequested: id => {
                        if (root.controller.selectScene(id)) {
                            dialog.close()
                        }
                    }
                }
                LocationEditor {
                    id: locationForm
                    objectName: "locationForm"
                    Layout.fillWidth: true
                    visible: tabs.currentIndex === 1 && (root.adding || locationList.current !== null)
                    texts: root.texts
                    controller: root.controller
                    item: root.adding ? null : locationList.current
                    onDone: root.finished(locationList)
                }
                CharacterEditor {
                    id: characterForm
                    objectName: "characterForm"
                    Layout.fillWidth: true
                    visible: tabs.currentIndex === 2 && (root.adding || characterList.current !== null)
                    texts: root.texts
                    controller: root.controller
                    characters: root.project.characters
                    item: root.adding ? null : characterList.current
                    onDone: root.finished(characterList)
                }
            }
        }
    }

    // A form for a new entry; the selection steps aside.
    function add(list, form) {
        list.currentId = ""
        adding = true
        form.load()
        form.reset()
    }

    // A change or a delete went through: the selection that is gone ends, the form for a new
    // entry closes. An edited entry stays selected, shown with what was saved.
    function finished(list) {
        adding = false
        if (list.current === null) {
            list.currentId = ""
        }
    }
}
