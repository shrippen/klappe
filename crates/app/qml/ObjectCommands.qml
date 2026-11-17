import QtQuick
import QtQuick.Controls as QQC2

// The commands of an object and of a bound label (LH-114 to LH-116, LH-118): mirror (Shift+M),
// reset size (Shift+G), label (Shift+L) and let a label go (Shift+U), and one command per
// library kind to change the kind of the selected object (`kind:<kind>`). AppHeader lists them
// in the command palette; the card's buttons call the same controller functions.
QtObject {
    id: commands

    required property var texts
    required property var controller
    property bool locked: false
    /// AppHeader's `keyName`: a sequence as the keyboard of the language names it.
    property var keyName: sequence => sequence

    readonly property string selectedKey: controller.selectedKey || ""
    readonly property bool objectSelected: selectedKey.indexOf("object:") === 0
    // An object, a figure or a mark: not the grip of one (`objectgrip:`, `figurevia:`).
    readonly property bool labellable: /^(object|figure|mark):/.test(selectedKey)
    // A note bound to an element; every edit raises the picture revision.
    readonly property bool noteBound: {
        controller.pictureRevision
        if (selectedKey.indexOf("note:") !== 0 || typeof controller.describe !== "function") {
            return false
        }
        const info = controller.describe(selectedKey)
        return info !== "" && JSON.parse(info).bound === true
    }
    readonly property bool open: controller.loaded && !locked

    readonly property list<QtObject> actions: [
        CommandAction {
            cmd: "object.mirror"
            group: "edit"
            text: commands.texts.t("object.mirror")
            shortcut: "Shift+M"
            keys: commands.keyName("Shift+M")
            enabled: commands.open && commands.objectSelected
            onTriggered: commands.controller.changeSelected("mirror", "")
        },
        CommandAction {
            cmd: "object.resetSize"
            group: "edit"
            text: commands.texts.t("object.resetSize")
            shortcut: "Shift+G"
            keys: commands.keyName("Shift+G")
            enabled: commands.open && commands.objectSelected
            onTriggered: commands.controller.changeSelected("resetSize", "")
        },
        CommandAction {
            cmd: "object.label"
            group: "edit"
            text: commands.texts.t("object.label")
            shortcut: "Shift+L"
            keys: commands.keyName("Shift+L")
            enabled: commands.open && commands.labellable
            onTriggered: commands.controller.labelSelected()
        },
        CommandAction {
            cmd: "note.unbind"
            group: "edit"
            text: commands.texts.t("note.unbind")
            shortcut: "Shift+U"
            keys: commands.keyName("Shift+U")
            enabled: commands.open && commands.noteBound
            onTriggered: commands.controller.changeSelected("unbind", "")
        }
    ]

    // One entry per library kind while an object is selected: change its kind (LH-114).
    function kindCommands() {
        if (!objectSelected || !controller.choicesJson) {
            return []
        }
        return (JSON.parse(controller.choicesJson).object || []).map(o => ({
            id: "kind:" + o.kind,
            text: texts.t("object.kindTo", { name: o.custom ? (o.label || o.kind) : texts.t("objectKind." + o.kind) }),
            group: texts.t("palette.group.edit"),
            enabled: open
        }))
    }
}
