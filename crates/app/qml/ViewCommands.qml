import QtQuick
import "ProjectData.js" as ProjectData

// The commands of the View menu that keep or clear what the plan shows: save and delete a
// view (LH-119, each asks in the form), and the sketch pen's one command (LH-120). Choosing a
// saved view is a menu or palette entry of the header, one per view.
Item {
    id: commands

    required property var texts
    required property var controller
    /// While the backup question is open nothing may change the project.
    property bool locked: false

    /// [{ id, name }] as the project view lists them.
    readonly property var savedViews: ProjectData.of(controller.projectJson).views || []
    readonly property alias saveAction: actSave
    readonly property alias removeAction: actRemove
    readonly property alias clearAction: actClear
    /// In the order of the palette.
    readonly property var actions: [actSave, actRemove, actClear]

    visible: false
    width: 0
    height: 0

    /// The palette's entries besides the actions: each saved view to choose or delete (LH-119),
    /// and each layer to fade one step (LH-15). The header's `run` knows their ids.
    function palette() {
        const group = texts.t("palette.group.view")
        const layers = controller.choicesJson ? (JSON.parse(controller.choicesJson).layer || []) : []
        const enabled = controller.loaded && !locked

        return savedViews.map(v => ({ id: "savedView:" + v.id, text: texts.t("view.apply", { name: v.name }), group: group, enabled: enabled }))
            .concat(savedViews.map(v => ({ id: "removeView:" + v.id, text: texts.t("view.removeCommand", { name: v.name }), group: group, enabled: enabled })))
            .concat(layers.map(name => ({ id: "fade:" + name, text: texts.t("layer.fadeCommand", { layer: texts.t("layer." + name) }), group: group, enabled: enabled })))
    }

    CommandAction {
        id: actSave
        cmd: "view.save"
        group: "view"
        text: commands.texts.t("view.save")
        enabled: commands.controller.loaded && !commands.locked
        onTriggered: form.show("save")
    }
    CommandAction {
        id: actRemove
        cmd: "view.remove"
        group: "view"
        text: commands.texts.t("menu.viewRemove")
        enabled: commands.controller.loaded && !commands.locked && commands.savedViews.length > 0
        onTriggered: form.show("remove")
    }
    CommandAction {
        id: actClear
        cmd: "sketch.clear"
        group: "edit"
        text: commands.texts.t("sketch.clear")
        enabled: commands.controller.loaded && !commands.locked
        onTriggered: commands.controller.clearSketches()
    }
    ViewForm {
        id: form
        objectName: "viewForm"
        texts: commands.texts
        controller: commands.controller
    }
}
