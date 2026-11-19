import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// E-09, rough first version: the screenplay as one text. The window's second working view
// beside the plan (View menu, Ctrl+E). The scenes and sections on the left, the Fountain text
// in a page-wide column in the middle. What is typed goes to the session after a short pause
// (`editScript`), so the scene list and the script panel follow; nothing is saved by it.
//
//  ┌ Szenen ──────────┬──────────────────────────────┐
//  │ # Akt I          │      INT. SAAL - TAG         │
//  │ 1 INT. SAAL      │                              │
//  │ 2 EXT. HOF       │      Sie kommt an.           │
//  └──────────────────┴──────────────────────────────┘
RowLayout {
    id: view

    required property var texts
    required property var controller
    /// How long typing rests before the text goes to the session.
    property int debounceMs: 500
    /// The width of the text column, in characters of the monospace font.
    readonly property int columns: 60
    /// The width of the scene list beside the text.
    readonly property real listWidth: Kirigami.Units.gridUnit * 16
    readonly property var entries: controller.outlineJson ? JSON.parse(controller.outlineJson) : []
    readonly property alias area: area
    /// The empty view's button: ask for a screenplay file.
    signal importRequested()

    spacing: 0

    /// Sends the text now if typing has not been sent yet (before a save).
    function flush() {
        if (!push.running) {
            return
        }
        push.stop()
        send()
    }

    /// The cursor at the start of text line `line` (from 0), that line at the top.
    function goToLine(line) {
        const rows = area.text.split("\n")
        let at = 0
        for (let i = 0; i < Math.min(line, rows.length); i++) {
            at += rows[i].length + 1
        }
        area.cursorPosition = at
        area.forceActiveFocus()

        const flick = scroll.contentItem
        const top = area.positionToRectangle(at).y
        flick.contentY = Math.max(0, Math.min(top, flick.contentHeight - flick.height))
    }

    function send() {
        if (area.text !== controller.scriptText) {
            controller.editScript(area.text)
        }
    }

    // Import, a restored backup or another project change the text outside the editor.
    // The text area gives back other text than it got (CRLF, no-break spaces), so what load sets
    // is no typing: it must never be sent.
    property bool _loading: false
    function load() {
        if (area.text === controller.scriptText) {
            return
        }
        _loading = true
        area.text = controller.scriptText
        _loading = false
    }

    Component.onCompleted: load()
    Connections {
        target: view.controller
        function onScriptTextChanged() { view.load() }
    }
    Timer {
        id: push
        interval: view.debounceMs
        onTriggered: view.send()
    }

    ColumnLayout {
        Layout.fillHeight: true
        // Fixed: the list's fillWidth would otherwise make the column fill and squeeze the text.
        Layout.fillWidth: false
        Layout.preferredWidth: view.listWidth
        Layout.maximumWidth: view.listWidth
        Layout.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: view.texts.t("screenplay.scenes") }

        QQC2.Label {
            visible: view.entries.length === 0
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            color: KanteStyle.mutedTextColor
            text: view.texts.t("screenplay.noScenes")
        }

        ListView {
            id: list
            objectName: "sceneList"
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: view.entries
            QQC2.ScrollBar.vertical: QQC2.ScrollBar {}
            delegate: KanteListRow {
                required property var modelData
                objectName: "scene." + modelData.line
                width: ListView.view.width
                density: KanteListRow.Density.Compact
                leadingText: modelData.kind === "section" ? "#".repeat(modelData.depth + 1) : modelData.number
                leadingWidth: Kirigami.Units.gridUnit * 2
                text: modelData.text
                onClicked: view.goToLine(modelData.line)
            }
        }
    }

    Kirigami.Separator { Layout.fillHeight: true }

    Item {
        Layout.fillWidth: true
        Layout.fillHeight: true

        FontMetrics { id: metrics; font: area.font }

        // Without text the view says why and offers the next step (D-17); it gives way to the
        // cursor once the text area has the focus.
        KanteEmptyState {
            objectName: "emptyBook"
            z: 1
            anchors.centerIn: parent
            width: Math.min(parent.width, Kirigami.Units.gridUnit * 24)
            visible: area.text === "" && !area.activeFocus
            title: view.texts.t("screenplay.empty")

            RowLayout {
                spacing: Kirigami.Units.smallSpacing
                KanteButton {
                    objectName: "loadScreenplay"
                    text: view.texts.t("script.import")
                    onClicked: view.importRequested()
                }
                KanteButton {
                    objectName: "writeScreenplay"
                    text: view.texts.t("screenplay.write")
                    onClicked: area.forceActiveFocus()
                }
            }
        }

        QQC2.ScrollView {
            id: scroll
            anchors.horizontalCenter: parent.horizontalCenter
            height: parent.height
            width: Math.min(parent.width, metrics.averageCharacterWidth * view.columns
                            + area.leftPadding + area.rightPadding)

            QQC2.TextArea {
                id: area
                objectName: "scriptArea"
                wrapMode: TextEdit.Wrap
                font.family: KanteStyle.monoFamily
                color: KanteStyle.textColor
                selectionColor: KanteStyle.selectionColor
                selectedTextColor: KanteStyle.textColor
                topPadding: Kirigami.Units.largeSpacing
                bottomPadding: Kirigami.Units.largeSpacing
                persistentSelection: true
                Accessible.name: view.texts.t("screenplay.editor")
                background: null
                onTextChanged: {
                    if (!view._loading) {
                        push.restart()
                    }
                }
                // A leaving focus is a good moment to send: a menu or dialog may save next.
                onActiveFocusChanged: {
                    if (!activeFocus) {
                        view.flush()
                    }
                }
            }
        }
    }
}
