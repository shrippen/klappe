import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-26: the persons of the team grouped by department in the catalogue's order, those without a
// department last. A box narrows the list to one department. A click picks a person; "New…"
// asks for a form to make one.
//
//  ┌ Department [Alle ▾] ┐
//  │ KAMERA              │
//  │ Jo Berg   Kamera    │
//  │ DARSTELLER          │
//  │ Lena Weber          │
//  │ OHNE DEPARTMENT     │
//  │ [ Neu… ]            │
//  └─────────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// The id of the person the form shows.
    property string currentId: ""

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var departments: project.departments || []
    /// The filter box: "all" first, then the departments.
    readonly property var choices: [""].concat(departments)
    property string department: ""
    /// The persons of the department chosen, by department in the catalogue's order and then by
    /// name; `group` is the department, "" for none.
    readonly property var rows: (project.persons || [])
        .filter(p => department === "" || p.department === department)
        .map(p => ({ person: p, group: p.department || "" }))
        .sort((a, b) => rank(a.group) - rank(b.group) || a.person.name.localeCompare(b.person.name))

    /// A row was clicked.
    signal picked(string id)
    /// "New…" was clicked.
    signal newRequested()

    // The department's place in the list; none after all of them.
    function rank(group) {
        const place = departments.indexOf(group)
        return place < 0 ? departments.length : place
    }

    spacing: Kirigami.Units.smallSpacing

    KanteSectionLabel { text: list.texts.t("team.filter") }
    KanteComboBox {
        id: filterBox
        objectName: "departmentFilter"
        Layout.fillWidth: true
        Accessible.name: list.texts.t("team.filter")
        model: list.choices.map(d => d === "" ? list.texts.t("team.all") : list.texts.t("department." + d))
        currentIndex: Math.max(0, list.choices.indexOf(list.department))
        onActivated: index => list.department = list.choices[index]
    }

    ListView {
        id: rowsView
        objectName: "personRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.rows
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        section.property: "group"
        section.delegate: KanteSectionLabel {
            required property string section
            width: ListView.view.width
            topPadding: Kirigami.Units.smallSpacing
            text: list.texts.t(section === "" ? "team.noDepartment" : "department." + section)
        }
        delegate: KanteListRow {
            required property var modelData
            objectName: "person." + modelData.person.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            text: modelData.person.name
            meta: modelData.person.function
            selected: modelData.person.id === list.currentId
            onClicked: list.picked(modelData.person.id)
        }
    }

    QQC2.Label {
        objectName: "noPersons"
        visible: list.rows.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("team.none")
    }

    KanteButton {
        objectName: "newPerson"
        text: list.texts.t("project.add")
        onClicked: list.newRequested()
    }
}
