import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// One list of the project dialog (scenes, scripted locations, locations or characters) with
// its button for a new entry; the scripted locations have none, the book's questions make them. It holds the selection; the form beside it (D-08) changes or deletes the entry.
ColumnLayout {
    id: list

    required property var texts
    // "scene", "place", "location" or "character".
    required property string kind
    // The rows: the project's own lists (see manage.rs).
    property var items: []
    property string currentId: ""
    readonly property var current: items.find(i => i.id === currentId) || null

    signal addRequested()

    spacing: Kirigami.Units.smallSpacing

    Component {
        id: sceneRow
        KanteListRow {
            required property var modelData
            width: ListView.view.width
            leadingText: modelData.number
            leadingWidth: Kirigami.Units.gridUnit * 3
            text: modelData.heading !== "" || modelData.inBook ? modelData.heading : list.texts.t("scene.notInBook")
            meta: modelData.ownLocation ? "" : modelData.locationName
            // A scene shot elsewhere than its scripted location says so (D-21).
            subtitle: modelData.ownLocation
                ? list.texts.t("scene.ownLocation", { location: modelData.locationName })
                : ""
            selected: modelData.id === list.currentId
            onClicked: list.currentId = modelData.id
        }
    }

    // The scripted location, the location it is shot at and its scenes; one without a location
    // says so in the warning colour (the row's subtitle has no colour of its own).
    Component {
        id: placeRow
        KanteListRow {
            required property var modelData
            readonly property int sceneCount: modelData.scenes.length
            width: ListView.view.width
            text: modelData.name
            subtitle: modelData.location_name === ""
                ? list.texts.t(sceneCount === 1 ? "place.count1" : "place.count", { count: sceneCount })
                : list.texts.t(sceneCount === 1 ? "place.where1" : "place.where",
                    { location: modelData.location_name, count: sceneCount })
            selected: modelData.id === list.currentId
            onClicked: list.currentId = modelData.id

            trailing: Text {
                objectName: "noLocation"
                visible: modelData.location_name === ""
                text: list.texts.t("place.noLocation")
                color: KanteStyle.warningColor
                font: KanteStyle.smallFont
            }
        }
    }

    Component {
        id: locationRow
        KanteListRow {
            required property var modelData
            width: ListView.view.width
            text: modelData.name
            subtitle: list.texts.t("location.size", {
                width: list.texts.num(modelData.width, 2),
                depth: list.texts.num(modelData.depth, 2)
            })
            count: String(modelData.scenes)
            selected: modelData.id === list.currentId
            onClicked: list.currentId = modelData.id
        }
    }

    Component {
        id: characterRow
        KanteListRow {
            required property var modelData
            width: ListView.view.width
            text: modelData.name
            meta: modelData.short
            subtitle: modelData.group > 0 ? list.texts.t("character.groupOf", { count: modelData.group }) : ""
            selected: modelData.id === list.currentId
            onClicked: list.currentId = modelData.id

            KanteSwatch {
                objectName: "swatch"
                swatchColor: KanteStyle.labelColor(modelData.color)
                size: KanteSwatch.Size.Small
            }
        }
    }

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.minimumHeight: Kirigami.Units.gridUnit * 12

        ListView {
            objectName: "rows"
            anchors.fill: parent
            clip: true
            model: list.items
            delegate: list.kind === "scene" ? sceneRow
                : list.kind === "place" ? placeRow
                : list.kind === "location" ? locationRow : characterRow
            QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        }
        KanteEmptyState {
            objectName: "empty"
            anchors.centerIn: parent
            width: parent.width - 2 * Kirigami.Units.largeSpacing
            visible: list.items.length === 0
            text: list.texts.t(list.kind === "scene" ? "project.noScenes"
                : list.kind === "place" ? "project.noPlaces"
                : list.kind === "location" ? "project.noLocations" : "project.noCharacters")
        }
    }

    KanteButton {
        objectName: "add"
        visible: list.kind !== "place"
        text: list.texts.t("project.add")
        onClicked: list.addRequested()
    }
}
