import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import "ProjectData.js" as ProjectData

// D-26 (planung/DESIGN.md): the casting module (the module bar, Ctrl+4). On the left the roles,
// on the right the form of the role chosen.
//
//  ┌ ROLLEN ───────────┬───────────────────────────────┐
//  │ 1 Anna  Lena Weber │ Anna                          │
//  │ 2 Bert  2 Vorsch.  │ Cast-Nummer … Besetzung …     │
//  └────────────────────┴───────────────────────────────┘
RowLayout {
    id: view

    required property var texts
    required property var controller

    /// The width of the list beside the form.
    readonly property real listWidth: Kirigami.Units.gridUnit * 20
    /// The role the form shows; "" for none.
    property string roleId: ""

    readonly property var project: ProjectData.of(controller.projectJson)
    readonly property var role: (project.characters || []).find(c => c.id === roleId) || null
    readonly property var casting: (project.castings || []).find(k => k.character === roleId) || null

    spacing: 0

    CastingList {
        objectName: "roleList"
        Layout.fillHeight: true
        Layout.preferredWidth: view.listWidth
        Layout.maximumWidth: view.listWidth
        Layout.margins: Kirigami.Units.smallSpacing
        texts: view.texts
        controller: view.controller
        currentId: view.roleId
        onPicked: id => view.roleId = id
    }

    Kirigami.Separator { Layout.fillHeight: true }

    ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.margins: Kirigami.Units.largeSpacing

        CastingEditor {
            objectName: "roleForm"
            visible: view.role !== null
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 36
            texts: view.texts
            controller: view.controller
            role: view.role
            casting: view.casting
            persons: view.project.persons || []
            locked: view.project.numbers_locked === true
        }

        // Nothing chosen: say how to get a form.
        KanteEmptyState {
            objectName: "roleHint"
            visible: view.role === null
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 24
            title: view.texts.t("casting.pick")
        }
        Item { Layout.fillHeight: true }
    }
}
