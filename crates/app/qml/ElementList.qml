import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-22: every element of the breakdown, grouped by category in the catalogue's order, with its
// status as the colour of the square and as text. A click picks an element; "New…" asks for a
// form to make one.
//
//  ┌ KOSTÜM ─────────────┐
//  │ ■ Ballkleid  in Arbeit │
//  │ ■ Hut        offen     │
//  │ REQUISITE              │
//  │ ■ Brief      bereit    │
//  │ [ Neu… ]               │
//  └────────────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// The id of the element the form shows.
    property string currentId: ""

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var categories: project.element_categories || []
    /// The elements by category, then by name.
    readonly property var rows: categories.reduce((all, c) => all.concat((project.elements || [])
        .filter(e => e.category === c)
        .sort((a, b) => a.name.localeCompare(b.name))), [])

    /// A row was clicked.
    signal picked(string id)
    /// "New…" was clicked.
    signal newRequested()

    spacing: Kirigami.Units.smallSpacing

    ListView {
        id: rowsView
        objectName: "elementRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.rows
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        section.property: "category"
        section.delegate: KanteSectionLabel {
            required property string section
            width: ListView.view.width
            topPadding: Kirigami.Units.smallSpacing
            text: list.texts.t("elementCategory." + section)
        }
        delegate: KanteListRow {
            required property var modelData
            objectName: "element." + modelData.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            text: modelData.name
            meta: list.texts.t("elementStatus." + modelData.status)
            selected: modelData.id === list.currentId
            onClicked: list.picked(modelData.id)

            ElementChip {
                compact: true
                status: modelData.status
                toolTip: list.texts.t("elementStatus." + modelData.status)
            }
        }
    }

    QQC2.Label {
        objectName: "noElements"
        visible: list.rows.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("elements.none")
    }

    KanteButton {
        objectName: "newElement"
        text: list.texts.t("project.add")
        onClicked: list.newRequested()
    }
}
