import QtQuick

// D-12: "Go to: <module>" for every module of the bar, as palette command and key Ctrl+1 to
// Ctrl+N in the bar's order. The window switches the module.
Item {
    id: goto

    required property var texts
    required property var controller
    /// The modules of the bar: `{ id, name, icon }`.
    required property var modules
    property bool locked: false
    property var keyName: sequence => sequence

    signal requested(string id)

    /// In the order of the bar.
    property var actions: []

    visible: false
    width: 0
    height: 0

    Instantiator {
        model: goto.modules
        delegate: CommandAction {
            required property var modelData
            required property int index

            cmd: "module." + modelData.id
            group: "view"
            text: goto.texts.t("module.goto", { name: goto.texts.t(modelData.name) })
            shortcut: "Ctrl+" + (index + 1)
            keys: goto.keyName("Ctrl+" + (index + 1))
            enabled: goto.controller.loaded && !goto.locked
            onTriggered: goto.requested(modelData.id)
        }
        onObjectAdded: (index, object) => goto.actions = goto.actions.concat([object])
        onObjectRemoved: (index, object) => goto.actions = goto.actions.filter(a => a !== object)
    }
}
