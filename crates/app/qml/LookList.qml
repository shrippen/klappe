import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import "ProjectData.js" as ProjectData

// D-25: every look, grouped by character in the order of the characters, with the number of
// scenes that put it on. A click picks a look; "New…" asks for a character and a name.
//
//  ┌ ANNA ───────────────────┐
//  │ Ölzeug, nass    2 Szenen │
//  │ Anzug           1 Szene  │
//  │ WÄRTER                   │
//  │ Uniform         0 Szenen │
//  │ [ Neu… ]                 │
//  └──────────────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// The id of the look the form shows.
    property string currentId: ""

    readonly property var project: ProjectData.of(controller.projectJson)
    readonly property var characters: project.characters || []
    /// The looks by character, then in the order they were made.
    readonly property var rows: characters.reduce((all, c) => all.concat((project.looks || [])
        .filter(l => l.character === c.id)
        .map(l => ({ id: l.id, name: l.name, figure: c.name, scenes: l.scenes.length }))), [])

    /// A row was clicked, or a look was made.
    signal picked(string id)

    spacing: Kirigami.Units.smallSpacing

    function count(n) {
        return texts.t(n === 1 ? "looks.count1" : "looks.count", { count: n })
    }

    ListView {
        objectName: "lookRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.rows
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        section.property: "figure"
        section.delegate: KanteSectionLabel {
            required property string section
            width: ListView.view.width
            topPadding: Kirigami.Units.smallSpacing
            text: section
        }
        delegate: KanteListRow {
            required property var modelData
            objectName: "look." + modelData.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            text: modelData.name
            meta: list.count(modelData.scenes)
            selected: modelData.id === list.currentId
            onClicked: list.picked(modelData.id)
        }
    }

    QQC2.Label {
        objectName: "noLooks"
        visible: list.rows.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t(list.characters.length === 0 ? "looks.noCharacter" : "looks.empty")
    }

    KanteButton {
        objectName: "newLook"
        enabled: list.characters.length > 0
        text: list.texts.t("project.add")
        onClicked: form.show("", "")
    }

    LookForm {
        id: form
        objectName: "listLookForm"
        texts: list.texts
        controller: list.controller
        characters: list.characters
        onMade: id => list.picked(id)
    }
}
