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
    /// How long no key is pressed before the pages and the scene lengths are laid out anew
    /// (DN-02: typing never waits for it).
    property int layoutRestMs: 5000
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
    /// What the text shows: `flow` to write in, `print` the pages as they print (D-15).
    readonly property var layouts: ["flow", "print"]
    property string layout: "flow"
    readonly property alias printView: printView
    readonly property alias titleCard: titleCard
    /// The last layout's page starts: `{ pages, breaks: [{ page, line }], scenes: [{ line, page }] }`
    /// (`scriptBreaksJson`). Shown as lines in the flowing text, used to find a scene's page.
    property var pageMap: ({ pages: 0, breaks: [], scenes: [] })
    /// The sections, scenes and synopses of the text (`scriptOutlineJson`) for the scene list,
    /// without what the folded sections hold. It follows the session's text, the last layout and
    /// what the list has folded, so it is read again when one changes.
    readonly property var outline: {
        controller.scriptText
        controller.projectJson
        return JSON.parse(controller.scriptOutlineJson(JSON.stringify(Object.keys(sceneList.collapsed))))
    }
    /// Typing since the last layout: the lines and the scene lengths are of an older text (DN-02).
    property bool stale: false
    /// The locked versions and the marks of what changed since (ScriptVersions).
    property alias versions: scriptVersions.versions
    property alias nextRevision: scriptVersions.nextRevision
    property alias showRevisions: scriptVersions.showRevisions
    property alias revisionLines: scriptVersions.revisionLines
    property alias revisionRanges: scriptVersions.revisionRanges
    readonly property alias versionMenu: versionMenu
    readonly property alias lockDialog: scriptVersions.lockDialog
    readonly property alias compareDialog: scriptVersions.compareDialog
    readonly property alias removeDialog: scriptVersions.removeDialog
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

    /// Reads the layout the controller holds now; what was typed up to it is laid out.
    function readLayout() {
        pageMap = JSON.parse(controller.scriptBreaksJson())
        stale = false
        readRevision()
    }

    /// The lines changed since the last version, read anew (ScriptVersions).
    function readRevision() {
        scriptVersions.readRevision()
    }

    /// The lines of `lines` as text positions (ScriptVersions).
    function lineRanges(lines) {
        return scriptVersions.lineRanges(lines)
    }

    /// The version's dialogs (ScriptVersions).
    function askLock() {
        scriptVersions.askLock()
    }

    function askCompare(from) {
        scriptVersions.askCompare(from)
    }

    /// Lays the book out and shows the result (the lines, the lengths in the scene list).
    function layOut() {
        controller.paginateScript()
        readLayout()
    }

    /// The page starts in `breaks` (by line, ascending) as `{ at }`, `at` the text position of the
    /// first line of the page, -1 when the text no longer has that line.
    function breakMarks(breaks) {
        return doc.lineStarts(breaks.map(b => b.line)).map(at => ({ at: at }))
    }

    readonly property var marks: breakMarks(pageMap.breaks)

    /// The cursor at the start of text line `line` (from 0), that line at the top.
    function goToLine(line) {
        const text = doc.read()
        const start = doc.lineStart(line)
        const at = start < 0 ? text.length : start
        doc.setCursor(at)
        area.forceActiveFocus()

        const flick = scroll.contentItem
        const top = doc.rectOf(at).y
        flick.contentY = Math.max(0, Math.min(top, flick.contentHeight - flick.height))
    }

    /// Shows `layout`; the pages are laid out from what was typed up to now. The pages take the
    /// keys: the hidden text must not change under them.
    function showLayout(name) {
        if (name === "print") {
            flush()
            layOut()
            printView.refresh()
        }
        layout = name
        if (name === "print") {
            printView.forceActiveFocus()
        }
    }

    /// The page of the first scene at or after text line `line` (a scene's own heading, or a
    /// section's first scene); 0 when no scene follows.
    function pageOfLine(line) {
        const found = pageMap.scenes.find(s => s.line >= line)
        return found ? found.page : 0
    }

    /// The cursor at the heading of `scene`, a row of the scene list: its `line` is the heading's
    /// text line, null when the book does not hold the scene, which leaves the cursor where it is.
    function goToScene(scene) {
        if (typeof scene.line !== "number") {
            return
        }
        // The print view stays: it scrolls to the page the heading stands on.
        if (layout === "print") {
            const page = pageOfLine(scene.line)
            if (page > 0) {
                printView.goToPage(page, pageMap.pages)
            }
            return
        }
        showLayout("flow")
        goToLine(scene.line)
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

    /// Sets each line as its kind.
    function restyle() {
        if (!tools.item) {
            return
        }
        flow.shown = flow.withPending(controller.scriptKinds(doc.read()))
        tools.item.setLooks(flow.shown, lineLooks)
    }

    ScriptDoc {
        id: doc
        area: area
        controller: view.controller
        tools: view.tools
        onEdited: view.sourceEdited()
    }
    readonly property alias doc: doc

    ScriptFlow {
        id: flow
        doc: doc
        controller: view.controller
        onPendingMoved: view.restyle()
    }
    readonly property alias flow: flow
    readonly property alias scriptMenu: scriptMenu
    readonly property alias search: search
    readonly property alias sceneList: sceneList
    onLineLooksChanged: restyle()

    /// The source changed (typed, replaced, a step back): it goes to the session after the pause,
    /// the lines are set as their kinds. The element flow may turn what was typed to capitals.
    function sourceEdited() {
        push.restart()
        relayout.restart()
        Qt.callLater(flow.updateSuggest)
        stale = true
        if (!_replaying && flow.capitalize()) {
            return
        }
        restyle()
    }

    /// Ctrl+Z and Ctrl+Shift+Z in the text: the editor's own history of the source, not the
    /// project's. A step back is no typing: its text keeps the case it had.
    property bool _replaying: false
    function undoText() {
        _replaying = true
        moveCursor(doc.undo())
        _replaying = false
    }

    function redoText() {
        _replaying = true
        moveCursor(doc.redo())
        _replaying = false
    }

    /// The share of the text's width the search card takes.
    readonly property real searchShare: 0.6

    /// A hit closer than this many lines to the bottom is scrolled to; it then stands this far
    /// down the view (a third), so the lines before it show too.
    readonly property int hitMarginLines: 2
    readonly property real hitHeight: 1 / 3
    /// Unfolds the title page card (DB-06) over the flowing text, with the cursor in its first
    /// field; the pages as they print give way to the text for it.
    function showTitlePage() {
        showLayout("flow")
        titleCard.show()
    }

    /// Selects `from` to `to` and scrolls it into view (a hit of the search).
    function showText(from, to) {
        showLayout("flow")
        doc.select(from, to)
        const flick = scroll.contentItem
        const top = doc.rectOf(from).y
        if (top < flick.contentY || top > flick.contentY + flick.height - metrics.height * hitMarginLines) {
            flick.contentY = Math.max(0, Math.min(top - flick.height * hitHeight, flick.contentHeight - flick.height))
        }
        dodgeSearch(from, to)
    }

    /// The search card goes to the other edge while the hit shown lies under it (DB-07), and
    /// stays where it is otherwise.
    property bool searchAtBottom: false
    function dodgeSearch(from, to) {
        const a = doc.rectOf(from)
        const b = doc.rectOf(to)
        // A hit that wraps covers its first line up to the text's right edge.
        const right = b.y > a.y ? area.width : b.x
        const p = area.mapToItem(search, a.x, a.y)
        const hit = Qt.rect(p.x, p.y, Math.max(right - a.x, 1), a.height)
        const under = hit.x < search.width && hit.x + hit.width > 0
            && hit.y < search.height && hit.y + hit.height > 0
        if (under) {
            searchAtBottom = !searchAtBottom
        }
    }

    /// Puts `text` in place of `from` to `to` as one undo step; a replacement is no typing,
    /// so it keeps its case.
    function putText(from, to, text) {
        _replaying = true
        doc.replace(from, to, text)
        _replaying = false
    }

    /// Up, Down and Esc while the choices are open (Enter and Tab take one: `flow`).
    function suggestKey(event) {
        const moves = { [Qt.Key_Down]: 1, [Qt.Key_Up]: -1 }
        if (event.key === Qt.Key_Escape) {
            flow.dismissSuggest()
        } else if (event.key in moves) {
            flow.moveSuggest(moves[event.key])
        } else {
            return false
        }
        return true
    }

    /// The keys of the text that are not typing: the element flow, the search, the text's undo.
    function keyPressed(event) {
        // The pad's Enter comes with the keypad modifier; it is a plain Enter all the same.
        const plain = (event.modifiers & ~Qt.KeypadModifier) === Qt.NoModifier
        if (plain && flow.suggestions && suggestKey(event)) {
            event.accepted = true
            return
        }
        const enter = event.key === Qt.Key_Return || event.key === Qt.Key_Enter
        if (plain && enter) {
            event.accepted = flow.enter()
            return
        }
        event.accepted = true
        if (plain && event.key === Qt.Key_Tab) {
            flow.tab()
        } else if (event.matches(StandardKey.Undo)) {
            undoText()
        } else if (event.matches(StandardKey.Redo)) {
            redoText()
        } else if (!clipboardKey(event) && !editKey(event)) {
            event.accepted = false
        }
    }

    /// Toggles `mark` ("bold", "italic", "underline") for the selection, or for what is typed next.
    /// An emphasis is no typing: the case of the text stays.
    function toggleMark(mark) {
        _replaying = true
        doc.toggleMark(mark)
        _replaying = false
    }

    /// Ctrl+B, Ctrl+I, Ctrl+U and the keys of the context menu; false for another key.
    function editKey(event) {
        const marks = [[StandardKey.Bold, "bold"], [StandardKey.Italic, "italic"],
                       [StandardKey.Underline, "underline"]]
        const found = marks.find(m => event.matches(m[0]))
        if (found) {
            toggleMark(found[1])
            return true
        }
        return scriptMenu.key(event)
    }

    /// The text shows the source as Fountain, nothing hidden (D-36); not kept with the project.
    readonly property bool sourceShown: doc.view === ScriptDoc.View.Source

    /// Shows the source as Fountain (`on`) or as the printed screenplay. The cursor, the selection
    /// and the scroll stay where they are; nothing is written.
    function showSource(on) {
        const flick = scroll.contentItem
        const top = flick.contentY
        doc.setView(on ? ScriptDoc.View.Source : ScriptDoc.View.Shown)
        restyle()
        flick.contentY = Math.max(0, Math.min(top, flick.contentHeight - flick.height))
    }

    /// Alt+Up and Alt+Down in the scene list (DB-21): the scene or section whose heading is at text
    /// line `line` swaps places with its neighbour, its text with it. One edit of the text (one
    /// step of its undo history), sent to the session at once; the list keeps its focus on the block.
    function moveBlock(line, way) {
        flush()
        const move = controller.moveOutline(line, way)
        if (move === "") {
            return
        }
        const m = JSON.parse(move)
        // The block may have stepped into a folded section: open the sections around it.
        sceneList.unfold(m.unfold)
        const text = doc.read()
        const [from, end] = doc.lineStarts([m.from, m.to])
        putText(from, end < 0 ? text.length : end, m.lines.join("\n") + (end < 0 ? "" : "\n"))
        flush()
        sceneList.focusAt(m.line)
    }

    /// Ctrl+C and Ctrl+X: the Fountain of the selection goes on the clipboard, as the text area
    /// would copy what it shows; false for another key, or without a selection.
    function clipboardKey(event) {
        const cut = event.matches(StandardKey.Cut)
        if ((!cut && !event.matches(StandardKey.Copy)) || !copySelection()) {
            return false
        }
        if (cut) {
            doc.cutSelection()
        }
        return true
    }

    /// Puts the source of the selection on the clipboard; false without a selection or tools.
    function copySelection() {
        if (!tools.item || !tools.item.copy || doc.selectedText === "") {
            return false
        }
        tools.item.copy(doc.selectedSource())
        return true
    }

    function moveCursor(at) {
        if (at >= 0) {
            doc.setCursor(at)
        }
    }

    function send() {
        if (doc.read() !== controller.scriptText) {
            controller.editScript(doc.read())
        }
    }

    // Import, a restored backup or another project change the text outside the editor.
    // The text area gives back other text than it got (CRLF, no-break spaces), so what load sets
    // is no typing: it must never be sent.
    function load() {
        if (doc.read() === controller.scriptText) {
            return
        }
        doc.load(controller.scriptText)
        restyle()
        // Another text from outside (another project, an import): what is typed in the card was
        // for the old one.
        titleCard.discard()
        if (tools.item) {
            tools.item.forget()
        }
        // No key was pressed: the pages and lengths follow the new text at once (DN-02).
        layOut()
    }

    Component.onCompleted: {
        load()
        scriptVersions.readVersions()
    }
    Connections {
        target: view.controller
        // Another project with the same text, or another paper, changes the project but not the
        // text: lay out again, unless typing is pending (DN-02: typing never lays out).
        function onProjectJsonChanged() {
            scriptVersions.readVersions()
            if (!view.stale) {
                view.layOut()
            }
        }
        function onScriptTextChanged() {
            view.load()
            // Typing reaches here after its pause: the card shows the title block as it stands.
            titleCard.follow()
            if (view.layout === "print") {
                printView.refresh()
            }
        }
    }
    Loader {
        id: tools
        source: view.toolsSource
        onLoaded: {
            item.document = doc.document
            item.mutedColor = Qt.binding(() => KanteStyle.mutedTextColor)
            view.restyle()
            doc.redrawRuns()
            item.forget()
        }
    }
    ScriptMenu {
        id: scriptMenu
        objectName: "scriptMenu"
        texts: view.texts
        editor: view
    }
    // Ctrl+F anywhere in the view, not only in the text (the scene list, the switch).
    Shortcut {
        sequences: [StandardKey.Find]
        enabled: view.visible
        onActivated: search.open()
    }
    Timer {
        id: push
        interval: view.debounceMs
        onTriggered: view.send()
    }
    Timer {
        id: relayout
        interval: view.layoutRestMs
        onTriggered: {
            view.flush()
            view.layOut()
        }
    }

    ScriptVersions {
        id: scriptVersions
        texts: view.texts
        controller: view.controller
        doc: doc
        onFlushRequested: view.flush()
        onLayoutRequested: view.layOut()
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
            id: sceneList
            objectName: "sceneList"
            Layout.fillWidth: true
            Layout.fillHeight: true
            texts: view.texts
            controller: view.controller
            lengthsStale: view.stale
            outline: view.outline
            onPicked: scene => view.goToScene(scene)
            onMoveRequested: (line, way) => view.moveBlock(line, way)
        }
    }

    Kirigami.Separator { Layout.fillHeight: true }

    ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: 0

        // Flowing text to write in, or the pages as they print (D-15).
        RowLayout {
            Layout.alignment: Qt.AlignRight
            Layout.margins: Kirigami.Units.smallSpacing
            spacing: Kirigami.Units.smallSpacing

            // Typed since the last layout (DN-02): the page lines and lengths are of the text before.
            QQC2.Label {
                objectName: "staleNote"
                visible: view.stale
                color: KanteStyle.mutedTextColor
                text: view.texts.t("screenplay.stale")
            }

            VersionMenu {
                id: versionMenu
                objectName: "versionMenu"
                enabled: doc.source !== ""
                texts: view.texts
                versions: view.versions
                showRevisions: view.showRevisions
                onLockRequested: view.askLock()
                onCompareRequested: from => view.askCompare(from)
                onRemoveRequested: version => removeDialog.ask(version)
                onRevisionsToggled: shown => scriptVersions.setRevisions(shown)
            }

            KanteSegmented {
                objectName: "layoutSwitch"
                model: view.layouts.map(l => view.texts.t("screenplay." + l))
                currentIndex: view.layouts.indexOf(view.layout)
                onActivated: index => view.showLayout(view.layouts[index])
            }
        }

        // The title page (DB-06): the top card above the flowing text, as wide as the text.
        TitleCard {
            id: titleCard
            objectName: "titleCard"
            visible: view.layout === "flow"
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            Layout.maximumWidth: metrics.averageCharacterWidth * view.columns + area.leftPadding + area.rightPadding
            Layout.bottomMargin: Kirigami.Units.smallSpacing
            texts: view.texts
            controller: view.controller
            doc: doc
            putText: view.putText
            onApplied: area.forceActiveFocus()
            onDismissed: area.forceActiveFocus()
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            FontMetrics { id: metrics; font: area.font }

            // Find and replace (DB-07), Ctrl+F in the view: a card over the top right of the
            // text, so the layout switch above keeps its place; at the bottom right while the
            // hit shown lies under it.
            ScriptSearch {
                id: search
                objectName: "scriptSearch"
                z: 3
                anchors.right: scroll.right
                anchors.rightMargin: Kirigami.Units.largeSpacing
                y: view.searchAtBottom ? parent.height - height - Kirigami.Units.largeSpacing
                                       : Kirigami.Units.largeSpacing
                width: Math.min(scroll.width - 2 * anchors.rightMargin,
                                Math.max(Kirigami.Units.gridUnit * 30, scroll.width * view.searchShare))
                texts: view.texts
                controller: view.controller
                doc: doc
                showText: view.showText
                putText: view.putText
                onClosed: area.forceActiveFocus()
            }

            PrintView {
                id: printView
                objectName: "printView"
                anchors.fill: parent
                z: 2
                visible: view.layout === "print" && doc.source !== ""
                controller: view.controller
                texts: view.texts
                // A title page comes before the pages the layout counts.
                titleIndex: sheet.pages.length > view.pageMap.pages ? 0 : -1
                onTitlePageClicked: view.showTitlePage()
            }

            // Without text the view says why and offers the next step (D-17); it gives way to the
            // cursor once the text area has the focus.
            KanteEmptyState {
                objectName: "emptyBook"
                z: 1
                anchors.centerIn: parent
                width: Math.min(parent.width, Kirigami.Units.gridUnit * 24)
                visible: doc.source === "" && !doc.focused
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
                        onClicked: {
                            view.showLayout("flow")
                            area.forceActiveFocus()
                        }
                    }
                }
            }

            QQC2.ScrollView {
                id: scroll
                visible: view.layout === "flow"
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
                    Keys.onPressed: event => view.keyPressed(event)
                    // The context menu is the editor's own (ScriptMenu); the style's would copy
                    // and cut the text without its marks, and cannot take a step back.
                    TapHandler {
                        objectName: "menuTap"
                        acceptedButtons: Qt.RightButton
                        onTapped: eventPoint => scriptMenu.openAt(eventPoint.position)
                    }
                    Component.onCompleted: {
                        for (const child of data) {
                            const right = child.acceptedButtons !== undefined
                                && (child.acceptedButtons & Qt.RightButton) !== 0
                            if (right && child.objectName !== "menuTap") {
                                child.enabled = false
                            }
                        }
                    }
                    // The page breaks of the last layout (DB-30): a thin line above the first line of
                    // each page, the page number at its right end. They sit on the text and move
                    // nothing; `contentHeight` and `width` re-place them when the lines wrap anew.
                    Repeater {
                        model: view.pageMap.breaks
                        delegate: Item {
                            required property var modelData
                            required property int index
                            readonly property int at: index < view.marks.length ? view.marks[index].at : -1
                            objectName: "pageBreak." + modelData.label
                            visible: at >= 0
                            readonly property real lineTop: area.contentHeight >= 0 && area.width > 0
                                ? doc.rectOf(at).y : 0
                            x: area.leftPadding
                            y: lineTop - height
                            width: area.width - area.leftPadding - area.rightPadding
                            height: pageNumber.implicitHeight
                            opacity: view.stale ? KanteStyle.dimOpacity : 1

                            Rectangle {
                                anchors.bottom: parent.bottom
                                width: parent.width
                                height: KanteStyle.borderWidth
                                color: KanteStyle.ruleColor
                            }
                            QQC2.Label {
                                id: pageNumber
                                objectName: "pageNumber." + modelData.label
                                anchors.right: parent.right
                                text: modelData.label + "."
                                color: KanteStyle.mutedTextColor
                                font: KanteStyle.smallFont
                                Accessible.name: view.texts.t("screenplay.pageNumber", { page: modelData.label })
                            }
                        }
                    }
                    RevisionMarks {
                        objectName: "revisionMarks"
                        visible: view.layout === "flow"
                        area: area
                        doc: doc
                        texts: view.texts
                        stale: view.stale
                        ranges: view.revisionRanges
                        ink: view.nextRevision ? revisionInk.color(view.nextRevision.color) : KanteStyle.textColor
                        RevisionInk { id: revisionInk }
                    }
                    // Later: while an edit runs, the cursor has moved but `text` is still the old one.
                    onCursorPositionChanged: {
                        Qt.callLater(flow.leaveIfMoved)
                        Qt.callLater(flow.updateSuggest)
                    }
                    ScriptSuggest {
                        objectName: "scriptSuggest"
                        flow: view.flow
                        area: area
                        doc: doc
                        texts: view.texts
                    }
                    // A leaving focus is a good moment to send: a menu or dialog may save next.
                    onActiveFocusChanged: {
                        Qt.callLater(flow.updateSuggest)
                        if (!activeFocus) {
                            view.flush()
                        }
                    }
                }
            }
        }
    }
}
