import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import "ProjectData.js" as ProjectData

// Besetzung (D-26): the roles of the project, the characters. A row shows the cast number, the
// name and who plays it (the fixed person) or how many are proposed. Groups of extras come last,
// with their size after the name.
//
//  ┌ ROLLEN ──────────────────────┐
//  │ 1  Anna        Lena Weber    │
//  │ 2  Bert        2 Vorschläge  │
//  │ 3  Clara       offen         │
//  │ –  Gäste ×12   offen         │
//  └──────────────────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// The id of the role the form shows.
    property string currentId: ""

    readonly property var project: ProjectData.of(controller.projectJson)
    readonly property var characters: project.characters || []
    /// The roles with their casting, the groups last.
    readonly property var rows: characters.filter(c => c.group === 0).concat(characters.filter(c => c.group > 0))
        .map(c => ({ role: c, casting: (project.castings || []).find(k => k.character === c.id) || null }))

    /// A row was clicked.
    signal picked(string id)

    function personName(id) {
        const person = (project.persons || []).find(p => p.id === id)
        return person ? person.name : ""
    }

    // Who plays the role, how many are proposed, or that it is open.
    function whoText(casting) {
        if (!casting) {
            return texts.t("casting.open")
        }
        if (casting.fixed !== null) {
            return personName(casting.fixed)
        }
        const count = casting.proposed.length
        if (count === 0) {
            return texts.t("casting.open")
        }
        return texts.t(count === 1 ? "casting.proposals1" : "casting.proposals", { count: count })
    }

    spacing: Kirigami.Units.smallSpacing

    KanteSectionLabel { text: list.texts.t("casting.roles"); rule: true; Layout.fillWidth: true }

    ListView {
        id: rowsView
        objectName: "roleRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.rows
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        delegate: KanteListRow {
            required property var modelData
            objectName: "role." + modelData.role.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            leadingText: modelData.casting && modelData.casting.number !== null ? String(modelData.casting.number) : "–"
            leadingWidth: Kirigami.Units.gridUnit * 2
            text: modelData.role.group > 0 ? modelData.role.name + " ×" + modelData.role.group : modelData.role.name
            meta: list.whoText(modelData.casting)
            selected: modelData.role.id === list.currentId
            onClicked: list.picked(modelData.role.id)
        }
    }

    QQC2.Label {
        objectName: "noRoles"
        visible: list.rows.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("casting.noRoles")
    }
}
