import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-26 (planung/DESIGN.md): the team module (the module bar, Ctrl+5). On the left the persons by
// department, on the right the form of the person chosen or of a new one.
//
//  ┌ Department [Alle] ┬────────────────────────────┐
//  │ KAMERA             │ Name, Department, Funktion │
//  │ Jo Berg            │ Telefon, Mail, Adresse     │
//  │ [ Neu… ]           │ [Übernehmen] [Löschen …]   │
//  └────────────────────┴────────────────────────────┘
RowLayout {
    id: view

    required property var texts
    required property var controller

    /// The width of the list beside the form.
    readonly property real listWidth: Kirigami.Units.gridUnit * 20
    /// The person the form shows; "" for none.
    property string personId: ""
    /// The form makes a new person instead of changing `person`.
    property bool creating: false

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var person: (project.persons || []).find(p => p.id === personId) || null

    function pick(id) {
        creating = false
        personId = id
    }

    spacing: 0

    TeamList {
        objectName: "personList"
        Layout.fillHeight: true
        Layout.preferredWidth: view.listWidth
        Layout.maximumWidth: view.listWidth
        Layout.margins: Kirigami.Units.smallSpacing
        texts: view.texts
        controller: view.controller
        currentId: view.creating ? "" : view.personId
        onPicked: id => view.pick(id)
        onNewRequested: {
            view.personId = ""
            view.creating = true
        }
    }

    Kirigami.Separator { Layout.fillHeight: true }

    ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.margins: Kirigami.Units.largeSpacing

        PersonEditor {
            id: form
            objectName: "personForm"
            visible: view.creating || view.person !== null
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 32
            texts: view.texts
            controller: view.controller
            item: view.creating ? null : view.person
            departments: view.project.departments || []
            // A new person is shown in its form afterwards.
            onDone: result => {
                if (view.creating) {
                    view.pick(result)
                }
            }
        }

        // Nothing chosen: say how to get a form.
        KanteEmptyState {
            objectName: "personHint"
            visible: !form.visible
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 24
            title: view.texts.t("team.pick")
        }
        Item { Layout.fillHeight: true }
    }
}
