import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-22 (planung/DESIGN.md): the breakdown module (the module bar, Ctrl+3). On the left the
// scene list or, switched by "Szenen | Elemente", the list of all elements; on the right the
// sheet of the scene on view and its text to mark elements in (D-23), or the form of the
// element. A scene of an element's chips is shown
// and the view goes back to the scenes.
//
//  ┌ [Szenen|Elemente] ┬──────────────────────────┬──────────────────┐
//  │ 1 INT. SAAL       │ 1  INT. SAAL – TAG       │ TEXT DER SZENE   │
//  │ 2 EXT. HOF        │ Spieltag, Vorstopp …     │ Anna öffnet den  │
//  │                   │ Figuren, Elemente …      │ Brief REQ …      │
//  └───────────────────┴──────────────────────────┴──────────────────┘
RowLayout {
    id: view

    required property var texts
    required property var controller

    /// The width of the list beside the sheet.
    readonly property real listWidth: Kirigami.Units.gridUnit * 16
    /// The width of the scene's text beside the sheet (D-23).
    readonly property real textWidth: Math.max(0, Math.round((width - listWidth) * 0.42))
    /// The segments: the scene sheet and the elements.
    readonly property var modes: ["scenes", "elements"]
    property string mode: "scenes"
    /// The element the form shows; "" for none.
    property string elementId: ""
    /// The form makes a new element instead of changing `element`.
    property bool creating: false

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var element: (project.elements || []).find(e => e.id === elementId) || null

    // Shows the scene `id`: the list and the sheet of the scenes follow.
    function showScene(id) {
        controller.selectScene(id)
        mode = "scenes"
    }

    function pick(id) {
        creating = false
        elementId = id
    }

    // The new element `id` is shown in its form afterwards.
    function made(id) {
        creating = false
        elementId = id
    }

    spacing: 0

    ColumnLayout {
        Layout.fillHeight: true
        Layout.preferredWidth: view.listWidth
        Layout.maximumWidth: view.listWidth
        Layout.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteSegmented {
            objectName: "modeSwitch"
            Accessible.name: view.texts.t("breakdown.mode")
            model: view.modes.map(m => view.texts.t("breakdown." + m))
            currentIndex: view.modes.indexOf(view.mode)
            onActivated: index => view.mode = view.modes[index]
        }

        SceneList {
            objectName: "sceneList"
            visible: view.mode === "scenes"
            Layout.fillWidth: true
            Layout.fillHeight: true
            texts: view.texts
            controller: view.controller
        }

        ElementList {
            objectName: "elementList"
            visible: view.mode === "elements"
            Layout.fillWidth: true
            Layout.fillHeight: true
            texts: view.texts
            controller: view.controller
            currentId: view.creating ? "" : view.elementId
            onPicked: id => view.pick(id)
            onNewRequested: {
                view.elementId = ""
                view.creating = true
            }
        }
    }

    Kirigami.Separator { Layout.fillHeight: true }

    SceneSheet {
        objectName: "sheet"
        visible: view.mode === "scenes"
        Layout.fillWidth: true
        Layout.fillHeight: true
        texts: view.texts
        controller: view.controller
    }

    Kirigami.Separator { visible: view.mode === "scenes"; Layout.fillHeight: true }

    // D-23: the scene's text beside its sheet, to mark elements in.
    SceneText {
        objectName: "sceneText"
        visible: view.mode === "scenes"
        Layout.fillHeight: true
        // Its lines would ask for their full length: the width is set (all three, or the long
        // hint's width fights the maximum), the sheet takes the rest.
        Layout.minimumWidth: view.textWidth
        Layout.preferredWidth: view.textWidth
        Layout.maximumWidth: view.textWidth
        Layout.margins: Kirigami.Units.largeSpacing
        texts: view.texts
        controller: view.controller
    }

    ColumnLayout {
        visible: view.mode === "elements"
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.margins: Kirigami.Units.largeSpacing

        ElementEditor {
            id: form
            objectName: "elementForm"
            visible: view.creating || view.element !== null
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 32
            texts: view.texts
            controller: view.controller
            item: view.creating ? null : view.element
            categories: view.project.element_categories || []
            statuses: view.project.element_statuses || []
            scenes: view.project.scenes || []
            onSceneRequested: id => view.showScene(id)
            onDone: result => {
                if (view.creating) {
                    view.made(result)
                }
            }
        }
        // Nothing chosen: say how to get a form (the right side was blank).
        KanteEmptyState {
            objectName: "elementHint"
            visible: !form.visible
            Layout.fillWidth: true
            Layout.maximumWidth: Kirigami.Units.gridUnit * 24
            title: view.texts.t("elements.pick")
        }
        Item { Layout.fillHeight: true }
    }
}
