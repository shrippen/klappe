import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The project (LH-01 to LH-03, D-08, D-21): scenes, scripted locations, locations and
// characters.
//
//   ┌ Szenen │ Spielorte │ Drehorte │ Figuren ┬─────────────────┐
//   │ 12 INT. SÄNGERSAAL – TAG               │ Szene bearbeiten │
//   │ 13 EXT. HOF                            │ Nummer [ 12    ] │
//   │                                        │ Szenenkopf [ …  ] │
//   │ [ Neu… ]                               │ [ Übernehmen ]   │
//   └────────────────────────────────────────┴──────────────────┘
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

    readonly property var numberings: project.numberings || []

    readonly property var places: project.places || []

    readonly property var project: controller.projectJson
        ? JSON.parse(controller.projectJson)
        : { scenes: [], places: [], locations: [], characters: [], numberings: [], numbering: "" }

    // The kinds, in the order of the tabs.
    readonly property var kinds: ["scene", "place", "location", "character"]

    // The dialog is wider than Kante's Large step (48 units): the four tabs with their counts
    // stand whole in the left column and the form keeps its room.
    readonly property real dialogUnits: 64

    // A location made from the plan's empty state (D-21) goes to this scripted location, or to
    // the scene on view when `""`.
    property bool assigning: false
    property string assignPlace: ""

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

    // Opens the form for a new location named `name`; once it is added, it becomes the
    // location of the scripted location `place`, or of the scene on view when `place` is empty.
    function openNewLocation(name, place) {
        dialog.open()
        tabs.currentIndex = kinds.indexOf("location")
        add(locationList, locationForm)
        locationForm.preset(name)
        assigning = true
        assignPlace = place
    }

    // Gives the location `id` that was just added to what asked for it, and closes the dialog.
    function assignNew(id) {
        assigning = false
        // An edit answers true, not the id of a new location.
        if (typeof id !== "string") {
            return
        }
        const place = places.find(p => p.id === assignPlace)
        const done = place
            ? controller.editPlace(place.id, place.name, JSON.stringify(place.variants), id)
            : controller.shootSceneAt(id)
        if (done) {
            dialog.close()
        }
    }

    function listOf(kind) {
        return [sceneList, placeList, locationList, characterList][kinds.indexOf(kind)]
    }

    implicitWidth: 0
    implicitHeight: 0

    KanteDialog {
        id: dialog
        objectName: "dialog"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        width: Math.min(Kirigami.Units.gridUnit * root.dialogUnits, QQC2.Overlay.overlay.width * 0.9)
        title: root.texts.t("project.title")
        standardButtons: QQC2.Dialog.Close
        rejectText: root.texts.t("project.close")

        onAboutToShow: {
            root.controller.clearStatus()
            root.adding = false
            root.assigning = false
            sceneList.currentId = root.controller.sceneId
            placeList.currentId = ""
            locationList.currentId = ""
            characterList.currentId = ""
            tabs.currentIndex = 0
        }

        ColumnLayout {
            width: parent ? parent.width : implicitWidth
            spacing: Kirigami.Units.largeSpacing

            RowLayout {
                Layout.fillWidth: true
                spacing: Kirigami.Units.largeSpacing

                ColumnLayout {
                    Layout.preferredWidth: tabs.implicitWidth
                    Layout.minimumWidth: tabs.implicitWidth
                    Layout.maximumWidth: tabs.implicitWidth
                    Layout.alignment: Qt.AlignTop
                    spacing: Kirigami.Units.smallSpacing

                    KanteTabBar {
                        id: tabs
                        objectName: "tabs"
                        Layout.fillWidth: true
                        model: [
                            root.texts.t("project.scenes"),
                            root.texts.t("project.places"),
                            root.texts.t("project.locations"),
                            root.texts.t("project.characters")
                        ]
                        counts: [
                            String(root.project.scenes.length),
                            String(root.places.length),
                            String(root.project.locations.length),
                            String(root.project.characters.length)
                        ]
                        onActivated: index => {
                            currentIndex = index
                            root.adding = false
                            root.assigning = false
                        }
                    }

                    StackLayout {
                        Layout.fillWidth: true
                        // Grows when the names under the characters need more room.
                        Layout.preferredHeight: Math.max(Kirigami.Units.gridUnit * 20,
                            characterColumn.implicitHeight)
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
                            id: placeList
                            objectName: "placeList"
                            texts: root.texts
                            kind: "place"
                            items: root.places
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
                        // The characters, and under them the names that are none (DB-50).
                        ColumnLayout {
                            id: characterColumn
                            spacing: Kirigami.Units.largeSpacing

                            ManageList {
                                id: characterList
                                objectName: "characterList"
                                Layout.fillWidth: true
                                Layout.fillHeight: true
                                texts: root.texts
                                kind: "character"
                                items: root.project.characters
                                onAddRequested: root.add(characterList, characterForm)
                                onCurrentIdChanged: root.adding = false
                            }
                            NotFigures {
                                objectName: "notFigures"
                                Layout.fillWidth: true
                                texts: root.texts
                                controller: root.controller
                                names: root.project.not_figures || []
                            }
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
                        places: root.places
                        item: root.adding ? null : sceneList.current
                        onDone: root.finished(sceneList)
                        onShowRequested: id => {
                            if (root.controller.selectScene(id)) {
                                dialog.close()
                            }
                        }
                    }
                    PlaceEditor {
                        id: placeForm
                        objectName: "placeForm"
                        Layout.fillWidth: true
                        visible: tabs.currentIndex === 1 && placeList.current !== null
                        texts: root.texts
                        controller: root.controller
                        locations: root.project.locations
                        item: placeList.current
                        onDone: root.finished(placeList)
                    }
                    LocationEditor {
                        id: locationForm
                        objectName: "locationForm"
                        Layout.fillWidth: true
                        visible: tabs.currentIndex === 2 && (root.adding || locationList.current !== null)
                        texts: root.texts
                        controller: root.controller
                        item: root.adding ? null : locationList.current
                        onDone: result => {
                            root.finished(locationList)
                            if (root.assigning) {
                                root.assignNew(result)
                            }
                        }
                    }
                    CharacterEditor {
                        id: characterForm
                        objectName: "characterForm"
                        Layout.fillWidth: true
                        visible: tabs.currentIndex === 3 && (root.adding || characterList.current !== null)
                        texts: root.texts
                        controller: root.controller
                        characters: root.project.characters
                        item: root.adding ? null : characterList.current
                        onDone: root.finished(characterList)
                    }
                }
            }

            // How new shots are numbered (LH-76): one scheme per project.
            ColumnLayout {
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing

                KanteSectionLabel { text: root.texts.t("project.numbering") }
                KanteComboBox {
                    id: numberingBox
                    objectName: "numberingBox"
                    Layout.fillWidth: true
                    Accessible.name: root.texts.t("project.numbering")
                    textRole: "text"
                    valueRole: "value"
                    model: root.numberings.map(n => ({ value: n, text: root.texts.t("numbering." + n) }))
                    currentIndex: root.numberings.indexOf(root.project.numbering)
                    onActivated: {
                        if (!root.controller.setNumbering(String(currentValue))) {
                            currentIndex = root.numberings.indexOf(root.project.numbering)
                        }
                    }
                }
                QQC2.Label {
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    color: KanteStyle.mutedTextColor
                    text: root.texts.t("project.numberingHint")
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
