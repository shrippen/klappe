import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// E-09, D-15: the screenplay as flowing text. The window's second working view beside the plan
// (the module bar, Ctrl+1). The project's scenes (SceneList) on the left, the Fountain text in
// the middle, its elements set in the columns of the page, scaled to the width of the text
// (ScriptLook; the text itself stays plain Fountain). What is typed goes to the session after a
// short pause (`editScript`), so the scene list and the script panel follow; nothing is saved by it.
//
//  ┌ Szenen ──────────┬──────────────────────────────────────┐
//  │ 1 INT. SAAL  T   │ INT. SAAL - TAG                      │
//  │ 2 EXT. HOF   T   │ Sie kommt an.                        │
//  │ 3 INT. KELLER    │               ANNA                   │
//  │                  │         Ihr kommt spät.              │
//  └──────────────────┴──────────────────────────────────────┘
RowLayout {
    id: view

    required property var texts
    required property var controller
    /// How long typing rests before the text goes to the session.
    property int debounceMs: 500
    /// The width of the text column at most, in characters of the monospace font: nearly the
    /// whole view (D-15), yet lines a reader can follow. The page's columns are scaled to it once,
    /// not to the view, so a resize never touches the text's undo history.
    readonly property int columns: 100
    /// The kinds of `scriptKinds` and the columns the page sets them in.
    readonly property var pageLooks: JSON.parse(controller.scriptLooks())
    /// The same in pixels, with the faces: what ScriptLook gets.
    readonly property var lineLooks: pixelLooks(metrics.averageCharacterWidth)
    /// The width of the scene list beside the text.
    readonly property real listWidth: Kirigami.Units.gridUnit * 16
    readonly property alias area: area
    /// What sets the lines as their kind and keeps the undo history; the tests give a stand-in.
    property url toolsSource: Qt.resolvedUrl("ScriptTools.qml")
    readonly property alias tools: tools.item
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

    /// The cursor at the heading of `scene`, a row of the scene list: its `line` is the heading's
    /// text line, null when the book does not hold the scene, which leaves the cursor where it is.
    function goToScene(scene) {
        if (typeof scene.line === "number") {
            goToLine(scene.line)
        }
    }

    /// Margins in pixels of the page's columns, scaled to `columns`; headings bold, what the page
    /// does not print (sections, synopses, notes) muted.
    function pixelLooks(charWidth) {
        const unit = charWidth * columns / pageLooks.columns
        const out = {}
        for (const letter in pageLooks.looks) {
            const look = pageLooks.looks[letter]
            const printed = look.x !== undefined
            out[letter] = {
                left: printed ? look.x * unit : 0,
                right: printed ? (pageLooks.columns - look.x - look.width) * unit : 0,
                align: printed ? look.align : "left",
                bold: look.kind === "heading",
                muted: look.kind === "section" || look.kind === "synopsis" || look.kind === "note"
            }
        }
        return out
    }

    /// Sets each line as its kind; `_styled` is the text it was done for.
    property string _styled: ""
    function restyle() {
        if (!tools.item) {
            return
        }
        _styled = area.text
        tools.item.setLooks(controller.scriptKinds(_styled), lineLooks)
    }
    onLineLooksChanged: restyle()

    /// Ctrl+Z and Ctrl+Shift+Z in the text: the editor's own history, not the project's.
    function undoText() {
        moveCursor(tools.item ? tools.item.undo() : -1)
    }

    function redoText() {
        moveCursor(tools.item ? tools.item.redo() : -1)
    }

    function moveCursor(at) {
        if (at >= 0) {
            area.cursorPosition = at
        }
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
        restyle()
        if (tools.item) {
            tools.item.forget()
        }
    }

    Component.onCompleted: load()
    Connections {
        target: view.controller
        function onScriptTextChanged() { view.load() }
    }
    Loader {
        id: tools
        source: view.toolsSource
        onLoaded: {
            item.document = area.textDocument
            item.mutedColor = Qt.binding(() => KanteStyle.mutedTextColor)
            view.restyle()
            item.forget()
        }
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

        SceneList {
            objectName: "sceneList"
            Layout.fillWidth: true
            Layout.fillHeight: true
            texts: view.texts
            controller: view.controller
            onPicked: scene => view.goToScene(scene)
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
                Keys.onPressed: event => {
                    if (event.matches(StandardKey.Undo)) {
                        view.undoText()
                        event.accepted = true
                    } else if (event.matches(StandardKey.Redo)) {
                        view.redoText()
                        event.accepted = true
                    }
                }
                onTextChanged: {
                    // A new look changes no text: nothing to send or set again.
                    if (view._loading || text === view._styled) {
                        return
                    }
                    push.restart()
                    view.restyle()
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
