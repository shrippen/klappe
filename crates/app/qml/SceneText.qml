import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtQml
import org.kde.kirigami as Kirigami
import Kante

// D-23: the text of the scene on view beside its sheet, read only. Words chosen with the mouse
// or the keyboard open a menu at the selection: elements of a similar name first, then the
// categories for a new one (arrows, Enter). Marked words are underlined in the colour of their
// category, its short name behind them, so colour is not the only sign (DB-22).
//
//  ┌ TEXT DER SZENE ─────────────────────────────┐
//  │ Nebel. Ein Stapel Seiten REQ liegt im …      │
//  │            ┌──────────────────────┐          │
//  │            │ Seite · Requisite    │          │
//  │            │ Neu: Kostüm          │          │
//  │            │ Neu: Requisite …     │          │
//  │            └──────────────────────┘          │
//  └──────────────────────────────────────────────┘
ColumnLayout {
    id: sceneText

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : ({})
    readonly property var lines: project.sheet ? project.sheet.text : []
    readonly property var categories: project.element_categories || []
    readonly property var elements: project.elements || []
    readonly property string sceneId: controller.sceneId || ""
    /// What the menu marks: line, the text before the words, the words. Kept while the menu
    /// closes, so its rows do not change under it.
    property var chosen: null
    /// The line the menu was opened on; its selection goes when the menu closes.
    property var chosenOn: null

    // The colour of a category: Kante's label colours in turn (seven for fifteen), so its short
    // name tells the category, the colour only groups.
    function colorOf(category) {
        const labels = KanteStyle.labels
        return KanteStyle.labelColor(labels[Math.max(0, categories.indexOf(category)) % labels.length])
    }

    function escaped(text) {
        return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    }

    // The short name shown behind a marked run, with its space.
    function tagOf(run) {
        return " " + texts.t("elementShort." + run.category)
    }

    // The line as rich text; spaces kept (`pre-wrap`), so a position counts like in `runs`.
    function html(runs) {
        const parts = runs.map(r => {
            if (!r.element) {
                return escaped(r.text)
            }
            const color = colorOf(r.category)
            return "<span style=\"text-decoration: underline; color: " + color + "\">" + escaped(r.text)
                + "</span><span style=\"font-size: small; color: " + color + "\">" + escaped(tagOf(r)) + "</span>"
        })
        return "<p style=\"white-space: pre-wrap\">" + parts.join("") + "</p>"
    }

    // The line's text before position `pos` of its shown text, short names left out.
    function sourceBefore(runs, pos) {
        let shown = 0
        let source = ""
        for (const r of runs) {
            if (pos <= shown + r.text.length) {
                return source + r.text.slice(0, pos - shown)
            }
            source += r.text
            shown += r.text.length
            if (r.element) {
                const tag = tagOf(r).length
                if (pos <= shown + tag) {
                    return source
                }
                shown += tag
            }
        }
        return source
    }

    // A letter or a digit: part of a word.
    function inWord(c) {
        return c !== undefined && (c.toLowerCase() !== c.toUpperCase() || (c >= "0" && c <= "9"))
    }

    // Whether the line's text from `from` to `to` reaches into a marked run: a second mark
    // there would not show, so it is not offered.
    function touchesMark(runs, from, to) {
        let at = 0
        for (const r of runs) {
            const end = at + r.text.length
            if (r.element && from < end && to > at) {
                return true
            }
            at = end
        }
        return false
    }

    // The words chosen in line `index`, whole: spaces and marks at the ends are left out, a
    // word cut at an end is taken in („pel Seiten. “ → „Stapel Seiten“). Null for none or for marked words.
    function choice(index, runs, start, end) {
        const line = runs.map(r => r.text).join("")
        let from = sourceBefore(runs, start).length
        let to = sourceBefore(runs, end).length
        while (from < to && !inWord(line[from])) {
            from++
        }
        while (to > from && !inWord(line[to - 1])) {
            to--
        }
        if (from === to) {
            return null
        }
        while (inWord(line[from - 1])) {
            from--
        }
        while (inWord(line[to])) {
            to++
        }
        if (touchesMark(runs, from, to)) {
            return null
        }
        return { line: index, prefix: line.slice(0, from), text: line.slice(from, to) }
    }

    // Elements of a similar name: one name holds the other, the case aside.
    function similar(text) {
        const t = text.toLowerCase()
        return elements.filter(e => {
            const n = e.name.toLowerCase()
            return n.includes(t) || t.includes(n)
        })
    }

    /// The menu's rows, set when it opens: a mark changes the elements, the rows stay.
    property var entries: []

    // The menu's rows for `text`: similar elements, then a new one per category.
    function rowsFor(text) {
        return similar(text).map(e => ({
            element: e.id, category: e.category,
            text: e.name + " · " + texts.t("elementCategory." + e.category)
        })).concat(categories.map(c => ({
            element: "", category: c, text: texts.t("textMark.new", { category: texts.t("elementCategory." + c) })
        })))
    }

    function offer(index, edit) {
        const runs = lines[index].runs
        chosen = choice(index, runs, edit.selectionStart, edit.selectionEnd)
        if (!chosen) {
            return
        }
        chosenOn = edit
        entries = rowsFor(chosen.text)
        const at = edit.positionToRectangle(edit.selectionEnd)
        const spot = edit.mapToItem(sceneText, at.x, at.y + at.height)
        menu.popup(sceneText, spot.x, spot.y)
    }

    function mark(entry) {
        controller.markText(sceneId, chosen.line, chosen.prefix, chosen.text, entry.element, entry.category)
    }

    spacing: Kirigami.Units.smallSpacing

    KanteSectionLabel {
        text: sceneText.texts.t("textMark.title")
        rule: true
        Layout.fillWidth: true
    }
    QQC2.Label {
        objectName: "textHint"
        text: sceneText.texts.t(sceneText.lines.length > 0 ? "textMark.hint" : "textMark.none")
        color: KanteStyle.mutedTextColor
        wrapMode: Text.WordWrap
        Layout.fillWidth: true
    }

    Flickable {
        id: flick
        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true
        contentWidth: width
        contentHeight: column.implicitHeight
        boundsBehavior: Flickable.StopAtBounds
        QQC2.ScrollBar.vertical: QQC2.ScrollBar {}

        Column {
            id: column
            width: flick.width
            spacing: Kirigami.Units.smallSpacing

            Repeater {
                model: sceneText.lines
                delegate: TextEdit {
                    id: line
                    required property var modelData
                    required property int index
                    objectName: "textLine." + index
                    readonly property bool speech: ["dialogue", "parenthetical"].includes(modelData.kind)
                    readonly property bool cue: modelData.kind === "cue"

                    x: cue ? column.width * 0.3 : speech ? column.width * 0.15 : 0
                    width: column.width - x - (speech ? column.width * 0.1 : 0)
                    readOnly: true
                    selectByMouse: true
                    selectByKeyboard: true
                    persistentSelection: true
                    activeFocusOnTab: true
                    wrapMode: TextEdit.Wrap
                    textFormat: TextEdit.RichText
                    font.family: KanteStyle.monoFamily
                    color: KanteStyle.textColor
                    selectionColor: KanteStyle.selectionColor
                    selectedTextColor: KanteStyle.textColor
                    text: sceneText.html(modelData.runs)
                    Accessible.role: Accessible.StaticText
                    Accessible.name: modelData.runs.map(r => r.text).join("")

                    // The menu opens when the mouse lets go of a selection (the handler only
                    // watches, the text still selects), and on Enter or the Menu key for one
                    // made with the keyboard.
                    PointHandler {
                        acceptedButtons: Qt.LeftButton
                        onActiveChanged: {
                            if (!active && line.selectedText !== "") {
                                sceneText.offer(line.index, line)
                            }
                        }
                    }
                    Keys.onPressed: event => {
                        const opens = [Qt.Key_Return, Qt.Key_Enter, Qt.Key_Menu].includes(event.key)
                        if (opens && line.selectedText !== "") {
                            sceneText.offer(line.index, line)
                            event.accepted = true
                        }
                    }
                }
            }
        }
    }

    QQC2.Menu {
        id: menu
        objectName: "markMenu"
        KantePopupSkin { popup: menu }
        onClosed: {
            if (sceneText.chosenOn) {
                sceneText.chosenOn.deselect()
            }
            sceneText.chosenOn = null
        }

        Instantiator {
            model: sceneText.entries
            delegate: KanteMenuItem {
                required property var modelData
                objectName: "mark." + (modelData.element || modelData.category)
                text: modelData.text
                onTriggered: sceneText.mark(modelData)
            }
            onObjectAdded: (index, object) => menu.insertItem(index, object)
            onObjectRemoved: (index, object) => menu.removeItem(object)
        }
    }
}
