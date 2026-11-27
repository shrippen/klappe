import QtQuick

// D-36: "Fountain-Quelltext" in the View menu and the palette. The text of the screenplay shows
// the source with all its marks instead of the printed look. The editor keeps the state
// (`source`) and hands it back; `toggled` names the area ("scriptSource"). Only the flowing
// text of the screenplay has it, the printed pages do not.
Item {
    id: views

    required property var texts
    required property var controller
    property bool locked: false
    /// The open module's id.
    property string module: ""
    /// The screenplay shows its flowing text, not the pages as they print.
    property bool flow: true
    /// The text shows the source as Fountain.
    property bool source: false

    signal toggled(string area, bool shown)

    readonly property alias sourceAction: actSource
    /// In the order of the palette.
    readonly property var actions: [actSource]

    visible: false
    width: 0
    height: 0

    CommandAction {
        id: actSource
        cmd: "script.source"
        group: "view"
        text: views.texts.t("script.source")
        checkable: true
        checked: views.source
        enabled: views.controller.loaded && !views.locked && views.module === "screenplay" && views.flow
        onToggled: views.toggled("scriptSource", checked)
    }
}
