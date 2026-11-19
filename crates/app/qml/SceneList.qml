import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-13: the list of every scene of the project, the same in each module. A row shows the number,
// the heading and small markers: T the scene is in the screenplay, P the number of its plans,
// S the number of its shots. A scene that is not in the screenplay is dimmed and says so. The
// scene on view is highlighted; a click shows it (`controller.selectScene`) and then tells the
// module with `picked`, so the screenplay view can move to its heading. The field above filters
// by number or heading.
//
//  ┌ Szenen filtern ───┐
//  │ 1  INT. SAAL  T P1 S5 │
//  │ 2  EXT. HOF   T       │
//  │ 3  INT. KELLER P2 S7  │  (dimmed, "nicht im Buch")
//  └───────────────────┘
ColumnLayout {
    id: list

    required property var texts
    required property var controller
    /// A row was clicked and its scene is on view: `{ id, number, heading, inBook, plans, shots }`.
    signal picked(var scene)

    readonly property var scenes: controller.projectJson ? JSON.parse(controller.projectJson).scenes : []
    readonly property string _needle: filter.text.trim().toLowerCase()
    readonly property var shown: scenes.filter(s => _needle === ""
        || s.number.toLowerCase().includes(_needle) || s.heading.toLowerCase().includes(_needle))

    spacing: Kirigami.Units.smallSpacing

    // What the row's meta figure holds: "T P1 S5", each part only when it applies.
    function marks(scene) {
        const parts = []
        if (scene.inBook) {
            parts.push(texts.t("scenelist.markText"))
        }
        if (scene.plans > 0) {
            parts.push(texts.t("scenelist.markPlan") + scene.plans)
        }
        if (scene.shots > 0) {
            parts.push(texts.t("scenelist.markShots") + scene.shots)
        }
        return parts.join(" ")
    }

    KanteTextField {
        id: filter
        objectName: "sceneFilter"
        visible: list.scenes.length > 0
        Layout.fillWidth: true
        placeholderText: list.texts.t("scenelist.filter")
        Accessible.name: list.texts.t("scenelist.filter")
    }

    QQC2.Label {
        objectName: "noScenes"
        visible: list.scenes.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("screenplay.noScenes")
    }

    QQC2.Label {
        objectName: "noMatch"
        visible: list.scenes.length > 0 && list.shown.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: list.texts.t("scenelist.noMatch")
    }

    ListView {
        objectName: "sceneRows"
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        model: list.shown
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
        delegate: KanteListRow {
            required property var modelData
            objectName: "scene." + modelData.id
            width: ListView.view.width
            density: KanteListRow.Density.Compact
            opacity: modelData.inBook ? 1 : KanteStyle.dimOpacity
            leadingText: modelData.number
            leadingWidth: Kirigami.Units.gridUnit * 2
            text: modelData.heading !== "" ? modelData.heading : list.texts.t("scene.notInBook")
            subtitle: !modelData.inBook && modelData.heading !== "" ? list.texts.t("scene.notInBook") : ""
            meta: list.marks(modelData)
            selected: modelData.id === list.controller.sceneId
            Accessible.description: list.texts.t("scenelist.marks", { plans: modelData.plans, shots: modelData.shots })
            onClicked: {
                list.controller.selectScene(modelData.id)
                list.picked(modelData)
            }
        }
    }
}
